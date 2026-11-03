package br.com.sorriprecos

import kotlin.test.Test
import kotlin.test.assertEquals

class CartTest {
    @Test fun completeBranchesRankBeforePartialSubtotals() {
        val at = 180_000L
        fun product(id: Long) = Product(id, "Produto $id", "Marca", 1, "un", "unidade", "Mercearia")
        fun offer(branch: String, price: Long, until: Long? = null) =
            Offer(branch, branch, "Rede", "Sorriso", "recent", effectivePriceCents = price,
                collectedAt = at, validUntil = until, sourceUrl = "https://example.com/$branch/$price")

        val quotes = quoteCart(
            listOf(CartLine(product(1), 2), CartLine(product(2))),
            mapOf(1L to listOf(offer("A", 400), offer("B", 100)),
                2L to listOf(offer("A", 200), offer("B", 100, at - 1), offer("C", 90, at - 1))),
            at
        )
        assertEquals(listOf("A", "B"), quotes.map { it.branch.branchId })
        assertEquals(listOf(1000L, 200L), quotes.map { it.subtotal })
        assertEquals(listOf(2, 1), quotes.map { it.priced })
        assertEquals(CartItemQuote(800, "https://example.com/A/400"), quotes[0].items[1])
        assertEquals(CartItemQuote(200, "https://example.com/B/100"), quotes[1].items[1])
        assertEquals(null, quotes[1].items[2])
    }
}
