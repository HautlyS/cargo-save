# Git Integration

## Overview

cargo-save uses git as the primary mechanism for detecting source code changes. This provides fast, accurate change detection without hashing every file.

## Why Git?

**Advantages:**
- **Fast**: Git already tracks file changes efficiently
- **Accurate**: Detects both committed and uncommitted changes
- **Reliable**: Uses git's proven change detection
- **Incremental**: Only hashes modified files

**Disadvantages:**
- Requires git repository
- Requires git in PATH
- Doesn't work for non-git projects

## Git Commands Used

### 1. List Tracked Files
```bash
git ls-tree -r HEAD <path>
```

**Purpose:** Get all tracked files and their git object hashes

**Output Format:**
```
100644 blob a1b2c3d4e5f6g7h8  src/main.rs
100644 blob 9a8b7c6d5e4f3g2h  Cargo.toml
```

**Usage in cargo-save:**
```rust
let output = Command::new("git")
    .args(["ls-tree", "-r", "HEAD"])
    .arg(package_path)
    .output()?;

if output.status.success() && !output.stdout.is_empty() {
    hasher.update(&output.stdout);
}
```

### 2. Check Uncommitted Changes
```bash
git status --porcelain <path>
```

**Purpose:** Detect modified, added, or deleted files

**Output Format:**
```
 M src/main.rs
?? new_file.rs
 D old_file.rs
```

**Usage in cargo-save:**
```rust
let status = Command::new("git")
    .args(["status", "--porcelain"])
    .arg(package_path)
    .output()?;

if status.status.success() && !status.stdout.is_empty() {
    hasher.update(&status.stdout);
    
    // Hash content of modified files
    for line in String::from_utf8_lossy(&status.stdout).lines() {
        if line.len() > 3 {
            let file_path = &line[3..];
            let full_path = package_path.join(file_path);
            if full_path.exists() && full_path.is_file() {
                let content = fs::read(&full_path)?;
                hasher.update(&content);
            }
        }
    }
}
```

## Change Detection Algorithm

### Full Algorithm
```rust
fn compute_source_hash(path: &Path) -> Result<String> {
    let mut hasher = Blake3Hasher::new();
    
    // Step 1: Try git ls-tree (tracked files)
    let git_tree = Command::new("git")
        .args(["ls-tree", "-r", "HEAD"])
        .arg(path)
        .output()?;
    
    if git_tree.status.success() && !git_tree.stdout.is_empty() {
        // Git is available and path is tracked
        hasher.update(&git_tree.stdout);
        
        // Step 2: Check for uncommitted changes
        let git_status = Command::new("git")
            .args(["status", "--porcelain"])
            .arg(path)
            .output()?;
        
        if git_status.status.success() && !git_status.stdout.is_empty() {
            hasher.update(&git_status.stdout);
            
            // Step 3: Hash modified file contents
            let status_str = String::from_utf8_lossy(&git_status.stdout);
            for line in status_str.lines() {
                if line.len() > 3 {
                    let file_path = &line[3..];
                    let full_path = path.join(file_path);
                    if full_path.exists() && full_path.is_file() {
                        let content = fs::read(&full_path)?;
                        hasher.update(full_path.to_string_lossy().as_bytes());
                        hasher.update(&content);
                    }
                }
            }
        }
        
        return Ok(hasher.finalize().to_hex().to_string());
    }
    
    // Step 4: Fallback to file-based hashing
    compute_source_hash_fallback(path)
}
```

## Fallback Mechanism

When git is not available or path is not in a git repository:

```rust
fn compute_source_hash_fallback(path: &Path) -> Result<String> {
    static GIT_WARNING_SHOWN: AtomicBool = AtomicBool::new(false);
    
    if !GIT_WARNING_SHOWN.swap(true, Ordering::Relaxed) {
        eprintln!(
            "{} Warning: Git not available. Using file-based hashing.",
            LOG_PREFIX
        );
    }
    
    let mut hasher = Blake3Hasher::new();
    
    for entry in WalkDir::new(path)
        .follow_links(false)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let path_str = entry.path().to_string_lossy();
            
            // Skip non-source directories
            if path_str.contains("/target/")
                || path_str.contains("/.git/")
                || path_str.contains("/node_modules/")
            {
                continue;
            }
            
            // Only hash source files
            if let Some(ext) = entry.path().extension() {
                if matches!(ext.to_str(), Some("rs") | Some("toml")) {
                    let content = fs::read(entry.path())?;
                    hasher.update(entry.path().to_string_lossy().as_bytes());
                    hasher.update(&content);
                }
            }
        }
    }
    
    Ok(hasher.finalize().to_hex().to_string())
}
```

## Git Status Parsing

### Status Codes
```
' M' - Modified (not staged)
'M ' - Modified (staged)
'MM' - Modified (staged and unstaged)
'A ' - Added
'D ' - Deleted
'R ' - Renamed
'C ' - Copied
'??' - Untracked
'!!' - Ignored
```

### Parsing Logic
```rust
fn parse_git_status(status_output: &[u8]) -> Vec<PathBuf> {
    let status_str = String::from_utf8_lossy(status_output);
    let mut modified_files = Vec::new();
    
    for line in status_str.lines() {
        if line.len() > 3 {
            let status_code = &line[..2];
            let file_path = &line[3..];
            
            // Only care about modified/added files
            if status_code.contains('M') || status_code.contains('A') {
                modified_files.push(PathBuf::from(file_path));
            }
        }
    }
    
    modified_files
}
```

## Performance Characteristics

### Git-based (Fast)
- **Time**: 10-50ms per package
- **Scales**: O(changed files) not O(all files)
- **Parallel**: Yes (rayon)

### Fallback (Slower)
- **Time**: 50-500ms per package
- **Scales**: O(all files)
- **Parallel**: Yes (rayon)

## Edge Cases

### 1. Submodules
**Status:** ✅ Implemented
**Behavior:** Submodule status is tracked via `git submodule status`
**Impact:** Submodule changes are detected and invalidate cache

### 2. Ignored Files
**Status:** ✅ Working as expected
**Behavior:** Ignored by git, not hashed
**Impact:** Changes to ignored files don't invalidate cache (correct behavior)

### 3. Symlinks
**Status:** ✅ Handled
**Behavior:** `follow_links(false)` in fallback mode
**Impact:** Symlinks are not followed, preventing external file hashing

### 4. Large Files
**Status:** ⚠️ Basic implementation
**Behavior:** Entire file content is hashed
**Impact:** May be slow for very large files (>100MB)
**Future:** Consider using git hash-object or metadata-only hashing

### 5. Binary Files
**Status:** ✅ Working as expected
**Behavior:** Only .rs and .toml files hashed in fallback mode
**Impact:** Binary assets not tracked (correct for Rust builds)

## Git Requirements

### Git Availability
Git is optional but strongly recommended. Without git, cargo-save falls back to slower file-based hashing.

**Fallback behavior:**
- Walks directory tree
- Hashes only .rs and .toml files
- Shows one-time warning message
- Skips target/, .git/, node_modules/

### Git in PATH
Git must be available in PATH for git-based hashing. No special configuration required.

## Integration with Cargo

### Cargo.toml Changes
Cargo.toml is tracked by git, so changes are detected automatically.

### Cargo.lock Changes
Cargo.lock is hashed separately (not part of source hash).

### .cargo/config.toml
**Status:** Not currently tracked
**Impact:** Changes to cargo config don't invalidate cache
**Workaround:** Use `cargo-save invalidate --all` after config changes

## Debugging Git Integration

### Check Git Status
```bash
# What git sees
git ls-tree -r HEAD src/

# Uncommitted changes
git status --porcelain src/
```

### Verify Hash Computation
```bash
# Show hashes
cargo-save status --hashes
```

### Check Repository Features
```bash
# Run git integration example
cargo run --example git_integration
```

## Advanced Git Features

cargo-save fully supports advanced Git configurations:

### 1. Submodule Support

**Detection:**
```rust
fn get_submodule_status(&self, path: &Path) -> Option<Vec<u8>> {
    let output = Command::new("git")
        .args(["submodule", "status"])
        .current_dir(path)
        .output()
        .ok()?;
    
    if output.status.success() {
        Some(output.stdout)
    } else {
        None
    }
}
```

**Integration:**
- Submodule status is included in source hash
- Changes to submodule commits invalidate cache
- Works with nested submodules

**Usage:**
```bash
# Submodules are automatically detected
cargo-save build

# Status shows submodule detection
cargo-save status
```

### 2. Sparse Checkout Support

**Detection:**
```rust
fn get_sparse_checkout_patterns(&self, repo_info: &GitRepoInfo) -> Option<Vec<String>> {
    let sparse_file = repo_info.git_dir.join("info/sparse-checkout");
    if sparse_file.exists() {
        fs::read_to_string(&sparse_file).ok().map(|content| {
            content.lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect()
        })
    } else {
        None
    }
}
```

**Integration:**
- Sparse checkout patterns are included in hash
- Only checked-out files are tracked
- Pattern changes invalidate cache

**Setup:**
```bash
# Enable sparse checkout
git sparse-checkout init
git sparse-checkout set src/ Cargo.toml

# cargo-save automatically detects it
cargo-save build
```

### 3. Git Worktree Support

**Detection:**
```rust
pub fn get_git_repo_info(&self, path: &Path) -> Option<GitRepoInfo> {
    let git_dir_output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path)
        .output()
        .ok()?;
    
    let git_dir = PathBuf::from(String::from_utf8_lossy(&git_dir_output.stdout).trim());
    
    let is_worktree = git_dir.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n != ".git")
        .unwrap_or(true);
    
    let worktree_root = if is_worktree {
        Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(path)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
                } else {
                    None
                }
            })
    } else {
        None
    };
    
    Some(GitRepoInfo {
        is_worktree,
        worktree_root,
        // ... other fields
    })
}
```

**Integration:**
- Worktrees are automatically detected
- Uses worktree root for hashing
- Each worktree has independent cache

**Usage:**
```bash
# Create worktree
git worktree add ../my-feature feature-branch

# In worktree directory
cd ../my-feature
cargo-save build  # Uses worktree-specific cache
```

### 4. Git LFS Support

**Detection:**
```rust
fn is_lfs_file(&self, path: &Path, repo_info: &GitRepoInfo) -> bool {
    if !repo_info.has_lfs {
        return false;
    }
    
    if let Ok(content) = fs::read_to_string(path) {
        content.starts_with("version https://git-lfs.github.com/spec/")
    } else {
        false
    }
}

fn get_lfs_pointer_hash(&self, path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().and_then(|content| {
        for line in content.lines() {
            if line.starts_with("oid sha256:") {
                return line.strip_prefix("oid sha256:")
                    .map(|s| s.trim().to_string());
            }
        }
        None
    })
}
```

**Integration:**
- LFS pointer files are detected automatically
- Hashes the OID instead of file content
- Prevents downloading large files unnecessarily

**LFS Pointer Format:**
```
version https://git-lfs.github.com/spec/v1
oid sha256:4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393
size 12345
```

**Usage:**
```bash
# LFS is automatically detected
cargo-save build

# Modified LFS files are tracked by OID
echo "new content" > large-file.bin
git add large-file.bin
cargo-save build  # Detects LFS pointer change
```

### 5. Shallow Clone Support

**Detection:**
```rust
let is_shallow = git_dir.join("shallow").exists();
```

**Integration:**
- Shallow clone status is detected
- Shallow file content is included in hash
- Works correctly with limited history

**Usage:**
```bash
# Clone with limited depth
git clone --depth 1 https://github.com/user/repo.git

# cargo-save works normally
cd repo
cargo-save build
```

### 6. Git Hooks Integration

**Installation:**
```rust
pub fn install_git_hooks(&self, workspace_root: &Path) -> Result<()> {
    let git_dir = Command::new("git")
        .args(["rev-parse", "--git-common-dir"])
        .current_dir(workspace_root)
        .output()?;
    
    let git_dir_path = PathBuf::from(String::from_utf8_lossy(&git_dir.stdout).trim());
    let hooks_dir = git_dir_path.join("hooks");
    
    fs::create_dir_all(&hooks_dir)?;
    
    // Install post-checkout hook
    let post_checkout_hook = hooks_dir.join("post-checkout");
    fs::write(&post_checkout_hook, POST_CHECKOUT_SCRIPT)?;
    
    // Install post-merge hook
    let post_merge_hook = hooks_dir.join("post-merge");
    fs::write(&post_merge_hook, POST_MERGE_SCRIPT)?;
    
    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&post_checkout_hook, 
            fs::Permissions::from_mode(0o755))?;
        fs::set_permissions(&post_merge_hook, 
            fs::Permissions::from_mode(0o755))?;
    }
    
    Ok(())
}
```

**Hooks Installed:**

**post-checkout:**
```bash
#!/bin/sh
# Invalidates cache when switching branches
if command -v cargo-save >/dev/null 2>&1; then
    if [ "$3" = "1" ]; then
        echo "[cargo-save] Branch changed, invalidating cache..."
        cargo-save invalidate --all 2>/dev/null || true
    fi
fi
```

**post-merge:**
```bash
#!/bin/sh
# Invalidates cache after merges
if command -v cargo-save >/dev/null 2>&1; then
    echo "[cargo-save] Merge completed, invalidating cache..."
    cargo-save invalidate --all 2>/dev/null || true
fi
```

**Usage:**
```bash
# Install hooks
cargo-save install-hooks

# Hooks run automatically
git checkout feature-branch  # Cache invalidated
git merge main              # Cache invalidated
```

## GitRepoInfo Structure

```rust
pub struct GitRepoInfo {
    /// Whether this is a git worktree
    pub is_worktree: bool,
    /// Whether this is a shallow clone
    pub is_shallow: bool,
    /// Whether Git LFS is being used
    pub has_lfs: bool,
    /// Whether sparse checkout is enabled
    pub is_sparse: bool,
    /// Path to the git directory
    pub git_dir: PathBuf,
    /// Path to the worktree root (for worktrees)
    pub worktree_root: Option<PathBuf>,
}
```

## GitFeaturesInfo Structure

```rust
pub struct GitFeaturesInfo {
    /// Whether submodules are present
    pub has_submodules: bool,
    /// Whether sparse checkout is enabled
    pub is_sparse: bool,
    /// Whether this is a worktree
    pub is_worktree: bool,
    /// Whether Git LFS is in use
    pub has_lfs: bool,
    /// Whether this is a shallow clone
    pub is_shallow: bool,
}
```

## Future Enhancements

Potential improvements for future versions:
- Track .cargo/config.toml changes
- Track build.rs changes
- Partial clone support (git clone --filter)
- Git bundle support
- Signed commit verification in hash
- Git notes integration for cache metadata
- Use git hash-object for large files
