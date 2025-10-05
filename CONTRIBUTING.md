# Contributing to microlayer

## Building

```bash
cargo build --release
```

The binary will be in `target/release/microlayer`.

## Testing

Run the test script:

```bash
./test.sh
```

## Code Structure

- `src/main.rs` - CLI entry point using clap
- `src/installers/` - Package installer implementations
  - `apt_get.rs` - Debian/Ubuntu apt-get installer
  - `apk.rs` - Alpine apk installer
  - `gh_release.rs` - GitHub release binary installer
- `src/utils/` - Utility modules
  - `command.rs` - Shell command execution helpers
  - `linux_info.rs` - Linux distribution detection

## Design Principles

1. **Minimal footprint**: Keep binary size small (currently ~1.7MB)
2. **Automatic cleanup**: Always clean up temporary files and caches
3. **Simple interface**: Follow nanolayer's command patterns for compatibility
4. **Safe defaults**: Fail on errors, no silent failures

## Adding New Installers

To add a new package manager installer:

1. Create a new module in `src/installers/`
2. Implement the `install()` function with signature: `pub fn install(...) -> Result<()>`
3. Follow the pattern: backup cache → install → cleanup → restore cache
4. Add the command to `src/main.rs`

## Release Process

1. Update version in `Cargo.toml`
2. Build release binary: `cargo build --release`
3. Strip binary: `strip target/release/microlayer`
4. Create GitHub release with binaries for different platforms

## Cross-compilation

To build for different targets:

```bash
# Install cross-compilation tool
cargo install cross

# Build for x86_64 Linux
cross build --release --target x86_64-unknown-linux-gnu

# Build for ARM64 Linux
cross build --release --target aarch64-unknown-linux-gnu
```
