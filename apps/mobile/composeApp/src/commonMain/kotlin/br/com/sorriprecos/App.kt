package br.com.sorriprecos

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

@Composable fun App(store: LocalStore) {
    var baseUrl by remember { mutableStateOf(store.get("api-url") ?: "http://127.0.0.1:8080") }
    var demo by remember { mutableStateOf(store.get("demo") == "true") }
    var historical by remember { mutableStateOf(!demo && store.get("historical") == "true") }
    var contrast by remember { mutableStateOf(store.get("ui-contrast") == "true") }
    var largeText by remember { mutableStateOf(store.get("ui-large-text") == "true") }
    val repo = remember(baseUrl) { Repository(store, baseUrl) }
    val synced by repo.updates.collectAsState()
    DisposableEffect(repo) { onDispose { repo.client.close() } }
    var screen by remember { mutableStateOf("home") }
    var pharmacy by remember { mutableStateOf(false) }
    var category by remember { mutableStateOf<QuickCategory?>(null) }
    var query by remember { mutableStateOf("") }
    var selected by remember { mutableStateOf<Product?>(null) }
    val homeVisible = screen == "home" && selected == null
    var homeHighlights by remember(repo, demo, historical) { mutableStateOf<List<HomeHighlight>>(emptyList()) }
    LaunchedEffect(repo, demo, historical, homeVisible, synced) {
        if (homeVisible && !historical) homeHighlights = repo.homeHighlights(demo)
    }
    var cart by remember(repo, demo) { mutableStateOf(repo.cart(demo)) }
    var favorites by remember(repo) { mutableStateOf(repo.favorites()) }
    var markets by remember { mutableStateOf<List<Supermarket>>(emptyList()) }
    var catalog by remember { mutableStateOf(CatalogStatus()) }
    var results by remember { mutableStateOf<List<Product>>(emptyList()) }
    var nextAfter by remember { mutableStateOf<Long?>(null) }
    var searching by remember { mutableStateOf(false) }
    var sourceLoading by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var offline by remember { mutableStateOf(false) }
    var searchOffline by remember { mutableStateOf(false) }
    var refresh by remember { mutableStateOf(0) }
    var lastSearchRefresh by remember(repo) { mutableStateOf(0) }
    var connectionDialog by remember { mutableStateOf(false) }
    var marketDialog by remember { mutableStateOf<Supermarket?>(null) }
    var paginationJob by remember { mutableStateOf<Job?>(null) }
    val scope = rememberCoroutineScope()
    val focus = LocalFocusManager.current
    val uri = LocalUriHandler.current
    val snack = remember { SnackbarHostState() }

    LaunchedEffect(repo) {
        while (true) {
            repo.syncKnownProducts()
            delay(60_000)
        }
    }
    LaunchedEffect(repo, refresh) {
        if (refresh > 0 && screen == "search") repo.syncKnownProducts()
    }
    LaunchedEffect(repo, synced) {
        if (synced > 0) {
            val current = results
            val updated = repo.latestProducts(current)
            if (results === current) {
                results = updated.data
                searchOffline = updated.offline
            }
        }
    }
    LaunchedEffect(repo, demo, historical, if (screen == "sources") refresh else 0) {
        sourceLoading = true; error = null
        try {
            val status = repo.status(); catalog = status.data
            val sources = repo.markets(demo, historical); markets = sources.data.items
            offline = status.offline || sources.offline
        } catch (e: CancellationException) { throw e }
        catch (e: Exception) { error = e.message }
        finally { sourceLoading = false }
    }
    LaunchedEffect(query, demo, historical, pharmacy, repo, refresh) {
        paginationJob?.cancelAndJoin()
        results = emptyList(); nextAfter = null; searchOffline = false
        if (query.trim().length < 2) { searching = false; return@LaunchedEffect }
        searching = true; error = null
        try {
            delay(350)
            val force = refresh != lastSearchRefresh
            val loaded = repo.search(query.trim(), demo, historical = historical && !pharmacy, pharmacy = pharmacy, force = force)
            lastSearchRefresh = refresh
            results = loaded.data.items; nextAfter = loaded.data.nextAfter; searchOffline = loaded.offline
        } catch (e: CancellationException) { throw e }
        catch (e: Exception) { error = e.message }
        finally { searching = false }
    }
    fun favorite(p: Product) {
        favorites = if (favorites.any { it.id == p.id }) favorites.filterNot { it.id == p.id } else favorites + p
        repo.saveFavorites(favorites)
    }
    fun setQuantity(product: Product, quantity: Int) {
        if (product.isHistorical || product.isDemo != demo) return
        val previous = cart.find { it.product.id == product.id }
        if (previous == null && cart.size >= CART_LIMIT) {
            scope.launch { snack.showSnackbar("Limite de $CART_LIMIT produtos no carrinho") }
            return
        }
        cart = when {
            quantity <= 0 -> cart.filterNot { it.product.id == product.id }
            previous == null -> cart + CartLine(product, quantity.coerceAtMost(CART_MAX_QUANTITY))
            else -> cart.map { if (it.product.id == product.id) it.copy(quantity = quantity.coerceAtMost(CART_MAX_QUANTITY)) else it }
        }
        repo.saveCart(demo, cart)
    }
    fun navigate(destination: String) { selected = null; screen = destination; focus.clearFocus() }
    fun search(value: String) { pharmacy = false; category = null; query = value; navigate("search") }
    fun explore() { search("") }
    fun openCategory(value: QuickCategory) { pharmacy = value == pharmacyCategory; category = value; query = ""; navigate("search") }
    fun toggleDemo() { historical = false; store.put("historical", "false"); demo = !demo; store.put("demo", demo.toString()); query = ""; selected = null }
    fun toggleHistory() { historical = !historical; demo = false; store.put("demo", "false"); store.put("historical", historical.toString()); query = ""; selected = null }
    fun setLarge(value: Boolean) { largeText = value; store.put("ui-large-text", value.toString()) }
    fun setContrast(value: Boolean) { contrast = value; store.put("ui-contrast", value.toString()) }
    NavigationBackHandler(selected != null || screen != "home") {
        if (selected != null) selected = null else navigate("home")
    }
    val density = LocalDensity.current
    CompositionLocalProvider(
        LocalDensity provides Density(density.density, density.fontScale * if (largeText) 1.2f else 1f),
        LocalPhotoOrigin provides baseUrl
    ) {
        MaterialTheme(colorScheme = if (contrast) AppContrast else AppLight) {
            PlatformAppearance(contrast, homeVisible)
            Scaffold(
                containerColor = MaterialTheme.colorScheme.background,
                contentWindowInsets = if (homeVisible)
                    WindowInsets.safeDrawing.only(WindowInsetsSides.Horizontal + WindowInsetsSides.Bottom)
                else ScaffoldDefaults.contentWindowInsets,
                snackbarHost = { SnackbarHost(snack) },
                topBar = {
                    if (selected != null || screen != "home") AppToolbar(
                        title = if (selected != null) "Comparar preços" else when (screen) {
                            "search" -> "Resultados da busca"; "favorites" -> "Favoritos"; "cart" -> if (demo) "Carrinho de demonstração" else "Meu carrinho"; "sources" -> "Fontes e filiais"; else -> "Ajustes"
                        },
                        onBack = { if (selected != null) selected = null else navigate("home") },
                        accessibility = selected != null, largeText = largeText, contrast = contrast,
                        onText = { setLarge(it) }, onContrast = { setContrast(!contrast) }
                    )
                },
                bottomBar = { if (selected == null) BottomNavigation(screen, cart.size, ::navigate) }
            ) { padding ->
                Column(Modifier.fillMaxSize().padding(padding).consumeWindowInsets(padding).imePadding()) {
                    if (homeVisible) Spacer(
                        Modifier.fillMaxWidth().background(Color(0xFF07572B)).windowInsetsTopHeight(WindowInsets.statusBars)
                    )
                    if (screen != "home" || selected != null) Box(Modifier.padding(horizontal = 18.dp, vertical = 5.dp)) { DataDisclosure(demo, catalog, selected?.isHistorical ?: (historical && screen != "cart" && !pharmacy)) }
                    if (selected != null) {
                        ComparisonScreen(repo, selected!!, favorites.any { it.id == selected!!.id },
                            onFavorite = { favorite(selected!!) }, refresh = refresh, onRefresh = { refresh++ },
                            cartQuantity = cart.find { it.product.id == selected!!.id }?.quantity ?: 0,
                            onAdd = selected!!.takeIf { !it.isHistorical && it.isDemo == demo }?.let { product ->
                                { setQuantity(product, (cart.find { it.product.id == product.id }?.quantity ?: 0) + 1) }
                            }, onCart = { navigate("cart") })
                    } else {
                        if (screen != "home" && (offline || searchOffline)) Notice("Offline · Mostrando dados salvos. Atualize antes de comprar.")
                        when (screen) {
                            "home" -> HomeScreen(demo, historical, offline, markets, homeHighlights,
                                favorites.map { it.id }.toSet(), ::search, ::explore, ::openCategory,
                                onProduct = { selected = it }, onFavorite = ::favorite,
                                onSources = { navigate("sources") }, onMarket = { marketDialog = it })
                            "search" -> LazyColumn(contentPadding = PaddingValues(18.dp, 10.dp, 18.dp, 20.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                                item {
                                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                        FilterChip(selected = !pharmacy, onClick = { pharmacy = false; category = null }, label = { Text("Mercado") })
                                        FilterChip(selected = pharmacy, onClick = { pharmacy = true; category = pharmacyCategory }, label = { Text("Farmácia") })
                                    }
                                }
                                item { SearchField(query, autofocus = category == null, onChange = { query = it.take(100) }, onSearch = { focus.clearFocus(); refresh++ }) }
                                if (query.trim().length < 2) {
                                    item { Text(category?.label ?: "Encontre o produto que você procura", fontWeight = FontWeight.Bold, modifier = Modifier.padding(top = 10.dp)) }
                                    val terms = category?.terms ?: if (historical) listOf("Chocolate", "Lacta", "Bacalhau") else quickProducts.map { it.query.replaceFirstChar(Char::uppercaseChar) }
                                    items(terms.chunked(2)) { row ->
                                        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                            row.forEach { term ->
                                                SuggestionChip(onClick = { query = term; focus.clearFocus() }, label = { Text(term) })
                                            }
                                        }
                                    }
                                } else {
                                    item { Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                                        Text(if (searching) "Buscando produtos…" else "${results.size} produtos encontrados${if(nextAfter!=null)" nesta página" else ""}", fontSize = 12.sp)
                                        IconButton(onClick = { refresh++ }, modifier = Modifier.size(36.dp).semantics { contentDescription = "Atualizar busca" }) { AppIcon(Glyph.Refresh, Modifier.size(18.dp)) }
                                    } }
                                    if (searching) item { LinearProgressIndicator(Modifier.fillMaxWidth()) }
                                    if (error != null) item { ErrorCard(error!!) { refresh++ } }
                                    if (!searching && results.isEmpty() && error == null) item { EmptyCard("Nenhum produto encontrado", "Tente outro nome ou embalagem. As fontes ainda podem estar pendentes.") }
                                    items(results, key = { it.id }) { p -> ProductCard(p, favorites.any { it.id == p.id }, onClick = { selected = p; focus.clearFocus() }, onFavorite = { favorite(p) }, offline = searchOffline) }
                                    if (nextAfter != null) item { OutlinedButton(onClick = {
                                        paginationJob = scope.launch {
                                            searching = true
                                            try { val loaded = repo.search(query.trim(), demo, nextAfter!!, historical && !pharmacy, pharmacy); results += loaded.data.items; nextAfter = loaded.data.nextAfter; searchOffline = loaded.offline }
                                            catch (e: CancellationException) { throw e } catch (e: Exception) { error = e.message } finally { searching = false }
                                        }
                                    }, enabled = !searching, modifier = Modifier.fillMaxWidth()) { Text("Carregar mais") } }
                                }
                            }
                            "cart" -> CartScreen(repo, cart, ::setQuantity, { navigate("search") }, { selected = it })
                            "favorites" -> LazyColumn(contentPadding = PaddingValues(18.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                                item { Text("Sua lista de favoritos", fontSize = 23.sp, fontWeight = FontWeight.Bold); Text("Salvos neste aparelho, sem cadastro.", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 6.dp, bottom = 8.dp)) }
                                if (favorites.isEmpty()) item { EmptyCard("Guarde o que você compra", "Toque no coração de um produto para acompanhar os preços depois.") }
                                items(favorites, key = { it.id }) { p -> ProductCard(p, true, onClick = { selected = p }, onFavorite = { favorite(p) }, offline = true) }
                            }
                            "sources" -> LazyColumn(contentPadding = PaddingValues(18.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                                item { Text("De onde vêm os preços", fontSize = 23.sp, fontWeight = FontWeight.Bold); Text("${catalog.discoveredSupermarkets} estabelecimentos mapeados · ${catalog.discoveredBranches} filiais referenciadas", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 6.dp)) }
                                item { Notice(if (demo) "Estas lojas são fictícias." else if(historical) "Pesquisa histórica do Procon: valores antigos, sem confirmação de preço ou estoque atual." else "Catálogos online das filiais de Sorriso-MT. Coleta a cada 10 minutos. Confirme preços e condições na origem.") }
                                item { TextButton(onClick = { refresh++ }) { AppIcon(Glyph.Refresh, Modifier.size(17.dp), MaterialTheme.colorScheme.primary); Spacer(Modifier.width(8.dp)); Text("Atualizar") } }
                                if (sourceLoading) item { LinearProgressIndicator(Modifier.fillMaxWidth()) }
                                if (error != null) item { ErrorCard(error!!) { refresh++ } }
                                items(markets, key = { it.id }) { m -> MarketRow(m) { marketDialog = m } }
                                item { OutlinedButton(onClick = { toggleDemo() }, modifier = Modifier.fillMaxWidth()) { Text(if (demo) "Ver fontes reais" else "Experimentar com dados DEMO") } }
                            }
                            else -> LazyColumn(contentPadding = PaddingValues(18.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                                item { Text("Do seu jeito", fontSize = 25.sp, fontWeight = FontWeight.Bold); Text("Mais conforto para comparar.", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 5.dp)) }
                                item { SettingsToggle("Texto maior", "Aumenta em 20% e respeita o tamanho do sistema.", largeText, ::setLarge) }
                                item { SettingsToggle("Alto contraste", "Fundo escuro e cores com mais contraste.", contrast, ::setContrast) }
                                item { SettingsLink("Fontes e filiais", "Veja a cobertura de Sorriso-MT.", Glyph.Cart) { navigate("sources") } }
                                item { SettingsLink("Conexão", baseUrl, Glyph.Refresh) { connectionDialog = true } }
                                item { SettingsToggle("Pesquisa histórica do Procon", "Consulta separada dos preços atuais dos mercados.", historical) { toggleHistory() } }
                                item { SettingsToggle("Modo DEMO", "Produtos e preços fictícios, separados dos reais.", demo) { toggleDemo() } }
                                item { EmptyCard("Feito para Sorriso", "Sem cadastro. Favoritos e preferências ficam neste aparelho. Registros históricos nunca são apresentados como ofertas atuais.") }
                            }
                        }
                    }
                }
            }
            if (connectionDialog) ConnectionDialog(baseUrl, onDismiss = { connectionDialog = false }, onSave = { baseUrl = it; store.put("api-url", it); query = ""; selected = null; connectionDialog = false; refresh++ })
            marketDialog?.let { market ->
                AlertDialog(onDismissRequest = { marketDialog = null }, title = { Text(market.name) }, text = { Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    Text("Sorriso-MT · ${market.branchCount} filial(is) referenciada(s)")
                    Text(if (market.isDemo) "Loja fictícia de demonstração." else if (market.productCount > 0) "${market.productCount} produtos com registros. Última observação: ${age(market.lastCollectedAt)}. ${if(historical) "Histórico não confirma preço nem estoque atual." else "Confira as condições na loja."}" else "Coleta pendente. Ainda não há preços dessa loja disponíveis no aplicativo.", fontSize = 14.sp)
                } }, confirmButton = { TextButton(onClick = { marketDialog = null }) { Text("Fechar") } }, dismissButton = {
                    market.website?.let { site -> TextButton(onClick = { try { uri.openUri(site) } catch (_: Exception) { scope.launch { snack.showSnackbar("Não foi possível abrir o site") } } }) { Text("Abrir site") } }
                })
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable private fun AppToolbar(title: String, onBack: () -> Unit, accessibility: Boolean, largeText: Boolean, contrast: Boolean, onText: (Boolean) -> Unit, onContrast: () -> Unit) {
    TopAppBar(
        title = { Text(title, fontWeight = FontWeight.Bold) },
        navigationIcon = {
            IconButton(onClick = onBack, modifier = Modifier.semantics { contentDescription = "Voltar" }) { AppIcon(Glyph.Back) }
        },
        colors = TopAppBarDefaults.topAppBarColors(containerColor = MaterialTheme.colorScheme.background),
        actions = {
            if (accessibility) {
                TextButton(onClick = { onText(false) }, contentPadding = PaddingValues(0.dp), modifier = Modifier.width(40.dp).semantics { contentDescription = "Tamanho de texto padrão" }) { Text("A", color = if (!largeText) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface, fontSize = 16.sp) }
                TextButton(onClick = { onText(true) }, contentPadding = PaddingValues(0.dp), modifier = Modifier.width(40.dp).semantics { contentDescription = "Aumentar texto" }) { Text("A+", fontSize = 17.sp, fontWeight = FontWeight.Bold, color = if (largeText) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface) }
                IconButton(onClick = onContrast, modifier = Modifier.semantics { contentDescription = if (contrast) "Desativar alto contraste" else "Ativar alto contraste" }) { AppIcon(Glyph.Contrast, Modifier.size(21.dp)) }
            }
        }
    )
}

@Composable private fun BottomNavigation(screen: String, cartCount: Int, onNavigate: (String) -> Unit) {
    NavigationBar(containerColor = MaterialTheme.colorScheme.surface, tonalElevation = 2.dp) {
        val tabs = listOf(
            Triple("home", Glyph.Home, "Início"), Triple("search", Glyph.Search, "Buscar"),
            Triple("cart", Glyph.Cart, "Carrinho"), Triple("favorites", Glyph.Heart, "Favoritos"),
            Triple("settings", Glyph.Settings, "Ajustes")
        )
        tabs.forEach { (id, icon, label) ->
            val selected = screen == id || (id == "settings" && screen == "sources")
            NavigationBarItem(
                selected = selected, onClick = { onNavigate(id) },
                icon = {
                    BadgedBox(badge = {
                        if (id == "cart" && cartCount > 0) Badge(
                            containerColor = MaterialTheme.colorScheme.error,
                            modifier = Modifier.semantics { contentDescription = "$cartCount produtos no carrinho" }
                        ) { Text(cartCount.toString()) }
                    }) {
                        AppIcon(icon, tint = if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant, filled = selected)
                    }
                },
                label = { Text(label, maxLines = 1, fontSize = 10.sp) },
                colors = NavigationBarItemDefaults.colors(indicatorColor = MaterialTheme.colorScheme.primaryContainer)
            )
        }
    }
}

@Composable private fun SearchField(query: String, autofocus: Boolean, onChange: (String) -> Unit, onSearch: () -> Unit) {
    val requester = remember { FocusRequester() }
    LaunchedEffect(Unit) { if (autofocus && query.isEmpty()) requester.requestFocus() }
    OutlinedTextField(value = query, onValueChange = onChange, modifier = Modifier.fillMaxWidth().focusRequester(requester), singleLine = true,
        placeholder = { Text("Produto, marca ou embalagem", fontSize = 13.sp) }, leadingIcon = { AppIcon(Glyph.Search, Modifier.size(20.dp)) },
        trailingIcon = { if (query.isNotEmpty()) IconButton(onClick = { onChange("") }, modifier = Modifier.semantics { contentDescription = "Limpar busca" }) { AppIcon(Glyph.Close, Modifier.size(18.dp)) } },
        shape = RoundedCornerShape(16.dp), keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search), keyboardActions = KeyboardActions(onSearch = { onSearch() }),
        colors = OutlinedTextFieldDefaults.colors(unfocusedContainerColor = MaterialTheme.colorScheme.surfaceVariant, focusedContainerColor = MaterialTheme.colorScheme.surface, unfocusedBorderColor = MaterialTheme.colorScheme.outlineVariant))
}

@Composable private fun ConnectionDialog(current: String, onDismiss: () -> Unit, onSave: (String) -> Unit) {
    var value by remember { mutableStateOf(current) }
    val valid = value.trim().let { (it.startsWith("https://") || it.startsWith("http://127.0.0.1:") || it.startsWith("http://localhost:") || it.startsWith("http://10.0.2.2:")) && it.length in 10..300 && !it.contains('@') && !it.contains(' ') }
    AlertDialog(onDismissRequest = onDismiss, title = { Text("Conectar ao SorriPreços") }, text = { Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text("No teste por USB, use a conexão local preparada com ADB.", fontSize = 13.sp)
        OutlinedTextField(value = value, onValueChange = { value = it }, label = { Text("Endereço da API") }, singleLine = true)
        Text("Favoritos ficam neste aparelho.", fontSize = 11.sp)
    } }, confirmButton = { TextButton(onClick = { onSave(value.trim().trimEnd('/')) }, enabled = valid) { Text("Salvar conexão") } }, dismissButton = { TextButton(onClick = onDismiss) { Text("Cancelar") } })
}
