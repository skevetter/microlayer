# Contributing to picolayer

## Building

```bash
cargo build --release
```

The binary will be in `target/release/picolayer`.

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
  - `gh_release.rs` - GitHub release binary installer with checksum verification
  - `run.rs` - pkgx command execution wrapper
- `src/utils/` - Utility modules
  - `command.rs` - Shell command execution helpers
  - `linux_info.rs` - Linux distribution detection

## Design Principles

1. **Minimal footprint**: Keep binary size reasonable (current target: <20MB)
2. **Automatic cleanup**: Always clean up temporary files and caches
3. **Simple interface**: Follow nanolayer's command patterns for compatibility
4. **Safe defaults**: Fail on errors, no silent failures
5. **Security**: Support checksum verification for downloaded binaries

## Adding New Installers

To add a new package manager installer:

1. Create a new module in `src/installers/`
2. Implement the `install()` function with signature: `pub fn install(...) -> Result<()>`
3. Follow the pattern: backup cache → install → cleanup → restore cache
4. Add the command to `src/main.rs`

## Release Process

Releases are automated via GitHub Actions:

1. **Nightly releases**: Automatically created on push to main branch with format `v2.0.0-nightly.YYYYMMDD`
2. **Release candidates**: Manually triggered via workflow_dispatch with format `v2.0.0-rc.N`
3. **Stable releases**: Created by pushing a version tag (e.g., `v2.0.0`)

All releases include pre-built binaries for:
- x86_64-unknown-linux-gnu
- aarch64-unknown-linux-gnu
- x86_64-apple-darwin
- aarch64-apple-darwin

### Manual Release Steps

1. Update version in `Cargo.toml` and `CHANGELOG.md`
2. Commit changes: `git commit -am "chore: bump version to X.Y.Z"`
3. Create and push tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
4. GitHub Actions will automatically build and create the release

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
