package br.com.sorriprecos

import androidx.compose.ui.window.ComposeUIViewController
import androidx.compose.runtime.Composable
import io.ktor.client.HttpClient
import io.ktor.client.engine.darwin.Darwin
import io.ktor.client.plugins.HttpTimeout
import platform.Foundation.NSDate
import platform.Foundation.NSDateFormatter
import platform.Foundation.NSUserDefaults
import platform.Foundation.timeIntervalSince1970

fun MainViewController()=ComposeUIViewController {App(IosStore())}
class IosStore:LocalStore {
    private val defaults=NSUserDefaults.standardUserDefaults
    override fun keys(prefix: String): List<String> = defaults.dictionaryRepresentation().keys.filterIsInstance<String>().filter { it.startsWith(prefix) }
    override fun get(key:String)=defaults.stringForKey(key)
    override fun put(key:String,value:String){defaults.setObject(value,forKey=key)}
    override fun remove(key:String){defaults.removeObjectForKey(key)}
}
actual fun platformClient()=HttpClient(Darwin) {install(HttpTimeout){requestTimeoutMillis=6500;connectTimeoutMillis=3000;socketTimeoutMillis=6500}}
actual fun platformNow()=NSDate().timeIntervalSince1970.toLong()
@Composable actual fun NavigationBackHandler(enabled:Boolean,onBack:()->Unit) { /* iOS uses the visible back control. */ }
actual fun formatDate(seconds:Long):String {
    val formatter=NSDateFormatter();formatter.dateFormat="dd/MM/yyyy 'às' HH:mm"
    return formatter.stringFromDate(NSDate(timeIntervalSince1970=seconds.toDouble()))
}

@Composable actual fun PlatformAppearance(highContrast:Boolean, greenHeader:Boolean) { /* System bars follow the host UIViewController. */ }
