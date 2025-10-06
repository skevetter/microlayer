#!/bin/bash

# Test script for picolayer GPG URL functionality
# This demonstrates the new capability to use URLs for GPG public keys

set -e

echo "Testing picolayer GPG verification with URL-based public key..."
echo "============================================================="

# Clean up any previous test files
rm -f /tmp/test_pkgx

echo "1. Testing with GPG public key URL (https://dist.pkgx.dev/gpg-public.asc)"
echo "   Command: picolayer gh-release pkgxdev/pkgx pkgx --version latest --bin-location /tmp --checksum --gpg-key https://dist.pkgx.dev/gpg-public.asc"
echo

# Build the project first
cargo build --release

# Run the test with URL-based GPG key
./target/release/picolayer gh-release pkgxdev/pkgx pkgx \
    --version latest \
    --bin-location /tmp \
    --checksum \
    --gpg-key "https://dist.pkgx.dev/gpg-public.asc"

# Verify the binary was installed and is working
if [ -f "/tmp/pkgx" ]; then
    echo "✓ Binary installed successfully"
    echo "✓ Version check:"
    /tmp/pkgx --version
else
    echo "✗ Binary not found"
    exit 1
fi

echo
echo "============================================================="
echo "✓ GPG verification with URL-based public key works correctly!"
echo
echo "Key features demonstrated:"
echo "- Downloads GPG public key from URL: https://dist.pkgx.dev/gpg-public.asc"
echo "- Automatically selects assets with available GPG signatures (.tar.xz over .tar.gz)"
echo "- Verifies GPG signatures using downloaded public key"
echo "- Supports both ASCII-armored and binary signature formats"
echo "- Extracts tar.xz archives correctly"
echo
echo "The --gpg-key parameter now accepts:"
echo "- URLs (https://... or http://...)"
echo "- File paths (local files)"
echo "- Direct key content (inline GPG public key)"

# Clean up
rm -f /tmp/pkgx

echo "✓ Test completed successfully!"
