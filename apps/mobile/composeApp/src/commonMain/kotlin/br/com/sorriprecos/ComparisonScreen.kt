package br.com.sorriprecos

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch

@Composable internal fun ComparisonScreen(repo: Repository, selected: Product, favorite: Boolean, onFavorite: () -> Unit, refresh: Int, onRefresh: () -> Unit, cartQuantity: Int, onAdd: (() -> Unit)?, onCart: () -> Unit) {
    var loaded by remember(selected.id, repo) { mutableStateOf<Loaded<Comparison>?>(null) }
    var loading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    var details by remember { mutableStateOf(false) }
    var showAll by remember(selected.id) { mutableStateOf(false) }
    val clipboard = LocalClipboardManager.current
    val uri = LocalUriHandler.current
    val snack = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()
    val refreshOffer: () -> Unit = {
        scope.launch {
            repo.syncKnownProducts(force = true, onlyId = selected.id)
            onRefresh()
        }
    }
    fun openSource(url: String) { try { uri.openUri(url) } catch (_: Exception) { scope.launch { snack.showSnackbar("Não foi possível abrir a fonte") } } }
    val synced by repo.updates.collectAsState()
    LaunchedEffect(selected.id, repo, refresh, synced) {
        loading = true; error = null
        try { loaded = repo.prices(selected.id, selected.isHistorical) } catch (e: CancellationException) { throw e }
        catch (e: Exception) { error = e.message } finally { loading = false }
    }
    val product = loaded?.data?.product ?: selected
    val offers = loaded?.data?.items.orEmpty().filter { offer ->
        if (product.isHistorical) offer.collectedAt != null && offer.priceCents != null
        else offer.isCurrent() && offer.effectivePriceCents != null
    }
    val best = offers.filter { it.isCurrent() }.mapNotNull { it.effectivePriceCents }.minOrNull()
    val historical = product.isHistorical
    Box(Modifier.fillMaxSize()) {
        LazyColumn(contentPadding = PaddingValues(18.dp, 12.dp, 18.dp, 24.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            if (loaded?.offline == true) item { Notice("Offline · Comparação salva. Preços podem ter mudado.") }
            item {
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    ProductArtwork(product)
                    Column(Modifier.weight(1f)) {
                        Text(product.name, fontSize = 20.sp, lineHeight = 24.sp, fontWeight = FontWeight.Bold)
                        Text("Marca: ${product.brand}", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 7.dp))
                        Text("Embalagem: ${product.pack}", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                    TextButton(onClick = onFavorite, contentPadding = PaddingValues(horizontal = 0.dp)) { AppIcon(Glyph.Heart, Modifier.size(16.dp), MaterialTheme.colorScheme.primary, filled = favorite); Spacer(Modifier.width(5.dp)); Text(if (favorite) "Salvo nos favoritos" else "Salvar nos favoritos", fontSize = 12.sp) }
                    TextButton(onClick = refreshOffer) { AppIcon(Glyph.Refresh, Modifier.size(15.dp), MaterialTheme.colorScheme.primary); Spacer(Modifier.width(5.dp)); Text("Atualizar", fontSize = 12.sp) }
                }
                if (onAdd != null && (offers.isNotEmpty() || cartQuantity > 0)) {
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        if (offers.isNotEmpty()) Button(onClick = onAdd, enabled = cartQuantity < CART_MAX_QUANTITY) {
                            AppIcon(Glyph.Cart, Modifier.size(17.dp), MaterialTheme.colorScheme.onPrimary)
                            Spacer(Modifier.width(6.dp))
                            Text("Adicionar ao carrinho")
                        }
                        if (cartQuantity > 0) TextButton(onClick = onCart) { Text("Ver carrinho ($cartQuantity)") }
                    }
                }
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    FilterChip(selected = !details, onClick = { details = false }, label = { Text("Preços", fontSize = 12.sp) })
                    FilterChip(selected = details, onClick = { details = true }, label = { Text("Detalhes", fontSize = 12.sp) })
                }
            }
            if (loading) item { LinearProgressIndicator(Modifier.fillMaxWidth()) }
            if (error != null) item { ErrorCard(error!!, refreshOffer) }
            if (details) {
                item { EmptyCard("A mesma embalagem, sempre", "${product.name}\nMarca: ${product.brand}\nConteúdo: ${product.pack}\nApresentação: ${product.presentation}\nGTIN: ${product.gtin ?: "Não informado"}\n${if(product.isDemo)"Produto sintético de demonstração." else "Correspondência conservadora por GTIN ou nome, marca e embalagem."}") }
                item { Text("Referências de origem", fontSize = 18.sp, fontWeight = FontWeight.Bold) }
                items(offers.filter { it.sourceUrl != null }, key = { it.branchId }) { o ->
                    TextButton(onClick = { openSource(o.sourceUrl!!) }) { Text("${o.supermarketName} · ${o.branchName}", Modifier.weight(1f, fill = false)); Spacer(Modifier.width(5.dp)); AppIcon(Glyph.External, Modifier.size(16.dp), MaterialTheme.colorScheme.primary) }
                }
                if (offers.none { it.sourceUrl != null }) item { Text(if(product.isDemo) "Dados fictícios. Não há site de oferta real." else "Sem URLs de produtos informadas pelas fontes.", fontSize = 13.sp) }
            } else {
                item {
                    Text("Preços por mercado", fontSize = 19.sp, fontWeight = FontWeight.Bold)
                    Text(if (historical) "Registros históricos · não são ofertas atuais" else "Do menor para o maior preço conhecido", fontSize = 12.sp, lineHeight = 17.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 4.dp))
                    if (historical) Text("Toque na filial para ver endereço e origem.", fontSize = 10.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 3.dp))
                }
                items(if(showAll) offers else offers.take(5), key = { it.branchId }) { o ->
                    val recent = o.isCurrent()
                    OfferRow(o, recent, best != null && recent && o.effectivePriceCents == best, ::openSource) {
                        clipboard.setText(AnnotatedString(o.address))
                        scope.launch { snack.showSnackbar("Endereço copiado") }
                    }
                }
                if (offers.size > 5) item { TextButton(onClick = { showAll = !showAll }, modifier = Modifier.fillMaxWidth()) { Text(if(showAll) "Mostrar menos filiais" else "Ver todas as ${offers.size} filiais") } }
                if (loaded != null && offers.isEmpty()) item { EmptyCard("Preço não disponível", "Nenhuma filial tem preço confirmado para este produto no momento.") }
                item {
                    Spacer(Modifier.height(4.dp))
                    Text("Histórico de preços", fontSize = 18.sp, fontWeight = FontWeight.Bold)
                    Surface(color = MaterialTheme.colorScheme.surfaceVariant, shape = RoundedCornerShape(15.dp), modifier = Modifier.fillMaxWidth().padding(top = 10.dp)) {
                        Row(Modifier.padding(18.dp), verticalAlignment = Alignment.CenterVertically) {
                            AppIcon(Glyph.Clock, Modifier.size(28.dp), MaterialTheme.colorScheme.onSurfaceVariant); Spacer(Modifier.width(14.dp))
                            Text("Ainda não há uma série diária disponível para este produto. As datas dos registros estão em cada filial.", fontSize = 12.sp, lineHeight = 18.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                }
            }
        }
        SnackbarHost(snack, Modifier.align(Alignment.BottomCenter))
    }
}

@Composable private fun OfferRow(
    offer: Offer, recent: Boolean, lowest: Boolean, onSource: (String) -> Unit, onCopyAddress: () -> Unit
) {
    var expanded by remember(offer.branchId) { mutableStateOf(false) }
    val colors = MaterialTheme.colorScheme
    val value = if (recent) offer.effectivePriceCents else if (offer.status in listOf("stale", "recent")) offer.priceCents else null
    val shape = RoundedCornerShape(15.dp)
    Box(Modifier.fillMaxWidth()) {
        Surface(
            modifier = Modifier.fillMaxWidth().padding(top = if (lowest) 8.dp else 0.dp),
            shape = shape, color = if (lowest) colors.primaryContainer else colors.surface,
            border = BorderStroke(1.dp, if (lowest) colors.primary.copy(alpha = .3f) else colors.outlineVariant),
            shadowElevation = if (lowest) 0.dp else 1.dp
        ) {
            Column {
                Row(
                    Modifier.fillMaxWidth().clickable { expanded = !expanded }
                        .padding(start = 10.dp, end = 10.dp, top = if (lowest) 19.dp else 9.dp, bottom = 9.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically
                ) {
                    Box(Modifier.size(if (lowest) 42.dp else 34.dp)
                        .background(if (lowest) colors.primary.copy(alpha = .12f) else colors.surfaceVariant, RoundedCornerShape(10.dp)),
                        contentAlignment = Alignment.Center) {
                        AppIcon(Glyph.Cart, Modifier.size(22.dp), if (lowest) colors.primary else colors.onSurfaceVariant)
                    }
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                        Text(offer.supermarketName, fontSize = 14.sp, lineHeight = 17.sp, fontWeight = FontWeight.Bold)
                        if (recent || expanded || offer.status == "stale") Row(verticalAlignment = Alignment.CenterVertically) {
                            AppIcon(Glyph.Pin, Modifier.size(12.dp), colors.onSurfaceVariant)
                            Spacer(Modifier.width(4.dp))
                            Text(offer.branchName, fontSize = 11.sp, color = colors.onSurfaceVariant)
                        }
                        val status = when {
                            recent -> age(offer.collectedAt).replaceFirst("Há ", "Verificado há ")
                            offer.status == "unavailable" -> "Indisponibilidade informada"
                            offer.status == "stale" || offer.status == "recent" ->
                                "Histórico · ${offer.collectedAt?.let { formatDate(it).substringBefore(" às") } ?: "Sem data"}"
                            else -> "Sem informação atual"
                        }
                        Row(
                            if (recent) Modifier.background(colors.primary.copy(alpha = .09f), RoundedCornerShape(12.dp))
                                .padding(horizontal = 5.dp, vertical = 2.dp) else Modifier,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            if (recent) {
                                AppIcon(Glyph.Clock, Modifier.size(11.dp), colors.primary)
                                Spacer(Modifier.width(4.dp))
                            }
                            Text(status, fontSize = 10.sp, lineHeight = 12.sp,
                                color = if (recent) colors.primary else colors.onSurfaceVariant)
                        }
                    }
                    Surface(color = if (lowest) colors.primary else colors.surfaceVariant, shape = RoundedCornerShape(11.dp)) {
                        Text(if (value == null) "Preço indisponível" else money(value),
                            fontSize = if (value != null) 18.sp else 10.sp,
                            lineHeight = if (value != null) 22.sp else 13.sp,
                            fontWeight = FontWeight.Bold, color = if (lowest) colors.onPrimary else colors.onSurfaceVariant,
                            modifier = Modifier.widthIn(max = 112.dp).padding(horizontal = 9.dp, vertical = 7.dp))
                    }
                }
                if (expanded) {
                    Surface(
                        modifier = Modifier.fillMaxWidth().padding(start = 6.dp, end = 6.dp, bottom = 6.dp),
                        shape = RoundedCornerShape(12.dp), color = colors.surface
                    ) {
                        Column(Modifier.padding(8.dp)) {
                            Row(Modifier.fillMaxWidth().clickable { expanded = false }.padding(vertical = 5.dp),
                                verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                                AppIcon(Glyph.Document, Modifier.size(17.dp), colors.primary)
                                Text("Detalhes da oferta", Modifier.weight(1f), fontSize = 14.sp, fontWeight = FontWeight.Bold)
                                AppIcon(Glyph.Collapse, Modifier.size(20.dp))
                            }
                            OfferDetail(Glyph.Pin, "Endereço", offer.address)
                            HorizontalDivider(color = colors.outlineVariant)
                            OfferDetail(Glyph.Calendar, "Observado em", offer.collectedAt?.let(::formatDate) ?: "Sem data")
                            HorizontalDivider(color = colors.outlineVariant)
                            val source = offer.sourceUrl
                            val sourceLabel = when {
                                source == null -> "Sem fonte publicada"
                                source.contains("procon.mt.gov.br") -> "Procon-MT"
                                source.contains("aiqfome.com/") -> "Vitrine no aiqfome"
                                else -> "Site do supermercado"
                            }
                            OfferDetail(Glyph.Link, "Fonte", sourceLabel)
                            offer.conditions?.takeIf { it.isNotBlank() }?.let {
                                Text(it, fontSize = 11.sp, lineHeight = 15.sp)
                                offer.promotionalPriceCents?.let { price ->
                                    Text("Preço condicionado: ${money(price)}", fontSize = 11.sp, fontWeight = FontWeight.Bold)
                                }
                            }
                            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                if (source != null) Button(onClick = { onSource(source) }, modifier = Modifier.weight(1.25f),
                                    shape = RoundedCornerShape(9.dp), contentPadding = PaddingValues(horizontal = 5.dp, vertical = 6.dp)) {
                                    AppIcon(Glyph.External, Modifier.size(14.dp), colors.onPrimary)
                                    Spacer(Modifier.width(4.dp))
                                    Text("Abrir fonte original", fontSize = 11.sp, lineHeight = 13.sp)
                                }
                                FilledTonalButton(onClick = onCopyAddress, modifier = Modifier.weight(1f),
                                    shape = RoundedCornerShape(9.dp), contentPadding = PaddingValues(horizontal = 5.dp, vertical = 6.dp)) {
                                    AppIcon(Glyph.Copy, Modifier.size(14.dp), colors.primary)
                                    Spacer(Modifier.width(4.dp))
                                    Text("Copiar endereço", fontSize = 11.sp, lineHeight = 13.sp, color = colors.primary)
                                }
                            }
                        }
                    }
                }
            }
        }
        if (lowest) Surface(Modifier.padding(start = 14.dp), shape = RoundedCornerShape(7.dp), color = colors.primary) {
            Row(Modifier.padding(horizontal = 8.dp, vertical = 3.dp), horizontalArrangement = Arrangement.spacedBy(4.dp),
                verticalAlignment = Alignment.CenterVertically) {
                AppIcon(Glyph.Star, Modifier.size(12.dp), colors.onPrimary)
                Text(if (offer.isDemo) "Menor preço · DEMO" else "Menor preço", fontSize = 10.sp,
                    lineHeight = 12.sp, fontWeight = FontWeight.Bold, color = colors.onPrimary)
            }
        }
    }
}

@Composable private fun OfferDetail(icon: Glyph, label: String, value: String) {
    Row(Modifier.fillMaxWidth().padding(vertical = 5.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        AppIcon(icon, Modifier.size(14.dp), MaterialTheme.colorScheme.onSurfaceVariant)
        Text(label, Modifier.weight(.4f), fontSize = 11.sp, lineHeight = 14.sp,
            fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Text(value, Modifier.weight(.6f), fontSize = 11.sp, lineHeight = 14.sp)
    }
}
