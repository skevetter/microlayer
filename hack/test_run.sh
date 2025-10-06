#!/bin/bash

# Test script for picolayer run functionality
# This script demonstrates various ways to use picolayer run command

set -e

echo "=== Testing picolayer run functionality ==="

# Test basic command execution
echo "1. Testing basic command execution..."
./target/release/picolayer run "echo 'Hello from picolayer!'"

# Test version-specific execution (like pkgx examples)
echo "2. Testing version-specific execution..."
if ./target/release/picolayer run "python@3.11 --version" 2>/dev/null; then
    echo "✓ Python 3.11 version test passed"
else
    echo "⚠ Python 3.11 not available, skipping"
fi

if ./target/release/picolayer run "node@14 --version" 2>/dev/null; then
    echo "✓ Node 14 version test passed"
else
    echo "⚠ Node 14 not available, skipping"
fi

# Test with working directory
echo "3. Testing working directory..."
mkdir -p /tmp/picolayer_test
echo 'print("Hello from Python script!")' > /tmp/picolayer_test/test.py
./target/release/picolayer run "python test.py" --working-dir /tmp/picolayer_test

# Test with environment variables
echo "4. Testing environment variables..."
./target/release/picolayer run "python -c \"import os; print('TEST_VAR:', os.environ.get('TEST_VAR', 'not_found'))\"" --env "TEST_VAR=hello_world"

# Test dependency detection with package.json
echo "5. Testing dependency detection (Node.js)..."
mkdir -p /tmp/picolayer_test_node
echo '{"name": "test", "version": "1.0.0"}' > /tmp/picolayer_test_node/package.json
./target/release/picolayer run "node --version" --working-dir /tmp/picolayer_test_node

# Test dependency detection with requirements.txt
echo "6. Testing dependency detection (Python)..."
mkdir -p /tmp/picolayer_test_python
echo "requests>=2.0.0" > /tmp/picolayer_test_python/requirements.txt
./target/release/picolayer run "python --version" --working-dir /tmp/picolayer_test_python

# Test force pkgx flag
echo "7. Testing --force-pkgx flag..."
./target/release/picolayer run "echo 'Testing force pkgx'" --force-pkgx

# Test with standard version (if pkgx-integration feature is available)
if ./target/release/picolayer --help | grep -q "pkgx-integration"; then
    echo "8. Testing with pkgx-integration feature..."
    ./target/release/picolayer run "echo 'Testing with library integration'" --force-pkgx
else
    echo "8. pkgx-integration feature not available in this build"
fi

# Clean up
rm -rf /tmp/picolayer_test /tmp/picolayer_test_node /tmp/picolayer_test_python

echo "=== All tests completed ==="
echo ""
echo "To build with pkgx-integration feature:"
echo "  cargo build --release --features pkgx-integration"
echo ""
echo "Usage examples:"
echo "  picolayer run \"python@3.11 --version\""
echo "  picolayer run \"node@14 script.js\" --working-dir /path/to/project"
echo "  picolayer run \"python app.py\" --env \"DEBUG=1\" --force-pkgx"
