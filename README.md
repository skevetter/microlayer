# Picolayer

A management tool to keep container layers as small as possible. Picolayer helps keep container layers small by automatically cleaning up installation leftovers such as apt-get update lists, caches, and temporary files. With Picolayer, you can also running any programming language, devcontainer-feature, or install any Github release with ease.

This project is inspired by [nanolayer](https://github.com/devcontainers-extra/nanolayer).

## Commands

- **apt-get**: Install Debian/Ubuntu packages with automatic cleanup
- **apk**: Install Alpine packages with automatic cleanup
- **brew**: Install packages using Homebrew
- **gh-release**: Install binaries from GitHub releases with checksum and GPG verification
- **x**: Execute commands with pkgx

## Installation

### From source

```bash
cargo install --git https://github.com/skevetter/picolayer
```

### From binary

Download the latest release from the [releases page](https://github.com/skevetter/picolayer/releases).
