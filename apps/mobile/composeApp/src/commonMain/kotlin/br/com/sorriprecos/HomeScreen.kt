package br.com.sorriprecos

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import br.com.sorriprecos.resources.*
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

@Composable internal fun HomeScreen(
    demo: Boolean, onSearch: (String) -> Unit, onExplore: () -> Unit, onCategory: (QuickCategory) -> Unit
) {
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val columns = if (maxWidth >= 365.dp && androidx.compose.ui.platform.LocalDensity.current.fontScale <= 1.15f) 4 else 2
        val rows = quickProducts.chunked(columns)
        LazyColumn(
            contentPadding = PaddingValues(start = 16.dp, top = 16.dp, end = 16.dp, bottom = 20.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            item {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    CartMark(45.dp)
                    Column {
                        Wordmark()
                        Text("Compare preços em Sorriso-MT", style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    FilledTonalButton(onClick = onExplore, modifier = Modifier.weight(1f)) {
                        AppIcon(Glyph.Store, Modifier.size(18.dp))
                        Spacer(Modifier.width(6.dp))
                        Text("Mercado")
                    }
                    FilledTonalButton(onClick = { onCategory(pharmacyCategory) }, modifier = Modifier.weight(1f)) {
                        AppIcon(Glyph.Pharmacy, Modifier.size(18.dp))
                        Spacer(Modifier.width(6.dp))
                        Text("Farmácia")
                    }
                }
            }
            item { HomeSearch(onExplore) }
            if (demo) item { Notice("MODO DEMO · Preços fictícios para testar o app", demo = true) }
            item {
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    Text("Produtos do dia a dia", Modifier.weight(1f), style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.Bold)
                    TextButton(onClick = onExplore, contentPadding = PaddingValues(horizontal = 7.dp)) {
                        Text("Explorar")
                        AppIcon(Glyph.Chevron, Modifier.size(15.dp), MaterialTheme.colorScheme.primary)
                    }
                }
            }
            items(rows.size) { index ->
                Row(Modifier.fillMaxWidth().height(IntrinsicSize.Min), horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                    rows[index].forEach { product ->
                        QuickProductCard(product, Modifier.weight(1f).fillMaxHeight()) { onSearch(product.query) }
                    }
                }
            }
            item {
                Text("Outras categorias", modifier = Modifier.padding(top = 7.dp),
                    style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
            }
            item {
                val categories = quickCategories + QuickCategory("Todos", Glyph.Grid, emptyList())
                Column(verticalArrangement = Arrangement.spacedBy(7.dp)) {
                    categories.chunked(if (columns == 4) 6 else 3).forEach { row ->
                        Row(Modifier.fillMaxWidth().height(IntrinsicSize.Min), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                            row.forEach { category ->
                                Surface(
                                    onClick = { if (category.terms.isEmpty()) onExplore() else onCategory(category) },
                                    modifier = Modifier.weight(1f).fillMaxHeight(), shape = RoundedCornerShape(13.dp),
                                    color = MaterialTheme.colorScheme.primaryContainer
                                ) {
                                    Column(Modifier.padding(horizontal = 2.dp, vertical = 8.dp),
                                        horizontalAlignment = Alignment.CenterHorizontally) {
                                        AppIcon(category.icon, Modifier.size(21.dp), MaterialTheme.colorScheme.primary)
                                        Text(category.label, Modifier.padding(top = 4.dp), fontSize = 10.sp,
                                            lineHeight = 12.sp, textAlign = TextAlign.Center)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable private fun QuickProductCard(product: QuickProduct, modifier: Modifier, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Surface(onClick = onClick, modifier = modifier, shape = RoundedCornerShape(16.dp),
        color = colors.surface, border = BorderStroke(1.dp, colors.outlineVariant)) {
        Column(
            Modifier.padding(horizontal = 5.dp, vertical = 7.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Image(painterResource(product.image), contentDescription = null,
                modifier = Modifier.fillMaxWidth().height(64.dp), contentScale = ContentScale.Fit)
            Text(product.name, Modifier.weight(1f).padding(top = 3.dp), fontWeight = FontWeight.SemiBold,
                fontSize = 12.sp, lineHeight = 14.sp, textAlign = TextAlign.Center)
            Surface(color = colors.primaryContainer, shape = RoundedCornerShape(16.dp),
                modifier = Modifier.padding(top = 4.dp)) {
                Text(product.pack, Modifier.padding(horizontal = 7.dp, vertical = 2.dp),
                    color = colors.onPrimaryContainer, fontSize = 10.sp, lineHeight = 12.sp)
            }
        }
    }
}
