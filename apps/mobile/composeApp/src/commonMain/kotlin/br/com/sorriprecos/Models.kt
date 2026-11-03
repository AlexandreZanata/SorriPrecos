package br.com.sorriprecos

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable data class Items<T>(val items:List<T>,@SerialName("next_after") val nextAfter:Long?=null,val versions:Map<String,String> = emptyMap())
@Serializable data class Product(
    val id:Long,val name:String,val brand:String,val quantity:Long,val unit:String,val presentation:String,val category:String,
    val gtin:String?=null,@SerialName("is_demo") val isDemo:Boolean=false,@SerialName("is_historical") val isHistorical:Boolean=false,
    @SerialName("min_price_cents") val minPriceCents:Long?=null,
    @SerialName("last_collected_at") val lastCollectedAt:Long?=null,
    @SerialName("branch_count") val branchCount:Int=0,
    @SerialName("reference_price_cents") val referencePriceCents:Long?=null,
    @SerialName("historical_branch_count") val historicalBranchCount:Int=0,
    @SerialName("photo_path") val photoPath:String?=null
) {val pack:String get()=if (quantity%1000L==0L && unit in listOf("g","ml")) "${quantity/1000} ${if(unit=="g") "kg" else "L"}" else "$quantity $unit"}
@Serializable data class Supermarket(
    val id:String,val name:String,val website:String?=null,
    @SerialName("integration_status") val integrationStatus:String,
    @SerialName("audit_class") val auditClass:String,
    @SerialName("last_collected_at") val lastCollectedAt:Long?=null,
    @SerialName("branch_count") val branchCount:Int=0,
    @SerialName("product_count") val productCount:Int=0,
    @SerialName("is_demo") val isDemo:Boolean=false
)
@Serializable data class CatalogStatus(
    @SerialName("discovered_supermarkets") val discoveredSupermarkets:Int=0,
    @SerialName("discovered_branches") val discoveredBranches:Int=0,
    @SerialName("automatic_integrations") val automaticIntegrations:Int=0,
    @SerialName("real_products") val realProducts:Int=0,
    @SerialName("demo_products") val demoProducts:Int=0,
    @SerialName("historical_products") val historicalProducts:Int=0,
    @SerialName("historical_supermarkets") val historicalSupermarkets:Int=0,
    @SerialName("historical_last_collected_at") val historicalLastCollectedAt:Long?=null,
    @SerialName("recent_products") val recentProducts:Int=0,
    @SerialName("imported_supermarkets") val importedSupermarkets:Int=0,
    @SerialName("last_collected_at") val lastCollectedAt:Long?=null
)
@Serializable data class Offer(
    @SerialName("branch_id") val branchId:String,
    @SerialName("branch_name") val branchName:String,
    @SerialName("supermarket_name") val supermarketName:String,
    val address:String,val status:String,
    @SerialName("price_cents") val priceCents:Long?=null,
    @SerialName("promotional_price_cents") val promotionalPriceCents:Long?=null,
    @SerialName("effective_price_cents") val effectivePriceCents:Long?=null,
    @SerialName("offer_type") val offerType:String?=null,
    val conditions:String?=null,
    @SerialName("source_url") val sourceUrl:String?=null,
    @SerialName("collected_at") val collectedAt:Long?=null,
    @SerialName("valid_until") val validUntil:Long?=null,
    @SerialName("is_demo") val isDemo:Boolean=false
)
@Serializable data class Comparison(val product:Product,val items:List<Offer>,@SerialName("coverage_note") val coverageNote:String="")

internal fun Product.isDiscoverable(at: Long = platformNow()): Boolean =
    isHistorical || (minPriceCents != null && branchCount > 0 &&
        lastCollectedAt?.let { it >= at - 172800 } == true)

fun money(cents:Long?):String {
    if(cents==null)return "Preço não disponível"
    val whole=(cents/100).toString().reversed().chunked(3).joinToString(".").reversed()
    return "R$ $whole,${(cents%100).toString().padStart(2,'0')}"
}
fun age(seconds:Long?):String {
    if(seconds==null)return "Sem coleta"
    val delta=(platformNow()-seconds).coerceAtLeast(0)
    return when {delta<60->"Verificado agora";delta<3600->"Há ${delta/60} min";delta<86400->"Há ${delta/3600} h";else->formatDate(seconds)}
}
