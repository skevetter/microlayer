# New Features and Configuration

## Environment Variables

Picolayer now supports the following environment variables for configuration:

### Logging Configuration

- **`PICOLAYER_LOG_FILE`**: Path to log file for dual logging (stdout + file)
  ```bash
  export PICOLAYER_LOG_FILE=/tmp/picolayer/picolayer.log
  picolayer apt-get curl
  ```

- **`RUST_LOG`**: Standard Rust logging level control (trace, debug, info, warn, error)
  ```bash
  export RUST_LOG=debug
  picolayer apt-get curl
  ```

### Cache Verification

- **`PICOLAYER_VERIFY_CACHE`**: Enable/disable cache verification (default: true)
  ```bash
  export PICOLAYER_VERIFY_CACHE=false
  picolayer apt-get curl
  ```

## Enhanced Features

### 1. Automatic Lock Management

All `apt`, `apt-get`, and `aptitude` commands now include automatic lock detection:
- Waits up to 60 seconds for apt/dpkg locks to be released
- Prevents conflicts with other package manager operations
- Logs informative messages when waiting

### 2. Cache Verification

New SHA256-based cache verification system ensures:
- Cache backups are created before operations
- Cache is restored to original state after operations
- Hash verification confirms no tampering occurred

### 3. Temporary File Management

All temporary files are stored with `/tmp/picolayer` prefix:
- Easy tracking of picolayer-created files
- Automatic cleanup when operations complete
- Lock files stored in centralized location

### 4. Integration Tests

New comprehensive integration tests verify:
- Cache restoration after apt-get operations
- Cache restoration after apt operations
- Temporary files are cleaned up properly
- System state matches before and after operations

Run integration tests with:
```bash
# On Linux with apt installed
cargo test --test cache_restoration_test
cargo test --test apt_get_test
```

### 5. File Logging

Dual logging support:
- All logs go to stdout/stderr (as before)
- Optionally also logged to file when `PICOLAYER_LOG_FILE` is set
- Thread-safe logging with automatic directory creation
- Timestamps and session markers included

### 6. Centralized Configuration

New `Config` module provides centralized settings:
- Configurable via environment variables
- Default values for all settings
- Used across all modules for consistency

## Testing

The project now includes:
- 60 unit tests (all passing)
- Multiple integration test suites
- Cache restoration verification tests
- Temporary file cleanup tests

Run all tests:
```bash
cargo test --lib          # Unit tests only
cargo test --all-targets  # All tests (requires appropriate OS)
```

## Example Usage

```bash
# Basic usage (logs to stdout only)
picolayer apt-get curl

# With file logging enabled
export PICOLAYER_LOG_FILE=/var/log/picolayer.log
picolayer apt-get curl

# With debug logging
export RUST_LOG=debug
export PICOLAYER_LOG_FILE=/tmp/picolayer.log
picolayer apt-get curl git vim

# Check the log file
cat /tmp/picolayer.log
```

## Troubleshooting

### Lock Wait Timeouts

If you encounter lock wait timeouts:
1. Check for other apt/dpkg processes: `ps aux | grep apt`
2. Wait for other package operations to complete
3. If stuck, manually remove stale locks (use with caution):
   ```bash
   sudo rm /var/lib/apt/lists/lock
   sudo rm /var/lib/dpkg/lock
   sudo rm /var/lib/dpkg/lock-frontend
   ```

### File Logging Issues

If file logging doesn't work:
1. Ensure the directory exists or picolayer has permission to create it
2. Check disk space
3. Verify `PICOLAYER_LOG_FILE` environment variable is set correctly

### Temporary File Buildup

All temporary files should be automatically cleaned up. If you notice buildup:
1. Check `/tmp/picolayer/` directory
2. Only lock files should persist (`.picolayer.lock`)
3. Old temp directories may be left if process was killed forcefully
4. Clean up manually: `rm -rf /tmp/picolayer/picolayer_*`
