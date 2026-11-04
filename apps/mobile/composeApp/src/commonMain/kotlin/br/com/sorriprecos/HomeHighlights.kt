package br.com.sorriprecos

internal data class HomeHighlight(val product: Product, val offer: Offer) {
    val discountPercent: Int?
        get() {
            val regular = offer.priceCents ?: return null
            val effective = offer.effectivePriceCents ?: return null
            if (effective <= 0 || effective >= regular || !offer.conditions.isNullOrBlank()) return null
            return ((regular - effective) * 100 / regular).toInt().takeIf { it > 0 }
        }
}

internal suspend fun Repository.homeHighlights(demo: Boolean): List<HomeHighlight> {
    val queries = listOf("arroz 5kg", "açúcar 5kg", "café 500g")
    val products = queries.mapNotNull { query ->
        try {
            val priced = search(query, demo).data.items.filter { it.minPriceCents != null && it.branchCount > 0 }
            priced.firstOrNull { it.photoPath != null } ?: priced.firstOrNull()
        } catch (_: CatalogUnavailable) {
            null
        }
    }.distinctBy(Product::id)
    val comparisons = pricesFor(products.map(Product::id))
    return products.mapNotNull { product ->
        val comparison = comparisons[product.id]?.data ?: return@mapNotNull null
        val offer = comparison.items.filter { it.isCurrent() && it.effectivePriceCents != null }
            .minByOrNull { it.effectivePriceCents!! } ?: return@mapNotNull null
        HomeHighlight(comparison.product, offer)
    }
}
