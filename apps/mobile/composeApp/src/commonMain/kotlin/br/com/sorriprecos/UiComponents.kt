package br.com.sorriprecos

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable internal fun DataDisclosure(demo: Boolean, catalog: CatalogStatus, historical: Boolean = false) {
    if (demo) Notice("MODO DEMO · Preços fictícios para testar o app", demo = true)
    else if (historical && catalog.historicalProducts > 0)
        Notice("DADOS REAIS HISTÓRICOS · Procon-MT / março de 2026. Nenhuma oferta atual confirmada.", historical = true)
}

@Composable internal fun Notice(text: String, demo: Boolean = false, historical: Boolean = false) {
    val contrast = MaterialTheme.colorScheme.background == Color.Black
    val background = if (historical && !contrast) Color(0xFFFFF6E5) else MaterialTheme.colorScheme.primaryContainer
    val foreground = if (historical && !contrast) Color(0xFF775015) else MaterialTheme.colorScheme.onPrimaryContainer
    Surface(color = background, shape = RoundedCornerShape(12.dp), modifier = Modifier.fillMaxWidth()) {
        Row(Modifier.padding(horizontal = 12.dp, vertical = 9.dp), verticalAlignment = Alignment.CenterVertically) {
            AppIcon(if (historical) Glyph.Clock else Glyph.Info, Modifier.size(17.dp), foreground)
            Spacer(Modifier.width(9.dp))
            Text(text, fontSize = 10.sp, lineHeight = 14.sp, fontWeight = if (demo) FontWeight.SemiBold else FontWeight.Normal, color = foreground)
        }
    }
}

@Composable internal fun MarketRow(m: Supermarket, onClick: () -> Unit) {
    Surface(color = MaterialTheme.colorScheme.surface, shape = RoundedCornerShape(14.dp), border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant), modifier = Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(Modifier.padding(14.dp), verticalAlignment = Alignment.CenterVertically) {
            AppIcon(Glyph.Cart, Modifier.size(25.dp), MaterialTheme.colorScheme.primary); Spacer(Modifier.width(12.dp))
            Column(Modifier.weight(1f)) {
                Text(m.name, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                Text("${m.branchCount} filial(is) · ${if (m.productCount>0) age(m.lastCollectedAt) else "Coleta pendente"}", fontSize = 10.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 3.dp))
            }
            Text(if (m.isDemo) "DEMO" else if (m.integrationStatus == "authorized") "Integrado" else if (m.productCount>0) "Histórico" else "Pendente", fontSize = 10.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable internal fun ProductCard(p: Product, favorite: Boolean, onClick: () -> Unit, onFavorite: () -> Unit, offline: Boolean) {
    val current = p.lastCollectedAt?.let { platformNow()-it <= 172800 } == true && p.minPriceCents != null
    val unconfirmed = !current && p.hasUnconfirmedPrice()
    val stockOnly = !current && !unconfirmed && p.hasStockWithoutPrice()
    val historical = !current && !unconfirmed && !stockOnly && p.historicalBranchCount > 0
    val value = if (current) p.minPriceCents else if (unconfirmed) p.unconfirmedPriceCents else if (historical) p.referencePriceCents else null
    Surface(shape = RoundedCornerShape(16.dp), color = MaterialTheme.colorScheme.surface, shadowElevation = 1.dp,
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant), modifier = Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
            ProductArtwork(p)
            Column(Modifier.weight(1f)) {
                Text(p.name, fontSize = 16.sp, lineHeight = 19.sp, fontWeight = FontWeight.Bold, maxLines = 3, overflow = TextOverflow.Ellipsis)
                Column(Modifier.padding(top = 4.dp)) {
                    p.knownBrand?.let { Text("Marca: $it", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                    Text("Embalagem: ${p.pack}", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                Surface(color = if (current) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant, shape = RoundedCornerShape(11.dp), modifier = Modifier.fillMaxWidth().padding(top = 7.dp)) {
                    Column(Modifier.padding(horizontal = 10.dp, vertical = 6.dp)) {
                        Text(when { p.isDemo -> "MENOR PREÇO DEMO"; historical -> "REGISTRO HISTÓRICO · DESATUALIZADO"; unconfirmed -> "ENCARTE · ESTOQUE NÃO CONFIRMADO"; stockOnly -> "ESTOQUE CONFIRMADO PARA RETIRADA"; offline -> "REFERÊNCIA SALVA"; else -> "MENOR PREÇO CONHECIDO" }, fontSize = 8.sp, lineHeight = 11.sp, color = if (current) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant, fontWeight = FontWeight.SemiBold)
                        Text(if (stockOnly) "Preço a consultar" else money(value), fontSize = if (value!=null) 23.sp else 12.sp, fontWeight = FontWeight.ExtraBold, color = if (current) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface)
                    }
                }
                Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(top = 5.dp)) {
                    AppIcon(Glyph.Cart, Modifier.size(12.dp), MaterialTheme.colorScheme.onSurfaceVariant); Spacer(Modifier.width(4.dp))
                    Text(if (current) "${p.branchCount} filiais com preço" else if (unconfirmed) "Estoque não confirmado" else if (stockOnly) "Veja as filiais" else if (historical) "${p.historicalBranchCount} registros antigos" else "Sem preço recente", fontSize = 9.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                IconButton(onClick = onFavorite, modifier = Modifier.size(34.dp).semantics { contentDescription = if (favorite) "Remover favorito ${p.name}" else "Favoritar ${p.name}" }) { AppIcon(Glyph.Heart, Modifier.size(18.dp), if (favorite) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant, filled = favorite) }
                IconButton(onClick = onClick, modifier = Modifier.size(34.dp).semantics { contentDescription = "Comparar" }) { AppIcon(Glyph.Chevron, Modifier.size(17.dp)) }
            }
        }
    }
}

@Composable internal fun EmptyCard(title: String, body: String) {
    Surface(shape = RoundedCornerShape(16.dp), color = MaterialTheme.colorScheme.surface, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant), modifier = Modifier.fillMaxWidth()) {
        Column(Modifier.padding(18.dp)) { Text(title, fontSize = 17.sp, fontWeight = FontWeight.Bold); Text(body, fontSize = 13.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 7.dp)) }
    }
}

@Composable internal fun ErrorCard(message: String, retry: () -> Unit) {
    Column { Text(message, color = MaterialTheme.colorScheme.error, fontSize = 13.sp); TextButton(onClick = retry) { Text("Tentar novamente") } }
}

@Composable internal fun SettingsToggle(title: String, subtitle: String, checked: Boolean, onChange: (Boolean) -> Unit) {
    Surface(shape = RoundedCornerShape(16.dp), color = MaterialTheme.colorScheme.surface, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)) {
        Row(Modifier.fillMaxWidth().toggleable(value = checked, role = Role.Switch, onValueChange = onChange).semantics { contentDescription = title }.padding(16.dp), verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f).padding(end = 10.dp)) { Text(title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold); Text(subtitle, fontSize = 12.sp, lineHeight = 17.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 5.dp)) }
            Switch(checked = checked, onCheckedChange = null)
        }
    }
}

@Composable internal fun SettingsLink(title: String, subtitle: String, icon: Glyph, onClick: () -> Unit) {
    Surface(shape = RoundedCornerShape(16.dp), color = MaterialTheme.colorScheme.surface, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant), modifier = Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(Modifier.padding(16.dp), verticalAlignment = Alignment.CenterVertically) {
            AppIcon(icon, tint = MaterialTheme.colorScheme.primary); Spacer(Modifier.width(12.dp))
            Column(Modifier.weight(1f)) { Text(title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold); Text(subtitle, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 5.dp)) }
            AppIcon(Glyph.Chevron, Modifier.size(18.dp))
        }
    }
}
