package br.com.sorriprecos

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.Alignment
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancelAndJoin

private val Green=Color(0xFF176B48)
private val Lime=Color(0xFFE3F7B1)
private val light=lightColorScheme(primary=Green,onPrimary=Color.White,primaryContainer=Color(0xFFE3EEDC),onPrimaryContainer=Color(0xFF153F2F),secondaryContainer=Color(0xFFD8E9DC),onSecondaryContainer=Green,background=Color(0xFFF6F7F2),onBackground=Color(0xFF182B23),surface=Color(0xFFF6F7F2),surfaceVariant=Color(0xFFE8ECE3),onSurface=Color(0xFF182B23),onSurfaceVariant=Color(0xFF526259),outlineVariant=Color(0xFFD9E0D5))
private val dark=darkColorScheme(primary=Color(0xFF9CD3AE),primaryContainer=Color(0xFF254C37),onPrimaryContainer=Color(0xFFE3F7B1),secondaryContainer=Color(0xFF254C37),onSecondaryContainer=Color(0xFFE3F7B1),background=Color(0xFF101B15),onBackground=Color(0xFFE5EDE4),surface=Color(0xFF101B15),surfaceVariant=Color(0xFF213027),onSurface=Color(0xFFE5EDE4),onSurfaceVariant=Color(0xFFBDCABC),outlineVariant=Color(0xFF35463A))

@Composable fun App(store:LocalStore) {
    var baseUrl by remember {mutableStateOf(store.get("api-url")?:"http://127.0.0.1:8080")}
    var demo by remember {mutableStateOf(store.get("demo")=="true")}
    val repo=remember(baseUrl){Repository(store,baseUrl)}
    DisposableEffect(repo){onDispose{repo.client.close()}}
    var screen by remember {mutableStateOf("home")}
    var query by remember {mutableStateOf("")}
    var selected by remember {mutableStateOf<Product?>(null)}
    var favorites by remember(repo){mutableStateOf(repo.favorites())}
    var markets by remember {mutableStateOf<List<Supermarket>>(emptyList())}
    var catalog by remember {mutableStateOf(CatalogStatus())}
    var results by remember {mutableStateOf<List<Product>>(emptyList())}
    var nextAfter by remember {mutableStateOf<Long?>(null)}
    var searching by remember {mutableStateOf(false)}
    var sourceLoading by remember {mutableStateOf(false)}
    var error by remember {mutableStateOf<String?>(null)}
    var offline by remember {mutableStateOf(false)}
    var searchOffline by remember {mutableStateOf(false)}
    var refresh by remember {mutableStateOf(0)}
    var settings by remember {mutableStateOf(false)}
    var paginationJob by remember {mutableStateOf<Job?>(null)}
    val scope=rememberCoroutineScope()
    val focus=LocalFocusManager.current

    LaunchedEffect(repo,demo,refresh) {
        sourceLoading=true;error=null
        try {
            val status=repo.status();catalog=status.data
            val sources=repo.markets(demo);markets=sources.data.items
            offline=status.offline || sources.offline
        } catch(e:CancellationException){throw e} catch(e:Exception){error=e.message} finally {sourceLoading=false}
    }
    LaunchedEffect(query,demo,repo,refresh) {
        paginationJob?.cancelAndJoin()
        results=emptyList();nextAfter=null;searchOffline=false
        if(query.trim().length<2){searching=false;return@LaunchedEffect}
        searching=true;error=null
        try {delay(350);val loaded=repo.search(query.trim(),demo);results=loaded.data.items;nextAfter=loaded.data.nextAfter;searchOffline=loaded.offline}
        catch(e:CancellationException){throw e} catch(e:Exception){error=e.message} finally{searching=false}
    }
    fun toggleFavorite(p:Product) {
        favorites=if(favorites.any{it.id==p.id})favorites.filterNot{it.id==p.id} else favorites+p
        repo.saveFavorites(favorites)
    }
    NavigationBackHandler(selected!=null || screen!="home" || query.isNotEmpty()) {
        if(selected!=null)selected=null else if(screen!="home")screen="home" else query=""
    }

    MaterialTheme(colorScheme=if(isSystemInDarkTheme())dark else light) {
        Scaffold(containerColor=MaterialTheme.colorScheme.background,
            bottomBar={if(selected==null)NavigationBar(containerColor=MaterialTheme.colorScheme.surface) {
                listOf(Triple("home","⌂","Início"),Triple("favorites","♡","Favoritos"),Triple("sources","≡","Fontes")).forEach { (id,icon,label)->
                    NavigationBarItem(selected=screen==id,onClick={screen=id;focus.clearFocus()},icon={Text(icon,fontSize=25.sp)},label={Text(label)})
                }
            }}
        ) { padding->
            Column(Modifier.fillMaxSize().padding(padding)) {
                Row(Modifier.fillMaxWidth().padding(horizontal=22.dp,vertical=14.dp),verticalAlignment=Alignment.CenterVertically) {
                    if(selected!=null)TextButton(onClick={selected=null}){Text("‹ Voltar")} else SmileMark()
                    Spacer(Modifier.width(10.dp))
                    Column(Modifier.weight(1f)) {
                        Text("SorriPreços",fontSize=23.sp,fontWeight=FontWeight.ExtraBold,letterSpacing=(-0.8).sp)
                        if(selected==null)Text("SORRISO, MATO GROSSO",fontSize=9.sp,letterSpacing=1.5.sp,color=MaterialTheme.colorScheme.onSurfaceVariant,fontWeight=FontWeight.SemiBold)
                    }
                    if(selected==null)TextButton(onClick={settings=true},modifier=Modifier.semantics{contentDescription="Configurar conexão"}){Text("Conexão",fontSize=12.sp)}
                }
                if(demo)Notice("MODO DEMO · Preços fictícios para testar o app",demo=true)
                if(!demo && catalog.historicalProducts>0 && catalog.recentProducts==0)Notice("DADOS REAIS HISTÓRICOS · Procon-MT / março de 2026. Nenhuma oferta atual confirmada.")
                if(selected!=null) {
                    ComparisonScreen(repo,selected!!,favorites.any{it.id==selected!!.id},onFavorite={toggleFavorite(selected!!)},refresh=refresh,onRefresh={refresh++})
                } else {
                    if(offline || searchOffline)Notice("Offline · Mostrando dados salvos. Atualize antes de comprar.")
                    when(screen) {
                        "home"->LazyColumn(Modifier.fillMaxSize(),contentPadding=PaddingValues(start=22.dp,end=22.dp,bottom=24.dp),verticalArrangement=Arrangement.spacedBy(15.dp)) {
                            item {
                                Spacer(Modifier.height(13.dp))
                                Text("Sua compra pode\ncustar menos.",fontSize=34.sp,lineHeight=37.sp,fontWeight=FontWeight.Bold,letterSpacing=(-1.2).sp)
                                Text("Compare preços nos mercados de Sorriso",color=MaterialTheme.colorScheme.onSurfaceVariant,fontSize=14.sp,modifier=Modifier.padding(top=9.dp,bottom=22.dp))
                                OutlinedTextField(value=query,onValueChange={query=it.take(100)},singleLine=true,label={Text("O que você procura?")},placeholder={Text("Ex.: arroz 5kg")},leadingIcon={Text("⌕",fontSize=28.sp)},trailingIcon={if(query.isNotEmpty())TextButton(onClick={query=""}){Text("Limpar")}},shape=RoundedCornerShape(16.dp),modifier=Modifier.fillMaxWidth(),keyboardOptions=KeyboardOptions(imeAction=ImeAction.Search),keyboardActions=KeyboardActions(onSearch={focus.clearFocus();refresh++}))
                            }
                            if(query.trim().length>=2) {
                                item {Row(Modifier.fillMaxWidth(),horizontalArrangement=Arrangement.SpaceBetween,verticalAlignment=Alignment.CenterVertically){Text("Resultados",fontWeight=FontWeight.Bold,fontSize=18.sp);TextButton(onClick={refresh++}){Text("Atualizar")}}}
                                if(searching)item{LinearProgressIndicator(Modifier.fillMaxWidth())}
                                if(error!=null)item{ErrorCard(error!!){refresh++}}
                                if(!searching && results.isEmpty() && error==null)item{EmptyCard("Nenhum produto encontrado","Tente outro nome ou embalagem. As fontes ainda podem estar pendentes.")}
                                items(results,key={it.id}) {p->ProductCard(p,favorites.any{it.id==p.id},onClick={selected=p;focus.clearFocus()},onFavorite={toggleFavorite(p)},offline=searchOffline)}
                                if(nextAfter!=null)item{OutlinedButton(onClick={paginationJob=scope.launch {searching=true;try{val loaded=repo.search(query.trim(),demo,nextAfter!!);results=results+loaded.data.items;nextAfter=loaded.data.nextAfter;searchOffline=loaded.offline}catch(e:CancellationException){throw e}catch(e:Exception){error=e.message}finally{searching=false}}},enabled=!searching,modifier=Modifier.fillMaxWidth()){Text("Carregar mais")}}
                            } else {
                                item {Row(horizontalArrangement=Arrangement.spacedBy(8.dp)){(if(!demo && catalog.historicalProducts>0)listOf("Chocolate","Lacta","Bacalhau") else listOf("Arroz 5kg","Café","Leite")).forEach{suggestion->SuggestionChip(onClick={query=suggestion},label={Text(suggestion)})}}}
                                item {Surface(shape=RoundedCornerShape(20.dp),color=Green) {
                                    Column(Modifier.padding(22.dp)) {
                                        Text(if(demo)"EXPLORE O MVP" else if(catalog.realProducts>0)"REGISTROS REAIS · SORRISO-MT" else "PREÇOS COM ORIGEM",fontSize=10.sp,fontWeight=FontWeight.Bold,letterSpacing=1.4.sp,color=Lime)
                                        Text(if(demo)"Compare. Favorite.\nTeste sem cadastro." else if(catalog.realProducts>0)"Registros reais.\nOrigem e data visíveis." else "Cada preço tem\numa filial e uma data.",fontSize=24.sp,lineHeight=29.sp,fontWeight=FontWeight.Bold,color=Color.White,modifier=Modifier.padding(vertical=13.dp))
                                        Text(if(demo)"12 produtos de demonstração. Nenhum valor representa uma oferta real." else if(catalog.historicalProducts>0)"O arquivo público do Procon contém preços observados em março de 2026. Use para testar a busca; esses valores estão desatualizados." else "Estamos preparando as primeiras fontes autorizadas. Você já pode conhecer a experiência.",fontSize=13.sp,color=Color(0xFFD8E9DC),lineHeight=20.sp)
                                    }
                                }}
                                item {Row(Modifier.fillMaxWidth(),horizontalArrangement=Arrangement.SpaceBetween,verticalAlignment=Alignment.CenterVertically){Text("Cobertura transparente",fontSize=19.sp,fontWeight=FontWeight.Bold);TextButton(onClick={screen="sources"}){Text("Ver fontes ›")}}}
                                item {Row(horizontalArrangement=Arrangement.spacedBy(10.dp)){Metric("${catalog.discoveredSupermarkets}","mapeados",Modifier.weight(1f));Metric("${catalog.importedSupermarkets}","com registros",Modifier.weight(1f));Metric("${catalog.realProducts}","produtos reais",Modifier.weight(1f))}}
                                if(sourceLoading)item{LinearProgressIndicator(Modifier.fillMaxWidth())}
                                if(error!=null)item{ErrorCard(error!!){refresh++}}
                                item {Text(if(demo)"Ambiente de demonstração" else if(catalog.importedSupermarkets>0)"Fontes e registros históricos" else "Fontes em preparação",fontSize=13.sp,fontWeight=FontWeight.SemiBold,color=MaterialTheme.colorScheme.onSurfaceVariant)}
                                items(markets.take(if(demo)3 else 4),key={it.id}) {MarketRow(it)}
                                item {OutlinedButton(onClick={demo=!demo;store.put("demo",demo.toString());query="";selected=null},modifier=Modifier.fillMaxWidth()){Text(if(demo)"Sair da demonstração" else "Experimentar com dados DEMO")}}
                                item {Text("Menor preço conhecido entre as fontes disponíveis. A cobertura da cidade ainda está em expansão.",fontSize=11.sp,lineHeight=17.sp,color=MaterialTheme.colorScheme.onSurfaceVariant)}
                            }
                        }
                        "favorites"->LazyColumn(contentPadding=PaddingValues(22.dp),verticalArrangement=Arrangement.spacedBy(14.dp)) {
                            item {Text("Sua lista de favoritos",fontSize=28.sp,fontWeight=FontWeight.Bold);Text("Salvos neste aparelho, sem cadastro.",color=MaterialTheme.colorScheme.onSurfaceVariant,modifier=Modifier.padding(top=7.dp,bottom=10.dp))}
                            if(favorites.isEmpty())item{EmptyCard("Guarde o que você compra","Toque no coração de um produto para acompanhar os preços depois.")}
                            items(favorites,key={it.id}) {p->ProductCard(p,true,onClick={selected=p},onFavorite={toggleFavorite(p)},offline=true)}
                        }
                        "sources"->LazyColumn(contentPadding=PaddingValues(22.dp),verticalArrangement=Arrangement.spacedBy(12.dp)) {
                            item {Text("De onde vêm os preços",fontSize=28.sp,lineHeight=32.sp,fontWeight=FontWeight.Bold);Text("${catalog.discoveredSupermarkets} estabelecimentos mapeados · ${catalog.discoveredBranches} filiais referenciadas",fontSize=13.sp,color=MaterialTheme.colorScheme.onSurfaceVariant,modifier=Modifier.padding(top=10.dp))}
                            item {Notice(if(demo)"Estas lojas são fictícias. Saia do DEMO para consultar o levantamento real." else "Registros públicos históricos não significam integração atual com a loja. Confira a data e a origem de cada preço.")}
                            item {Row(Modifier.fillMaxWidth(),horizontalArrangement=Arrangement.SpaceBetween){Text(if(demo)"Fontes DEMO" else "Levantamento inicial",fontWeight=FontWeight.Bold);TextButton(onClick={refresh++}){Text("Atualizar")}}}
                            if(sourceLoading)item{LinearProgressIndicator(Modifier.fillMaxWidth())}
                            if(error!=null)item{ErrorCard(error!!){refresh++}}
                            items(markets,key={it.id}) {MarketRow(it)}
                            item {OutlinedButton(onClick={demo=!demo;store.put("demo",demo.toString());query=""},modifier=Modifier.fillMaxWidth()){Text(if(demo)"Ver fontes reais" else "Experimentar com dados DEMO")}}
                        }
                    }
                }
            }
        }
        if(settings)ConnectionDialog(baseUrl,onDismiss={settings=false},onSave={baseUrl=it;store.put("api-url",it);query="";selected=null;settings=false;refresh++})
    }
}

@Composable private fun SmileMark() {
    Canvas(Modifier.size(39.dp).clip(RoundedCornerShape(12.dp)).background(Green).padding(9.dp)) {
        drawLine(Lime,Offset(size.width*.18f,size.height*.18f),Offset(size.width*.18f,size.height*.3f),strokeWidth=3.dp.toPx(),cap=StrokeCap.Round)
        drawLine(Lime,Offset(size.width*.82f,size.height*.18f),Offset(size.width*.82f,size.height*.3f),strokeWidth=3.dp.toPx(),cap=StrokeCap.Round)
        drawArc(Lime,10f,160f,false,Offset(0f,size.height*.2f),Size(size.width,size.height*.7f),style=Stroke(width=3.dp.toPx(),cap=StrokeCap.Round))
    }
}
@Composable private fun Notice(text:String,demo:Boolean=false) {
    Surface(color=if(demo)MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant,modifier=Modifier.fillMaxWidth()) {
        Text(text,modifier=Modifier.padding(horizontal=22.dp,vertical=10.dp),fontSize=11.sp,lineHeight=16.sp,fontWeight=if(demo)FontWeight.Bold else FontWeight.Normal)
    }
}
@Composable private fun Metric(value:String,label:String,modifier:Modifier=Modifier) {
    Surface(modifier=modifier,shape=RoundedCornerShape(14.dp),color=MaterialTheme.colorScheme.surfaceVariant) {
        Column(Modifier.padding(13.dp)){Text(value,fontWeight=FontWeight.Bold,fontSize=25.sp);Text(label,fontSize=10.sp,color=MaterialTheme.colorScheme.onSurfaceVariant)}
    }
}
@Composable private fun EmptyCard(title:String,body:String) {
    Surface(shape=RoundedCornerShape(16.dp),color=MaterialTheme.colorScheme.surfaceVariant,modifier=Modifier.fillMaxWidth()) {
        Column(Modifier.padding(20.dp)){Text(title,fontSize=17.sp,fontWeight=FontWeight.Bold);Text(body,fontSize=13.sp,lineHeight=20.sp,color=MaterialTheme.colorScheme.onSurfaceVariant,modifier=Modifier.padding(top=8.dp))}
    }
}
@Composable private fun ErrorCard(message:String,retry:()->Unit) {
    Column {Text(message,color=MaterialTheme.colorScheme.error,fontSize=13.sp);TextButton(onClick=retry){Text("Tentar novamente")}}
}
@Composable private fun MarketRow(m:Supermarket) {
    Row(Modifier.fillMaxWidth().padding(vertical=7.dp),verticalAlignment=Alignment.CenterVertically) {
        Box(Modifier.size(9.dp).clip(RoundedCornerShape(5.dp)).background(if(m.productCount>0 && m.lastCollectedAt?.let{platformNow()-it<=172800}==true)MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outline))
        Spacer(Modifier.width(12.dp))
        Column(Modifier.weight(1f)){Text(m.name,fontSize=14.sp,fontWeight=FontWeight.SemiBold);Text("${m.branchCount} filial(is) · ${if(m.productCount>0)age(m.lastCollectedAt) else "Coleta pendente"}",fontSize=11.sp,color=MaterialTheme.colorScheme.onSurfaceVariant)}
        Text(if(m.isDemo)"DEMO" else if(m.integrationStatus=="authorized")"Integrado" else if(m.productCount>0)"Histórico" else "Pendente",fontSize=10.sp,color=MaterialTheme.colorScheme.onSurfaceVariant)
    }
}
@Composable private fun ProductCard(p:Product,favorite:Boolean,onClick:()->Unit,onFavorite:()->Unit,offline:Boolean) {
    Surface(shape=RoundedCornerShape(17.dp),color=MaterialTheme.colorScheme.surface,modifier=Modifier.fillMaxWidth().border(1.dp,MaterialTheme.colorScheme.outlineVariant,RoundedCornerShape(17.dp)).clickable(onClick=onClick)) {
        Column(Modifier.padding(17.dp)) {
            Row(verticalAlignment=Alignment.Top) {
                Column(Modifier.weight(1f)) {Text(p.category.uppercase(),fontSize=9.sp,letterSpacing=1.sp,color=MaterialTheme.colorScheme.primary,fontWeight=FontWeight.Bold);Text(p.name,fontWeight=FontWeight.SemiBold,fontSize=17.sp,modifier=Modifier.padding(top=6.dp));Text("${p.brand} · ${p.pack} · ${p.presentation}",fontSize=12.sp,color=MaterialTheme.colorScheme.onSurfaceVariant,modifier=Modifier.padding(top=5.dp))}
                TextButton(onClick=onFavorite,modifier=Modifier.size(46.dp).semantics{contentDescription=if(favorite)"Remover favorito ${p.name}" else "Favoritar ${p.name}"},contentPadding=PaddingValues(0.dp)){Text(if(favorite)"♥" else "♡",fontSize=26.sp)}
            }
            HorizontalDivider(Modifier.padding(vertical=13.dp),color=MaterialTheme.colorScheme.outlineVariant)
            Row(verticalAlignment=Alignment.CenterVertically) {
                Column(Modifier.weight(1f)) {
                    Text(if(!p.isDemo && p.minPriceCents==null && p.historicalBranchCount>0)"REGISTRO HISTÓRICO · DESATUALIZADO" else if(offline)"REFERÊNCIA SALVA" else if(p.isDemo)"MENOR PREÇO DEMO" else "MENOR PREÇO CONHECIDO",fontSize=9.sp,letterSpacing=.7.sp,color=MaterialTheme.colorScheme.onSurfaceVariant)
                    val current=p.lastCollectedAt?.let{platformNow()-it<=172800}==true
                    val reference=if(current && p.minPriceCents!=null)p.minPriceCents else if(p.historicalBranchCount>0)p.referencePriceCents else null
                    Text(money(reference),fontWeight=FontWeight.Bold,fontSize=if(reference!=null)25.sp else 13.sp,color=MaterialTheme.colorScheme.primary,modifier=Modifier.padding(top=3.dp))
                }
                Column(horizontalAlignment=Alignment.End){Text("Comparar ›",fontSize=12.sp,fontWeight=FontWeight.Bold);Text(if(p.branchCount>0)"${p.branchCount} filiais" else if(p.historicalBranchCount>0)"${p.historicalBranchCount} registros antigos" else "Sem preço recente",fontSize=10.sp,color=MaterialTheme.colorScheme.onSurfaceVariant)}
            }
        }
    }
}

@Composable private fun ComparisonScreen(repo:Repository,selected:Product,favorite:Boolean,onFavorite:()->Unit,refresh:Int,onRefresh:()->Unit) {
    var loaded by remember(selected.id,repo){mutableStateOf<Loaded<Comparison>?>(null)}
    var loading by remember {mutableStateOf(true)}
    var error by remember {mutableStateOf<String?>(null)}
    var details by remember {mutableStateOf(false)}
    val uri=LocalUriHandler.current
    val snack=remember {SnackbarHostState()};val scope=rememberCoroutineScope()
    LaunchedEffect(selected.id,repo,refresh) {
        loading=true;error=null
        try {loaded=repo.prices(selected.id)}catch(e:CancellationException){throw e}catch(e:Exception){error=e.message}finally{loading=false}
    }
    val product=loaded?.data?.product?:selected
    val offers=loaded?.data?.items.orEmpty()
    fun isCurrent(o:Offer)=o.status=="recent" && o.collectedAt?.let{platformNow()-it<=172800}==true && (o.validUntil==null || o.validUntil>=platformNow())
    val best=offers.filter{isCurrent(it)}.mapNotNull{it.effectivePriceCents}.minOrNull()
    Box(Modifier.fillMaxSize()) {
        LazyColumn(contentPadding=PaddingValues(start=22.dp,end=22.dp,bottom=28.dp),verticalArrangement=Arrangement.spacedBy(13.dp)) {
            if(loaded?.offline==true)item{Notice("Offline · Comparação salva. Preços podem ter mudado.")}
            item {
                Spacer(Modifier.height(8.dp))
                Text(product.category.uppercase(),fontSize=10.sp,fontWeight=FontWeight.Bold,color=MaterialTheme.colorScheme.primary,letterSpacing=1.sp)
                Text(product.name,fontSize=30.sp,lineHeight=35.sp,fontWeight=FontWeight.Bold,modifier=Modifier.padding(top=9.dp))
                Text("${product.brand} · ${product.pack}",fontSize=16.sp,color=MaterialTheme.colorScheme.onSurfaceVariant,modifier=Modifier.padding(top=8.dp))
                Row(Modifier.fillMaxWidth(),horizontalArrangement=Arrangement.SpaceBetween){TextButton(onClick=onFavorite){Text(if(favorite)"♥ Salvo nos favoritos" else "♡ Salvar nos favoritos")};TextButton(onClick=onRefresh){Text("Atualizar")}}
                Row(horizontalArrangement=Arrangement.spacedBy(10.dp)){FilterChip(selected=!details,onClick={details=false},label={Text("Comparar preços")});FilterChip(selected=details,onClick={details=true},label={Text("Detalhes")})}
            }
            if(loading)item{LinearProgressIndicator(Modifier.fillMaxWidth())}
            if(error!=null)item{ErrorCard(error!!,onRefresh)}
            if(details) {
                item{EmptyCard("A mesma embalagem, sempre","${product.name}\nMarca: ${product.brand}\nConteúdo: ${product.pack}\nApresentação: ${product.presentation}\nGTIN: ${product.gtin?:"Não informado"}\n${if(product.isDemo)"Produto sintético de demonstração." else "Correspondência conservadora por GTIN ou nome, marca e embalagem."}")}
                item{Text("Referências de origem",fontSize=18.sp,fontWeight=FontWeight.Bold)}
                items(offers.filter{it.sourceUrl!=null},key={it.branchId}) {o->TextButton(onClick={try{uri.openUri(o.sourceUrl!!)}catch(_:Exception){scope.launch{snack.showSnackbar("Não foi possível abrir a fonte")}}}){Text("${o.supermarketName} ›")}}
                if(offers.none{it.sourceUrl!=null})item{Text(if(product.isDemo)"Dados fictícios. Não há site de oferta real." else "Sem URLs de produtos informadas pelas fontes.",fontSize=13.sp,color=MaterialTheme.colorScheme.onSurfaceVariant)}
            } else {
                item {Text(if(product.historicalBranchCount>0 && product.branchCount==0)"Registros históricos · não são ofertas atuais" else "${offers.count{isCurrent(it)}} filiais com preço recente",fontSize=13.sp,color=MaterialTheme.colorScheme.onSurfaceVariant);Text(if(product.historicalBranchCount>0 && product.branchCount==0)"Observe a data de cada registro e confirme os valores com a loja." else "Menor preço conhecido. Cobertura incompleta.",fontSize=11.sp,color=MaterialTheme.colorScheme.onSurfaceVariant,modifier=Modifier.padding(top=4.dp))}
                items(offers,key={it.branchId}) {o->
                    val recent=isCurrent(o);val lowest=recent && o.effectivePriceCents==best && best!=null
                    Surface(shape=RoundedCornerShape(17.dp),color=if(lowest)MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surface,modifier=Modifier.fillMaxWidth().border(1.dp,if(lowest)MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outlineVariant,RoundedCornerShape(17.dp))) {
                        Column(Modifier.padding(18.dp)) {
                            if(lowest)Text(if(o.isDemo)"MENOR PREÇO · DEMO" else "MENOR PREÇO CONHECIDO",fontSize=9.sp,letterSpacing=1.sp,fontWeight=FontWeight.Bold,color=MaterialTheme.colorScheme.primary,modifier=Modifier.padding(bottom=10.dp))
                            Text(o.supermarketName,fontWeight=FontWeight.Bold,fontSize=17.sp)
                            Text(o.branchName,fontSize=12.sp,color=MaterialTheme.colorScheme.onSurfaceVariant,modifier=Modifier.padding(top=4.dp))
                            Text(o.address,fontSize=11.sp,lineHeight=16.sp,color=MaterialTheme.colorScheme.onSurfaceVariant,modifier=Modifier.padding(top=4.dp))
                            val value=if(recent)o.effectivePriceCents else if(o.status=="stale" || o.status=="recent")o.priceCents else null
                            Text(money(value),fontSize=if(value!=null)29.sp else 15.sp,fontWeight=FontWeight.Bold,modifier=Modifier.padding(top=14.dp),color=if(recent)MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant)
                            Text(when {o.status=="unavailable"->"Indisponibilidade informada pela fonte";o.status=="stale" || (o.status=="recent" && !recent)->"Preço desatualizado · ${age(o.collectedAt)}";recent->age(o.collectedAt);else->"Sem informação atual"},fontSize=11.sp,color=MaterialTheme.colorScheme.onSurfaceVariant,modifier=Modifier.padding(top=5.dp))
                            if(!o.conditions.isNullOrBlank())Text(o.conditions,fontSize=12.sp,lineHeight=18.sp,modifier=Modifier.padding(top=10.dp))
                            if(o.promotionalPriceCents!=null && !o.conditions.isNullOrBlank())Text("Preço condicionado: ${money(o.promotionalPriceCents)}",fontSize=12.sp,fontWeight=FontWeight.SemiBold,modifier=Modifier.padding(top=5.dp))
                            if(o.sourceUrl!=null)TextButton(onClick={try{uri.openUri(o.sourceUrl)}catch(_:Exception){scope.launch{snack.showSnackbar("Não foi possível abrir a fonte")}}},contentPadding=PaddingValues(0.dp)){Text("Ver na fonte original ↗",fontSize=12.sp)}
                        }
                    }
                }
                if(loaded!=null && offers.isEmpty())item{EmptyCard("Preço não disponível","Este produto ainda não possui ofertas vinculadas às filiais.")}
            }
        }
        SnackbarHost(snack,Modifier.align(Alignment.BottomCenter))
    }
}

@Composable private fun ConnectionDialog(current:String,onDismiss:()->Unit,onSave:(String)->Unit) {
    var value by remember {mutableStateOf(current)}
    val valid=value.trim().let{(it.startsWith("https://") || it.startsWith("http://127.0.0.1:") || it.startsWith("http://localhost:") || it.startsWith("http://10.0.2.2:")) && it.length in 10..300 && !it.contains('@') && !it.contains(' ')}
    AlertDialog(onDismissRequest=onDismiss,title={Text("Conectar ao SorriPreços")},text={Column(verticalArrangement=Arrangement.spacedBy(12.dp)){Text("Use o endereço da API. No teste por USB, a conexão local é preparada com ADB.",fontSize=13.sp);OutlinedTextField(value=value,onValueChange={value=it},label={Text("Endereço da API")},singleLine=true);Text("Somente o SorriPreços recebe suas buscas. Favoritos ficam neste aparelho.",fontSize=11.sp)}},confirmButton={TextButton(onClick={onSave(value.trim().trimEnd('/'))},enabled=valid){Text("Salvar conexão")}},dismissButton={TextButton(onClick=onDismiss){Text("Cancelar")}})
}
