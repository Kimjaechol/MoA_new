# Tauri WebView bridge — keep JNI and reflection targets
-keep class com.moa.agent.** { *; }
-keep class app.tauri.** { *; }

# Keep WebView JavaScript interface
-keepclassmembers class * {
    @android.webkit.JavascriptInterface <methods>;
}

# AndroidX / Material
-dontwarn com.google.android.material.**
-keep class com.google.android.material.** { *; }
-dontwarn androidx.**
-keep class androidx.** { *; }
-keep interface androidx.** { *; }

# Kotlin metadata
-dontwarn kotlin.**
-keep class kotlin.Metadata { *; }
