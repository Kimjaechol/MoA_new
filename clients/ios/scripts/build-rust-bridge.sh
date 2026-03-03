#!/usr/bin/env bash
set -euo pipefail

# Build the ZeroClaw iOS bridge (Rust static library → universal binary)
#
# Targets:
#   aarch64-apple-ios       — iPhone / iPad (ARM64)
#   aarch64-apple-ios-sim   — Simulator on Apple Silicon
#   x86_64-apple-ios        — Simulator on Intel Mac
#
# Output:
#   ../lib/libzeroclaw_ios.a  (universal binary combining all targets)
#
# Prerequisites:
#   - Rust toolchain with iOS targets:
#     rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
#   - Xcode with iOS SDK installed

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IOS_DIR="$(dirname "$SCRIPT_DIR")"
BRIDGE_DIR="$IOS_DIR/../ios-bridge"
OUTPUT_DIR="$IOS_DIR/lib"

# Ensure output directory exists
mkdir -p "$OUTPUT_DIR"

echo "=== Building ZeroClaw iOS Bridge ==="
echo "Bridge source: $BRIDGE_DIR"
echo "Output:        $OUTPUT_DIR/libzeroclaw_ios.a"

# Ensure Rust targets are installed
for target in aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios; do
    if ! rustup target list --installed | grep -q "$target"; then
        echo "Installing Rust target: $target"
        rustup target add "$target"
    fi
done

# Build for each target
echo ""
echo "--- Building for aarch64-apple-ios (iPhone) ---"
cargo build --manifest-path "$BRIDGE_DIR/Cargo.toml" --release --target aarch64-apple-ios

echo ""
echo "--- Building for aarch64-apple-ios-sim (Simulator ARM64) ---"
cargo build --manifest-path "$BRIDGE_DIR/Cargo.toml" --release --target aarch64-apple-ios-sim

echo ""
echo "--- Building for x86_64-apple-ios (Simulator Intel) ---"
cargo build --manifest-path "$BRIDGE_DIR/Cargo.toml" --release --target x86_64-apple-ios

# Determine cargo target dir
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$BRIDGE_DIR/target}"

# Create universal binary for simulator (ARM64 + Intel)
echo ""
echo "--- Creating universal simulator binary ---"
lipo -create \
    "$CARGO_TARGET_DIR/aarch64-apple-ios-sim/release/libzeroclaw_ios.a" \
    "$CARGO_TARGET_DIR/x86_64-apple-ios/release/libzeroclaw_ios.a" \
    -output "$OUTPUT_DIR/libzeroclaw_ios_sim.a"

# Copy device binary
cp "$CARGO_TARGET_DIR/aarch64-apple-ios/release/libzeroclaw_ios.a" \
   "$OUTPUT_DIR/libzeroclaw_ios_device.a"

# Create XCFramework (preferred for modern Xcode)
echo ""
echo "--- Creating XCFramework ---"
rm -rf "$OUTPUT_DIR/ZeroClawBridge.xcframework"
xcodebuild -create-xcframework \
    -library "$OUTPUT_DIR/libzeroclaw_ios_device.a" \
    -headers "$BRIDGE_DIR/include" \
    -library "$OUTPUT_DIR/libzeroclaw_ios_sim.a" \
    -headers "$BRIDGE_DIR/include" \
    -output "$OUTPUT_DIR/ZeroClawBridge.xcframework"

# Also copy a single device .a for direct linking (simpler Xcode setup)
cp "$OUTPUT_DIR/libzeroclaw_ios_device.a" "$OUTPUT_DIR/libzeroclaw_ios.a"

echo ""
echo "=== Build complete ==="
echo "XCFramework: $OUTPUT_DIR/ZeroClawBridge.xcframework"
echo "Static lib:  $OUTPUT_DIR/libzeroclaw_ios.a (device only)"
echo ""
echo "To use in Xcode:"
echo "  1. Drag ZeroClawBridge.xcframework into your project"
echo "  2. Or add libzeroclaw_ios.a to 'Link Binary With Libraries'"
echo "  3. Ensure the bridging header imports zeroclaw_bridge.h"
