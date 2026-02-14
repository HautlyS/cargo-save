# Tool Integration Guide

This guide explains how `cargo-save` integrates with other Rust build tools.

## Overview

`cargo-save` operates at the **workspace package level**, while other tools work at different layers of the build process.

## Integration Architecture

```
┌─────────────────────────────────────────────────┐
│  cargo-save: Package-level caching              │
│  - Tracks which packages changed                │
│  - Skips rebuilding unchanged packages          │
│  - Workspace dependency graph aware             │
└─────────────────────────────────────────────────┘
                    ↓ calls
┌─────────────────────────────────────────────────┐
│  cargo: Build orchestration                     │
│  - Manages build process                        │
│  - Calls rustc for compilation                  │
└─────────────────────────────────────────────────┘
                    ↓ calls
┌─────────────────────────────────────────────────┐
│  sccache: Compiler-level caching                │
│  - Caches compiled object files                 │
│  - Shares cache across ALL projects             │
│  - Can use remote storage (S3, etc.)            │
└─────────────────────────────────────────────────┘
```

## sccache Integration (Recommended)

### Why Use Both?

`cargo-save` and `sccache` complement each other:

- **cargo-save**: Determines which packages need rebuilding in your workspace
- **sccache**: Caches the actual compilation artifacts across all your projects

### Setup

```bash
# Interactive setup (recommended)
cargo-save setup-sccache

# Or manual setup
cargo install sccache
export RUSTC_WRAPPER=sccache  # Add to ~/.bashrc, ~/.zshrc, etc.
```

cargo-save will prompt you to setup sccache on first build if not configured.

### How It Works

#### Scenario 1: First Build in a Workspace

```bash
cd my-workspace
cargo-save build
```

1. cargo-save detects all packages need building (first time)
2. cargo-save calls `cargo build`
3. sccache intercepts rustc calls and caches compiled objects
4. cargo-save stores package hashes and build metadata

Result: Full build, but sccache populates its cache

#### Scenario 2: Rebuild After Changing One File

```bash
# Edit src/lib.rs in one package
cargo-save build
```

1. cargo-save detects only 1 package changed
2. cargo-save skips rebuilding other packages
3. For the changed package, sccache may reuse some cached objects
4. cargo-save updates cache for the changed package

Result: Only 1 package rebuilds

#### Scenario 3: Building a Different Project

```bash
cd ../another-project
cargo-save build
```

1. cargo-save detects this is a new workspace (different cache)
2. cargo-save determines which packages need building
3. sccache reuses cached objects from previous projects (e.g., tokio, serde)
4. cargo-save creates new cache for this workspace

Result: cargo-save rebuilds packages, but sccache speeds up compilation

### Verification

```bash
cargo-save doctor        # Check integration status
sccache --show-stats     # View cache statistics
```

### Performance Benefits

| Scenario | Without sccache | With sccache |
|:---------|:---------------:|:------------:|
| First build in workspace | 100% | 100% |
| Rebuild 1 changed package | ~10% | ~10% |
| Build different project (same deps) | 100% | ~30-50% |
| Clean build after `cargo clean` | 100% | ~30-50% |

## cargo-cache Integration

`cargo-cache` is a cleanup utility for disk space management.

### Setup

```bash
cargo install cargo-cache
```

### Usage

```bash
# Clean old cargo registry caches
cargo-cache --autoclean

# Clean old cargo-save caches
cargo-save clean --days 30

# Clean everything
cargo-cache --autoclean && cargo-save clean --days 30
```

### What Each Tool Cleans

- **cargo-cache**: Cleans `~/.cargo/registry`, `~/.cargo/git`, and target directories
- **cargo-save**: Cleans `~/.cache/cargo-save` (or equivalent on your OS)

## Environment Check

Use the `doctor` command to verify your setup:

```bash
cargo-save doctor
```

This checks:
- Git availability and version
- sccache integration status
- Cache size and location
- Optimization recommendations

## Best Practices

### 1. Use sccache with cargo-save

```bash
# In your shell config (~/.bashrc, ~/.zshrc, etc.)
export RUSTC_WRAPPER=sccache
```

### 2. Regular Cache Cleanup

```bash
# Weekly cleanup
cargo-cache --autoclean
cargo-save clean --days 30
```

### 3. CI/CD Integration

```yaml
# GitHub Actions example
- name: Setup sccache
  uses: mozilla-actions/sccache-action@v0.0.3

- name: Install cargo-save
  run: cargo install cargo-save

- name: Build with caching
  run: cargo-save build --release
  env:
    RUSTC_WRAPPER: sccache
```

### 4. Monitor Cache Performance

```bash
cargo-save stats         # Check cargo-save stats
sccache --show-stats     # Check sccache stats

# Reset sccache stats to measure a specific build
sccache --zero-stats
cargo-save build
sccache --show-stats
```

## Troubleshooting

### sccache Not Working

Symptom: `cargo-save doctor` shows "RUSTC_WRAPPER: Not set"

Solution:
```bash
which sccache            # Check if installed
cargo install sccache    # Install if needed
echo 'export RUSTC_WRAPPER=sccache' >> ~/.bashrc
source ~/.bashrc
```

### Cache Growing Too Large

Symptom: Cache directories using too much disk space

Solution:
```bash
cargo-save stats         # Check sizes
sccache --show-stats
cargo-save clean --days 14
cargo-cache --autoclean

# For sccache, set size limit in ~/.config/sccache/config
# See: https://github.com/mozilla/sccache#configuration
```

### Builds Not Using Cache

Symptom: cargo-save always rebuilds everything

Solution:
```bash
cargo-save status --hashes

# Verify git is working
git status
cargo-save doctor        # Check for environment variable changes
```

## Advanced: Remote sccache

For teams, configure sccache to use remote storage:

```bash
# AWS S3
export SCCACHE_BUCKET=my-build-cache
export SCCACHE_REGION=us-west-2

# Redis
export SCCACHE_REDIS=redis://localhost:6379
```

See [sccache documentation](https://github.com/mozilla/sccache#storage-options) for more options.

## Summary

| Tool | Purpose | Scope | Use With cargo-save? |
|:-----|:--------|:------|:--------------------:|
| **cargo-save** | Package-level incremental builds | Per workspace | N/A |
| **sccache** | Compilation caching | Cross-project | Yes (recommended) |
| **cargo-cache** | Disk space cleanup | System-wide | Yes (for cleanup) |

Recommended setup:
```bash
# Install tools
cargo install cargo-save sccache cargo-cache

# Configure sccache
export RUSTC_WRAPPER=sccache

# Use cargo-save for builds
cargo-save build

# Periodic cleanup
cargo-save clean --days 30
cargo-cache --autoclean
```

This provides:
- Fast workspace incremental builds (cargo-save)
- Cross-project compilation caching (sccache)
- Automatic disk space management (cargo-cache)
