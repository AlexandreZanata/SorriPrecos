package br.com.sorriprecos

import io.ktor.client.HttpClient
import io.ktor.client.request.get
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.client.statement.bodyAsText
import io.ktor.http.encodeURLParameter
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import androidx.compose.runtime.Composable

interface LocalStore {
    fun get(key: String): String?
    fun put(key: String, value: String)
    fun remove(key: String)
    val catalogue: LocalStore get() = this
    fun keys(prefix: String): List<String> = emptyList()
}
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
            val cached=store.catalogue.get(key) ?: store.get(key)
            if(cached!=null) {
                try {return Loaded(codec.decodeFromString<T>(cached),offline=true,savedAt=(store.catalogue.get("time:$key") ?: store.get("time:$key"))?.toLongOrNull()?:0)} catch(_:Exception) {store.catalogue.remove(key);store.remove(key)}
            }
            throw CatalogUnavailable()
        }
    }
    fun save(key:String,text:String) {
        val cache = store.catalogue
        cache.put(key,text);cache.put("time:$key",platformNow().toString())
        val keys=(cache.get("cache-keys")?.split('\n').orEmpty().filter{it.isNotEmpty() && it!=key}+key)
        keys.dropLast(60).forEach {cache.remove(it);cache.remove("time:$it")}
        cache.put("cache-keys",keys.takeLast(60).joinToString("\n"))
    }
    private val disk = store.catalogue
    private val syncLock = Mutex()
    private val _updates = MutableStateFlow(0)
    internal val updates = _updates.asStateFlow()
    private var lastFailedAttempt: Long? = null
    private val syncFailed = MutableStateFlow(false)
    private val prefix = "products:v1:$baseUrl:"

    private fun key(id: Long, historical: Boolean) = "$prefix$historical:$id"
    private fun record(id: Long, historical: Boolean): ProductRecord? =
        disk.get(key(id, historical))?.let { codec.decodeFromString<ProductRecord>(it) }

    private fun saveRecord(id: Long, historical: Boolean, record: ProductRecord) =
        disk.put(key(id, historical), codec.encodeToString(record))

    private fun legacyComparison(id: Long, historical: Boolean): ProductRecord? {
        val legacy = "cache:v2:$baseUrl:/v1/products/$id/prices?historical=$historical"
        val comparison = store.get(legacy)?.let { codec.decodeFromString<Comparison>(it) } ?: return null
        val product = record(id, historical)?.product ?: comparison.product
        return ProductRecord(product, comparison.copy(product = product), checkedAt = 0)
            .also { saveRecord(id, historical, it) }
    }

    suspend fun search(query: String, demo: Boolean, after: Long = 0, historical: Boolean = false): Loaded<Items<Product>> =
        withContext(Dispatchers.Default) {
            val path = "/v1/products/search?q=${query.encodeURLParameter()}&demo=$demo&after=$after&historical=$historical"
            val key = "search:v1:$baseUrl:$path"
            val page = disk.get(key)?.let { codec.decodeFromString<SearchPage>(it) }
            if (page != null) {
                val stored = page.ids.mapNotNull { record(it, historical)?.product }.filter { it.isDiscoverable() }
                if (stored.isNotEmpty() || (page.ids.isEmpty() && platformNow() - page.savedAt in 0 until PRODUCT_SYNC_INTERVAL)) {
                    return@withContext Loaded(Items(stored, page.nextAfter), offline = syncFailed.value, savedAt = page.savedAt)
                }
            }
            val legacyKey = "cache:v2:$baseUrl:$path"
            val legacy = if (page == null) store.get(legacyKey)?.let { codec.decodeFromString<Items<Product>>(it) } else null
            val loaded = try {
                legacy?.let { Loaded(it, offline = true, savedAt = 0) } ?: request<Items<Product>>(path)
            } catch (e: CatalogUnavailable) {
                if (page != null) return@withContext Loaded(Items(emptyList(), page.nextAfter), offline = true, savedAt = page.savedAt)
                throw e
            }
            syncLock.withLock {
                val at = if (loaded.offline) 0 else platformNow()
                for (product in loaded.data.items) {
                    if (record(product.id, historical) == null) {
                        saveRecord(product.id, historical, ProductRecord(product = product, version = loaded.data.versions[product.id.toString()], checkedAt = at))
                    }
                }
                disk.put(key, codec.encodeToString(SearchPage(loaded.data.items.map(Product::id), loaded.data.nextAfter, at)))
            }
            loaded.copy(data = loaded.data.copy(items = loaded.data.items.filter { it.isDiscoverable() }))
        }

    internal suspend fun latestProducts(products: List<Product>): Loaded<List<Product>> =
        withContext(Dispatchers.Default) {
            Loaded(products.mapNotNull { product ->
                val latest = record(product.id, product.isHistorical)
                if (latest?.removed == true) null else latest?.product ?: product
            }.filter { it.isDiscoverable() }, offline = syncFailed.value)
        }

    suspend fun prices(id: Long, historical: Boolean = false): Loaded<Comparison> =
        pricesFor(listOf(id), historical)[id] ?: throw CatalogUnavailable()

    internal suspend fun pricesFor(ids: List<Long>, historical: Boolean = false): Map<Long, Loaded<Comparison>?> =
        withContext(Dispatchers.Default) {
            fun cachedPrices(failed: Boolean = false) = ids.associateWith { id ->
                val saved = record(id, historical)
                saved?.comparison?.let { Loaded(it, offline = syncFailed.value || failed, savedAt = saved.checkedAt) }
            }
            fun needsComparison(saved: ProductRecord?) = saved?.comparison == null && saved?.removed != true
            if (ids.none { needsComparison(record(it, historical)) }) return@withContext cachedPrices()

            syncLock.withLock {
                val missing = ids.distinct().mapNotNull { id ->
                    val existing = record(id, historical)
                    if (!needsComparison(existing) || legacyComparison(id, historical) != null) null
                    else SyncProduct(id, historical, comparison = true)
                }
                var failed = false
                for (batch in missing.chunked(PRODUCT_SYNC_BATCH)) {
                    try {
                        syncBatch(batch)
                    } catch (e: CancellationException) {
                        throw e
                    } catch (_: Exception) {
                        failed = true
                    }
                }
                cachedPrices(failed)
            }
        }

    internal suspend fun syncKnownProducts(force: Boolean = false, onlyId: Long? = null) = withContext(Dispatchers.Default) {
        syncLock.withLock {
            val at = platformNow()
            if (!force && lastFailedAttempt?.let { at - it in 0 until PRODUCT_SYNC_INTERVAL } == true) return@withLock
            val due = disk.keys(prefix).mapNotNull { key ->
                val row = disk.get(key)?.let { codec.decodeFromString<ProductRecord>(it) } ?: return@mapNotNull null
                val id = key.substringAfterLast(':').toLongOrNull() ?: return@mapNotNull null
                if ((onlyId != null && id != onlyId) || (!force && at - row.checkedAt in 0 until PRODUCT_SYNC_INTERVAL)) return@mapNotNull null
                val historical = key.removePrefix(prefix).substringBefore(':') == "true"
                SyncProduct(id, historical, row.comparison != null, row.version)
            }
            if (due.isEmpty()) return@withLock
            var wrote = false
            try {
                for (batch in due.chunked(PRODUCT_SYNC_BATCH)) {
                    syncBatch(batch)
                    wrote = true
                }
                lastFailedAttempt = null
                syncFailed.value = false
            } catch (e: CancellationException) {
                throw e
            } catch (_: Exception) {
                lastFailedAttempt = at
                syncFailed.value = true
            } finally {
                if (wrote || syncFailed.value) _updates.value++
            }
        }
    }

    private suspend fun syncBatch(batch: List<SyncProduct>) {
        val response = client.post(baseUrl.trimEnd('/') + "/v1/products/sync") {
            contentType(ContentType.Application.Json)
            setBody(codec.encodeToString(SyncRequest(batch)))
        }
        if (response.status.value !in 200..299) throw CatalogUnavailable()
        val updates = codec.decodeFromString<SyncResponse>(response.bodyAsText()).products
        val at = platformNow()
        for (update in updates) {
            val prior = record(update.id, update.historical)
            val data = update.data
            val next = when {
                update.removed -> ProductRecord(checkedAt = at, removed = true)
                data != null -> ProductRecord(
                    product = data.product, comparison = data.items?.let { Comparison(data.product, it, data.coverageNote) },
                    version = update.version, checkedAt = at
                )
                prior != null && update.freshness != null -> prior.withFreshness(update.freshness, at)
                else -> continue
            }
            saveRecord(update.id, update.historical, next)
        }
    }

    suspend fun markets(demo:Boolean,historical:Boolean=false)=request<Items<Supermarket>>("/v1/supermarkets?demo=$demo&historical=$historical")
    suspend fun status()=request<CatalogStatus>("/v1/catalog/status")
    internal fun cart(demo:Boolean):List<CartLine> = try {
        codec.decodeFromString<List<CartLine>>(store.get("cart:$baseUrl:$demo") ?: "[]")
            .filter { it.product.isDemo == demo && !it.product.isHistorical && it.quantity in 1..CART_MAX_QUANTITY }
            .distinctBy { it.product.id }.take(CART_LIMIT)
    } catch (_:Exception) { emptyList() }
    internal fun saveCart(demo:Boolean, lines:List<CartLine>) {
        store.put("cart:$baseUrl:$demo",codec.encodeToString(lines))
    }
    fun favorites():List<Product> = try {codec.decodeFromString(store.get("favorites:$baseUrl")?:"[]")}catch(_:Exception){emptyList()}
    fun saveFavorites(items:List<Product>) {store.put("favorites:$baseUrl",codec.encodeToString(items.take(200)))}
}
