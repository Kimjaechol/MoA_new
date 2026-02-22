import SwiftUI
import Tauri
import WebKit

/// Main iOS/macOS application entry point for MoA (Master of AI).
///
/// Uses Tauri 2's Swift bindings to host the web-based frontend in a
/// native WKWebView. All business logic runs in the Rust backend via
/// Tauri's command bridge.
@main
struct MoAApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

/// Content view that hosts the Tauri WebView.
struct ContentView: View {
    var body: some View {
        TauriWebView()
            .ignoresSafeArea()
    }
}

/// Wrapper for the Tauri-managed WKWebView.
struct TauriWebView: UIViewRepresentable {
    func makeUIView(context: Context) -> WKWebView {
        let webView = WKWebView()
        webView.scrollView.bounces = false
        webView.isOpaque = false
        webView.backgroundColor = .clear
        // Tauri will inject its bridge and load the frontend.
        return webView
    }

    func updateUIView(_ webView: WKWebView, context: Context) {}
}
