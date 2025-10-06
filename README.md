# picolayer

Ensures minimal container layers - A Rust clone of [nanolayer](https://github.com/devcontainers-extra/nanolayer).

`picolayer` helps keep container layers as small as possible by automatically cleaning up installation leftovers such as apt-get update lists, caches, and temporary files.

## Features

- **apt-get**: Install Debian/Ubuntu packages with automatic cleanup
- **apk**: Install Alpine packages with automatic cleanup
- **brew**: Install packages using Homebrew
- **gh-release**: Install binaries from GitHub releases with checksum and GPG verification
- **run**: Execute commands with pkgx for automatic dependency management
- **Minimal footprint**: Optimized for small binary size and minimal dependencies

## Versions

picolayer is available in two versions:

- **Lite** (~4MB): Uses the pkgx binary if available on PATH
- **Standard** (~4MB): Includes optional pkgx library integration (feature flag: `pkgx-integration`)

Both versions are well under their respective size limits (5MB for lite, 15MB for standard).

## Installation

### From source (requires Rust):

```bash
cargo install --git https://github.com/skevetter/picolayer
```

### From binary:

Download the latest release from the [releases page](https://github.com/skevetter/picolayer/releases).

Choose between:
- `picolayer-lite-*`: Smaller binary, uses pkgx binary if available
- `picolayer-*`: Standard version with optional pkgx library integration

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

### Install Homebrew packages

```bash
picolayer brew jq,tree
```

### Install from GitHub release

```bash
picolayer gh-release cli/cli gh --version latest
```

With checksum verification:

```bash
picolayer gh-release jesseduffield/lazygit lazygit --version latest --checksum
```

With GPG signature verification:

```bash
picolayer gh-release pkgxdev/pkgx pkgx --version latest --checksum --gpg-key /path/to/public-key.asc
```

### Run commands with pkgx

Run any version of any tool using pkgx (similar to `pkgx node@14 --version`):

```bash
# Run specific versions
picolayer run "python@3.11 --version"
picolayer run "node@14 --version"

# Run with working directory
picolayer run "python script.py" --working-dir /path/to/project

# Run with environment variables
picolayer run "python app.py" --env "DEBUG=1" --env "PORT=8000"

# Force pkgx library integration (when available)
picolayer run "python script.py" --force-pkgx
```

The `run` command automatically detects dependencies from your project files:
- `package.json` → Node.js
- `requirements.txt`, `pyproject.toml` → Python
- `Cargo.toml` → Rust
- `go.mod` → Go
- `Gemfile` → Ruby
- And more...

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

### Lite version (default)

```bash
cargo build --release
```

### Standard version (with pkgx integration)

```bash
cargo build --release --features pkgx-integration
```

The binary will be in `target/release/picolayer`.

## License

MIT
