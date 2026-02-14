# cargo-save

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/cargo-save.svg)](https://crates.io/crates/cargo-save)
[![Documentation](https://docs.rs/cargo-save/badge.svg)](https://docs.rs/cargo-save)
[![CI](https://github.com/HautlyS/cargo-save/workflows/CI/badge.svg)](https://github.com/HautlyS/cargo-save/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Downloads](https://img.shields.io/crates/d/cargo-save.svg)](https://crates.io/crates/cargo-save)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

Smart caching cargo wrapper with git-based incremental builds for Rust workspaces.

**Skip redundant builds. Save time. Stay productive.**

</div>

## Overview

`cargo-save` is a build cache for Rust that caches entire package builds and intelligently determines which packages need rebuilding in a workspace.

### Key Features

- **Package-Level Incremental Caching** - Only rebuilds packages that changed
- **Git-Based Change Detection** - Uses git tree hashes for fast, accurate change detection
- **Workspace-Aware** - Understands Cargo workspace dependencies and transitive changes
- **Build Log Caching** - Query previous build outputs without rebuilding
- **Advanced Git Support** - Handles submodules, worktrees, sparse checkout, LFS, and shallow clones
- **Smart Cache Invalidation** - Tracks dependencies, features, environment variables, and build profiles
- **Parallel Hashing** - Uses rayon for fast hash computation
- **CI/CD Ready** - Generate cache keys and integrate with GitHub Actions, GitLab CI, etc.

## Why cargo-save?

In large Rust workspaces, rebuilding unchanged packages wastes time. `cargo-save` solves this by:

1. Tracking package-level changes instead of individual source files
2. Understanding workspace dependencies to rebuild only affected packages
3. Storing complete build outputs for instant replay
4. Integrating with git for accurate change detection

## Quick Start

```bash
# Install
cargo install cargo-save

# Use just like cargo
cargo-save build
cargo-save test
cargo-save check

# Or use the cargo subcommand syntax
cargo save build
cargo save test --release
```

## Installation

### From crates.io (Recommended)

```bash
cargo install cargo-save
```

### From source

```bash
git clone https://github.com/HautlyS/cargo-save.git
cd cargo-save
cargo install --path .
```

### Quick install script

```bash
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/HautlyS/cargo-save/main/install.sh | sh
```

## Usage

### Basic Commands

```bash
# Build with caching (both syntaxes work)
cargo-save build
cargo-save save build  # equivalent

# Other cargo commands
cargo-save check
cargo-save test
cargo-save clippy
cargo-save build --release
```

### Cache Management

```bash
# Show workspace status and cache state
cargo-save status
cargo-save status --hashes  # show git hashes

# List cached builds
cargo-save list
cargo-save list --verbose
cargo-save list --workspace  # only current workspace

# Query build logs
cargo-save query tail          # last 50 lines
cargo-save query head 100      # first 100 lines
cargo-save query grep "error"  # search for pattern
cargo-save query all           # full output
cargo-save query errors        # only error lines

# Clean old caches
cargo-save clean               # remove caches older than 7 days
cargo-save clean --days 30     # custom age
cargo-save clean --keep 10     # keep only last 10 builds

# Invalidate caches
cargo-save invalidate --all
cargo-save invalidate my-package

# Show statistics
cargo-save stats

# Check environment and integration
cargo-save doctor
```

### CI Integration

```bash
# Generate cache key for CI systems
cargo-save cache-key --platform github
cargo-save cache-key --platform gitlab
```

### Pre-warming Cache

```bash
# Pre-compute hashes for all packages
cargo-save warm
cargo-save warm --release
```

### Git Hooks

```bash
# Install hooks for automatic cache invalidation
cargo-save install-hooks
```

This installs:
- `post-checkout`: Invalidates cache when switching branches
- `post-merge`: Invalidates cache after merging

### Environment Check

```bash
# Check integration status and recommendations
cargo-save doctor
```

This shows:
- Git availability and version
- sccache integration status
- Cache size and statistics
- Optimization recommendations

## Integration with Other Tools

### Using with sccache

`cargo-save` and `sccache` work at different caching layers:

- **cargo-save**: Workspace-level incremental builds (skips unchanged packages)
- **sccache**: Compilation-level caching (shares compiled dependencies across projects)

#### Setup

```bash
# Interactive setup (recommended)
cargo-save setup-sccache

# Or manual setup
cargo install sccache
export RUSTC_WRAPPER=sccache  # Add to ~/.bashrc or ~/.zshrc
```

cargo-save will prompt you to setup sccache on first build if not configured.

#### How They Work Together

```
Project A: cargo-save build
  ├─ cargo-save: Detects 3 packages need rebuild
  └─ sccache: Caches tokio, serde compilation

Project B: cargo-save build  
  ├─ cargo-save: Detects 5 packages need rebuild
  └─ sccache: Reuses tokio, serde from Project A
```

#### Verification

```bash
cargo-save doctor  # Check integration status
sccache --show-stats  # View cache statistics
```

### Using with cargo-cache

`cargo-cache` is a cleanup utility for managing disk space:

```bash
# Install cargo-cache
cargo install cargo-cache

# Clean old cargo caches
cargo-cache --autoclean

# Clean old cargo-save caches
cargo-save clean --days 30
```

Use both for complete cache management:
- `cargo-cache`: Cleans `~/.cargo` registry and git caches
- `cargo-save`: Cleans workspace build caches

## Comparison with Other Tools

### cargo-save vs sccache vs cargo-cache

| Feature | cargo-save | sccache | cargo-cache |
|:--------|:----------:|:-------:|:-----------:|
| **Caching Level** | Package-level | Compiler-level | Target directory |
| **Granularity** | Per package | Per source file | Entire target |
| **Git Integration** | Native (tree hashes) | None | None |
| **Workspace Aware** | Yes | No | Partial |
| **Transitive Dep Tracking** | Yes | N/A | No |
| **Build Log Storage** | Yes | No | No |
| **Distributed Caching** | No | Yes | No |
| **CI Integration** | Native | Via config | Manual |
| **Setup Complexity** | Zero config | Requires daemon | Manual cleanup |
| **Best For** | Workspaces | Individual builds | Disk cleanup |

### When to Use Each Tool

**Use cargo-save + sccache together** for maximum performance - they complement each other at different caching layers.

#### Use cargo-save when:
- Working with large Cargo workspaces (5+ packages)
- Need package-level incremental builds
- Need build log caching and querying
- Want git-aware change detection
- Want zero-configuration setup
- Need workspace dependency tracking
- Switching branches frequently

#### Use sccache when:
- Need distributed caching across multiple machines
- Want to share compiler caches between projects
- Primarily work on single-crate projects
- Need to cache C/C++ compilation too
- Have a team sharing a central cache server
- Want cloud storage backends (S3, Azure, GCS)

#### Use cargo-cache when:
- Need to clean up disk space from old builds
- Want to inspect target directory contents
- Need simple cache statistics
- Want to manually manage cargo's cache

### Detailed Feature Comparison

#### Caching Strategy

| Aspect | cargo-save | sccache | cargo-cache |
|:-------|:-----------|:--------|:------------|
| Cache Key Components | Source hash, deps, env, features, profile | Compiler args, source hash | N/A (cleanup only) |
| Cache Invalidation | Smart (dependency-aware) | Hash-based | Manual |
| Incremental Builds | Package-level | Object-level | Not applicable |
| Cross-Project Caching | No | Yes | No |
| Cache Persistence | Filesystem | Filesystem/Remote | N/A |

#### Git Integration

| Feature | cargo-save | sccache | cargo-cache |
|:--------|:----------:|:-------:|:-----------:|
| Git tree hashes | Native | No | No |
| Submodule tracking | Yes | No | No |
| Worktree support | Yes | No | No |
| Sparse checkout | Yes | No | No |
| LFS support | Yes | No | No |
| Shallow clones | Yes | No | No |

#### Workspace Features

| Feature | cargo-save | sccache | cargo-cache |
|:--------|:----------:|:-------:|:-----------:|
| Workspace-aware | Yes | No | Partial |
| Dependency graph | Full | No | No |
| Transitive invalidation | Yes | No | No |
| Package-level caching | Yes | No | No |

#### Additional Features

| Feature | cargo-save | sccache | cargo-cache |
|:--------|:----------:|:-------:|:-----------:|
| Build log caching | Yes | No | No |
| Log querying | Yes | No | No |
| CI cache keys | Built-in | Manual | No |
| Cache statistics | Yes | Yes | Yes |
| Git hooks | Yes | No | No |
| Pre-warm command | Yes | No | No |

### Performance Characteristics

| Metric | cargo-save | sccache | cargo-cache |
|:-------|:----------:|:-------:|:-----------:|
| First Build | Same as cargo | Same as cargo | Same as cargo |
| Rebuild (no changes) | ~0ms (cached) | Fast (object cache) | N/A |
| Rebuild (1 pkg changed) | ~1 package | ~1 package + deps | All packages |
| Cache Lookup Speed | Fast (git-based) | Fast (hash-based) | N/A |
| Cache Size | Medium | Large | N/A |
| Network Required | No | Optional (dist) | No |
| Memory Overhead | Low | Medium (daemon) | None |

### Real-World Scenarios

**Scenario 1: Large workspace (50 packages), change 1 file in 1 package**

```
cargo build           → Builds 1 package (~30s)
cargo-save build      → Builds 1 package (~30s, cached result)
sccache cargo build   → Builds 1 package (~30s, cached objects)
```

**Scenario 2: Switch git branches, build again**

```
cargo build           → May rebuild everything (incorrect incremental)
cargo-save build      → Correctly detects changes, rebuilds only changed
sccache cargo build   → Uses object cache, may miss semantic changes
```

**Scenario 3: CI/CD with cache**

```
cargo-save            → Generates cache key, integrates natively
sccache              → Requires S3/Azure/GCS setup
cargo-cache          → Manual cache management
```

**Scenario 4: Review build logs from yesterday**

```
cargo-save query tail --last 1    # Shows last build's output
sccache                          # No log storage
cargo-cache                     # No log storage
```

### Architecture Differences

```
┌─────────────────────────────────────────────────────────────┐
│                    cargo-save                               │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Package-level cache (entire build results)       │
│  Layer 2: Dependency graph tracking                        │
│  Layer 1: Git-based change detection                       │
└─────────────────────────────────────────────────────────────┘
                           vs
┌─────────────────────────────────────────────────────────────┐
│                    sccache                                  │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Compiler-level cache (object files)              │
│  Layer 1: Source file hashing                              │
└─────────────────────────────────────────────────────────────┘
                           vs
┌─────────────────────────────────────────────────────────────┐
│                    cargo-cache                              │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: Target directory management (cleanup only)       │
└─────────────────────────────────────────────────────────────┘
```

## How It Works

### Architecture

1. **Change Detection**: Computes git-based hashes for each package's source files
2. **Dependency Tracking**: Builds a dependency graph to detect transitive changes
3. **Cache Validation**: Checks if cached artifacts are valid based on:
   - Source code changes (git tree hashes)
   - Cargo.lock changes
   - Environment variables (RUSTFLAGS, etc.)
   - Build profile (debug/release)
   - Feature flags
4. **Selective Rebuild**: Only rebuilds packages that changed or depend on changed packages
5. **Output Caching**: Stores complete build logs for later querying

### Cache Strategy

Cache entries are keyed by:
- Package name and source hash
- Command hash (cargo command + args)
- Environment hash (RUSTFLAGS, etc.)
- Build profile (debug/release)
- Features hash (feature flags)
- Cargo.lock hash

A cache is valid only if ALL factors match and target files exist.

## Configuration

### Environment Variables

- `CARGO_SAVE_CACHE_DIR`: Custom cache directory (default: OS cache dir)

### Cache Location

By default, caches are stored in:
- Linux: `~/.cache/cargo-save/v4/`
- macOS: `~/Library/Caches/cargo-save/v4/`
- Windows: `%LOCALAPPDATA%\cargo-save\v4\`

## Using as a Library

```rust
use cargo_save::CacheManager;

fn main() -> anyhow::Result<()> {
    let cache = CacheManager::new()?;
    let workspace = cache.compute_workspace_state(&[])?;
    
    // Build with caching
    let (_, exit_code, _, _) = 
        cache.run_cargo_with_cache("build", &[], &workspace)?;
    
    Ok(())
}
```

See the [examples/](examples/) directory for more usage examples.

## Requirements

- Rust 1.70+
- Git (recommended for accurate change detection)
- Cargo workspace

## Limitations

- **Git dependency**: Works best in git repositories. Falls back to file hashing without git.
- **Workspace-focused**: Optimized for workspace builds, less useful for single-crate projects.
- **No distributed caching**: Unlike sccache, doesn't support shared remote caches.
- **Build scripts**: Changes to build.rs are not yet tracked.
- **Cargo config**: Changes to .cargo/config.toml are not yet tracked.

## Roadmap

- [ ] Track build.rs changes
- [ ] Track .cargo/config.toml changes
- [ ] Support for path dependencies outside workspace
- [ ] Distributed caching support
- [ ] Integration with more CI systems
- [ ] Cache compression
- [ ] Build artifact caching

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Inspired by the need for better workspace build caching in Rust
- Thanks to the Rust community for excellent tools and libraries
- Built with [cargo_metadata](https://crates.io/crates/cargo_metadata), [blake3](https://crates.io/crates/blake3), and [clap](https://crates.io/crates/clap)

## Related Projects

- [sccache](https://github.com/mozilla/sccache) - Shared compilation cache
- [cargo-cache](https://github.com/matthiaskrgr/cargo-cache) - Cargo cache management
- [cargo-sweep](https://github.com/holmgr/cargo-sweep) - Clean up old build files
- [cargo-build-cache](https://github.com/matklad/cargo-build-cache) - Build caching experiment
