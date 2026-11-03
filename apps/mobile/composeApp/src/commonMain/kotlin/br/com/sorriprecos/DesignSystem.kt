package br.com.sorriprecos

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.withTransform
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.sin

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

internal enum class Glyph { Cart, Search, Home, Heart, Settings, Back, Chevron, Close, Refresh, Contrast, Pin, Clock, Info, Check }

@Composable internal fun AppIcon(glyph: Glyph, modifier: Modifier = Modifier, tint: Color = MaterialTheme.colorScheme.onSurface, filled: Boolean = false) {
    Canvas(modifier.size(24.dp)) {
        withTransform({ scale(size.width / 24f, size.height / 24f, pivot = Offset.Zero) }) {
            val stroke = Stroke(1.8f, cap = StrokeCap.Round)
            fun line(x: Float, y: Float, x2: Float, y2: Float) = drawLine(tint, Offset(x,y), Offset(x2,y2), 1.8f, StrokeCap.Round)
            fun path(vararg points: Pair<Float,Float>) {
                drawPath(Path().apply { points.forEachIndexed { i,p -> if(i==0)moveTo(p.first,p.second) else lineTo(p.first,p.second) } }, tint, style=stroke)
            }
            when(glyph) {
                Glyph.Cart -> { path(2f to 3f,5f to 3f,8f to 16f,19f to 16f,22f to 7f,6f to 7f); drawCircle(tint,1.5f,Offset(9f,21f));drawCircle(tint,1.5f,Offset(19f,21f)) }
                Glyph.Search -> { drawCircle(tint,7f,Offset(10f,10f),style=stroke);line(15f,15f,22f,22f) }
                Glyph.Home -> { val p=Path().apply {moveTo(3f,11f);lineTo(12f,3f);lineTo(21f,11f);lineTo(21f,22f);lineTo(15f,22f);lineTo(15f,15f);lineTo(9f,15f);lineTo(9f,22f);lineTo(3f,22f);close()};drawPath(p,tint,style=if(filled) androidx.compose.ui.graphics.drawscope.Fill else stroke) }
                Glyph.Heart -> { val p=Path().apply {moveTo(12f,21f);cubicTo(7f,17f,2f,13f,2f,8f);cubicTo(2f,2f,9f,1f,12f,6f);cubicTo(15f,1f,22f,2f,22f,8f);cubicTo(22f,13f,17f,17f,12f,21f);close()};drawPath(p,tint,style=if(filled) androidx.compose.ui.graphics.drawscope.Fill else stroke) }
                Glyph.Settings -> {drawCircle(tint,7f,Offset(12f,12f),style=stroke);drawCircle(tint,2.5f,Offset(12f,12f),style=stroke);repeat(8){val a=it*PI/4;line((12+8*cos(a)).toFloat(),(12+8*sin(a)).toFloat(),(12+10.5*cos(a)).toFloat(),(12+10.5*sin(a)).toFloat())}}
                Glyph.Back -> {path(11f to 4f,3f to 12f,11f to 20f);line(3f,12f,22f,12f)}
                Glyph.Chevron -> path(9f to 5f,16f to 12f,9f to 19f)
                Glyph.Close -> {line(5f,5f,19f,19f);line(19f,5f,5f,19f)}
                Glyph.Refresh -> {drawArc(tint,40f,280f,false,Offset(3f,3f),Size(18f,18f),style=stroke);path(21f to 3f,21f to 10f,14f to 10f)}
                Glyph.Contrast -> {drawCircle(tint,9f,Offset(12f,12f),style=stroke);drawArc(tint,90f,180f,true,Offset(3f,3f),Size(18f,18f))}
                Glyph.Pin -> {val p=Path().apply {moveTo(12f,22f);cubicTo(9f,17f,4f,12f,4f,8f);cubicTo(4f,-2f,20f,-2f,20f,8f);cubicTo(20f,12f,15f,17f,12f,22f);close()};drawPath(p,tint,style=stroke);drawCircle(tint,2.5f,Offset(12f,8f),style=stroke)}
                Glyph.Clock -> {drawCircle(tint,9f,Offset(12f,12f),style=stroke);path(12f to 6f,12f to 12f,16f to 14f)}
                Glyph.Info -> {drawCircle(tint,9f,Offset(12f,12f),style=stroke);line(12f,11f,12f,17f);drawCircle(tint,1f,Offset(12f,7f))}
                Glyph.Check -> path(4f to 12f,9f to 17f,20f to 6f)
            }
        }
    }
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

/** Category illustrations, not retailer photographs or a representation of an exact package. */
@Composable internal fun ProductArtwork(product: Product, modifier: Modifier = Modifier) {
    val name=product.name.lowercase()
    val egg=name.contains("ovo") && (name.contains("pascoa") || name.contains("páscoa"))
    val chocolate=egg || name.contains("chocolate")
    val fish=name.contains("bacalhau") || name.contains("tilápia") || name.contains("tilapia")
    val accent=when {chocolate->Color(0xFF6E4B9A);fish->Color(0xFF3185A7);name.contains("café") || name.contains("cafe")->Color(0xFF965B36);else->Color(0xFFC7433B)}
    Column(modifier.width(76.dp),horizontalAlignment=Alignment.CenterHorizontally) {
        Canvas(Modifier.fillMaxWidth().height(93.dp).clip(RoundedCornerShape(10.dp)).background(MaterialTheme.colorScheme.surfaceVariant).semantics{contentDescription="Ilustração de categoria. Foto do produto não disponível."}) {
            withTransform({scale(size.width/76f,size.height/93f,pivot=Offset.Zero)}) {
                drawOval(Color.Black.copy(alpha=.08f),Offset(13f,81f),Size(52f,6f))
                if(egg) {
                    val p=Path().apply {moveTo(38f,6f);cubicTo(16f,10f,9f,57f,15f,70f);cubicTo(22f,89f,55f,89f,61f,70f);cubicTo(67f,57f,60f,10f,38f,6f);close()}
                    drawPath(p,Brush.linearGradient(listOf(accent,accent.copy(alpha=.6f)),Offset(15f,30f),Offset(63f,70f)))
                    drawOval(Color.White.copy(alpha=.9f),Offset(20f,39f),Size(36f,24f))
                    drawOval(Color(0xFFBA863E),Offset(30f,42f),Size(17f,19f))
                    drawLine(Color(0xFFF7DA97),Offset(16f,70f),Offset(59f,70f),3f)
                    drawOval(Color.White.copy(alpha=.18f),Offset(23f,16f),Size(7f,18f))
                } else {
                    val p=Path().apply {moveTo(15f,8f);lineTo(61f,8f);lineTo(65f,81f);quadraticTo(38f,87f,11f,81f);close()}
                    drawPath(p,Color.White);drawPath(p,Color(0xFFD1D6DC),style=Stroke(1f))
                    drawRoundRect(accent,Offset(15f,8f),Size(46f,24f),androidx.compose.ui.geometry.CornerRadius(3f))
                    drawLine(accent,Offset(12f,77f),Offset(64f,77f),6f)
                    if(fish) {
                        drawOval(accent,Offset(25f,43f),Size(25f,14f));drawPath(Path().apply{moveTo(49f,50f);lineTo(59f,43f);lineTo(59f,57f);close()},accent);drawCircle(Color.White,1.5f,Offset(30f,48f))
                    } else {
                        drawOval(Color(0xFFEDE5C9),Offset(22f,40f),Size(32f,25f))
                        repeat(7) { i->drawOval(Color(0xFFD2B974),Offset(26f+(i%3)*8f,44f+(i/3)*6f),Size(6f,3f)) }
                    }
                    drawLine(Color.White.copy(alpha=.35f),Offset(18f,11f),Offset(18f,28f),2f)
                }
            }
        }
        Text("Ilustração",fontSize=8.sp,color=MaterialTheme.colorScheme.onSurfaceVariant,textAlign=TextAlign.Center,modifier=Modifier.padding(top=3.dp))
    }
}
