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
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable

internal const val CART_LIMIT = 25
internal const val CART_MAX_QUANTITY = 99

@Serializable
internal data class CartLine(val product: Product, val quantity: Int = 1)

internal data class CartItemQuote(val total: Long, val sourceUrl: String?)

internal data class CartQuote(val branch: Offer, val items: Map<Long, CartItemQuote>) {
    val subtotal: Long = items.values.sumOf { it.total }
    val priced: Int = items.size
}

internal fun Offer.isCurrent(at: Long = platformNow()): Boolean =
    status == "recent" && collectedAt?.let { it >= at - 172800 } == true &&
        (validUntil == null || validUntil >= at)

internal fun Offer.hasUnconfirmedStock(at: Long = platformNow()): Boolean =
    status == "unconfirmed_stock" && priceCents != null &&
        collectedAt?.let { it >= at - 172800 } == true && validUntil?.let { it >= at } == true

internal fun quoteCart(lines: List<CartLine>, offers: Map<Long, List<Offer>>, at: Long): List<CartQuote> {
    val byProduct = offers.mapValues { (_, prices) -> prices.associateBy(Offer::branchId) }
    return offers.values.flatten().distinctBy(Offer::branchId).mapNotNull { branch ->
        val values = lines.mapNotNull { line ->
            val offer = byProduct[line.product.id]?.get(branch.branchId)?.takeIf { it.isCurrent(at) }
            offer?.effectivePriceCents?.let { price ->
                line.product.id to CartItemQuote(price * line.quantity, offer.sourceUrl)
            }
        }.toMap()
        values.takeIf { it.isNotEmpty() }?.let { CartQuote(branch, it) }
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
    val synced by repo.updates.collectAsState()
    LaunchedEffect(repo, ids, attempt, synced) {
        loading = ids.isNotEmpty()
        try {
            results = repo.pricesFor(ids)
        } finally { loading = false }
    }
    val currentLines = lines.map { line ->
        results[line.product.id]?.data?.product?.let { line.copy(product = it) } ?: line
    }
    val quotes = quoteCart(lines, results.mapNotNull { (id, result) ->
        result?.let { id to it.data.items }
    }.toMap(), platformNow())
    val complete = quotes.filter { lines.isNotEmpty() && it.priced == lines.size }
    val best = complete.firstOrNull()
    val savings = best?.let { complete.last().subtotal - it.subtotal } ?: 0
    var expandedBranch by remember(repo, ids) { mutableStateOf<String?>(null) }
    val colors = MaterialTheme.colorScheme
    val uri = LocalUriHandler.current
    val snackbar = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()
    Box(Modifier.fillMaxSize()) {
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
            items(currentLines, key = { it.product.id }) { line ->
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
                        IconButton(onClick = {
                        scope.launch {
                            loading = true
                            try { repo.syncKnownProducts(force = true) } finally { attempt++ }
                        }
                    }, enabled = !loading,
                            modifier = Modifier.semantics { contentDescription = "Atualizar totais" }) {
                            AppIcon(Glyph.Refresh, tint = if (loading) colors.onSurfaceVariant else colors.primary)
                        }
                    }
                    if (loading) LinearProgressIndicator(Modifier.fillMaxWidth())
                    if (results.values.any { it == null }) Notice("Alguns produtos não puderam ser consultados. Atualize para tentar novamente.")
                    if (results.values.any { it?.offline == true }) Notice("Offline · Alguns preços vieram de consultas salvas.")
                }
                if (!loading && quotes.isEmpty() && results.values.none { it == null }) {
                    item { EmptyCard("Sem preços para esta lista", "Adicione outro produto ou atualize os totais mais tarde.") }
                }
                items(quotes, key = { it.branch.branchId }) { quote ->
                    val expanded = expandedBranch == quote.branch.branchId
                    CartBranchCard(quote, currentLines, quote == best, savings, expanded,
                        onDetails = { expandedBranch = if (expanded) null else quote.branch.branchId },
                        onSource = { url ->
                            try { uri.openUri(url) }
                            catch (_: IllegalArgumentException) {
                                scope.launch { snackbar.showSnackbar("Não foi possível abrir a fonte") }
                            }
                        }
                    )
                }
            }
        }
        SnackbarHost(snackbar, Modifier.align(Alignment.BottomCenter))
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
    quote: CartQuote, lines: List<CartLine>, lowest: Boolean, savings: Long, expanded: Boolean,
    onDetails: () -> Unit, onSource: (String) -> Unit
) {
    val colors = MaterialTheme.colorScheme
    val foreground = colors.onSurface
    Box(Modifier.fillMaxWidth()) {
        OutlinedCard(
            modifier = Modifier.fillMaxWidth().padding(top = if (lowest) 10.dp else 0.dp),
            shape = MaterialTheme.shapes.large,
            border = BorderStroke(1.dp, if (lowest) colors.primary.copy(alpha = .5f) else colors.outlineVariant),
            colors = CardDefaults.outlinedCardColors(containerColor = colors.surface, contentColor = foreground)
        ) {
            Column(Modifier.padding(start = 11.dp, end = 11.dp, top = if (lowest) 20.dp else 11.dp, bottom = 8.dp)) {
                Row(
                    Modifier.fillMaxWidth().then(if (!lowest)
                        Modifier.clickable(onClickLabel = if (expanded) "Ocultar detalhes" else "Ver detalhes", onClick = onDetails)
                    else Modifier),
                    horizontalArrangement = Arrangement.spacedBy(9.dp), verticalAlignment = Alignment.CenterVertically
                ) {
                    Box(Modifier.size(36.dp), contentAlignment = Alignment.Center) {
                        AppIcon(if (lowest) Glyph.Cart else Glyph.Store, Modifier.size(20.dp), foreground)
                    }
                    Column(Modifier.weight(1f)) {
                        Text(quote.branch.supermarketName, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                        Text(quote.branch.branchName, style = MaterialTheme.typography.bodySmall)
                        if (!lowest) Text("${quote.priced}/${lines.size} com preço", style = MaterialTheme.typography.labelSmall)
                    }
                    Column(horizontalAlignment = Alignment.End) {
                        Text(money(quote.subtotal), style = if (lowest) MaterialTheme.typography.titleLarge else MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold, color = if (lowest) colors.primary else foreground)
                        if (!lowest) Text(if (quote.priced == lines.size) "Total da lista" else "Subtotal parcial",
                            style = MaterialTheme.typography.labelSmall)
                    }
                }
                if (lowest) {
                    HorizontalDivider(Modifier.padding(top = 9.dp))
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(4.dp),
                        verticalAlignment = Alignment.CenterVertically) {
                        Column(Modifier.weight(1f)) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                AppIcon(Glyph.Check, Modifier.size(14.dp), foreground)
                                Spacer(Modifier.width(4.dp))
                                Text("${quote.priced} de ${lines.size} com preço", style = MaterialTheme.typography.labelMedium)
                            }
                            if (savings > 0) Text("Economize até ${money(savings)}", Modifier.padding(top = 4.dp),
                                style = MaterialTheme.typography.labelMedium)
                        }
                        TextButton(onClick = onDetails, contentPadding = PaddingValues(horizontal = 3.dp),
                            colors = ButtonDefaults.textButtonColors(contentColor = foreground)) {
                            Text(if (expanded) "Ocultar detalhes" else "Ver detalhes")
                            AppIcon(if (expanded) Glyph.Collapse else Glyph.Chevron, Modifier.size(16.dp), foreground)
                        }
                    }
                }
                if (expanded) {
                    HorizontalDivider(Modifier.padding(top = 5.dp))
                    lines.forEach { line ->
                        CartOfferItem(line, quote.items[line.product.id], onSource)
                    }
                }
            }
        }
        if (lowest) Surface(Modifier.padding(start = 12.dp), shape = MaterialTheme.shapes.extraLarge, color = colors.primary) {
            Row(Modifier.padding(horizontal = 10.dp, vertical = 4.dp), horizontalArrangement = Arrangement.spacedBy(4.dp),
                verticalAlignment = Alignment.CenterVertically) {
                AppIcon(Glyph.Star, Modifier.size(14.dp), colors.onPrimary)
                Text("Menor preço", style = MaterialTheme.typography.labelMedium, color = colors.onPrimary)
            }
        }
    }
}


@Composable private fun CartOfferItem(line: CartLine, offer: CartItemQuote?, onSource: (String) -> Unit) {
    val product = line.product
    val source = offer?.sourceUrl?.takeIf { it.startsWith("https://") || it.startsWith("http://") }
    Row(
        Modifier.fillMaxWidth().padding(vertical = 5.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        if (product.photoPath != null) ProductArtwork(product, Modifier.size(36.dp))
        Column(Modifier.weight(1f)) {
            Text("${line.quantity} × ${product.name}",
                modifier = if (source == null) Modifier else Modifier.clickable(onClickLabel = "Abrir fonte de ${product.name}") { onSource(source) },
                textDecoration = if (source != null) TextDecoration.Underline else null,
                style = MaterialTheme.typography.bodySmall)
            if (source != null) {
                TextButton(onClick = { onSource(source) }, contentPadding = PaddingValues(0.dp),
                    colors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.onSurface)) {
                    AppIcon(Glyph.External, Modifier.size(14.dp))
                    Spacer(Modifier.width(4.dp))
                    Text("Abrir fonte", style = MaterialTheme.typography.labelMedium)
                }
            }
        }
        Text(money(offer?.total), style = MaterialTheme.typography.labelMedium)
    }
}
