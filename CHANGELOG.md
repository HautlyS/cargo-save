# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive library API with full documentation
- Examples directory with 4 runnable examples
- Unit tests for core functionality
- Git hooks support for automatic cache invalidation
- Sparse checkout support
- Git LFS support
- Shallow clone support
- Git worktree support
- Git submodule tracking
- CI/CD cache key generation

### Changed
- Restructured codebase into lib.rs and main.rs
- Improved documentation with rustdoc comments
- Enhanced error messages and logging

## [0.2.0] - 2026-02-14

### Added
- Package-level incremental caching
- Git-based change detection
- Workspace-aware dependency tracking
- Parallel hash computation with rayon
- Build log caching and querying
- Cache management commands (clean, invalidate, list)
- Statistics command
- Status command with hash display
- Warm command for pre-computing hashes
- Support for release and debug profiles
- Feature flag tracking
- Environment variable tracking (RUSTFLAGS, etc.)
- Cross-platform cache directory support

### Changed
- Improved hash computation accuracy
- Better handling of git repositories
- More efficient cache invalidation

## [0.1.0] - 2026-02-10

### Added
- Initial release
- Basic cargo wrapper functionality
- Simple caching mechanism
- Build output capture

[Unreleased]: https://github.com/HautlyS/cargo-save/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/HautlyS/cargo-save/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/HautlyS/cargo-save/releases/tag/v0.1.0
