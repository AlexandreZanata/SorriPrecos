package br.com.sorriprecos

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
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

internal data class CartQuote(val branch: Offer, val lineTotals: Map<Long, Long>) {
    val subtotal: Long = lineTotals.values.sum()
    val priced: Int = lineTotals.size
}

internal fun Offer.isCurrent(at: Long = platformNow()): Boolean =
    status == "recent" && collectedAt?.let { it >= at - 172800 } == true &&
        (validUntil == null || validUntil >= at)

internal fun quoteCart(lines: List<CartLine>, offers: Map<Long, List<Offer>>, at: Long): List<CartQuote> {
    val byProduct = offers.mapValues { (_, prices) -> prices.associateBy(Offer::branchId) }
    return offers.values.flatten().distinctBy(Offer::branchId).map { branch ->
        val values = lines.mapNotNull { line ->
            byProduct[line.product.id]?.get(branch.branchId)?.takeIf { it.isCurrent(at) }
                ?.effectivePriceCents?.let { line.product.id to it * line.quantity }
        }.toMap()
        CartQuote(branch, values)
    }.sortedWith(compareByDescending<CartQuote> { it.priced }.thenBy { it.subtotal }.thenBy { it.branch.supermarketName })
}

@Composable internal fun CartScreen(
    repo: Repository, lines: List<CartLine>, onQuantity: (Product, Int) -> Unit,
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
    val complete = quotes.filter { lines.isNotEmpty() && it.priced == lines.size }
    val best = complete.firstOrNull()
    val savings = best?.let { complete.last().subtotal - it.subtotal } ?: 0
    var expandedBranch by remember(repo, ids) { mutableStateOf<String?>(null) }
    val colors = MaterialTheme.colorScheme
    LazyColumn(
        contentPadding = PaddingValues(start = 16.dp, top = 6.dp, end = 16.dp, bottom = 22.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        item {
            Text("Compare o custo da sua lista por filial", style = MaterialTheme.typography.bodyMedium, color = colors.onSurfaceVariant)
            Text("Sem frete · Valores dos produtos", style = MaterialTheme.typography.bodySmall, color = colors.onSurfaceVariant)
        }
        item {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically) {
                Text("Sua lista", Modifier.weight(1f), style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
                Surface(color = colors.primaryContainer, shape = MaterialTheme.shapes.extraLarge) {
                    Text(if (lines.size == 1) "1 produto" else "${lines.size} produtos",
                        Modifier.padding(horizontal = 12.dp, vertical = 6.dp),
                        style = MaterialTheme.typography.labelMedium, color = colors.onPrimaryContainer)
                }
            }
        }
        if (lines.isEmpty()) item { EmptyCard("Carrinho vazio", "Abra um produto para adicioná-lo à lista.") }
        items(lines, key = { it.product.id }) { line ->
            CartProductCard(line, onOpen = { onOpen(line.product) },
                onQuantity = { onQuantity(line.product, it) })
        }
        item {
            TextButton(onClick = onSearch) {
                AppIcon(Glyph.Add, Modifier.size(18.dp), colors.primary)
                Spacer(Modifier.width(5.dp))
                Text("Buscar produtos")
            }
        }
        if (ids.isNotEmpty()) {
            item {
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    Column(Modifier.weight(1f)) {
                        Text("Totais por filial", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
                        Text("Listas completas primeiro", style = MaterialTheme.typography.bodySmall, color = colors.onSurfaceVariant)
                    }
                    IconButton(onClick = { attempt++ }, enabled = !loading,
                        modifier = Modifier.semantics { contentDescription = "Atualizar totais" }) {
                        AppIcon(Glyph.Refresh, tint = if (loading) colors.onSurfaceVariant else colors.primary)
                    }
                }
                if (loading) LinearProgressIndicator(Modifier.fillMaxWidth())
                if (results.values.any { it == null }) Notice("Alguns produtos não puderam ser consultados. Atualize para tentar novamente.")
                if (results.values.any { it?.offline == true }) Notice("Offline · Alguns preços vieram de consultas salvas.")
            }
            items(quotes, key = { it.branch.branchId }) { quote ->
                val expanded = expandedBranch == quote.branch.branchId
                CartBranchCard(quote, lines, quote == best, savings, expanded) {
                    expandedBranch = if (expanded) null else quote.branch.branchId
                }
            }
        }
    }
}

@Composable private fun CartProductCard(line: CartLine, onOpen: () -> Unit, onQuantity: (Int) -> Unit) {
    val colors = MaterialTheme.colorScheme
    val product = line.product
    OutlinedCard(shape = MaterialTheme.shapes.large, colors = CardDefaults.outlinedCardColors(containerColor = colors.surface)) {
        Row(Modifier.fillMaxWidth().padding(11.dp), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Surface(shape = MaterialTheme.shapes.medium, color = colors.surfaceVariant,
                modifier = Modifier.size(width = 62.dp, height = 76.dp).clickable(onClick = onOpen)) {
                Box(contentAlignment = Alignment.Center) {
                    if (product.photoPath == "/v1/products/${product.id}/photo") {
                        ProductArtwork(product, Modifier.fillMaxSize().padding(5.dp))
                    } else {
                        AppIcon(Glyph.Package, tint = colors.onSurfaceVariant)
                    }
                }
            }
            Column(Modifier.weight(1f)) {
                Column(Modifier.fillMaxWidth().clickable(onClick = onOpen)) {
                    Text(product.name, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                    Text(product.pack, style = MaterialTheme.typography.bodySmall, color = colors.onSurfaceVariant)
                }
                Row(Modifier.fillMaxWidth().padding(top = 5.dp), horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalAlignment = Alignment.CenterVertically) {
                    Text("Quantidade", Modifier.weight(1f), style = MaterialTheme.typography.bodySmall, color = colors.onSurfaceVariant)
                    Surface(shape = MaterialTheme.shapes.medium, color = colors.surfaceVariant,
                        border = BorderStroke(1.dp, colors.outlineVariant)) {
                        Row(Modifier.height(IntrinsicSize.Min), verticalAlignment = Alignment.CenterVertically) {
                            IconButton(onClick = { onQuantity(line.quantity - 1) },
                                modifier = Modifier.semantics { contentDescription = "Diminuir quantidade de ${product.name}" }) {
                                AppIcon(Glyph.Remove, Modifier.size(18.dp))
                            }
                            VerticalDivider(Modifier.height(18.dp))
                            Text(line.quantity.toString(), Modifier.padding(horizontal = 8.dp), fontWeight = FontWeight.Bold)
                            VerticalDivider(Modifier.height(18.dp))
                            IconButton(onClick = { onQuantity(line.quantity + 1) }, enabled = line.quantity < CART_MAX_QUANTITY,
                                modifier = Modifier.semantics { contentDescription = "Aumentar quantidade de ${product.name}" }) {
                                AppIcon(Glyph.Add, Modifier.size(18.dp),
                                    if (line.quantity == CART_MAX_QUANTITY) colors.onSurfaceVariant else colors.onSurface)
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable private fun CartBranchCard(
    quote: CartQuote, lines: List<CartLine>, lowest: Boolean, savings: Long, expanded: Boolean, onDetails: () -> Unit
) {
    val colors = MaterialTheme.colorScheme
    val container = if (lowest) colors.primaryContainer else colors.surface
    val foreground = if (lowest) colors.onPrimaryContainer else colors.onSurface
    Box(Modifier.fillMaxWidth()) {
        OutlinedCard(
            modifier = Modifier.fillMaxWidth().padding(top = if (lowest) 10.dp else 0.dp),
            shape = MaterialTheme.shapes.large,
            border = BorderStroke(1.dp, if (lowest) colors.primary.copy(alpha = .5f) else colors.outlineVariant),
            colors = CardDefaults.outlinedCardColors(containerColor = container, contentColor = foreground)
        ) {
            Column(Modifier.padding(start = 11.dp, end = 11.dp, top = if (lowest) 20.dp else 11.dp, bottom = 8.dp)) {
                Row(
                    Modifier.fillMaxWidth().then(if (!lowest && quote.priced > 0)
                        Modifier.clickable(onClickLabel = if (expanded) "Ocultar detalhes" else "Ver detalhes", onClick = onDetails)
                    else Modifier),
                    horizontalArrangement = Arrangement.spacedBy(9.dp), verticalAlignment = Alignment.CenterVertically
                ) {
                    if (quote.priced > 0) {
                        Surface(color = if (lowest) colors.primary.copy(alpha = .10f) else colors.surfaceVariant,
                            shape = MaterialTheme.shapes.medium) {
                            Box(Modifier.size(36.dp), contentAlignment = Alignment.Center) {
                                AppIcon(if (lowest) Glyph.Cart else Glyph.Store, Modifier.size(20.dp),
                                    if (lowest) colors.primary else colors.onSurfaceVariant)
                            }
                        }
                    }
                    Column(Modifier.weight(1f)) {
                        Text(quote.branch.supermarketName, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                        Text(if (quote.priced == 0) "Nenhum preço encontrado para a lista" else quote.branch.branchName,
                            style = MaterialTheme.typography.bodySmall, color = colors.onSurfaceVariant)
                        if (!lowest && quote.priced > 0) Text("${quote.priced}/${lines.size} com preço",
                            style = MaterialTheme.typography.labelSmall, color = colors.onSurfaceVariant)
                    }
                    if (quote.priced > 0) {
                        Column(horizontalAlignment = Alignment.End) {
                            Text(money(quote.subtotal), style = if (lowest) MaterialTheme.typography.titleLarge else MaterialTheme.typography.titleMedium,
                                fontWeight = FontWeight.Bold, color = if (lowest) colors.onPrimaryContainer else colors.onSurface)
                            if (!lowest) Text(if (quote.priced == lines.size) "Total da lista" else "Subtotal parcial",
                                style = MaterialTheme.typography.labelSmall, color = colors.onSurfaceVariant)
                        }
                    } else Surface(color = colors.surfaceVariant, shape = MaterialTheme.shapes.large) {
                        Text("Sem preço", Modifier.padding(horizontal = 8.dp, vertical = 5.dp),
                            style = MaterialTheme.typography.labelSmall, color = colors.onSurfaceVariant)
                    }
                }
                if (lowest) {
                    HorizontalDivider(Modifier.padding(top = 9.dp))
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(4.dp),
                        verticalAlignment = Alignment.CenterVertically) {
                        Column(Modifier.weight(1f)) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                AppIcon(Glyph.Check, Modifier.size(14.dp), colors.primary)
                                Spacer(Modifier.width(4.dp))
                                Text("${quote.priced} de ${lines.size} com preço", style = MaterialTheme.typography.labelMedium)
                            }
                            if (savings > 0) Surface(color = colors.primary.copy(alpha = .10f), shape = MaterialTheme.shapes.extraLarge,
                                modifier = Modifier.padding(top = 4.dp)) {
                                Text("Economize até ${money(savings)}", Modifier.padding(horizontal = 8.dp, vertical = 3.dp),
                                    style = MaterialTheme.typography.labelMedium)
                            }
                        }
                        TextButton(onClick = onDetails, contentPadding = PaddingValues(horizontal = 3.dp)) {
                            Text(if (expanded) "Ocultar detalhes" else "Ver detalhes")
                            AppIcon(if (expanded) Glyph.Collapse else Glyph.Chevron, Modifier.size(16.dp), colors.primary)
                        }
                    }
                }
                if (expanded) {
                    HorizontalDivider(Modifier.padding(top = 5.dp))
                    lines.forEach { line ->
                        Row(Modifier.fillMaxWidth().padding(vertical = 5.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            Text("${line.quantity} × ${line.product.name}", Modifier.weight(1f), style = MaterialTheme.typography.bodySmall)
                            Text(money(quote.lineTotals[line.product.id]), style = MaterialTheme.typography.labelMedium)
                        }
                    }
                }
            }
        }
        if (lowest) Surface(Modifier.padding(start = 12.dp), shape = MaterialTheme.shapes.extraLarge, color = colors.primary) {
            Row(Modifier.padding(horizontal = 10.dp, vertical = 4.dp), horizontalArrangement = Arrangement.spacedBy(4.dp),
                verticalAlignment = Alignment.CenterVertically) {
                AppIcon(Glyph.Star, Modifier.size(14.dp), colors.onPrimary)
                Text("Menor total", style = MaterialTheme.typography.labelMedium, color = colors.onPrimary)
            }
        }
    }
}
