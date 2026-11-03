package br.com.sorriprecos

import io.ktor.client.HttpClient
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.request.get
import io.ktor.client.statement.bodyAsText
import io.ktor.http.encodeURLParameter
import kotlinx.coroutines.CancellationException
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import androidx.compose.runtime.Composable

interface LocalStore { fun get(key:String):String?; fun put(key:String,value:String); fun remove(key:String) }
expect fun platformClient():HttpClient
expect fun platformNow():Long
expect fun formatDate(seconds:Long):String
@Composable expect fun PlatformAppearance(highContrast:Boolean)
@Composable expect fun NavigationBackHandler(enabled:Boolean,onBack:()->Unit)

val codec=Json { ignoreUnknownKeys=true }
data class Loaded<T>(val data:T,val offline:Boolean=false,val savedAt:Long=platformNow())
class CatalogUnavailable:Exception("Não foi possível conectar. Confira o endereço da API e tente novamente.")

class Repository(val store:LocalStore,val baseUrl:String,val client:HttpClient=platformClient()) {
    suspend inline fun <reified T> request(path:String):Loaded<T> {
        val key="cache:v2:$baseUrl:$path"
        try {
            val response=client.get(baseUrl.trimEnd('/')+path)
            if(response.status.value !in 200..299)throw CatalogUnavailable()
            val text=response.bodyAsText()
            val value=codec.decodeFromString<T>(text)
            save(key,text)
            return Loaded(value)
        } catch(e:CancellationException) {throw e} catch(e:Exception) {
            val cached=store.get(key)
            if(cached!=null) {
                try {return Loaded(codec.decodeFromString<T>(cached),offline=true,savedAt=store.get("time:$key")?.toLongOrNull()?:0)} catch(_:Exception) {store.remove(key)}
            }
            throw CatalogUnavailable()
        }
    }
    fun save(key:String,text:String) {
        store.put(key,text);store.put("time:$key",platformNow().toString())
        val keys=(store.get("cache-keys")?.split('\n').orEmpty().filter{it.isNotEmpty() && it!=key}+key)
        keys.dropLast(60).forEach {store.remove(it);store.remove("time:$it")}
        store.put("cache-keys",keys.takeLast(60).joinToString("\n"))
    }
    suspend fun search(query:String,demo:Boolean,after:Long=0,historical:Boolean=false)=request<Items<Product>>("/v1/products/search?q=${query.encodeURLParameter()}&demo=$demo&after=$after&historical=$historical")
    suspend fun prices(id:Long,historical:Boolean=false)=request<Comparison>("/v1/products/$id/prices?historical=$historical")
    suspend fun markets(demo:Boolean,historical:Boolean=false)=request<Items<Supermarket>>("/v1/supermarkets?demo=$demo&historical=$historical")
    suspend fun status()=request<CatalogStatus>("/v1/catalog/status")
    fun favorites():List<Product> = try {codec.decodeFromString(store.get("favorites:$baseUrl")?:"[]")}catch(_:Exception){emptyList()}
    fun saveFavorites(items:List<Product>) {store.put("favorites:$baseUrl",codec.encodeToString(items.take(200)))}
}
