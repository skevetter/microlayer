# picolayer

Ensures minimal container layers - A Rust clone of [nanolayer](https://github.com/devcontainers-extra/nanolayer).

`picolayer` helps keep container layers as small as possible by automatically cleaning up installation leftovers such as apt-get update lists, caches, and temporary files.

## What's New in v2.0.0

**Breaking Changes:**
- Updated binary size target from 3MB to 20MB to accommodate new features
- Enhanced `gh-release` command with new options (backward compatible with existing usage)

**New Features:**
- **Multi-platform releases**: Pre-built binaries for Linux (x86_64, aarch64) and macOS (x86_64, aarch64)
- **pkgx integration**: New `run` command for executing scripts with automatic dependency management
- **Regex pattern matching**: Filter GitHub release assets with `--pattern` flag
- **Checksum verification**: Verify downloads with `--verify-checksum` flag for enhanced security
- **Nightly and RC releases**: Automated pre-release pipelines for testing

## Features

- **apt-get**: Install Debian/Ubuntu packages with automatic cleanup
- **apk**: Install Alpine packages with automatic cleanup
- **gh-release**: Install binaries from GitHub releases with regex filtering and checksum verification
- **run**: Execute commands using pkgx for automatic dependency management
- **Minimal footprint**: Optimized for small binary size and minimal dependencies

## Installation

### From source (requires Rust):

```bash
cargo install --git https://github.com/skevetter/picolayer
```

### From binary:

Download the latest release from the [releases page](https://github.com/skevetter/picolayer/releases).

## Usage

### Install apt-get packages

```bash
picolayer apt-get htop,curl,git
```

With PPAs:

```bash
picolayer apt-get neovim --ppas ppa:neovim-ppa/stable
```

### Install apk packages

```bash
picolayer apk htop,curl,git
```

### Install from GitHub release

```bash
picolayer gh-release cli/cli gh --version latest
```

With regex pattern filtering:

```bash
picolayer gh-release pkgxdev/pkgx pkgx --pattern "pkgx-.*\+linux\+x86_64\.tar\.xz"
```

With checksum verification:

```bash
picolayer gh-release cli/cli gh --verify-checksum
```

### Run commands with pkgx

```bash
# Run a Python script (pkgx will auto-install Python if needed)
picolayer run "python script.py"

# Run a Node.js application
picolayer run "node app.js" --working-dir ./my-app

# Run with environment variables
picolayer run "python app.py" --env "DEBUG=true" --env "PORT=8080"
```

## Docker Example

### Before (without picolayer):

```dockerfile
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y htop curl
```

Layer size: **~25MB**

### After (with picolayer):

```dockerfile
FROM ubuntu:22.04
COPY picolayer /usr/local/bin/picolayer
RUN picolayer apt-get htop,curl
```

Layer size: **~2MB**

Or download directly in the Dockerfile:

```dockerfile
FROM ubuntu:22.04
RUN curl -sfL https://github.com/skevetter/picolayer/releases/latest/download/picolayer-x86_64-unknown-linux-gnu.tar.gz | tar xz -C /usr/local/bin && \
    picolayer apt-get htop,curl && \
    rm /usr/local/bin/picolayer
```

## Building

```bash
cargo build --release
```

The binary will be in `target/release/picolayer`.

For smallest binary size:

```bash
cargo build --release
strip target/release/picolayer
```

## License

MIT
