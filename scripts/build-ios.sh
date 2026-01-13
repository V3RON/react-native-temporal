#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_DIR="$PROJECT_ROOT/rust/temporal-rn"
IOS_DIR="$PROJECT_ROOT/ios"
TARGET_DIR="$RUST_DIR/target"

echo "Building temporal-rn for iOS..."
echo "Project root: $PROJECT_ROOT"

cd "$RUST_DIR"

# Install Rust targets if not present
echo "Installing iOS Rust targets..."
rustup target add aarch64-apple-ios 2>/dev/null || true
rustup target add aarch64-apple-ios-sim 2>/dev/null || true
rustup target add x86_64-apple-ios 2>/dev/null || true

# Build for device (arm64)
echo "Building for iOS device (aarch64-apple-ios)..."
cargo build --release --target aarch64-apple-ios

# Build for simulator (arm64 - Apple Silicon)
echo "Building for iOS simulator arm64 (aarch64-apple-ios-sim)..."
cargo build --release --target aarch64-apple-ios-sim

# Build for simulator (x86_64 - Intel Macs)
echo "Building for iOS simulator x86_64 (x86_64-apple-ios)..."
cargo build --release --target x86_64-apple-ios

# Create output directory
mkdir -p "$IOS_DIR/libs"

# Create universal simulator library (combining arm64 and x86_64)
echo "Creating universal simulator library..."
lipo -create \
    "$TARGET_DIR/aarch64-apple-ios-sim/release/libtemporal_rn.a" \
    "$TARGET_DIR/x86_64-apple-ios/release/libtemporal_rn.a" \
    -output "$IOS_DIR/libs/libtemporal_rn_sim.a"

# Copy device library
echo "Copying device library..."
cp "$TARGET_DIR/aarch64-apple-ios/release/libtemporal_rn.a" "$IOS_DIR/libs/libtemporal_rn_device.a"

# Generate C header using cbindgen
echo "Generating C header..."
if command -v cbindgen &> /dev/null; then
    cbindgen --config cbindgen.toml --crate temporal-rn --output "$IOS_DIR/temporal_rn.h"
else
    echo "cbindgen not found, creating header manually..."
    cat > "$IOS_DIR/temporal_rn.h" << 'EOF'
/* temporal-rn C bindings */
#ifndef TEMPORAL_RN_H
#define TEMPORAL_RN_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Returns the current instant as an ISO 8601 string (e.g., "2024-01-15T10:30:45.123Z").
 * The caller is responsible for freeing the returned string using `temporal_free_string`.
 *
 * Returns NULL on error.
 */
char *temporal_instant_now(void);

/**
 * Frees a string allocated by temporal functions.
 */
void temporal_free_string(char *s);

#ifdef __cplusplus
}
#endif

#endif /* TEMPORAL_RN_H */
EOF
fi

echo ""
echo "iOS build complete!"
echo "Output files:"
ls -la "$IOS_DIR/libs/"
ls -la "$IOS_DIR/temporal_rn.h"
