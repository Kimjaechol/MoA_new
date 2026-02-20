package com.moa.agent

/**
 * Main Android Activity for MoA (Master of AI).
 *
 * This extends Tauri's TauriActivity which hosts the WebView and bridges
 * Rust commands to the Android platform. The Rust library (moa_agent_lib)
 * is loaded automatically by Tauri's mobile entry point.
 */
class MainActivity : TauriActivity()
