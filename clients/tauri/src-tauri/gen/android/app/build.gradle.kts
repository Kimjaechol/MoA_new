import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

android {
    compileSdk = 34
    namespace = "com.moa.agent"

    defaultConfig {
        applicationId = "com.moa.agent"
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        getByName("debug") {
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {
                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            isMinifyEnabled = true
            proguardFiles(
                *fileTree("proguard") {
                    include("*.pro")
                }.toList().toTypedArray()
            )
        }
    }

    kotlinOptions {
        jvmTarget = "1.8"
    }

    buildFeatures {
        buildConfig = true
    }

    // Tauri uses product flavors with an "abi" dimension to generate
    // per-architecture and universal APKs. The Tauri CLI expects tasks
    // like assembleUniversalRelease, which require product flavors
    // (not splits.abi). The RustPlugin.kt in buildSrc/ registers these
    // flavors automatically when the project is properly initialized
    // via `npx tauri android init`.
    flavorDimensions += "abi"
    productFlavors {
        create("universal") {
            dimension = "abi"
            ndk {
                abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86", "x86_64")
            }
        }
        create("arm64") {
            dimension = "abi"
            ndk {
                abiFilters += listOf("arm64-v8a")
            }
        }
        create("arm") {
            dimension = "abi"
            ndk {
                abiFilters += listOf("armeabi-v7a")
            }
        }
        create("x86") {
            dimension = "abi"
            ndk {
                abiFilters += listOf("x86")
            }
        }
        create("x86_64") {
            dimension = "abi"
            ndk {
                abiFilters += listOf("x86_64")
            }
        }
    }
}

dependencies {
    implementation("androidx.webkit:webkit:1.8.0")
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("com.google.android.material:material:1.12.0")
}

// Apply Tauri-generated Gradle build script.
apply(from = "tauri.build.gradle.kts")
