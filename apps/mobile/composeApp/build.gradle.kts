import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    kotlin("multiplatform")
    kotlin("plugin.serialization")
    id("org.jetbrains.kotlin.plugin.compose")
    id("org.jetbrains.compose")
    id("com.android.application")
}
kotlin {
    androidTarget { compilerOptions { jvmTarget.set(JvmTarget.JVM_17) } }
    listOf(iosArm64(),iosSimulatorArm64()).forEach {
        it.binaries.framework { baseName="SorriPrecos"; isStatic=true }
    }
    sourceSets {
        commonMain.dependencies {
            implementation("org.jetbrains.compose.components:components-resources:1.10.3")
            implementation("org.jetbrains.compose.runtime:runtime:1.11.1")
            implementation("org.jetbrains.compose.foundation:foundation:1.11.1")
            implementation("org.jetbrains.compose.material3:material3:1.9.0")
            implementation("org.jetbrains.compose.material:material-icons-extended:1.7.3")
            implementation("org.jetbrains.compose.ui:ui:1.11.1")
            implementation("io.ktor:ktor-client-core:3.5.2")
            implementation("io.coil-kt.coil3:coil-compose:3.4.0")
            implementation("io.coil-kt.coil3:coil-network-ktor3:3.4.0")
            implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.11.0")
            implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.11.0")
        }
        androidMain.dependencies {
            implementation("androidx.activity:activity-compose:1.13.0")
            implementation("io.ktor:ktor-client-cio:3.5.2")
        }
        iosMain.dependencies { implementation("io.ktor:ktor-client-darwin:3.5.2") }
        commonTest.dependencies {
            implementation(kotlin("test"))
            implementation("io.ktor:ktor-client-mock:3.5.2")
            implementation("org.jetbrains.kotlinx:kotlinx-coroutines-test:1.11.0")
        }
    }
}
android {
    namespace="br.com.sorriprecos"
    compileSdk=36
    defaultConfig {applicationId="br.com.sorriprecos";minSdk=26;targetSdk=36;versionCode=1;versionName="0.1.0"}
    compileOptions {sourceCompatibility=JavaVersion.VERSION_17;targetCompatibility=JavaVersion.VERSION_17}
    buildFeatures {compose=true}
    packaging {resources.excludes += "/META-INF/{AL2.0,LGPL2.1}"}
}

compose.resources {
    packageOfResClass = "br.com.sorriprecos.resources"
    generateResClass = always
}
