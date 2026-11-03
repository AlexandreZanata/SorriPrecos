package br.com.sorriprecos

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch

@Composable internal fun ComparisonScreen(repo: Repository, selected: Product, favorite: Boolean, onFavorite: () -> Unit, refresh: Int, onRefresh: () -> Unit) {
    var loaded by remember(selected.id, repo) { mutableStateOf<Loaded<Comparison>?>(null) }
    var loading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    var details by remember { mutableStateOf(false) }
    var showAll by remember(selected.id) { mutableStateOf(false) }
    val uri = LocalUriHandler.current
    val snack = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()
    fun openSource(url: String) { try { uri.openUri(url) } catch (_: Exception) { scope.launch { snack.showSnackbar("Não foi possível abrir a fonte") } } }
    LaunchedEffect(selected.id, repo, refresh) {
        loading = true; error = null
        try { loaded = repo.prices(selected.id, selected.isHistorical) } catch (e: CancellationException) { throw e }
        catch (e: Exception) { error = e.message } finally { loading = false }
    }
    val product = loaded?.data?.product ?: selected
    val offers = loaded?.data?.items.orEmpty()
    fun current(o: Offer) = o.status == "recent" && o.collectedAt?.let { platformNow()-it<=172800 } == true && (o.validUntil==null || o.validUntil>=platformNow())
    val best = offers.filter { current(it) }.mapNotNull { it.effectivePriceCents }.minOrNull()
    val historical = product.historicalBranchCount > 0 && product.branchCount == 0
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
                    TextButton(onClick = onFavorite, contentPadding = PaddingValues(horizontal = 0.dp)) { Text(if (favorite) "♥ Salvo nos favoritos" else "♡ Salvar nos favoritos", fontSize = 12.sp) }
                    TextButton(onClick = onRefresh) { AppIcon(Glyph.Refresh, Modifier.size(15.dp), MaterialTheme.colorScheme.primary); Spacer(Modifier.width(5.dp)); Text("Atualizar", fontSize = 12.sp) }
                }
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    FilterChip(selected = !details, onClick = { details = false }, label = { Text("Preços", fontSize = 12.sp) })
                    FilterChip(selected = details, onClick = { details = true }, label = { Text("Detalhes", fontSize = 12.sp) })
                }
            }
            if (loading) item { LinearProgressIndicator(Modifier.fillMaxWidth()) }
            if (error != null) item { ErrorCard(error!!, onRefresh) }
            if (details) {
                item { EmptyCard("A mesma embalagem, sempre", "${product.name}\nMarca: ${product.brand}\nConteúdo: ${product.pack}\nApresentação: ${product.presentation}\nGTIN: ${product.gtin ?: "Não informado"}\n${if(product.isDemo)"Produto sintético de demonstração." else "Correspondência conservadora por GTIN ou nome, marca e embalagem."}") }
                item { Text("Referências de origem", fontSize = 18.sp, fontWeight = FontWeight.Bold) }
                items(offers.filter { it.sourceUrl != null }, key = { it.branchId }) { o ->
                    TextButton(onClick = { openSource(o.sourceUrl!!) }) { Text("${o.supermarketName} · ${o.branchName} ↗") }
                }
                if (offers.none { it.sourceUrl != null }) item { Text(if(product.isDemo) "Dados fictícios. Não há site de oferta real." else "Sem URLs de produtos informadas pelas fontes.", fontSize = 13.sp) }
            } else {
                item {
                    Text("Preços por mercado", fontSize = 19.sp, fontWeight = FontWeight.Bold)
                    Text(if (historical) "Registros históricos · não são ofertas atuais" else "Do menor para o maior preço conhecido", fontSize = 12.sp, lineHeight = 17.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 4.dp))
                    if (historical) Text("Toque na filial para ver endereço e origem.", fontSize = 10.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 3.dp))
                }
                items(if(showAll) offers else offers.take(5), key = { it.branchId }) { o ->
                    OfferRow(o, current(o), best != null && current(o) && o.effectivePriceCents == best, ::openSource)
                }
                if (offers.size > 5) item { TextButton(onClick = { showAll = !showAll }, modifier = Modifier.fillMaxWidth()) { Text(if(showAll) "Mostrar menos filiais" else "Ver todas as ${offers.size} filiais") } }
                if (loaded != null && offers.isEmpty()) item { EmptyCard("Preço não disponível", "Este produto ainda não possui ofertas vinculadas às filiais.") }
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

@Composable private fun OfferRow(offer: Offer, recent: Boolean, lowest: Boolean, onSource: (String) -> Unit) {
    var expanded by remember(offer.branchId) { mutableStateOf(false) }
    val value = if (recent) offer.effectivePriceCents else if (offer.status == "stale" || offer.status == "recent") offer.priceCents else null
    Surface(shape = RoundedCornerShape(13.dp), color = if (lowest) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surface,
        border = BorderStroke(1.dp, if(lowest) MaterialTheme.colorScheme.primary.copy(alpha=.2f) else MaterialTheme.colorScheme.outlineVariant), shadowElevation = if(lowest) 0.dp else 1.dp, modifier = Modifier.fillMaxWidth()) {
        Column {
            Row(Modifier.fillMaxWidth().clickable { expanded = !expanded }.padding(horizontal = 11.dp, vertical = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                AppIcon(Glyph.Cart, Modifier.size(23.dp), if(lowest) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant)
                Spacer(Modifier.width(10.dp))
                Column(Modifier.weight(1f).padding(end = 6.dp)) {
                    Text(offer.supermarketName, fontSize = 13.sp, lineHeight = 16.sp, fontWeight = FontWeight.Bold)
                    Text(offer.branchName, fontSize = 10.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 2.dp))
                    Text(when {
                        offer.status=="unavailable" -> "Indisponibilidade informada"
                        offer.status=="stale" || offer.status=="recent" && !recent -> "Histórico · ${offer.collectedAt?.let { formatDate(it).substringBefore(" às") } ?: "Sem data"}"
                        recent -> age(offer.collectedAt)
                        else -> "Sem informação atual"
                    }, fontSize = 9.sp, lineHeight = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 2.dp))
                    if (recent && !offer.conditions.isNullOrBlank()) Text("Condições especiais · toque para ver", fontSize = 9.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                Surface(color = if(lowest) MaterialTheme.colorScheme.primary else if(value==null) MaterialTheme.colorScheme.surfaceVariant else MaterialTheme.colorScheme.surface, shape = RoundedCornerShape(8.dp)) {
                    Text(money(value), fontSize = if(value!=null) 18.sp else 10.sp, lineHeight = if(value!=null) 23.sp else 13.sp, fontWeight = if(value!=null) FontWeight.Bold else FontWeight.Normal,
                        color = if(lowest) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurface, modifier = Modifier.widthIn(max = 115.dp).padding(horizontal = 8.dp, vertical = 6.dp))
                }
            }
            if (expanded) {
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                Column(Modifier.padding(13.dp), verticalArrangement = Arrangement.spacedBy(7.dp)) {
                    Text(offer.address, fontSize = 12.sp, lineHeight = 17.sp)
                    offer.collectedAt?.let { Text("Observado em ${formatDate(it)}", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                    if (lowest) Text(if(offer.isDemo) "MENOR PREÇO · DEMO" else "MENOR PREÇO CONHECIDO", fontSize = 10.sp, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Bold)
                    offer.conditions?.takeIf { it.isNotBlank() }?.let { Text(it, fontSize = 11.sp, lineHeight = 16.sp) }
                    if (offer.promotionalPriceCents!=null && !offer.conditions.isNullOrBlank()) Text("Preço condicionado: ${money(offer.promotionalPriceCents)}", fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                    offer.sourceUrl?.let { url -> TextButton(onClick = { onSource(url) }, contentPadding = PaddingValues(0.dp)) { Text("Ver na fonte original ↗", fontSize = 12.sp) } }
                }
            }
        }
    }
}
