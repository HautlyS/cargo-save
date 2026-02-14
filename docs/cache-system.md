# Cache System

## Overview

cargo-save implements a two-level caching system:
1. **Build-level cache**: Complete build logs and metadata
2. **Package-level cache**: Incremental artifacts per package

## Cache Key Generation

### Package Cache Key
```rust
format!(
    "{}-{}-{}-{}-{}-{}",
    package.name,
    &source_hash[..16],
    command_hash,
    env_hash,
    profile,  // "debug" or "release"
    features_hash
)
```

**Example:**
```
my-crate-a1b2c3d4e5f6g7h8-build-9a8b7c6d-debug-f1e2d3c4
```

### Build Cache ID
```rust
format!(
    "{}-{}",
    timestamp,  // YYYYMMDD_HHMMSS
    &command_hash[..8]
)
```

**Example:**
```
20250214_103045-abc12345
```

## Cache Storage

### Directory Structure
```
~/.cache/cargo-save/v3/
├── 20250214_103045-abc12345.log
├── 20250214_103046-def67890.log
├── metadata/
│   ├── 20250214_103045-abc12345.json
│   └── 20250214_103046-def67890.json
└── incremental/
    ├── my-crate-a1b2c3d4-build-9a8b-debug-f1e2.json
    ├── my-crate-a1b2c3d4-build-9a8b-release-f1e2.json
    └── other-crate-e5f6g7h8-check-1a2b-debug-c3d4.json
```

### Build Cache Metadata
```json
{
  "cache_id": "20250214_103045-abc12345",
  "command": "cargo build --release",
  "subcommand": "build",
  "args": ["--release"],
  "timestamp": "2025-02-14T10:30:45-03:00",
  "exit_code": 0,
  "workspace_state": { ... },
  "is_release": true,
  "target_dir": "/path/to/target",
  "lines_count": 1234,
  "duration_ms": 45678,
  "env_hash": "9a8b7c6d5e4f3g2h"
}
```

### Incremental Cache Entry
```json
{
  "package_name": "my-crate",
  "package_version": "0.1.0",
  "source_hash": "a1b2c3d4e5f6g7h8...",
  "cargo_lock_hash": "1a2b3c4d5e6f7g8h...",
  "command_hash": "build",
  "env_hash": "9a8b7c6d5e4f3g2h",
  "is_release": false,
  "features_hash": "f1e2d3c4b5a69788",
  "target_files": [
    ["/path/to/target/debug/.fingerprint/my-crate-abc/lib-my-crate", 1024],
    ["/path/to/target/debug/deps/libmy_crate.rlib", 524288]
  ],
  "artifact_paths": [
    "/path/to/target/debug/deps/libmy_crate.rlib",
    "/path/to/target/debug/deps/libmy_crate.rmeta"
  ],
  "timestamp": "2025-02-14T10:30:45-03:00",
  "build_success": true,
  "duration_ms": 5678
}
```

## Cache Validation

### Validation Steps
1. **Load cache entry** from incremental directory
2. **Check source hash** - Has code changed?
3. **Check Cargo.lock hash** - Have dependencies changed?
4. **Check environment hash** - Have build flags changed?
5. **Check features hash** - Have feature flags changed?
6. **Check target files** - Do all artifacts exist with correct sizes?

### Validation Logic
```rust
fn is_cache_valid(cache: &IncrementalCache, package: &PackageHash) -> bool {
    cache.source_hash == package.source_hash
        && cache.cargo_lock_hash == workspace.cargo_lock_hash
        && cache.env_hash == current_env_hash
        && cache.features_hash == current_features_hash
        && cache.target_files.iter().all(|(path, size)| {
            fs::metadata(path).map(|m| m.len() == *size).unwrap_or(false)
        })
        && cache.build_success
}
```

## Cache Invalidation

### Automatic Invalidation
Cache is automatically invalidated when:
- Source code changes (git detects changes)
- Cargo.lock changes (dependencies updated)
- Environment variables change (RUSTFLAGS, etc.)
- Feature flags change
- Build profile changes (debug ↔ release)
- Target files missing or corrupted

### Manual Invalidation
```bash
# Invalidate specific package
cargo-save invalidate my-package

# Invalidate all packages
cargo-save invalidate --all
```

### Transitive Invalidation
When package B changes and A depends on B:
1. B's cache is invalidated (source changed)
2. Dependency graph identifies A depends on B
3. A's cache is also invalidated (transitive)
4. Both A and B are rebuilt

## Cache Maintenance

### Cleaning Old Caches
```bash
# Remove caches older than 7 days
cargo-save clean --days 7

# Keep only last 10 builds
cargo-save clean --keep 10
```

**Implementation:**
- Sorts cache files by modification time
- Removes oldest entries first
- Cleans both logs and metadata
- Cleans corresponding incremental caches

### Cache Statistics
```bash
cargo-save stats
```

**Displays:**
- Total cache size
- Number of log files
- Number of metadata files
- Number of incremental caches
- Cache location

## Hash Computation

### Source Hash (Git-based)
```rust
fn compute_source_hash(path: &Path) -> String {
    let mut hasher = Blake3Hasher::new();
    
    // Get git tree hash
    let git_tree = Command::new("git")
        .args(["ls-tree", "-r", "HEAD"])
        .arg(path)
        .output()?;
    hasher.update(&git_tree.stdout);
    
    // Include uncommitted changes
    let git_status = Command::new("git")
        .args(["status", "--porcelain"])
        .arg(path)
        .output()?;
    hasher.update(&git_status.stdout);
    
    // Hash modified file contents
    for modified_file in parse_git_status(&git_status.stdout) {
        let content = fs::read(&modified_file)?;
        hasher.update(&content);
    }
    
    hasher.finalize().to_hex().to_string()
}
```

### Source Hash (Fallback)
```rust
fn compute_source_hash_fallback(path: &Path) -> String {
    let mut hasher = Blake3Hasher::new();
    
    for entry in WalkDir::new(path).max_depth(10) {
        if is_source_file(&entry) {
            hasher.update(entry.path().as_bytes());
            hasher.update(&fs::read(entry.path())?);
        }
    }
    
    hasher.finalize().to_hex().to_string()
}
```

### Environment Hash
```rust
fn compute_env_hash() -> String {
    let mut hasher = Blake3Hasher::new();
    
    for var in ENV_VARS_THAT_AFFECT_BUILD {
        if let Ok(value) = std::env::var(var) {
            hasher.update(var.as_bytes());
            hasher.update(value.as_bytes());
        }
    }
    
    hasher.finalize().to_hex().to_string()
}
```

### Features Hash
```rust
fn compute_features_hash(args: &[String]) -> String {
    let mut hasher = Blake3Hasher::new();
    
    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--features" => {
                if let Some(features) = args.get(i + 1) {
                    hasher.update(features.as_bytes());
                }
            }
            "--all-features" => hasher.update(b"--all-features"),
            "--no-default-features" => hasher.update(b"--no-default-features"),
            _ if arg.starts_with("--features=") => {
                hasher.update(arg.as_bytes());
            }
            _ => {}
        }
    }
    
    hasher.finalize().to_hex().to_string()
}
```

## Cache Versioning

**Current Version:** v3

**Version Changes:**
- v1: Initial implementation (deprecated)
- v2: Added features hash (deprecated)
- v3: Current - Added environment hash, improved validation

**Migration:**
- Old cache versions are ignored
- No automatic migration
- Users must rebuild to populate new cache

## Performance Optimization

### Parallel Hash Computation
```rust
let packages: Vec<PackageHash> = metadata
    .workspace_packages()
    .par_iter()  // Rayon parallel iterator
    .filter_map(|pkg| compute_package_hash(pkg).ok())
    .collect();
```

### Cache Lookup Optimization
- O(1) hash map lookup by cache key
- File existence check before reading
- Lazy loading of cache entries

### Memory Management
- Stream build output (don't buffer entire log)
- Incremental cache entries are small (<10KB each)
- Build logs stored separately (can be large)

## Future Enhancements

1. **Compression**: Compress large build logs
2. **Distributed Cache**: Share cache across machines (S3, Redis)
3. **Artifact Caching**: Cache actual build artifacts, not just metadata
4. **Smart Cleanup**: Remove least-used caches first
5. **Cache Analytics**: Track hit rate, time saved, etc.
6. **Checksum Validation**: Detect corrupted cache files
