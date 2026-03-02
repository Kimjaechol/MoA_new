#!/usr/bin/env bash
# build-tauri.sh — Build ZeroClaw + MoA Tauri app with sidecar bundling.
#
# Usage:
#   ./scripts/build-tauri.sh              # Build for current platform
#   ./scripts/build-tauri.sh --debug      # Debug build (faster)
#   ./scripts/build-tauri.sh --target aarch64-unknown-linux-gnu
#
# This script:
# 1. Builds the ZeroClaw binary (cargo build)
# 2. Copies it to the Tauri sidecar location with correct target-triple suffix
# 3. Runs the Tauri build (npm run tauri build)
#
# Requirements:
# - Rust toolchain (cargo)
# - Node.js + npm
# - Tauri CLI (npm install)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TAURI_DIR="${REPO_ROOT}/clients/tauri"
SIDECAR_DIR="${TAURI_DIR}/src-tauri/binaries"

# Parse arguments
BUILD_MODE="release"
CARGO_FLAGS="--release"
TARGET_TRIPLE=""
SKIP_FRONTEND=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --debug)
            BUILD_MODE="debug"
            CARGO_FLAGS=""
            shift
            ;;
        --target)
            TARGET_TRIPLE="$2"
            shift 2
            ;;
        --skip-frontend)
            SKIP_FRONTEND="1"
            shift
            ;;
        *)
            echo "Unknown argument: $1"
            echo "Usage: $0 [--debug] [--target <triple>] [--skip-frontend]"
            exit 1
            ;;
    esac
done

# Detect target triple if not specified
if [[ -z "$TARGET_TRIPLE" ]]; then
    TARGET_TRIPLE="$(rustc -vV | grep '^host:' | awk '{print $2}')"
fi

echo "==> Building ZeroClaw for ${TARGET_TRIPLE} (${BUILD_MODE})"

# Step 1: Build ZeroClaw binary
cd "${REPO_ROOT}"

TARGET_FLAG=""
if [[ -n "$TARGET_TRIPLE" ]]; then
    TARGET_FLAG="--target ${TARGET_TRIPLE}"
fi

# shellcheck disable=SC2086
cargo build ${CARGO_FLAGS} ${TARGET_FLAG}

# Step 2: Locate the built binary
if [[ -n "$TARGET_FLAG" ]]; then
    BINARY_PATH="${REPO_ROOT}/target/${TARGET_TRIPLE}/${BUILD_MODE}/zeroclaw"
else
    BINARY_PATH="${REPO_ROOT}/target/${BUILD_MODE}/zeroclaw"
fi

# On Windows, the binary has .exe extension
if [[ "$TARGET_TRIPLE" == *"windows"* ]]; then
    BINARY_PATH="${BINARY_PATH}.exe"
fi

if [[ ! -f "$BINARY_PATH" ]]; then
    echo "ERROR: ZeroClaw binary not found at ${BINARY_PATH}"
    exit 1
fi

BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
echo "==> ZeroClaw binary built: ${BINARY_PATH} (${BINARY_SIZE})"

# Step 3: Copy to Tauri sidecar location
# Tauri sidecar naming convention: <name>-<target-triple>[.exe]
mkdir -p "${SIDECAR_DIR}"

SIDECAR_NAME="zeroclaw-${TARGET_TRIPLE}"
if [[ "$TARGET_TRIPLE" == *"windows"* ]]; then
    SIDECAR_NAME="${SIDECAR_NAME}.exe"
fi

cp "${BINARY_PATH}" "${SIDECAR_DIR}/${SIDECAR_NAME}"
echo "==> Sidecar copied to: ${SIDECAR_DIR}/${SIDECAR_NAME}"

# Step 4: Build the Tauri app
if [[ -n "$SKIP_FRONTEND" ]]; then
    echo "==> Skipping Tauri frontend build (--skip-frontend)"
else
    echo "==> Building MoA Tauri app..."
    cd "${TAURI_DIR}"

    # Install frontend dependencies if needed
    if [[ ! -d "node_modules" ]]; then
        echo "==> Installing npm dependencies..."
        npm install
    fi

    # Build Tauri app
    if [[ "$BUILD_MODE" == "debug" ]]; then
        npx tauri build --debug
    else
        npx tauri build
    fi

    echo "==> MoA Tauri app built successfully!"
fi

echo ""
echo "==> Build complete!"
echo "    ZeroClaw binary: ${SIDECAR_DIR}/${SIDECAR_NAME}"
if [[ -z "$SKIP_FRONTEND" ]]; then
    echo "    Tauri bundle: ${TAURI_DIR}/src-tauri/target/${BUILD_MODE}/bundle/"
fi
