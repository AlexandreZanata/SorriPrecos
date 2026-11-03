package br.com.sorriprecos

import android.os.Bundle
import android.content.Context
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.compose.BackHandler
import androidx.compose.runtime.Composable
import androidx.core.content.edit
import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.HttpTimeout
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

class MainActivity:ComponentActivity() {
    override fun onCreate(savedInstanceState:Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val store=AndroidStore(this)
        setContent {App(store)}
    }
}
class AndroidStore(context:Context):LocalStore {
    private val prefs=context.getSharedPreferences("sorriprecos",Context.MODE_PRIVATE)
    override fun get(key:String)=prefs.getString(key,null)
    override fun put(key:String,value:String){prefs.edit{putString(key,value)}}
    override fun remove(key:String){prefs.edit{remove(key)}}
}
actual fun platformClient()=HttpClient(CIO) {install(HttpTimeout){requestTimeoutMillis=6500;connectTimeoutMillis=3000;socketTimeoutMillis=6500}}
actual fun platformNow()=System.currentTimeMillis()/1000
@Composable actual fun NavigationBackHandler(enabled:Boolean,onBack:()->Unit) {BackHandler(enabled,onBack)}
actual fun formatDate(seconds:Long)=SimpleDateFormat("dd/MM/yyyy 'às' HH:mm",Locale.forLanguageTag("pt-BR")).format(Date(seconds*1000))
