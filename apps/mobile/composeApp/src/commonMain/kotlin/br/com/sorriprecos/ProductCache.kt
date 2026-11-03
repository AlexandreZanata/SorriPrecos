package br.com.sorriprecos

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

internal const val PRODUCT_SYNC_INTERVAL = 600L
internal const val PRODUCT_SYNC_BATCH = 40

@Serializable
internal data class SearchPage(val ids: List<Long>, val nextAfter: Long?, val savedAt: Long)

@Serializable
internal data class ProductRecord(
    val product: Product? = null,
    val comparison: Comparison? = null,
    val version: String? = null,
    val checkedAt: Long,
    val removed: Boolean = false
)

@Serializable
internal data class SyncProduct(
    val id: Long, val historical: Boolean, val comparison: Boolean, val version: String? = null
)

@Serializable internal data class SyncRequest(val products: List<SyncProduct>)
@Serializable internal data class SyncResponse(val products: List<SyncUpdate>)

@Serializable
internal data class SyncUpdate(
    val id: Long, val historical: Boolean, val version: String? = null,
    val removed: Boolean = false, val data: SyncData? = null, val freshness: SyncFreshness? = null
)

@Serializable
internal data class SyncData(
    val product: Product, val items: List<Offer>? = null,
    @SerialName("coverage_note") val coverageNote: String = ""
)

@Serializable
internal data class SyncFreshness(
    @SerialName("last_collected_at") val lastCollectedAt: Long? = null,
    val offers: List<OfferFreshness> = emptyList()
)

@Serializable
internal data class OfferFreshness(
    @SerialName("branch_id") val branchId: String,
    @SerialName("collected_at") val collectedAt: Long,
    @SerialName("valid_until") val validUntil: Long? = null
)

internal fun ProductRecord.withFreshness(freshness: SyncFreshness, checkedAt: Long): ProductRecord {
    val product = product?.copy(lastCollectedAt = freshness.lastCollectedAt)
    val observations = freshness.offers.associateBy(OfferFreshness::branchId)
    val offers = comparison?.items?.map { offer ->
        val observation = observations[offer.branchId]
        offer.copy(collectedAt = observation?.collectedAt, validUntil = observation?.validUntil)
    }
    val comparison = if (comparison != null && product != null && offers != null)
        comparison.copy(product = product, items = offers) else null
    return copy(product = product, comparison = comparison, checkedAt = checkedAt)
}
