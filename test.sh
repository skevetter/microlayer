#!/bin/bash
set -e

echo "Building picolayer..."
cargo build --release

echo ""
echo "Testing CLI interface..."
echo "========================"

# Test version
echo "Testing --version..."
./target/release/picolayer --version

echo ""
echo "Testing --help..."
./target/release/picolayer --help

echo ""
echo "Testing apt-get --help..."
./target/release/picolayer apt-get --help

echo ""
echo "Testing apk --help..."
./target/release/picolayer apk --help

echo ""
echo "Testing gh-release --help..."
./target/release/picolayer gh-release --help

echo ""
echo "================================"
echo "All CLI tests passed successfully!"
echo "Binary size: $(du -h target/release/picolayer | cut -f1)"
echo "================================"
