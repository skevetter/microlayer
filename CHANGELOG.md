# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2024-10-05

### Added
- **New `run` command**: Execute commands using pkgx for automatic dependency management
  - Support for working directory specification (`--working-dir`)
  - Environment variable passing (`--env`)
  - Automatic detection and installation of dependencies via pkgx
- **Enhanced `gh-release` command**:
  - Regex pattern matching for asset filtering (`--pattern`)
  - Checksum verification support (`--verify-checksum`)
  - Support for .tar.xz archives
  - SHA256 checksum validation with multiple checksum file formats
- **Multi-platform GitHub Actions workflow**:
  - Matrix build for 4 platforms: x86_64-linux, aarch64-linux, x86_64-darwin, aarch64-darwin
  - Automated nightly and RC release pipelines
  - Pre-built binaries for all supported platforms
- Dependency additions: `sha2`, `hex`, `regex` for checksum verification

### Changed
- **BREAKING**: Binary size limit increased from 3MB to 20MB
- Version bumped to 2.0.0 to indicate breaking changes
- Updated CI workflow to reflect new binary size target
- Enhanced asset detection to support more archive formats

### Documentation
- Added comprehensive examples for new `run` command
- Documented regex pattern matching and checksum verification
- Added "What's New in v2.0.0" section to README
- Created CHANGELOG.md for tracking version changes

## [0.1.0] - 2024-01-01

### Added
- Initial release
- `apt-get` command for Debian/Ubuntu package installation
- `apk` command for Alpine package installation
- `gh-release` command for GitHub release binary installation
- Automatic cleanup of package manager caches
- PPA support for Ubuntu systems
