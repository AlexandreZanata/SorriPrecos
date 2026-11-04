package br.com.sorriprecos

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import br.com.sorriprecos.resources.*
import kotlinx.coroutines.launch
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource

internal data class QuickProduct(
    val name: String, val pack: String, val query: String, val image: DrawableResource
)

internal val quickProducts = listOf(
    QuickProduct("Arroz", "5 kg", "arroz 5kg", Res.drawable.home_arroz),
    QuickProduct("Feijão", "1 kg", "feijão 1kg", Res.drawable.home_feijao),
    QuickProduct("Açúcar", "5 kg", "açúcar 5kg", Res.drawable.home_acucar),
    QuickProduct("Café", "500 g", "café 500g", Res.drawable.home_cafe),
    QuickProduct("Leite", "1 L", "leite 1l", Res.drawable.home_leite),
    QuickProduct("Óleo", "900 ml", "óleo 900ml", Res.drawable.home_oleo),
    QuickProduct("Farinha de trigo", "1 kg", "farinha de trigo 1kg", Res.drawable.home_farinha),
    QuickProduct("Carne moída", "500 g", "carne moída 500g", Res.drawable.home_carne)
)

internal data class QuickCategory(val label: String, val icon: Glyph, val terms: List<String>)

internal val pharmacyCategory = QuickCategory("Farmácia", Glyph.Pharmacy, listOf("Protetor solar", "Hidratante", "Sabonete"))

internal val quickCategories = listOf(
    QuickCategory("Bebidas", Glyph.Bottle, listOf("Refrigerante", "Suco", "Água mineral")),
    QuickCategory("Padaria", Glyph.Bread, listOf("Pão", "Bolo", "Biscoito")),
    QuickCategory("Limpeza", Glyph.Spray, listOf("Detergente", "Sabão", "Desinfetante")),
    QuickCategory("Higiene", Glyph.Pump, listOf("Sabonete", "Shampoo", "Creme dental")),
    QuickCategory("Pet", Glyph.Paw, listOf("Ração", "Petisco", "Areia para gatos"))
)

private val homeMarketIds = listOf("sorriso", "casa-aurora", "delmoro", "machado", "sorrinorte", "atacadao")
private val groceryOrder = listOf(0, 1, 4, 5, 7, 2, 3, 6)
private val bannerImages = listOf(Res.drawable.home_banner_mercado, Res.drawable.home_banner_farmacia)
private val bannerTitles = listOf(
    "Compare preços nos mercados de Sorriso-MT",
    "Encontre produtos nas farmácias de Sorriso-MT"
)

@Composable internal fun HomeScreen(
    demo: Boolean,
    historical: Boolean,
    offline: Boolean,
    markets: List<Supermarket>,
    highlights: List<HomeHighlight>,
    favorites: Set<Long>,
    onSearch: (String) -> Unit,
    onExplore: () -> Unit,
    onCategory: (QuickCategory) -> Unit,
    onProduct: (Product) -> Unit,
    onFavorite: (Product) -> Unit,
    onSources: () -> Unit,
    onMarket: (Supermarket) -> Unit
) {
    var barcode by remember { mutableStateOf<String?>(null) }
    var more by remember { mutableStateOf(false) }
    val large = LocalDensity.current.fontScale > 1.15f
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val shortcutWidth = ((maxWidth - 60.dp) / 5).coerceAtLeast(if (large) 87.dp else 62.dp)
        val productWidth = ((maxWidth - 48.dp) / 3).coerceIn(if (large) 150.dp else 106.dp, 170.dp)
        LazyColumn(verticalArrangement = Arrangement.spacedBy(9.dp), contentPadding = PaddingValues(bottom = 24.dp)) {
            item { HomeHeader(onExplore, { barcode = "" }, onSources) }
            if (demo || historical || offline) item {
                Box(Modifier.padding(horizontal = 16.dp)) {
                    Notice(
                        when {
                            demo -> "Modo DEMO · Produtos e preços fictícios"
                            historical -> "Pesquisa histórica do Procon · Preços de referência"
                            else -> "Offline · Confira os preços na loja antes de comprar"
                        },
                        demo = demo, historical = historical
                    )
                }
            }
            item {
                LazyRow(contentPadding = PaddingValues(horizontal = 12.dp), horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                    item { HomeShortcut("Mercado", Glyph.Basket, Modifier.width(shortcutWidth), true, onExplore) }
                    item { HomeShortcut("Farmácia", Glyph.Pharmacy, Modifier.width(shortcutWidth)) { onCategory(pharmacyCategory) } }
                    items(listOf(quickCategories[4], quickCategories[2])) { category ->
                        HomeShortcut(category.label, category.icon, Modifier.width(shortcutWidth)) { onCategory(category) }
                    }
                    item { HomeShortcut("Mais", Glyph.Grid, Modifier.width(shortcutWidth)) { more = true } }
                }
            }
            item { HomeBanners(onExplore, { onCategory(pharmacyCategory) }) }
            if (highlights.isNotEmpty() && !historical) {
                item { HomeSectionTitle("Ofertas em destaque", "Explorar", onExplore, Glyph.Flame) }
                item {
                    LazyRow(contentPadding = PaddingValues(horizontal = 12.dp, vertical = 2.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        items(highlights, key = { it.product.id }) { highlight ->
                            HighlightCard(highlight, highlight.product.id in favorites, Modifier.width(productWidth),
                                onOpen = { onProduct(highlight.product) },
                                onFavorite = { onFavorite(highlight.product) })
                        }
                    }
                }
            }
            item { HomeSectionTitle("Compre por categoria", "Ver todas", { more = true }) }
            item {
                LazyRow(contentPadding = PaddingValues(horizontal = 12.dp), horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                    items(groceryOrder.map(quickProducts::get), key = { it.query }) { product ->
                        Surface(onClick = { onSearch(product.query) }, modifier = Modifier.width(shortcutWidth),
                            shape = RoundedCornerShape(14.dp), color = MaterialTheme.colorScheme.surfaceVariant) {
                            Column(Modifier.padding(horizontal = 3.dp, vertical = 6.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                                Image(painterResource(product.image), contentDescription = null,
                                    modifier = Modifier.fillMaxWidth().height(45.dp), contentScale = ContentScale.Fit)
                                Text(product.name, fontSize = 11.sp, lineHeight = 13.sp, textAlign = TextAlign.Center,
                                    fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
                            }
                        }
                    }
                }
            }
            val cityMarkets = if (demo) markets.take(4) else homeMarketIds.mapNotNull { id -> markets.find { it.id == id } }
            if (cityMarkets.isNotEmpty()) {
                item { HomeSectionTitle(if (demo) "Lojas de demonstração" else "Mercados da sua cidade", "Ver todos", onSources, Glyph.Store) }
                item {
                    LazyRow(contentPadding = PaddingValues(horizontal = 12.dp, vertical = 3.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        items(cityMarkets, key = { it.id }) { market ->
                            HomeMarket(market, onClick = { onMarket(market) })
                        }
                    }
                }
            }
        }
    }
    if (more) AlertDialog(
        onDismissRequest = { more = false },
        title = { Text("Compre por categoria") },
        text = {
            LazyColumn {
                items(listOf(pharmacyCategory) + quickCategories, key = { it.label }) { category ->
                    ListItem(
                        headlineContent = { Text(category.label) },
                        leadingContent = { AppIcon(category.icon, tint = MaterialTheme.colorScheme.primary) },
                        modifier = Modifier.clickable { more = false; onCategory(category) }
                    )
                }
                items(groceryOrder.map(quickProducts::get), key = { it.query }) { product ->
                    ListItem(
                        headlineContent = { Text(product.name) },
                        supportingContent = { Text(product.pack) },
                        leadingContent = { Image(painterResource(product.image), null, Modifier.size(24.dp)) },
                        modifier = Modifier.clickable { more = false; onSearch(product.query) }
                    )
                }
            }
        },
        confirmButton = { TextButton(onClick = { more = false }) { Text("Fechar") } }
    )
    barcode?.let { code ->
        AlertDialog(
            onDismissRequest = { barcode = null },
            title = { Text("Buscar por código de barras") },
            text = {
                OutlinedTextField(value = code, onValueChange = { barcode = it.filter { c -> c in '0'..'9' }.take(14) },
                    label = { Text("Digite o código da embalagem") }, singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number))
            },
            confirmButton = {
                TextButton(enabled = code.length in setOf(8, 12, 13, 14), onClick = { barcode = null; onSearch(code) }) { Text("Buscar") }
            },
            dismissButton = { TextButton(onClick = { barcode = null }) { Text("Cancelar") } }
        )
    }
}

@Composable private fun HomeHeader(onSearch: () -> Unit, onBarcode: () -> Unit, onSources: () -> Unit) {
    val white = Color.White
    val lime = Color(0xFF9AF755)
    Column(
        Modifier.fillMaxWidth().clip(RoundedCornerShape(bottomStart = 24.dp, bottomEnd = 24.dp))
            .background(Brush.linearGradient(listOf(Color(0xFF087320), Color(0xFF064627), Color(0xFF07811E))))
            .padding(start = 14.dp, end = 14.dp, bottom = 13.dp)
    ) {
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            AppIcon(Glyph.Pin, Modifier.size(21.dp), white)
            Text("Sorriso-MT", Modifier.weight(1f).padding(start = 5.dp), color = white, fontWeight = FontWeight.Bold)
            IconButton(onClick = onSources, modifier = Modifier.size(42.dp).semantics { contentDescription = "Atualizações das fontes" }) {
                AppIcon(Glyph.Notifications, tint = white)
            }
        }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            AppIcon(Glyph.Cart, Modifier.size(45.dp), white)
            Column {
                Text(buildAnnotatedString {
                    withStyle(SpanStyle(color = white)) { append("Sorri") }
                    withStyle(SpanStyle(color = lime)) { append("Preços") }
                }, fontWeight = FontWeight.Black, fontStyle = FontStyle.Italic, fontSize = 30.sp, lineHeight = 32.sp)
                Text("Compare preços e economize na sua cidade", color = white, fontSize = 11.sp, lineHeight = 14.sp)
            }
        }
        Spacer(Modifier.height(12.dp))
        Surface(shape = RoundedCornerShape(18.dp), color = MaterialTheme.colorScheme.surface,
            shadowElevation = 2.dp, modifier = Modifier.fillMaxWidth()) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Row(Modifier.weight(1f).heightIn(min = 48.dp).clickable(onClick = onSearch).padding(horizontal = 12.dp),
                    verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    AppIcon(Glyph.Search, Modifier.size(23.dp))
                    Text("Buscar produtos (ex.: arroz, leite…)", fontSize = 12.sp, lineHeight = 15.sp,
                        color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                Box(Modifier.width(1.dp).height(30.dp).background(MaterialTheme.colorScheme.outlineVariant))
                IconButton(onClick = onBarcode, modifier = Modifier.semantics { contentDescription = "Digitar código de barras" }) {
                    AppIcon(Glyph.Barcode, Modifier.size(22.dp))
                }
            }
        }
    }
}

@Composable private fun HomeShortcut(
    label: String, icon: Glyph, modifier: Modifier, emphasized: Boolean = false, onClick: () -> Unit
) {
    val colors = MaterialTheme.colorScheme
    Surface(onClick = onClick, modifier = modifier, shape = RoundedCornerShape(15.dp),
        color = if (emphasized) colors.primary else colors.surfaceVariant,
        contentColor = if (emphasized) colors.onPrimary else colors.onSurface) {
        Column(Modifier.heightIn(min = 66.dp).padding(horizontal = 2.dp, vertical = 7.dp),
            horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.Center) {
            AppIcon(icon, Modifier.size(26.dp), if (emphasized) colors.onPrimary else colors.primary)
            Text(label, Modifier.padding(top = 3.dp), fontSize = 11.sp, lineHeight = 13.sp, fontWeight = FontWeight.Bold, textAlign = TextAlign.Center)
        }
    }
}

@Composable private fun HomeBanners(onMarket: () -> Unit, onPharmacy: () -> Unit) {
    val pager = rememberPagerState { bannerTitles.size }
    val scope = rememberCoroutineScope()
    val shape = RoundedCornerShape(19.dp)
    val height = if (LocalDensity.current.fontScale > 1.15f) 154.dp else 120.dp
    Surface(modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp), shape = shape,
        color = Color(0xFF064627), shadowElevation = 3.dp) {
        Box {
            HorizontalPager(state = pager, modifier = Modifier.fillMaxWidth()) { page ->
                Box(Modifier.fillMaxWidth().heightIn(min = height).clickable(onClick = if (page == 0) onMarket else onPharmacy)) {
                    Image(painterResource(bannerImages[page]), null, Modifier.matchParentSize(), contentScale = ContentScale.Crop)
                    Box(Modifier.matchParentSize().background(Brush.horizontalGradient(
                        0f to Color(0xE8054727), .48f to Color(0xC3054727), .68f to Color.Transparent)))
                    Column(Modifier.fillMaxWidth(.57f).padding(start = 17.dp, top = 15.dp, bottom = 24.dp)) {
                        Text(bannerTitles[page], color = Color.White, fontWeight = FontWeight.ExtraBold,
                            fontSize = 20.sp, lineHeight = 22.sp)
                        Spacer(Modifier.height(6.dp))
                        Box(Modifier.width(64.dp).height(3.dp).clip(CircleShape).background(Color(0xFF9AF755)))
                    }
                }
            }
            Row(Modifier.align(Alignment.BottomCenter)) {
                bannerTitles.forEachIndexed { index, title ->
                    Box(Modifier.size(width = 26.dp, height = 28.dp)
                        .semantics { contentDescription = "Mostrar banner: $title" }
                        .clickable { scope.launch { pager.animateScrollToPage(index) } }, contentAlignment = Alignment.Center) {
                        Box(Modifier.size(7.dp).clip(CircleShape).background(Color.White.copy(alpha = if (index == pager.currentPage) 1f else .5f)))
                    }
                }
            }
        }
    }
}

@Composable private fun HomeSectionTitle(title: String, action: String, onAction: () -> Unit, icon: Glyph? = null) {
    Row(Modifier.fillMaxWidth().padding(horizontal = 12.dp), verticalAlignment = Alignment.CenterVertically) {
        if (icon != null) {
            AppIcon(icon, Modifier.size(21.dp), if (icon == Glyph.Flame) Color(0xFFEC222B) else MaterialTheme.colorScheme.primary)
            Spacer(Modifier.width(5.dp))
        }
        Text(title, Modifier.weight(1f), fontWeight = FontWeight.ExtraBold, fontSize = 18.sp, lineHeight = 20.sp)
        TextButton(onClick = onAction, modifier = Modifier.height(42.dp), contentPadding = PaddingValues(start = 6.dp)) {
            Text(action, fontSize = 12.sp, fontWeight = FontWeight.Bold)
            AppIcon(Glyph.Chevron, Modifier.size(17.dp), MaterialTheme.colorScheme.primary)
        }
    }
}

@Composable private fun HighlightCard(
    highlight: HomeHighlight, favorite: Boolean, modifier: Modifier, onOpen: () -> Unit, onFavorite: () -> Unit
) {
    val (product, offer) = highlight
    val colors = MaterialTheme.colorScheme
    val discount = highlight.discountPercent
    Surface(onClick = onOpen, modifier = modifier, color = colors.surface, shape = RoundedCornerShape(16.dp),
        border = BorderStroke(1.dp, colors.outlineVariant), shadowElevation = 2.dp) {
        Column(Modifier.padding(8.dp)) {
            Box(Modifier.fillMaxWidth().height(76.dp)) {
                ProductArtwork(product, Modifier.fillMaxSize().padding(top = 12.dp))
                if (discount != null) Surface(Modifier.align(Alignment.TopStart), shape = CircleShape,
                    color = Color(0xFFC90C24), contentColor = Color.White) {
                    Text("−$discount%", Modifier.padding(horizontal = 6.dp, vertical = 2.dp), fontSize = 10.sp, fontWeight = FontWeight.Bold)
                }
                IconButton(onClick = onFavorite, modifier = Modifier.align(Alignment.TopEnd).size(42.dp)
                    .semantics { contentDescription = if (favorite) "Remover favorito" else "Favoritar ${product.name}" }) {
                    AppIcon(Glyph.Heart, Modifier.size(20.dp), if (favorite) colors.primary else colors.onSurface, filled = favorite)
                }
            }
            Text(product.name, Modifier.padding(top = 4.dp).heightIn(min = 30.dp), fontSize = 12.sp, lineHeight = 14.sp,
                fontWeight = FontWeight.Bold, maxLines = 2, overflow = TextOverflow.Ellipsis)
            Text(product.pack, fontSize = 10.sp, color = colors.onSurfaceVariant)
            Surface(color = colors.primaryContainer, shape = RoundedCornerShape(9.dp),
                modifier = Modifier.fillMaxWidth().padding(top = 5.dp)) {
                Column(Modifier.padding(horizontal = 6.dp, vertical = 4.dp)) {
                    Text(if (discount != null) money(offer.priceCents) else "Preço atual",
                        fontSize = 10.sp, lineHeight = 12.sp,
                        textDecoration = if (discount != null) TextDecoration.LineThrough else null,
                        color = colors.onSurfaceVariant)
                    Text(money(offer.effectivePriceCents), fontSize = 16.sp, lineHeight = 18.sp,
                        fontWeight = FontWeight.Black, color = colors.onPrimaryContainer)
                }
            }
            Row(Modifier.padding(top = 6.dp), verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                AppIcon(Glyph.Store, Modifier.size(13.dp), colors.primary)
                Text(offer.supermarketName, Modifier.weight(1f), fontSize = 10.sp, lineHeight = 12.sp,
                    maxLines = 2, overflow = TextOverflow.Ellipsis)
                AppIcon(Glyph.Chevron, Modifier.size(12.dp))
            }
        }
    }
}

@Composable private fun HomeMarket(market: Supermarket, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val badgeColor = when (market.id) {
        "sorriso" -> Color(0xFFC90C24)
        "casa-aurora" -> Color(0xFFB84E08)
        "delmoro" -> Color(0xFF144DBA)
        else -> Color(0xFF087D13)
    }
    Surface(onClick = onClick, shape = RoundedCornerShape(14.dp), color = colors.surface, shadowElevation = 2.dp,
        border = BorderStroke(1.dp, colors.outlineVariant)) {
        Row(Modifier.width(117.dp).heightIn(min = 56.dp).padding(7.dp), verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            Box(Modifier.size(28.dp).clip(CircleShape).background(badgeColor), contentAlignment = Alignment.Center) {
                AppIcon(Glyph.Cart, Modifier.size(17.dp), Color.White)
            }
            Text(market.name, fontSize = 11.sp, lineHeight = 13.sp, fontWeight = FontWeight.Bold, maxLines = 2,
                overflow = TextOverflow.Ellipsis)
        }
    }
}
