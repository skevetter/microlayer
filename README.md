# Picolayer

A minimal container layer management tool. Picolayer helps keep container layers small by automatically cleaning up installation leftovers such as apt-get update lists, caches, and temporary files.

This project is inspired by the [nanolayer](https://github.com/devcontainers-extra/nanolayer) repository.

## Commands

- **apt-get**: Install Debian/Ubuntu packages with automatic cleanup
- **apk**: Install Alpine packages with automatic cleanup
- **brew**: Install packages using Homebrew
- **gh-release**: Install binaries from GitHub releases with checksum and GPG verification
- **run**: Execute commands with pkgx for automatic dependency management

## Installation

### From source

```bash
cargo install --git https://github.com/skevetter/picolayer
```

### From binary

Download the latest release from the [releases page](https://github.com/skevetter/picolayer/releases).

## Usage

### Install apt-get packages

Install packages:

```bash
picolayer apt-get htop,curl,git
```

With PPAs:

```bash
picolayer apt-get neovim --ppas ppa:neovim-ppa/stable
```

Force PPAs on non-Ubuntu systems:

```bash
picolayer apt-get neovim --ppas ppa:neovim-ppa/stable --force-ppas-on-non-ubuntu
```

### Install apk packages

Install Alpine packages:

```bash
picolayer apk htop,curl,git
```

### Install Homebrew packages

```bash
picolayer brew jq,tree
```

### Install from GitHub release

Install the latest release:

```bash
picolayer gh-release cli/cli gh --version latest
```

Install a specific version:

```bash
picolayer gh-release cli/cli gh --version v2.40.0
```

With checksum verification:

```bash
picolayer gh-release jesseduffield/lazygit lazygit --version latest --checksum
```

With custom binary location:

```bash
picolayer gh-release cli/cli gh --version latest --bin-location /usr/local/bin
```

With asset filtering:

```bash
picolayer gh-release cli/cli gh --version latest --filter "linux.*amd64"
```

With GPG signature verification:

```bash
picolayer gh-release pkgxdev/pkgx pkgx --version latest --checksum --gpg-key /path/to/public-key.asc
```

### Run commands with pkgx

Run any version of any tool using pkgx for automatic dependency management:

```bash
# Run specific versions
picolayer run python@3.11 --version

picolayer run node@18 --version
```

Run with working directory:

```bash
picolayer run python script.py --working-dir /path/to/project
```

Run with environment variables:

```bash
picolayer run python app.py --env "DEBUG=1" --env "PORT=8000"
```

Run in ephemeral mode (cleanup packages after execution):

```bash
picolayer run "python@3.11" script.py --ephemeral
```

Force use of pkgx:

```bash
picolayer run "python" script.py --force-pkgx
```

Delete pkgx installation:

```bash
picolayer run --delete
```

## Docker Example

### Before (without picolayer)

```dockerfile
FROM ubuntu:22.04

RUN apt-get update && apt-get install -y htop curl
```

Layer size: **~25MB**

### After (with picolayer)

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

## License

MIT
