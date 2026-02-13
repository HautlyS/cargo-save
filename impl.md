# cargo-save v3.0

Smart caching cargo wrapper with git-based incremental builds. Prevents unnecessary recompilations by tracking package-level changes using git hashes, environment variables, Cargo.lock, and build artifacts.

## Problem

1. **Cargo's default cache** often recompiles entire workspace even when only one crate changed
2. **LLMs** often need to inspect build logs multiple times (head, tail, grep), causing full recompilations each time (~10s+ wasted)
3. **CI/CD** rebuilds everything from scratch without smart caching
4. **Workspace builds** are slow because cargo doesn't cache at package granularity
5. **Feature flags** changes cause unnecessary rebuilds
6. **Environment variables** (RUSTFLAGS, etc.) affect builds but aren't tracked
7. **Dependency changes** (Cargo.lock) aren't properly tracked per-package

## Solution

`cargo-save` provides:

1. **Git-based incremental caching** - Uses git tree hashes + blake3 to detect if a crate actually changed
2. **Package-level caching** - Only rebuilds packages that changed (and their dependents)
3. **Environment tracking** - Tracks RUSTFLAGS and other env vars that affect builds
4. **Cargo.lock tracking** - Invalidates cache when dependencies change
5. **Feature flag tracking** - Caches builds per feature set
6. **Artifact caching** - Tracks actual build artifacts (.rlib, .rmeta files)
7. **Log caching** - Captures build output once for instant queries without recompiling
8. **CI-friendly** - Works with `--release`, custom target directories, and CI environments
9. **Parallel processing** - Computes hashes in parallel for better performance
10. **Dependency graph** - Properly handles transitive workspace dependencies

## Installation

```bash
# Install from source
cargo build --release
sudo cp target/release/cargo-save /usr/local/bin/

# Or use cargo install
cargo install --path .
```

## Usage

### Basic Commands

```bash
# Build with smart caching
cargo-save save build

# Build specific package
cargo-save save build -p my-package

# Release build (cached separately)
cargo-save save build --release

# Check, clippy, test - all work the same
cargo-save save check
cargo-save save clippy
cargo-save save test
cargo-save save doc

# With extra args and features
cargo-save save build --features "feature1 feature2"
cargo-save save build --all-features
```

### Query Cached Logs (LLM-Friendly)

```bash
# Query latest build
cargo-save query head 50        # First 50 lines
cargo-save query tail 30        # Last 30 lines
cargo-save query grep "error"   # Search pattern
cargo-save query range 10-20    # Lines 10-20
cargo-save query all            # Full output
cargo-save query errors         # Show only errors
cargo-save query warnings       # Show only warnings

# Query specific cache
cargo-save query tail 20 --id 20250213_120000-abc12345

# Query from last N runs
cargo-save query tail 50 --last 3
```

### Cache Management

```bash
# List cached builds
cargo-save list
cargo-save list --verbose       # Detailed info
cargo-save list --workspace     # Only current workspace

# Show statistics
cargo-save stats

# Show workspace cache status
cargo-save status
cargo-save status --hashes      # With git hashes

# Warm cache (pre-compute hashes)
cargo-save warm                 # Debug builds
cargo-save warm --release       # Also for release

# Clean old caches
cargo-save clean --days 7       # Remove older than 7 days
cargo-save clean --keep 10      # Keep only last 10
cargo-save clean --force        # Skip confirmation

# Invalidate cache
cargo-save invalidate my-package  # Invalidate specific package
cargo-save invalidate --all       # Invalidate all
```

### CI/GitHub Actions Integration

```bash
# Generate cache key for CI
cargo-save cache-key              # GitHub Actions format
cargo-save cache-key --platform github
cargo-save cache-key --platform gitlab
cargo-save cache-key --platform generic
```

## How It Works

### 1. Git-Based Change Detection

For each package in the workspace:
- Computes git tree hash of source files (`*.rs`, `*.toml`)
- Uses `git ls-tree` for tracked files (fast and accurate)
- Falls back to manual file walking if not a git repo
- Excludes `target/`, `.git/`, `Cargo.lock`
- Uses blake3 for fast, collision-resistant hashing

### 2. Package-Level Caching with Multiple Factors

Cache key includes:
- Package name and source hash
- Command hash (cargo command + args)
- Environment hash (RUSTFLAGS, etc.)
- Profile (debug/release)
- Features hash (feature flags)
- Cargo.lock hash (dependency versions)

```
{package}-{source_hash}-{command_hash}-{env_hash}-{profile}-{features_hash}.json
```

### 3. Environment Variable Tracking

Automatically tracks these environment variables that affect builds:
- `RUSTFLAGS`, `RUSTDOCFLAGS`
- `CARGO_TARGET_DIR`, `CARGO_HOME`
- `CARGO_BUILD_JOBS`, `CARGO_BUILD_TARGET`
- `CARGO_PROFILE_*` settings
- `CC`, `CXX`, `AR`, `LINKER`

### 4. Smart Build Execution

```
1. Parse workspace structure with cargo_metadata
2. Compute hashes in parallel for all packages
3. Check which packages need rebuild
4. If all cached → skip build
5. If some changed → cargo build (only those packages)
6. Save new cache entries for rebuilt packages
7. Capture and cache build output
```

### 5. Dependency Graph Tracking

- Builds a dependency graph of workspace members
- When package A depends on B, and B changes, A is rebuilt
- Uses cargo_metadata to determine workspace-local dependencies only
- Ignores external crates (handled by Cargo.lock hash)

### 6. Log Caching

- All cargo output (stdout + stderr) captured
- Saved to cache directory with unique ID
- Query without recompiling
- Perfect for LLM workflows and debugging

## Cache Structure

```
~/.cache/cargo-save/v3/
├── 20250213_120000-abc12345.log          # Build log
├── 20250213_120000-def67890.log
├── metadata/
│   ├── 20250213_120000-abc12345.json     # Build metadata
│   └── 20250213_120000-def67890.json
├── incremental/
│   ├── my-crate-a1b2c3d4-*-*-*-*-debug.json    # Package cache
│   ├── my-crate-a1b2c3d4-*-*-*-*-release.json
│   └── other-crate-e5f6g7h8-*-*-*-*-debug.json
└── artifacts/
    └── (future: cached build artifacts)
```

## For LLMs: Efficient Build Log Analysis

**After running `cargo-save save build`, query logs without recompiling:**

```bash
# 1. Build once
cargo-save save build

# 2. Analyze in parts (instant, no recompile!)
cargo-save query head 30          # See build start
cargo-save query errors           # Find all errors
cargo-save query grep "warning:"  # Find warnings
cargo-save query tail 50          # See build end
cargo-save query range 100-200    # Specific section

# 3. Check specific error context
cargo-save query grep "error\[E"  # Find error codes
```

**Never run `cargo build` twice to see different parts. Use queries instead.**

## CI/CD Integration

### GitHub Actions (Recommended)

```yaml
name: Build

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Install cargo-save
        run: cargo install cargo-save
      
      - name: Generate cache key
        id: cache-key
        run: echo "key=$(cargo-save cache-key)" >> $GITHUB_OUTPUT
      
      - name: Cache cargo-save
        uses: actions/cache@v4
        with:
          path: ~/.cache/cargo-save
          key: ${{ runner.os }}-cargo-save-${{ steps.cache-key.outputs.key }}
          restore-keys: |
            ${{ runner.os }}-cargo-save-
      
      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Cache target directory
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ runner.os }}-cargo-target-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Warm cache
        run: cargo-save warm --release
      
      - name: Build with cache
        run: cargo-save save build --release
      
      - name: Run clippy
        run: cargo-save save clippy -- -D warnings
      
      - name: Run tests
        run: cargo-save save test
      
      - name: Upload build log on failure
        if: failure()
        run: cargo-save query all --last 1 > build.log
```

### GitLab CI

```yaml
variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo

.cache_template: &cache
  key:
    files:
      - Cargo.lock
  paths:
    - target/
    - .cargo/
    - ~/.cache/cargo-save/

build:
  <<: *cache
  script:
    - cargo install cargo-save
    - cargo-save save build --release
  after_script:
    - '[[ "$CI_JOB_STATUS" == "failed" ]] && cargo-save query all --last 1'
```

### Docker/Multi-stage Builds

```dockerfile
# Stage 1: Builder with cache
FROM rust:1.75 as builder

# Install cargo-save
RUN cargo install cargo-save

WORKDIR /app

# Copy manifests first (for better Docker layer caching)
COPY Cargo.toml Cargo.lock ./
COPY */Cargo.toml ./*/Cargo.toml

# Copy source code
COPY . .

# Build with cargo-save caching
RUN cargo-save save build --release

# Stage 2: Runtime
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/myapp /usr/local/bin/
CMD ["myapp"]
```

## Environment Variables

```bash
# Custom cache directory
export CARGO_SAVE_CACHE_DIR=/custom/path

# Disable incremental caching (force full rebuild)
export CARGO_SAVE_DISABLE_INCREMENTAL=1

# Verbose logging
export CARGO_SAVE_DEBUG=1
```

## Troubleshooting

### Build not cached

1. Check git is initialized: `git status`
2. Check status: `cargo-save status`
3. Check environment: `env | grep RUST`
4. Invalidate and retry: `cargo-save invalidate --all`

### Cache grows too large

```bash
# Clean old entries
cargo-save clean --days 3

# Keep only recent
cargo-save clean --keep 20
```

### Wrong package rebuilt

The tool tracks dependencies. If package A depends on B, and B changes, both are rebuilt (correct behavior).

To force rebuild: `cargo-save invalidate <package>`

### CI cache not working

1. Ensure `~/.cache/cargo-save` is persisted between CI runs
2. Check that cargo's `target/` directory is also cached
3. Use `cargo-save cache-key` to generate consistent keys
4. Verify RUSTFLAGS are the same across runs

## Implementation Details

### Core Algorithm

```rust
// 1. Compute workspace state (parallel)
let workspace = compute_workspace_state()?;

// 2. Build dependency graph
let graph = build_dependency_graph(&workspace);

// 3. Check cache for each package
let to_rebuild: Vec<Package> = workspace.packages
    .par_iter()
    .filter(|p| !is_cached(p, &env_hash, &cargo_lock_hash))
    .collect();

// 4. Compute transitive dependents
let transitive = compute_transitive_dependents(&to_rebuild, &graph);

// 5. Run cargo if needed
if !to_rebuild.is_empty() {
    run_cargo_build()?;
    
    // 6. Save cache for each rebuilt package
    for pkg in &to_rebuild {
        save_incremental_cache(pkg)?;
    }
}
```

### Hash Computation

```rust
fn compute_source_hash(path: &Path) -> String {
    let mut hasher = Blake3Hasher::new();
    
    // Try git first (fastest and most accurate)
    if let Ok(git_tree) = git_ls_tree(path) {
        hasher.update(&git_tree);
        
        // Include uncommitted changes
        if let Ok(status) = git_status(path) {
            hasher.update(&status);
        }
        return hasher.finalize().to_hex().to_string();
    }
    
    // Fallback: manual file hashing
    for file in walkdir(path) {
        if is_source_file(&file) {
            hasher.update(&fs::read(file)?);
        }
    }
    
    hasher.finalize().to_hex().to_string()
}
```

### Cache Validation

A cache entry is valid only if ALL of these match:
1. ✅ Source hash (code changes)
2. ✅ Cargo.lock hash (dependency changes)
3. ✅ Environment hash (RUSTFLAGS, etc.)
4. ✅ Features hash (feature flags)
5. ✅ Toolchain hash (rustc version)
6. ✅ Target files exist and haven't changed size

## Performance

- **Hash computation**: ~10-50ms per package (parallel)
- **Cache lookup**: ~1ms per package
- **Log query**: Instant (<1ms)
- **Build skip**: Instant (when all cached)
- **Dependency graph**: ~5ms for 50 packages

## Comparison with Default Cargo

| Scenario | Default Cargo | cargo-save |
|----------|---------------|------------|
| No changes | Recompiles everything | Skips build (instant) |
| One file changed | Recompiles workspace | Recompiles only affected packages |
| LLM queries log | Rebuilds each time | Queries cache instantly |
| CI rebuild | Full rebuild | Incremental rebuild |
| --release build | Separate from debug | Cached separately automatically |
| Feature flags change | Recompiles | Only affected packages |
| RUSTFLAGS change | Recompiles | Only if cache invalidated |
| Cargo.lock change | Full rebuild | Only affected packages |

## Future Features

- [ ] Distributed caching (S3, Redis)
- [ ] Compression for large logs
- [ ] Build artifact caching across machines
- [ ] Integration with cargo-nextest
- [ ] WASM target support
- [ ] Remote cache synchronization
- [ ] Build time analytics
- [ ] Automatic cache optimization suggestions

## License

MIT OR Apache-2.0
