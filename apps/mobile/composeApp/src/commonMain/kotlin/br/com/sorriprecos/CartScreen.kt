package br.com.sorriprecos

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.serialization.Serializable

internal const val CART_LIMIT = 25
internal const val CART_MAX_QUANTITY = 99

@Serializable
internal data class CartLine(val product: Product, val quantity: Int = 1)

internal data class CartQuote(val branch: Offer, val subtotal: Long, val priced: Int)

internal fun Offer.isCurrent(at: Long = platformNow()): Boolean =
    status == "recent" && collectedAt?.let { it >= at - 172800 } == true &&
        (validUntil == null || validUntil >= at)

internal fun quoteCart(lines: List<CartLine>, offers: Map<Long, List<Offer>>, at: Long): List<CartQuote> {
    val byProduct = offers.mapValues { (_, prices) -> prices.associateBy(Offer::branchId) }
    return offers.values.flatten().distinctBy(Offer::branchId).map { branch ->
        val values = lines.mapNotNull { line ->
            byProduct[line.product.id]?.get(branch.branchId)?.takeIf { it.isCurrent(at) }
                ?.effectivePriceCents?.times(line.quantity)
        }
        CartQuote(branch, values.sum(), values.size)
    }.sortedWith(compareByDescending<CartQuote> { it.priced }.thenBy { it.subtotal }.thenBy { it.branch.supermarketName })
}

@Composable internal fun CartScreen(
    repo: Repository, lines: List<CartLine>, demo: Boolean, onQuantity: (Product, Int) -> Unit,
    onSearch: () -> Unit, onOpen: (Product) -> Unit
) {
    val ids = lines.map { it.product.id }
    var results by remember(repo, ids) { mutableStateOf<Map<Long, Loaded<Comparison>?>>(emptyMap()) }
    var attempt by remember { mutableIntStateOf(0) }
    var loading by remember(repo, ids) { mutableStateOf(ids.isNotEmpty()) }
    LaunchedEffect(repo, ids, attempt) {
        loading = ids.isNotEmpty()
        try {
            results = buildMap {
                for (batch in ids.chunked(4)) {
                    putAll(coroutineScope {
                        batch.map { id ->
                            async {
                                id to try { repo.prices(id) }
                                catch (e: CancellationException) { throw e }
                                catch (_: Exception) { null }
                            }
                        }.awaitAll()
                    })
                }
            }
        } finally { loading = false }
    }
    val quotes = quoteCart(lines, results.mapNotNull { (id, result) ->
        result?.let { id to it.data.items }
    }.toMap(), platformNow())
    LazyColumn(contentPadding = PaddingValues(18.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        item {
            Text(if (demo) "Carrinho de demonstração" else "Meu carrinho", style = MaterialTheme.typography.headlineSmall)
            Text("Compare o custo da lista em cada filial, sem frete.")
            TextButton(onClick = onSearch) { Text("Buscar produtos") }
        }
        if (lines.isEmpty()) item { EmptyCard("Carrinho vazio", "Abra um produto para adicioná-lo à lista.") }
        items(lines, key = { it.product.id }) { line ->
            ElevatedCard {
                Row(Modifier.fillMaxWidth().padding(10.dp), verticalAlignment = Alignment.CenterVertically) {
                    Column(Modifier.weight(1f)) {
                        TextButton(onClick = { onOpen(line.product) }, contentPadding = PaddingValues(0.dp)) { Text(line.product.name) }
                        Text(line.product.pack, style = MaterialTheme.typography.bodySmall)
                    }
                    IconButton(onClick = { onQuantity(line.product, line.quantity - 1) },
                        modifier = Modifier.semantics { contentDescription = "Diminuir quantidade de ${line.product.name}" }) { Text("−") }
                    Text(line.quantity.toString())
                    IconButton(onClick = { onQuantity(line.product, line.quantity + 1) }, enabled = line.quantity < CART_MAX_QUANTITY,
                        modifier = Modifier.semantics { contentDescription = "Aumentar quantidade de ${line.product.name}" }) { Text("+") }
                }
            }
        }
        if (ids.isNotEmpty()) {
            item {
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    Text("Totais por filial", Modifier.weight(1f), style = MaterialTheme.typography.titleLarge)
                    TextButton(onClick = { attempt++ }, enabled = !loading) { Text("Atualizar totais") }
                }
                if (loading) LinearProgressIndicator(Modifier.fillMaxWidth())
                if (results.values.any { it == null }) Notice("Alguns produtos não puderam ser consultados. Atualize para tentar novamente.")
                if (results.values.any { it?.offline == true }) Notice("Offline · Alguns preços vieram de consultas salvas.")
            }
            items(quotes, key = { it.branch.branchId }) { quote ->
                ElevatedCard {
                    Row(Modifier.fillMaxWidth().padding(12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically) {
                        Column(Modifier.weight(1f)) {
                            Text(quote.branch.supermarketName, fontWeight = FontWeight.Bold)
                            Text(quote.branch.branchName, style = MaterialTheme.typography.bodySmall)
                            Text("${quote.priced} de ${lines.size} produtos com preço", style = MaterialTheme.typography.bodySmall)
                        }
                        Column(horizontalAlignment = Alignment.End) {
                            if (quote.priced > 0) {
                                Text(if (quote.priced == lines.size) "Total" else "Subtotal parcial", style = MaterialTheme.typography.labelMedium)
                                Text(money(quote.subtotal), fontWeight = FontWeight.Bold)
                            } else Text("Sem preço", style = MaterialTheme.typography.labelMedium)
                        }
                    }
                }
            }
        }
    }
}
