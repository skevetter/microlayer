#!/bin/bash
set -e

echo "Building microlayer..."
cargo build --release

echo ""
echo "Testing CLI interface..."
echo "========================"

# Test version
echo "Testing --version..."
./target/release/microlayer --version

echo ""
echo "Testing --help..."
./target/release/microlayer --help

echo ""
echo "Testing apt-get --help..."
./target/release/microlayer apt-get --help

echo ""
echo "Testing apk --help..."
./target/release/microlayer apk --help

echo ""
echo "Testing gh-release --help..."
./target/release/microlayer gh-release --help

echo ""
echo "================================"
echo "All CLI tests passed successfully!"
echo "Binary size: $(du -h target/release/microlayer | cut -f1)"
echo "================================"
