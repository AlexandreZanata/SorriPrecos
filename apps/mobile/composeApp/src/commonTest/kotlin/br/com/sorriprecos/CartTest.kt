package br.com.sorriprecos

import kotlin.test.Test
import kotlin.test.assertEquals

class CartTest {
    @Test fun completeBranchesRankBeforePartialSubtotals() {
        val at = 180_000L
        fun product(id: Long) = Product(id, "Produto $id", "Marca", 1, "un", "unidade", "Mercearia")
        fun offer(branch: String, price: Long, until: Long? = null) =
            Offer(branch, branch, "Rede", "Sorriso", "recent", effectivePriceCents = price,
                collectedAt = at, validUntil = until)

        val quotes = quoteCart(
            listOf(CartLine(product(1), 2), CartLine(product(2))),
            mapOf(1L to listOf(offer("A", 400), offer("B", 100)),
                2L to listOf(offer("A", 200), offer("B", 100, at - 1))),
            at
        )
        assertEquals(listOf("A", "B"), quotes.map { it.branch.branchId })
        assertEquals(listOf(1000L, 200L), quotes.map { it.subtotal })
        assertEquals(listOf(2, 1), quotes.map { it.priced })
    }
}
