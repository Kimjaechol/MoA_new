#!/usr/bin/env bash
# MoA Mobile Build Setup Script
#
# Prerequisites:
#   - Rust toolchain (rustup)
#   - Node.js 18+ and npm
#   - Android: Android Studio, SDK 34+, NDK 26+
#   - iOS: Xcode 15+, CocoaPods
#
# Usage:
#   ./scripts/mobile-setup.sh android   # Set up Android build
#   ./scripts/mobile-setup.sh ios       # Set up iOS build
#   ./scripts/mobile-setup.sh all       # Set up both platforms

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=== MoA Mobile Build Setup ==="
echo "Root: $ROOT_DIR"
echo ""

# ── Install Rust targets ────────────────────────────────────────

setup_rust_targets() {
    echo "--- Installing Rust cross-compilation targets ---"

    if [[ "$1" == "android" || "$1" == "all" ]]; then
        echo "  Android targets:"
        rustup target add aarch64-linux-android   # ARM64 (most modern phones)
        rustup target add armv7-linux-androideabi  # ARM32 (older devices)
        rustup target add i686-linux-android       # x86 (emulator)
        rustup target add x86_64-linux-android     # x86_64 (emulator)
        echo "  Done."
    fi

    if [[ "$1" == "ios" || "$1" == "all" ]]; then
        echo "  iOS targets:"
        rustup target add aarch64-apple-ios          # iPhone (ARM64)
        rustup target add aarch64-apple-ios-sim      # Simulator (Apple Silicon)
        rustup target add x86_64-apple-ios           # Simulator (Intel)
        echo "  Done."
    fi
}

# ── Install Node dependencies ───────────────────────────────────

install_node_deps() {
    echo "--- Installing Node dependencies ---"
    cd "$ROOT_DIR"
    npm install
    echo "  Done."
}

# ── Android setup ────────────────────────────────────────────────

setup_android() {
    echo "--- Android Setup ---"

    # Check ANDROID_HOME
    if [[ -z "${ANDROID_HOME:-}" ]]; then
        echo "  WARNING: ANDROID_HOME not set."
        echo "  Please set it to your Android SDK path, e.g.:"
        echo "    export ANDROID_HOME=\$HOME/Android/Sdk"
        echo "    export NDK_HOME=\$ANDROID_HOME/ndk/26.1.10909125"
        echo ""
    else
        echo "  ANDROID_HOME: $ANDROID_HOME"
    fi

    # Initialize Tauri Android project if not already done
    if [[ ! -f "$ROOT_DIR/src-tauri/gen/android/app/tauri.build.gradle.kts" ]]; then
        echo "  Running: tauri android init"
        cd "$ROOT_DIR"
        npx tauri android init
    else
        echo "  Android project already initialized."
    fi

    echo ""
    echo "  Build commands:"
    echo "    npm run tauri:android          # Dev build + run on device"
    echo "    npx tauri android build        # Release APK/AAB"
    echo ""
    echo "  Android setup complete."
}

# ── iOS setup ────────────────────────────────────────────────────

setup_ios() {
    echo "--- iOS Setup ---"

    # Check Xcode
    if ! command -v xcodebuild &>/dev/null; then
        echo "  ERROR: Xcode not found. Install Xcode from the App Store."
        exit 1
    fi

    xcode_version=$(xcodebuild -version | head -1)
    echo "  $xcode_version"

    # Initialize Tauri iOS project if not already done
    if [[ ! -d "$ROOT_DIR/src-tauri/gen/apple/MoA - Master of AI.xcodeproj" ]]; then
        echo "  Running: tauri ios init"
        cd "$ROOT_DIR"
        npx tauri ios init
    else
        echo "  iOS project already initialized."
    fi

    echo ""
    echo "  IMPORTANT: Set your Apple Development Team ID in:"
    echo "    src-tauri/tauri.conf.json -> bundle.iOS.developmentTeam"
    echo ""
    echo "  Build commands:"
    echo "    npm run tauri:ios              # Dev build + run on simulator"
    echo "    npx tauri ios build            # Release IPA"
    echo ""
    echo "  iOS setup complete."
}

# ── Main ─────────────────────────────────────────────────────────

platform="${1:-all}"

case "$platform" in
    android)
        setup_rust_targets android
        install_node_deps
        setup_android
        ;;
    ios)
        setup_rust_targets ios
        install_node_deps
        setup_ios
        ;;
    all)
        setup_rust_targets all
        install_node_deps
        setup_android
        setup_ios
        ;;
    *)
        echo "Usage: $0 {android|ios|all}"
        exit 1
        ;;
esac

echo ""
echo "=== Mobile setup complete ==="
