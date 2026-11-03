package br.com.sorriprecos

import kotlin.test.*
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.test.runTest

class MemoryStore:LocalStore {
    val values=mutableMapOf<String,String>()
    override fun get(key:String)=values[key]
    override fun put(key:String,value:String){values[key]=value}
    override fun remove(key:String){values.remove(key)}
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
        val good=Repository(store,"https://example.test",HttpClient(MockEngine{request ->
            assertEquals("true",request.url.parameters["historical"])
            respond("{\"items\":[]}")
        }))
        good.search("arroz",false,historical=true);good.client.close()
        val offline=Repository(store,"https://example.test",HttpClient(MockEngine{error("offline")}))
        assertFailsWith<CatalogUnavailable>{offline.search("arroz",false)}
        assertTrue(offline.search("arroz",false,historical=true).offline);offline.client.close()
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
}
