#!/usr/bin/env bash
set -euo pipefail

# Xcode Run Script Build Phase for ZeroClaw Rust Bridge
#
# Add this as a "Run Script" build phase in Xcode BEFORE "Compile Sources":
#   bash "${SRCROOT}/scripts/xcode-build-phase.sh"
#
# This script builds the Rust static library for the current Xcode target platform.

BRIDGE_DIR="${SRCROOT}/../ios-bridge"
LIB_OUTPUT_DIR="${SRCROOT}/lib"

mkdir -p "$LIB_OUTPUT_DIR"

# Determine Rust target from Xcode environment
case "$PLATFORM_NAME" in
    iphoneos)
        RUST_TARGET="aarch64-apple-ios"
        ;;
    iphonesimulator)
        if [ "$NATIVE_ARCH" = "arm64" ] || [ "${ARCHS}" = "arm64" ]; then
            RUST_TARGET="aarch64-apple-ios-sim"
        else
            RUST_TARGET="x86_64-apple-ios"
        fi
        ;;
    *)
        echo "warning: Unsupported platform: $PLATFORM_NAME"
        exit 0
        ;;
esac

# Ensure target is installed
if ! rustup target list --installed 2>/dev/null | grep -q "$RUST_TARGET"; then
    echo "Installing Rust target: $RUST_TARGET"
    rustup target add "$RUST_TARGET"
fi

# Build
echo "Building zeroclaw-ios-bridge for $RUST_TARGET (${CONFIGURATION})"

CARGO_PROFILE="release"
if [ "${CONFIGURATION}" = "Debug" ]; then
    CARGO_PROFILE="debug"
    cargo build --manifest-path "$BRIDGE_DIR/Cargo.toml" --target "$RUST_TARGET"
else
    cargo build --manifest-path "$BRIDGE_DIR/Cargo.toml" --target "$RUST_TARGET" --release
fi

# Determine target dir
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$BRIDGE_DIR/target}"

# Copy the built library
cp "$CARGO_TARGET_DIR/$RUST_TARGET/$CARGO_PROFILE/libzeroclaw_ios.a" \
   "$LIB_OUTPUT_DIR/libzeroclaw_ios.a"

echo "Rust bridge built: $LIB_OUTPUT_DIR/libzeroclaw_ios.a ($RUST_TARGET, $CARGO_PROFILE)"
