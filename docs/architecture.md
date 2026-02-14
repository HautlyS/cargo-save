# Architecture Overview

## System Design

cargo-save is a smart caching wrapper around cargo that provides package-level incremental builds using git-based change detection.

## Core Components

### 1. CacheManager
Central orchestrator that manages all caching operations.

**Responsibilities:**
- Cache directory management
- Workspace state computation
- Incremental cache validation
- Build execution and output capture
- Cache queries and maintenance

**Key Methods:**
- `compute_workspace_state()` - Analyzes workspace and computes hashes
- `check_incremental_cache()` - Validates cached artifacts
- `run_cargo_with_cache()` - Executes cargo with caching
- `get_changed_packages()` - Determines what needs rebuilding

### 2. Hash Computation System
Uses Blake3 for fast, collision-resistant hashing.

**Hash Types:**
- **Source Hash**: Git tree hash + uncommitted changes
- **Cargo.lock Hash**: Dependency version tracking
- **Environment Hash**: Build-affecting environment variables
- **Features Hash**: Feature flag combinations
- **Toolchain Hash**: Rust compiler version
- **Command Hash**: Cargo command and arguments

### 3. Dependency Graph
Tracks workspace-internal dependencies for transitive invalidation.

**Structure:**
```rust
DependencyGraph {
    packages: HashMap<String, PackageNode>
}

PackageNode {
    name: String,
    dependencies: Vec<String>,
    reverse_dependencies: Vec<String>
}
```

**Purpose:**
- When package B changes and A depends on B, both are rebuilt
- Only tracks workspace members (external deps handled by Cargo.lock)

### 4. Cache Storage

**Directory Structure:**
```
~/.cache/cargo-save/v3/
├── {cache_id}.log              # Build output logs
├── metadata/
│   └── {cache_id}.json         # Build metadata
└── incremental/
    └── {cache_key}.json        # Package-level cache
```

**Cache Key Format:**
```
{package}-{source_hash}-{command_hash}-{env_hash}-{profile}-{features_hash}
```

## Data Flow

### Build Flow
```
1. Parse CLI arguments
   ↓
2. Load cargo metadata (workspace structure)
   ↓
3. Compute workspace state (parallel hash computation)
   ↓
4. Build dependency graph
   ↓
5. Check incremental cache for each package
   ↓
6. Determine changed packages + transitive dependents
   ↓
7. If all cached → skip build
   If some changed → run cargo build
   ↓
8. Capture stdout/stderr in real-time
   ↓
9. Save build logs and metadata
   ↓
10. Update incremental cache for rebuilt packages
```

### Cache Validation Flow
```
1. Load cache entry for package
   ↓
2. Validate source hash matches
   ↓
3. Validate Cargo.lock hash matches
   ↓
4. Validate environment hash matches
   ↓
5. Validate features hash matches
   ↓
6. Check all target files exist with correct sizes
   ↓
7. If all valid → cache hit
   If any invalid → cache miss
```

## Parallelization

Uses Rayon for parallel processing:
- Package hash computation (all packages in parallel)
- File walking during fallback hashing
- Dependency graph construction

## Git Integration

**Primary Method (Fast):**
```bash
git ls-tree -r HEAD <path>  # Get tracked files
git status --porcelain <path>  # Get uncommitted changes
```

**Fallback Method (Slower):**
- Walk directory tree
- Hash all .rs and .toml files
- Skip target/, .git/, node_modules/

## Cache Invalidation Strategy

A cache entry is invalidated if ANY of these change:
1. Source code (git tree hash)
2. Cargo.lock (dependency versions)
3. Environment variables (RUSTFLAGS, etc.)
4. Feature flags
5. Build profile (debug/release)
6. Toolchain version
7. Target files missing or size changed

## Performance Characteristics

**Time Complexity:**
- Hash computation: O(n) where n = number of files
- Cache lookup: O(1) per package
- Dependency graph: O(p + d) where p = packages, d = dependencies
- Transitive invalidation: O(p²) worst case, O(p) typical

**Space Complexity:**
- Cache storage: O(b × l) where b = builds, l = log size
- Memory usage: O(p) where p = packages

**Typical Performance:**
- Hash computation: 10-50ms per package (parallel)
- Cache lookup: <1ms per package
- Full workspace scan: 100-500ms for 50 packages
- Log query: <1ms (instant)

## Error Handling

**Strategy:**
- Use `anyhow::Result` for error propagation
- Context added at each layer
- Non-critical errors logged but don't fail build
- Cache corruption silently ignored (falls back to rebuild)

**Critical Errors (fail fast):**
- Cargo metadata unavailable
- Cache directory creation failed
- Cargo process spawn failed

**Non-Critical Errors (warn and continue):**
- Git not available (fallback to file hashing)
- Cache file corrupted (rebuild)
- Old cache format (ignore)

## Concurrency Model

**Thread Safety:**
- Read-only operations are thread-safe
- Write operations are sequential (single build at a time)
- No locks needed (process-level isolation)

**Parallel Operations:**
- Hash computation (rayon)
- File walking (rayon)
- Stdout/stderr capture (separate threads)

## Extension Points

**Future Enhancements:**
1. Distributed caching (S3, Redis)
2. Build artifact caching (not just metadata)
3. Cross-machine cache sharing
4. Compression for large logs
5. Build analytics and metrics
6. Custom hash strategies per package
