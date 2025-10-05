# Migration Guide: v0.1.0 to v2.0.0

This guide helps you upgrade from picolayer v0.1.0 to v2.0.0.

## Breaking Changes

### Binary Size Target

**Changed:** Binary size limit increased from 3MB to 20MB

**Why:** To accommodate new features including checksum verification, regex pattern matching, and pkgx integration.

**Impact:** Minimal. The actual binary size is still only 3.1MB. This change affects CI validation thresholds only.

**Action Required:** None for end users. If you have custom CI checks that validate binary size, update the threshold to 20MB.

## New Features

### 1. New `run` Command

Execute commands with automatic dependency management via pkgx.

**Before (manual pkgx usage):**
```bash
pkgx python@3.11 script.py
```

**After (picolayer v2.0.0):**
```bash
picolayer run "python script.py"
```

**Additional Features:**
- Working directory specification: `--working-dir`
- Environment variables: `--env KEY=VALUE`
- Automatic pkgx installation check

**Requirements:**
- pkgx must be installed on your system
- Installation: `curl -fsS https://pkgx.sh | sh`

### 2. Enhanced `gh-release` Command

#### Regex Pattern Matching

**New Feature:** Filter GitHub release assets using regex patterns.

**Use Case:** When asset names don't follow standard conventions.

**Example:**
```bash
# Before: Limited to standard platform detection
picolayer gh-release pkgxdev/pkgx pkgx

# After: Use regex to match specific asset formats
picolayer gh-release pkgxdev/pkgx pkgx --pattern "pkgx-.*\+linux\+x86_64\.tar\.xz"
```

#### Checksum Verification

**New Feature:** Verify downloaded assets using SHA256 checksums.

**Security Benefit:** Ensures downloaded binaries haven't been tampered with.

**Example:**
```bash
# Enable checksum verification
picolayer gh-release cli/cli gh --verify-checksum
```

**Supported Checksum Formats:**
- `<asset>.sha256`
- `<asset>.sha256sum`
- `<asset>.asc`
- `checksums.txt`
- `SHA256SUMS`
- `sha256sums.txt`

**Note:** If no checksum file is found, the download proceeds with a warning.

#### Archive Format Support

**New Feature:** Support for `.tar.xz` archives in addition to `.tar.gz` and `.zip`.

**Benefit:** Broader compatibility with different GitHub projects.

## Backward Compatibility

All existing commands remain backward compatible:

```bash
# These commands work exactly the same in v2.0.0
picolayer apt-get htop,curl
picolayer apk htop,curl
picolayer gh-release cli/cli gh
```

The new `--pattern` and `--verify-checksum` flags are optional and don't affect existing usage.

## Multi-Platform Releases

v2.0.0 introduces automated multi-platform releases:

**Available Pre-built Binaries:**
- `picolayer-x86_64-unknown-linux-gnu.tar.gz` - Linux x86_64
- `picolayer-aarch64-unknown-linux-gnu.tar.gz` - Linux ARM64
- `picolayer-x86_64-apple-darwin.tar.gz` - macOS Intel
- `picolayer-aarch64-apple-darwin.tar.gz` - macOS Apple Silicon

**Release Types:**
- **Stable**: `v2.0.0` - Production-ready releases
- **Nightly**: `v2.0.0-nightly.20241005` - Daily builds from main branch
- **Release Candidate**: `v2.0.0-rc.1` - Pre-release testing versions

## Installation

### From Binary

**v0.1.0 Method (still works):**
```bash
curl -sfL https://github.com/skevetter/picolayer/releases/latest/download/picolayer-x86_64-unknown-linux-gnu.tar.gz | tar xz -C /usr/local/bin
```

**v2.0.0 Improvements:**
- More platforms available
- Checksums provided for verification
- Automated release process ensures consistency

### From Source

**No changes:**
```bash
cargo install --git https://github.com/skevetter/picolayer
```

## Dockerfile Updates

**No changes required** for basic usage:

```dockerfile
FROM ubuntu:22.04
RUN curl -sfL https://github.com/skevetter/picolayer/releases/latest/download/picolayer-x86_64-unknown-linux-gnu.tar.gz | tar xz -C /usr/local/bin && \
    picolayer apt-get htop,curl && \
    rm /usr/local/bin/picolayer
```

**New Option:** Use with pkgx for dynamic dependency management:

```dockerfile
FROM ubuntu:22.04
RUN curl -fsS https://pkgx.sh | sh
COPY picolayer /usr/local/bin/picolayer
COPY script.py /app/script.py
WORKDIR /app
RUN picolayer run "python script.py"
```

## Troubleshooting

### Issue: "pkgx is not available"

**Solution:** Install pkgx:
```bash
curl -fsS https://pkgx.sh | sh
```

### Issue: "Checksum verification failed"

**Possible Causes:**
1. Network interruption during download
2. GitHub release asset was modified
3. Proxy or firewall interference

**Solution:**
1. Retry the download
2. Check GitHub release page for known issues
3. Temporarily disable verification: remove `--verify-checksum` flag
4. Report suspicious checksum failures to the repository maintainers

### Issue: "No suitable asset found for this platform"

**Solution:**
1. Check if your platform is supported in the release
2. Use `--pattern` flag to manually specify asset pattern
3. Build from source for unsupported platforms

## Getting Help

- **Documentation**: [README.md](README.md)
- **Changelog**: [CHANGELOG.md](CHANGELOG.md)
- **Contributing**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **Issues**: [GitHub Issues](https://github.com/skevetter/picolayer/issues)

## Summary

picolayer v2.0.0 is a feature-rich upgrade that maintains backward compatibility while adding powerful new capabilities:

✅ All existing commands work unchanged
✅ New `run` command for pkgx integration
✅ Enhanced security with checksum verification
✅ Flexible asset matching with regex patterns
✅ Multi-platform automated releases

Upgrade today to take advantage of these new features!
