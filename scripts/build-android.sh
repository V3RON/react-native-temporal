#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_DIR="$PROJECT_ROOT/rust/temporal-rn"
ANDROID_DIR="$PROJECT_ROOT/android"
JNILIBS_DIR="$ANDROID_DIR/src/main/jniLibs"

echo "Building temporal-rn for Android..."
echo "Project root: $PROJECT_ROOT"

# Check for Android NDK
if [ -z "$ANDROID_NDK_HOME" ] && [ -z "$NDK_HOME" ]; then
    # Try to find NDK in common locations
    if [ -d "$HOME/Library/Android/sdk/ndk" ]; then
        NDK_VERSION=$(ls "$HOME/Library/Android/sdk/ndk" | sort -V | tail -1)
        export ANDROID_NDK_HOME="$HOME/Library/Android/sdk/ndk/$NDK_VERSION"
    elif [ -d "$ANDROID_HOME/ndk" ]; then
        NDK_VERSION=$(ls "$ANDROID_HOME/ndk" | sort -V | tail -1)
        export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/$NDK_VERSION"
    fi
fi

if [ -z "$ANDROID_NDK_HOME" ] && [ -z "$NDK_HOME" ]; then
    echo "Error: ANDROID_NDK_HOME or NDK_HOME environment variable not set"
    echo "Please set it to your Android NDK installation path"
    exit 1
fi

echo "Using NDK: ${ANDROID_NDK_HOME:-$NDK_HOME}"

cd "$RUST_DIR"

# Install cargo-ndk if not present
if ! command -v cargo-ndk &> /dev/null; then
    echo "Installing cargo-ndk..."
    cargo install cargo-ndk
fi

# Install Android Rust targets
echo "Installing Android Rust targets..."
rustup target add aarch64-linux-android 2>/dev/null || true
rustup target add armv7-linux-androideabi 2>/dev/null || true
rustup target add i686-linux-android 2>/dev/null || true
rustup target add x86_64-linux-android 2>/dev/null || true

# Create output directories
mkdir -p "$JNILIBS_DIR/arm64-v8a"
mkdir -p "$JNILIBS_DIR/armeabi-v7a"
mkdir -p "$JNILIBS_DIR/x86"
mkdir -p "$JNILIBS_DIR/x86_64"

# Build for all Android architectures using cargo-ndk
echo "Building for Android (all architectures)..."
cargo ndk \
    -t arm64-v8a \
    -t armeabi-v7a \
    -t x86 \
    -t x86_64 \
    -o "$JNILIBS_DIR" \
    build --release

echo ""
echo "Android build complete!"
echo "Output files:"
find "$JNILIBS_DIR" -name "*.so" -exec ls -la {} \;
