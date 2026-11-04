package br.com.sorriprecos

import kotlin.test.*
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import io.ktor.http.encodedPath
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.test.runTest

class MemoryStore:LocalStore {
    val values=mutableMapOf<String,String>()
    override fun get(key:String)=values[key]
    override fun put(key:String,value:String){values[key]=value}
    override fun remove(key:String){values.remove(key)}
    override fun keys(prefix:String)=values.keys.filter { it.startsWith(prefix) }
}
class RepositoryTest {
    @Test fun amountsAndPackaging() {
        assertEquals("R$ 23,99",money(2399))
        assertEquals("R$ 1.234,56",money(123456))
        assertEquals("Preço não disponível",money(null))
        assertEquals("5 kg",Product(1,"Arroz","DEMO",5000,"g","pacote","Mercearia").pack)
        assertEquals("350 ml",Product(2,"Refrigerante","DEMO",350,"ml","lata","Bebidas").pack)
    }
    @Test fun apiContractPreservesDemoAndCents() {
        val p=codec.decodeFromString<Product>("""{"id":1,"name":"Arroz DEMO","brand":"DEMO","quantity":5000,"unit":"g","presentation":"pacote","category":"Mercearia","is_demo":true,"min_price_cents":2399,"branch_count":2,"future_field":true}""")
        assertTrue(p.isDemo);assertEquals(2399,p.minPriceCents);assertEquals(2,p.branchCount)
    }
    @Test fun failedConnectionReturnsPersistentCache()=runTest {
        val store=MemoryStore()
        val good=Repository(store,"https://example.test",HttpClient(MockEngine{respond("{\"items\":[]}")}))
        assertFalse(good.markets(false).offline);good.client.close()
        val offline=Repository(store,"https://example.test",HttpClient(MockEngine{error("offline")}))
        assertTrue(offline.markets(false).offline);offline.client.close()
    }
    @Test fun historicalCacheNeverLeaksIntoCurrentSearch()=runTest {
        val store=MemoryStore()
        val historic = Product(1,"Arroz","Marca",5000,"g","pacote","Mercearia",isHistorical=true)
        val good=Repository(store,"https://example.test",HttpClient(MockEngine{request ->
            assertEquals("true",request.url.parameters["historical"])
            respond(codec.encodeToString(Items(listOf(historic))))
        }))
        good.search("arroz",false,historical=true);good.client.close()
        val offline=Repository(store,"https://example.test",HttpClient(MockEngine{error("offline")}))
        assertFailsWith<CatalogUnavailable>{offline.search("arroz",false)}
        assertEquals(historic,offline.search("arroz",false,historical=true).data.items.single())
        offline.client.close()
    }
    @Test fun cancellationNeverFallsBackToOldResults()=runTest {
        val store=MemoryStore();store.put("cache:v2:https://example.test:/v1/supermarkets?demo=false&historical=false","{\"items\":[]}")
        val repo=Repository(store,"https://example.test",HttpClient(MockEngine{throw CancellationException("obsolete")}))
        assertFailsWith<CancellationException>{repo.markets(false)};repo.client.close()
    }
    @Test fun cachesDoNotCrossServers()=runTest {
        val store=MemoryStore();store.put("cache:v2:https://one.test:/v1/supermarkets?demo=false&historical=false","{\"items\":[]}")
        val repo=Repository(store,"https://two.test",HttpClient(MockEngine{respond("{}",HttpStatusCode.ServiceUnavailable)}))
        assertFailsWith<CatalogUnavailable>{repo.markets(false)};repo.client.close()
    }

    @Test fun productsSurviveRepositoryRestartAndReconcileThroughOneBatch()=runTest {
        val disk=MemoryStore()
        val product=Product(1,"Arroz","Marca",5000,"g","pacote","Mercearia",minPriceCents=2399,lastCollectedAt=platformNow(),branchCount=1)
        val first=Repository(disk,"https://example.test",HttpClient(MockEngine { request ->
            assertEquals("/v1/products/search",request.url.encodedPath)
            respond(codec.encodeToString(Items(listOf(product))))
        }))
        assertEquals(product,first.search("arroz",false).data.items.single())
        first.client.close()

        var calls=0
        val second=Repository(disk,"https://example.test",HttpClient(MockEngine { request ->
            calls++
            assertEquals("/v1/products/sync",request.url.encodedPath)
            respond(codec.encodeToString(SyncResponse(listOf(SyncUpdate(
                id=1,historical=false,version="next",data=SyncData(product.copy(minPriceCents=if (calls == 1) 2399 else 2199),emptyList())
            )))))
        }))
        assertEquals(product,second.search("arroz",false).data.items.single())
        second.syncKnownProducts()
        assertEquals(0,calls)
        assertEquals(2399L,second.prices(1).data.product.minPriceCents)
        assertEquals(2399L,second.prices(1).data.product.minPriceCents)
        assertEquals(1,calls)

        val key = disk.keys("products:v1:").single()
        val cached = codec.decodeFromString<ProductRecord>(disk.get(key)!!)
        disk.put(key, codec.encodeToString(cached.copy(checkedAt = platformNow() - PRODUCT_SYNC_INTERVAL)))
        second.syncKnownProducts()
        assertEquals(2199L,second.search("arroz",false).data.items.single().minPriceCents)
        assertEquals(2199L,second.prices(1).data.product.minPriceCents)
        second.syncKnownProducts()
        assertEquals(2,calls)
        second.client.close()
    }
    @Test fun unconfirmedFlyerExpiresAndNeverEntersCartTotals() {
        val now = platformNow()
        val product = Product(5, "Arroz", "Marca", 5000, "g", "pacote", "Mercearia",
            unconfirmedPriceCents = 1850, unconfirmedValidUntil = now + 60, lastCollectedAt = now)
        assertTrue(product.isDiscoverable(now))
        assertFalse(product.isDiscoverable(now + 61))
        val offer = Offer("atacadao-sorriso", "Sorriso", "Atacadão", "Sorriso", "unconfirmed_stock",
            priceCents = 1850, collectedAt = now, validUntil = now + 60)
        assertTrue(offer.hasUnconfirmedStock(now))
        assertFalse(offer.hasUnconfirmedStock(now + 61))
        assertFalse(offer.isCurrent(now))
        assertTrue(quoteCart(listOf(CartLine(product)), mapOf(5L to listOf(offer)), now).isEmpty())
    }

    @Test fun staleSearchRefreshesAndKeepsOfflineResults() = runTest {
        val disk = MemoryStore()
        val old = Product(5, "Arroz", "Marca", 5000, "g", "pacote", "Mercearia",
            minPriceCents = 2200, branchCount = 1, lastCollectedAt = platformNow())
        val newer = Product(6, "Arroz", "Marca", 5000, "g", "pacote", "Mercearia",
            unconfirmedPriceCents = 1850, unconfirmedValidUntil = platformNow() + 3600, lastCollectedAt = platformNow())
        val initial = Repository(disk, "https://example.test", HttpClient(MockEngine { respond(codec.encodeToString(Items(listOf(old)))) }))
        initial.search("arroz", false); initial.client.close()
        val searchKey = disk.keys("search:v1:").single()
        val page = codec.decodeFromString<SearchPage>(disk.get(searchKey)!!)
        disk.put(searchKey, codec.encodeToString(page.copy(savedAt = platformNow() - PRODUCT_SYNC_INTERVAL)))
        val offline = Repository(disk, "https://example.test", HttpClient(MockEngine { error("offline") }))
        assertEquals(listOf(old), offline.search("arroz", false).data.items)
        offline.client.close()
        val online = Repository(disk, "https://example.test", HttpClient(MockEngine { respond(codec.encodeToString(Items(listOf(old, newer)))) }))
        assertEquals(listOf(old, newer), online.search("arroz", false).data.items)
        online.client.close()
    }
}
