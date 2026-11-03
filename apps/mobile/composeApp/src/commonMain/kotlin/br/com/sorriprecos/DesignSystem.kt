package br.com.sorriprecos

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.outlined.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.layout.ContentScale
import coil3.compose.AsyncImage
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

internal val BrandGreen = Color(0xFF087D13)
internal val BrandNavy = Color(0xFF082A50)
internal val AppLight = lightColorScheme(
    primary = BrandGreen, onPrimary = Color.White,
    primaryContainer = Color(0xFFE3F9E4), onPrimaryContainer = Color(0xFF076511),
    secondaryContainer = Color(0xFFE9F8EA), onSecondaryContainer = BrandGreen,
    background = Color(0xFFFAFBFD), onBackground = Color(0xFF101724),
    surface = Color.White, onSurface = Color(0xFF101724),
    surfaceVariant = Color(0xFFF1F3F7), onSurfaceVariant = Color(0xFF566174),
    outline = Color(0xFF758093), outlineVariant = Color(0xFFE6E9EF)
)
internal val AppContrast = darkColorScheme(
    primary = Color(0xFF9CF29F), onPrimary = Color.Black,
    primaryContainer = Color(0xFF173C1B), onPrimaryContainer = Color.White,
    secondaryContainer = Color(0xFF193B1D), onSecondaryContainer = Color.White,
    background = Color.Black, onBackground = Color.White,
    surface = Color(0xFF101510), onSurface = Color.White,
    surfaceVariant = Color(0xFF202820), onSurfaceVariant = Color(0xFFE8F0E8),
    outline = Color(0xFFC6D2C6), outlineVariant = Color(0xFF7D907D)
)

internal enum class Glyph {
    Cart, Search, Home, Heart, Settings, Back, Chevron, Close, Refresh, Contrast, Pin, Clock, Info, Check, Calendar, Document, Link, External, Copy, Star, Bottle, Bread, Spray, Pump, Paw, Grid, Add, Remove, Store, Package, Collapse, Pharmacy
}

@Composable internal fun AppIcon(
    glyph: Glyph, modifier: Modifier = Modifier.size(24.dp),
    tint: Color = MaterialTheme.colorScheme.onSurface, filled: Boolean = false
) {
    val icon = when (glyph) {
        Glyph.Cart -> Icons.Outlined.ShoppingCart
        Glyph.Search -> Icons.Outlined.Search
        Glyph.Home -> Icons.Outlined.Home
        Glyph.Heart -> if (filled) Icons.Filled.Favorite else Icons.Outlined.FavoriteBorder
        Glyph.Settings -> Icons.Outlined.Settings
        Glyph.Back -> Icons.AutoMirrored.Outlined.ArrowBack
        Glyph.Chevron -> Icons.Outlined.ChevronRight
        Glyph.Close -> Icons.Outlined.Close
        Glyph.Refresh -> Icons.Outlined.Refresh
        Glyph.Contrast -> Icons.Outlined.Contrast
        Glyph.Pin -> Icons.Outlined.LocationOn
        Glyph.Clock -> Icons.Outlined.Schedule
        Glyph.Info -> Icons.Outlined.Info
        Glyph.Check -> Icons.Outlined.Check
        Glyph.Calendar -> Icons.Outlined.CalendarToday
        Glyph.Document -> Icons.Outlined.Description
        Glyph.Link -> Icons.Outlined.Link
        Glyph.External -> Icons.AutoMirrored.Outlined.OpenInNew
        Glyph.Copy -> Icons.Outlined.ContentCopy
        Glyph.Star -> Icons.Outlined.StarBorder
        Glyph.Bottle -> Icons.Outlined.LocalDrink
        Glyph.Bread -> Icons.Outlined.BakeryDining
        Glyph.Spray -> Icons.Outlined.CleaningServices
        Glyph.Pump -> Icons.Outlined.Soap
        Glyph.Paw -> Icons.Outlined.Pets
        Glyph.Grid -> Icons.Outlined.GridView
        Glyph.Add -> Icons.Outlined.Add
        Glyph.Remove -> Icons.Outlined.Remove
        Glyph.Store -> Icons.Outlined.Storefront
        Glyph.Package -> Icons.Outlined.Inventory2
        Glyph.Pharmacy -> Icons.Outlined.LocalPharmacy
        Glyph.Collapse -> Icons.Outlined.ExpandLess
    }
    Icon(icon, contentDescription = null, modifier = modifier, tint = tint)
}

@Composable internal fun CartMark(size: Dp = 52.dp) {
    Box(Modifier.size(size).clip(RoundedCornerShape(size*.27f)).background(Brush.linearGradient(listOf(Color(0xFF08CD28),Color(0xFF00780C)))),contentAlignment=Alignment.Center) {
        AppIcon(Glyph.Cart,Modifier.size(size*.66f),Color.White)
    }
}

@Composable internal fun Wordmark() {
    val navy = if(MaterialTheme.colorScheme.background == Color.Black) Color.White else BrandNavy
    Text(buildAnnotatedString {withStyle(SpanStyle(color=navy)){append("Sorri")};withStyle(SpanStyle(color=MaterialTheme.colorScheme.primary)){append("Preços")}},fontSize=32.sp,fontWeight=FontWeight.ExtraBold,letterSpacing=(-1.2).sp)
}


internal val LocalPhotoOrigin = staticCompositionLocalOf { "" }

@Composable internal fun ProductArtwork(product: Product, modifier: Modifier = Modifier.padding(end = 10.dp).size(width = 76.dp, height = 93.dp).clip(RoundedCornerShape(10.dp))) {
    val path = product.photoPath ?: return
    if (path != "/v1/products/${product.id}/photo") return
    AsyncImage(
        model = LocalPhotoOrigin.current.trimEnd('/') + path,
        contentDescription = "Foto de ${product.name}",
        contentScale = ContentScale.Fit,
        modifier = modifier
    )
}
