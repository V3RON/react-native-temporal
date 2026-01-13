#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "============================================"
echo "Building temporal-rn for all platforms"
echo "============================================"
echo ""

# Build iOS
echo ">>> Building for iOS..."
"$SCRIPT_DIR/build-ios.sh"
echo ""

# Build Android
echo ">>> Building for Android..."
"$SCRIPT_DIR/build-android.sh"
echo ""

echo "============================================"
echo "All builds completed successfully!"
echo "============================================"
