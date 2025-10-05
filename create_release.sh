#!/bin/bash
#
# Script to create release packages for RJX
#

set -e

VERSION="0.1.0"
RELEASE_DIR="release"
PLATFORMS=("macos" "linux" "windows")

# Create release directory
mkdir -p "$RELEASE_DIR"

# Build for current platform (macOS)
echo "Building release binary..."
cargo build --release

# Create macOS package
echo "Creating macOS package..."
mkdir -p "$RELEASE_DIR/rjx-$VERSION-macos"
cp target/release/rjx "$RELEASE_DIR/rjx-$VERSION-macos/"
cp README.md LICENSE "$RELEASE_DIR/rjx-$VERSION-macos/"
(cd "$RELEASE_DIR" && tar -czf "rjx-$VERSION-macos.tar.gz" "rjx-$VERSION-macos")

echo "macOS package created at $RELEASE_DIR/rjx-$VERSION-macos.tar.gz"

# Note: For a complete release process, you would typically use cross-compilation 
# or CI/CD platforms to build for Linux and Windows
echo ""
echo "To create a complete release for all platforms:"
echo "1. Use cross-compilation tools or CI/CD services like GitHub Actions"
echo "2. For Linux: cargo build --release --target x86_64-unknown-linux-gnu"
echo "3. For Windows: cargo build --release --target x86_64-pc-windows-msvc"

echo ""
echo "Release binary for current platform is ready at: target/release/rjx"
echo "Release package for macOS is ready at: $RELEASE_DIR/rjx-$VERSION-macos.tar.gz"
