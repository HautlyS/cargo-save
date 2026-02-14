//! cargo-save - Smart caching cargo wrapper with git-based incremental builds
//!
//! [![Crates.io](https://img.shields.io/crates/v/cargo-save.svg)](https://crates.io/crates/cargo-save)
//! [![Documentation](https://docs.rs/cargo-save/badge.svg)](https://docs.rs/cargo-save)
//! [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
//! [![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
//!
//! This library provides package-level incremental caching for Rust workspaces
//! using git-based change detection. It can be used both as a CLI tool and
//! as a library for custom build tooling.
//!
//! # Overview
//!
//! `cargo-save` provides:
//!
//! - **Package-level incremental caching**: Only rebuilds packages that actually changed
//! - **Git-based change detection**: Uses git for fast, accurate change tracking
//! - **Advanced Git support**: Submodules, worktrees, sparse checkout, LFS, shallow clones
//! - **Smart cache invalidation**: Considers dependencies, features, environment variables
//! - **Build log caching**: Query previous build outputs without rebuilding
//! - **Workspace-aware**: Optimized for multi-package workspaces
//!
//! # Quick Start
//!
//! ```no_run
//! use cargo_save::CacheManager;
//! use anyhow::Result;
//!
//! fn main() -> Result<()> {
//!     // Initialize cache manager
//!     let cache = CacheManager::new()?;
//!     
//!     // Compute current workspace state
//!     let workspace = cache.compute_workspace_state(&[])?;
//!     
//!     // Run cargo build with caching
//!     let (cache_id, exit_code, lines, duration) = cache
//!         .run_cargo_with_cache("build", &[], &workspace)?;
//!     
//!     println!("Build completed in {}ms", duration);
//!     Ok(())
//! }
//! ```
//!
//! # Installation
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! cargo-save = "0.2"
//! ```
//!
//! Or install as a CLI tool:
//!
//! ```bash
//! cargo install cargo-save
//! ```
//!
//! # Architecture
//!
//! The library is organized into several key components:
//!
//! - [`CacheManager`]: Central orchestrator for all caching operations
//! - [`WorkspaceState`]: Represents the current state of a Cargo workspace
//! - [`PackageHash`]: Contains hash information for individual packages
//! - [`DependencyGraph`]: Tracks workspace dependencies for transitive invalidation
//! - [`IncrementalCache`]: Stores cached build information
//!
//! # Cache Strategy
//!
//! Cache entries are keyed by multiple factors to ensure correctness:
//!
//! 1. **Package source hash**: Git tree hash or file content hash
//! 2. **Command hash**: The cargo subcommand and arguments
//! 3. **Environment hash**: Relevant environment variables (RUSTFLAGS, etc.)
//! 4. **Build profile**: Debug vs release
//! 5. **Features hash**: Enabled feature flags
//! 6. **Cargo.lock hash**: Dependency versions
//!
//! A cache is only considered valid if ALL these factors match.
//!
//! # Safety and Thread Safety
//!
//! All public types in this crate are thread-safe and can be safely shared between threads:
//!
//! - [`CacheManager`] uses only immutable internal state after construction
//! - All operations that modify the filesystem are atomic where possible
//! - Concurrent reads from the cache are safe
//!
//! # Error Handling
//!
//! This crate uses [`anyhow`] for error handling. All fallible operations return
//! `anyhow::Result<T>`, which provides rich error context and easy error propagation.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use cargo_save::CacheManager;
//!
//! # fn main() -> anyhow::Result<()> {
//! let cache = CacheManager::new()?;
//! let workspace = cache.compute_workspace_state(&[])?;
//!
//! // Build with caching
//! let (_, exit_code, _, _) = cache.run_cargo_with_cache("build", &[], &workspace)?;
//! assert_eq!(exit_code, Some(0));
//! # Ok(())
//! # }
//! ```
//!
//! ## Query Build Logs
//!
//! ```no_run
//! use cargo_save::CacheManager;
//!
//! # fn main() -> anyhow::Result<()> {
//! let cache = CacheManager::new()?;
//!
//! // Query recent errors from cached builds
//! cache.query_logs("errors", None, None, Some(5))?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Cache Management
//!
//! ```no_run
//! use cargo_save::CacheManager;
//!
//! # fn main() -> anyhow::Result<()> {
//! let cache = CacheManager::new()?;
//!
//! // Show statistics
//! cache.show_stats()?;
//!
//! // Clean old caches
//! cache.clean_old_caches(7, None, false)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Build Script
//!
//! ```no_run
//! use cargo_save::CacheManager;
//!
//! # fn main() -> anyhow::Result<()> {
//! let cache = CacheManager::new()?;
//! let workspace = cache.compute_workspace_state(&[])?;
//!
//! // Check which packages need rebuilding
//! let command_hash = cache.compute_command_hash("build", &["--release".to_string()]);
//! let env_hash = cache.compute_env_hash();
//! let changed = cache.get_changed_packages(&workspace, &command_hash, &env_hash, true, &["--release".to_string()]);
//!
//! println!("Packages to rebuild: {}", changed.len());
//! for pkg in &changed {
//!     println!("  - {}", pkg.name);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Feature Flags
//!
//! This crate does not currently use feature flags. All functionality is enabled by default.
//!
//! # Platform Support
//!
//! - **Linux**: Full support
//! - **macOS**: Full support
//! - **Windows**: Full support
//!
//! # Minimum Supported Rust Version (MSRV)
//!
//! This crate requires Rust 1.70.0 or later.
//!
//! # Comparison with Similar Tools
//!
//! | Feature | cargo-save | sccache | cargo-cache |
//! |:--------|:----------:|:-------:|:-----------:|
//! | Caching Level | Package-level | Compiler-level | Target directory |
//! | Git Integration | Native | None | None |
//! | Workspace Aware | ✅ | ❌ | ⚠️ |
//! | Build Log Storage | ✅ | ❌ | ❌ |
//! | Distributed Caching | ❌ | ✅ | ❌ |
//! | Setup Complexity | Zero | Daemon | Manual |
//!
//! See the [README](https://github.com/HautlyS/cargo-save) for a detailed comparison.

// All modules are defined inline in this file for simplicity

use anyhow::{Context, Result};
use blake3::Hasher as Blake3Hasher;
use cargo_metadata::{Metadata, MetadataCommand, Package};
use clap::Parser;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

/// Command-line interface for cargo-save
///
/// This enum defines all the subcommands available in the cargo-save CLI.
#[derive(Parser)]
#[command(name = "cargo-save")]
#[command(
    about = "Smart caching cargo wrapper with git-based incremental builds",
    version
)]
pub enum Cli {
    /// Save subcommand (called as `cargo save`)
    #[command(name = "save")]
    Save {
        /// The cargo subcommand to run
        subcommand: String,
        /// Arguments to pass to cargo
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Direct invocation (called as `cargo-save`)
    #[command(hide = true)]
    Direct {
        /// The cargo subcommand to run
        subcommand: String,
        /// Arguments to pass to cargo
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Query cached build logs
    #[command(name = "query")]
    Query {
        /// Query mode: head, tail, grep, range, errors, warnings, all
        mode: String,
        /// Parameter for the query (line count, pattern, range)
        param: Option<String>,
        /// Specific cache ID to query
        #[arg(short, long)]
        id: Option<String>,
        /// Query the Nth most recent build
        #[arg(short, long)]
        last: Option<usize>,
    },

    /// List cached builds
    #[command(name = "list")]
    List {
        /// Show verbose information
        #[arg(short, long)]
        verbose: bool,
        /// Only show caches for current workspace
        #[arg(short, long)]
        workspace: bool,
    },

    /// Clean old cache files
    #[command(name = "clean")]
    Clean {
        /// Remove caches older than this many days
        #[arg(short, long, default_value = "7")]
        days: u64,
        /// Keep only this many most recent caches
        #[arg(short, long)]
        keep: Option<usize>,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Show cache statistics
    #[command(name = "stats")]
    Stats,

    /// Invalidate caches
    #[command(name = "invalidate")]
    Invalidate {
        /// Package names to invalidate
        packages: Vec<String>,
        /// Invalidate all caches
        #[arg(short, long)]
        all: bool,
    },

    /// Show workspace status
    #[command(name = "status")]
    Status {
        /// Show package hashes
        #[arg(long)]
        hashes: bool,
    },

    /// Generate cache key for CI systems
    #[command(name = "cache-key")]
    CacheKey {
        /// CI platform: github, gitlab, etc.
        #[arg(short, long, default_value = "github")]
        platform: String,
    },

    /// Pre-warm cache by computing hashes
    #[command(name = "warm")]
    Warm {
        /// Use release profile
        #[arg(long)]
        release: bool,
    },

    /// Install git hooks for auto-invalidation
    #[command(name = "install-hooks")]
    InstallHooks,

    /// Check environment and integration status
    #[command(name = "doctor")]
    Doctor,

    /// Setup sccache for cross-project caching
    #[command(name = "setup-sccache")]
    SetupSccache,
}

const CACHE_VERSION: &str = "v4";
const LOG_PREFIX: &str = "[cargo-save]";
const HASH_DISPLAY_LEN: usize = 16;

/// Environment variables that can affect the build output.
/// These are included in the cache key to ensure cache correctness.
pub const ENV_VARS_THAT_AFFECT_BUILD: &[&str] = &[
    "RUSTFLAGS",
    "RUSTDOCFLAGS",
    "CARGO_TARGET_DIR",
    "CARGO_HOME",
    "CARGO_NET_OFFLINE",
    "CARGO_BUILD_JOBS",
    "CARGO_BUILD_TARGET",
    "CARGO_BUILD_RUSTFLAGS",
    "CARGO_INCREMENTAL",
    "CARGO_PROFILE_DEV_DEBUG",
    "CARGO_PROFILE_RELEASE_DEBUG",
    "CARGO_PROFILE_RELEASE_OPT_LEVEL",
    "CARGO_PROFILE_RELEASE_LTO",
    "CC",
    "CXX",
    "AR",
    "LINKER",
];

/// Git repository information for advanced git features support.
#[derive(Debug, Clone)]
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

/// Represents a cached build with all metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCache {
    /// Unique identifier for this cache entry
    pub cache_id: String,
    /// Full command that was executed
    pub command: String,
    /// Cargo subcommand used
    pub subcommand: String,
    /// Arguments passed to cargo
    pub args: Vec<String>,
    /// Timestamp of the build
    pub timestamp: String,
    /// Exit code of the build (None if killed)
    pub exit_code: Option<i32>,
    /// Workspace state at build time
    pub workspace_state: WorkspaceState,
    /// Whether this was a release build
    pub is_release: bool,
    /// Target directory used
    pub target_dir: Option<PathBuf>,
    /// Number of lines in the build log
    pub lines_count: usize,
    /// Build duration in milliseconds
    pub duration_ms: u64,
    /// Hash of relevant environment variables
    pub env_hash: String,
}

/// Represents an incremental cache entry for a single package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalCache {
    /// Name of the package
    pub package_name: String,
    /// Version of the package
    pub package_version: String,
    /// Hash of the package source
    pub source_hash: String,
    /// Hash of Cargo.lock
    pub cargo_lock_hash: String,
    /// Hash of the command
    pub command_hash: String,
    /// Hash of environment variables
    pub env_hash: String,
    /// Whether this was a release build
    pub is_release: bool,
    /// Hash of feature flags
    pub features_hash: String,
    /// Target files and their sizes
    pub target_files: Vec<(PathBuf, u64)>,
    /// Paths to built artifacts
    pub artifact_paths: Vec<PathBuf>,
    /// Timestamp of the build
    pub timestamp: String,
    /// Whether the build succeeded
    pub build_success: bool,
    /// Build duration in milliseconds
    pub duration_ms: u64,
}

/// Represents the current state of a Cargo workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceState {
    /// Root directory of the workspace
    pub root: PathBuf,
    /// All packages in the workspace
    pub packages: Vec<PackageHash>,
    /// Hash of Cargo.lock
    pub cargo_lock_hash: String,
    /// Hash of the Rust toolchain
    pub toolchain_hash: String,
    /// Timestamp when state was computed
    pub timestamp: String,
    /// Information about git features in use
    pub git_features: Option<GitFeaturesInfo>,
}

/// Information about Git features being used.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Hash information for a single package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageHash {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Path to the package manifest directory
    pub path: PathBuf,
    /// Hash of the package source
    pub source_hash: String,
    /// Names of workspace dependencies
    pub dependencies: Vec<String>,
    /// Hash of feature flags
    pub features_hash: String,
}

/// Dependency graph for workspace packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Map of package names to their dependency information
    pub packages: HashMap<String, PackageNode>,
}

/// Node in the dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageNode {
    /// Package name
    pub name: String,
    /// Names of packages this package depends on
    pub dependencies: Vec<String>,
    /// Names of packages that depend on this package
    pub reverse_dependencies: Vec<String>,
}

/// Central manager for all caching operations.
///
/// This is the main interface for using cargo-save as a library.
/// It handles cache storage, computation, and retrieval.
///
/// # Example
///
/// ```no_run
/// use cargo_save::CacheManager;
///
/// # fn main() -> anyhow::Result<()> {
/// let cache = CacheManager::new()?;
/// let workspace = cache.compute_workspace_state(&[])?;
///
/// // Check which packages need rebuilding
/// let changed = cache.get_changed_packages(&workspace, "hash", "env", false, &[]);
/// println!("{} packages need rebuilding", changed.len());
/// # Ok(())
/// # }
/// ```
pub struct CacheManager {
    /// Directory for general cache files
    pub cache_dir: PathBuf,
    /// Directory for incremental cache files
    pub incremental_dir: PathBuf,
    /// Directory for metadata files
    pub metadata_dir: PathBuf,
}

impl CacheManager {
    /// Creates a new CacheManager with the default cache directory.
    ///
    /// The cache directory is determined by:
    /// 1. The `CARGO_SAVE_CACHE_DIR` environment variable, if set
    /// 2. The system cache directory (`~/.cache/cargo-save` on Linux)
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directories cannot be created.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cargo_save::CacheManager;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let cache = CacheManager::new()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        let cache_dir = if let Ok(custom_dir) = std::env::var("CARGO_SAVE_CACHE_DIR") {
            PathBuf::from(custom_dir)
        } else {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("cargo-save")
        }
        .join(CACHE_VERSION);

        let incremental_dir = cache_dir.join("incremental");
        let metadata_dir = cache_dir.join("metadata");

        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(&incremental_dir)?;
        fs::create_dir_all(&metadata_dir)?;

        Ok(Self {
            cache_dir,
            incremental_dir,
            metadata_dir,
        })
    }

    /// Gets Cargo metadata for the current workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if cargo metadata cannot be retrieved.
    pub fn get_cargo_metadata(&self) -> Result<Metadata> {
        let metadata = MetadataCommand::new()
            .exec()
            .context("Failed to get cargo metadata")?;
        Ok(metadata)
    }

    /// Computes a hash of the current Rust toolchain.
    ///
    /// This includes the rustc and cargo versions.
    pub fn compute_toolchain_hash(&self) -> Result<String> {
        let mut hasher = Blake3Hasher::new();

        if let Ok(output) = Command::new("rustc").args(["--version"]).output() {
            if output.status.success() {
                hasher.update(&output.stdout);
            }
        }

        if let Ok(output) = Command::new("cargo").args(["--version"]).output() {
            if output.status.success() {
                hasher.update(&output.stdout);
            }
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Computes a hash of the Cargo.lock file.
    ///
    /// # Errors
    ///
    /// Returns an error if the Cargo.lock file cannot be read.
    pub fn compute_cargo_lock_hash(&self, workspace_root: &Path) -> Result<String> {
        let lock_file = workspace_root.join("Cargo.lock");

        if lock_file.exists() {
            let content = fs::read(&lock_file)?;
            let mut hasher = Blake3Hasher::new();
            hasher.update(&content);
            Ok(hasher.finalize().to_hex().to_string())
        } else {
            Ok("no-lock-file".to_string())
        }
    }

    /// Computes a hash of relevant environment variables.
    ///
    /// See [`ENV_VARS_THAT_AFFECT_BUILD`] for the list of variables included.
    pub fn compute_env_hash(&self) -> String {
        let mut hasher = Blake3Hasher::new();

        for var in ENV_VARS_THAT_AFFECT_BUILD {
            if let Ok(value) = std::env::var(var) {
                hasher.update(var.as_bytes());
                hasher.update(value.as_bytes());
            }
        }

        hasher.finalize().to_hex().to_string()
    }

    /// Computes a hash of feature flags from command arguments.
    ///
    /// Recognizes `--features`, `--all-features`, and `--no-default-features`.
    pub fn compute_features_hash(&self, args: &[String]) -> String {
        let mut hasher = Blake3Hasher::new();

        for (i, arg) in args.iter().enumerate() {
            if arg == "--features" {
                if let Some(features) = args.get(i + 1) {
                    hasher.update(features.as_bytes());
                }
            } else if arg.starts_with("--features=") {
                if let Some(features) = arg.strip_prefix("--features=") {
                    hasher.update(features.as_bytes());
                }
            } else if arg == "--all-features" {
                hasher.update(b"--all-features");
            } else if arg == "--no-default-features" {
                hasher.update(b"--no-default-features");
            }
        }

        hasher.finalize().to_hex().to_string()
    }

    /// Gets information about the git repository at the given path.
    ///
    /// Returns `None` if the path is not in a git repository.
    pub fn get_git_repo_info(&self, path: &Path) -> Option<GitRepoInfo> {
        let git_dir_output = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(path)
            .output()
            .ok()?;

        if !git_dir_output.status.success() {
            return None;
        }

        let git_dir_str = String::from_utf8_lossy(&git_dir_output.stdout);
        let git_dir = PathBuf::from(git_dir_str.trim());

        let is_worktree = git_dir
            .file_name()
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

        let is_shallow = git_dir.join("shallow").exists();

        let has_lfs = Command::new("git")
            .args(["lfs", "status"])
            .current_dir(path)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let is_sparse = git_dir.join("info/sparse-checkout").exists();

        Some(GitRepoInfo {
            is_worktree,
            is_shallow,
            has_lfs,
            is_sparse,
            git_dir,
            worktree_root,
        })
    }

    /// Checks if a file is managed by Git LFS.
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

    /// Gets the SHA256 hash from an LFS pointer file.
    fn get_lfs_pointer_hash(&self, path: &Path) -> Option<String> {
        fs::read_to_string(path).ok().and_then(|content| {
            for line in content.lines() {
                if line.starts_with("oid sha256:") {
                    return line
                        .strip_prefix("oid sha256:")
                        .map(|s| s.trim().to_string());
                }
            }
            None
        })
    }

    /// Computes a hash of the source files in a package.
    ///
    /// Uses git tree hashes when available, falling back to file content hashing.
    /// Handles git submodules, LFS files, sparse checkouts, and worktrees.
    ///
    /// # Errors
    ///
    /// Returns an error if source files cannot be read.
    pub fn compute_source_hash(&self, path: &Path, _args: &[String]) -> Result<String> {
        let mut hasher = Blake3Hasher::new();

        let repo_info = self.get_git_repo_info(path);

        let effective_path = if let Some(ref info) = repo_info {
            if info.is_worktree {
                if let Some(ref worktree_root) = info.worktree_root {
                    worktree_root.as_path()
                } else {
                    path
                }
            } else {
                path
            }
        } else {
            path
        };

        // Try to use git for fast tree hashing
        if let Ok(output) = Command::new("git")
            .args(["ls-tree", "-r", "HEAD"])
            .arg(effective_path)
            .output()
        {
            if output.status.success() && !output.stdout.is_empty() {
                hasher.update(&output.stdout);

                // Include uncommitted changes
                if let Ok(status_output) = Command::new("git")
                    .args(["status", "--porcelain"])
                    .arg(effective_path)
                    .output()
                {
                    if status_output.status.success() && !status_output.stdout.is_empty() {
                        hasher.update(&status_output.stdout);

                        let status_str = String::from_utf8_lossy(&status_output.stdout);
                        for line in status_str.lines() {
                            if line.len() > 3 {
                                let file_path = &line[3..];
                                let full_path = path.join(file_path);
                                if full_path.exists() && full_path.is_file() {
                                    self.hash_file_with_lfs_support(
                                        &full_path,
                                        &repo_info,
                                        &mut hasher,
                                    )?;
                                }
                            }
                        }
                    }
                }

                // Include submodule status
                if let Some(submodule_status) = self.get_submodule_status(effective_path) {
                    if !submodule_status.is_empty() {
                        hasher.update(b"SUBMODULES:");
                        hasher.update(&submodule_status);
                    }
                }

                // Include sparse checkout patterns
                if let Some(ref info) = repo_info {
                    if info.is_sparse {
                        if let Some(patterns) = self.get_sparse_checkout_patterns(info) {
                            hasher.update(b"SPARSE:");
                            for pattern in patterns {
                                hasher.update(pattern.as_bytes());
                            }
                        }
                    }
                }

                // Include shallow clone info
                if let Some(ref info) = repo_info {
                    if info.is_shallow {
                        hasher.update(b"SHALLOW_CLONE");
                        let shallow_file = info.git_dir.join("shallow");
                        if let Ok(content) = fs::read(&shallow_file) {
                            hasher.update(&content);
                        }
                    }
                }

                return Ok(hasher.finalize().to_hex().to_string());
            }
        }

        // Fallback to file-based hashing
        static GIT_WARNING_SHOWN: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        if !GIT_WARNING_SHOWN.swap(true, std::sync::atomic::Ordering::Relaxed) {
            eprintln!(
                "{} Warning: Git not available or not in a git repository. Using file-based hashing (less accurate).",
                LOG_PREFIX
            );
        }

        for entry in WalkDir::new(path)
            .follow_links(false)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path_str = entry.path().to_string_lossy();

                if path_str.contains("/target/")
                    || path_str.contains("/.git/")
                    || path_str.contains("/node_modules/")
                {
                    continue;
                }

                if let Some(ext) = entry.path().extension() {
                    if matches!(ext.to_str(), Some("rs") | Some("toml")) {
                        if let Ok(content) = fs::read(entry.path()) {
                            hasher.update(entry.path().to_string_lossy().as_bytes());
                            hasher.update(&content);
                        }
                    }
                }
            }
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Helper function to hash a file, handling LFS files specially.
    fn hash_file_with_lfs_support(
        &self,
        path: &Path,
        repo_info: &Option<GitRepoInfo>,
        hasher: &mut Blake3Hasher,
    ) -> Result<()> {
        if let Some(ref info) = repo_info {
            if self.is_lfs_file(path, info) {
                if let Some(oid) = self.get_lfs_pointer_hash(path) {
                    hasher.update(b"LFS:");
                    hasher.update(oid.as_bytes());
                    return Ok(());
                }
            }
        }

        if let Ok(content) = fs::read(path) {
            hasher.update(path.to_string_lossy().as_bytes());
            hasher.update(&content);
        }

        Ok(())
    }

    /// Gets the status of git submodules.
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

    /// Gets sparse checkout patterns from the git repository.
    fn get_sparse_checkout_patterns(&self, repo_info: &GitRepoInfo) -> Option<Vec<String>> {
        let sparse_file = repo_info.git_dir.join("info/sparse-checkout");
        if sparse_file.exists() {
            fs::read_to_string(&sparse_file).ok().map(|content| {
                content
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .collect()
            })
        } else {
            None
        }
    }

    /// Computes a hash for a single package.
    ///
    /// # Errors
    ///
    /// Returns an error if the package manifest directory cannot be determined
    /// or if source hashing fails.
    pub fn compute_package_hash(
        &self,
        package: &Package,
        metadata: &Metadata,
        args: &[String],
    ) -> Result<PackageHash> {
        let manifest_dir = package
            .manifest_path
            .parent()
            .context("No manifest directory")?;

        let source_hash = self.compute_source_hash(manifest_dir.as_std_path(), args)?;
        let features_hash = self.compute_features_hash(args);

        let mut dependencies = Vec::new();

        for dep in &package.dependencies {
            if metadata.workspace_members.iter().any(|member_id| {
                metadata
                    .packages
                    .iter()
                    .find(|p| &p.id == member_id)
                    .map(|p| p.name == dep.name)
                    .unwrap_or(false)
            }) {
                dependencies.push(dep.name.clone());
            }
        }

        Ok(PackageHash {
            name: package.name.clone(),
            version: package.version.to_string(),
            path: manifest_dir.as_std_path().to_path_buf(),
            source_hash,
            dependencies,
            features_hash,
        })
    }

    /// Computes the current state of the entire workspace.
    ///
    /// This is the main entry point for determining what needs to be built.
    /// It computes hashes for all packages, the Cargo.lock file, and the toolchain.
    ///
    /// # Errors
    ///
    /// Returns an error if cargo metadata cannot be retrieved or if hashing fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cargo_save::CacheManager;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let cache = CacheManager::new()?;
    /// let workspace = cache.compute_workspace_state(&[])?;
    ///
    /// println!("Workspace has {} packages", workspace.packages.len());
    /// for pkg in &workspace.packages {
    ///     println!("  - {}", pkg.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn compute_workspace_state(&self, args: &[String]) -> Result<WorkspaceState> {
        let metadata = self.get_cargo_metadata()?;
        let root: PathBuf = metadata.workspace_root.clone().into();

        let packages: Vec<PackageHash> = metadata
            .workspace_packages()
            .par_iter()
            .filter_map(|package| self.compute_package_hash(package, &metadata, args).ok())
            .collect();

        let cargo_lock_hash = self.compute_cargo_lock_hash(&root)?;
        let toolchain_hash = self.compute_toolchain_hash()?;

        let git_features = self.get_git_repo_info(&root).map(|info| {
            let has_submodules = self
                .get_submodule_status(&root)
                .map(|s| !s.is_empty())
                .unwrap_or(false);

            GitFeaturesInfo {
                has_submodules,
                is_sparse: info.is_sparse,
                is_worktree: info.is_worktree,
                has_lfs: info.has_lfs,
                is_shallow: info.is_shallow,
            }
        });

        Ok(WorkspaceState {
            root,
            packages,
            cargo_lock_hash,
            toolchain_hash,
            timestamp: chrono::Local::now().to_rfc3339(),
            git_features,
        })
    }

    /// Builds a dependency graph from the workspace state.
    ///
    /// This graph is used to determine transitive dependencies - when a package
    /// changes, all packages that depend on it also need to be rebuilt.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cargo_save::CacheManager;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let cache = CacheManager::new()?;
    /// let workspace = cache.compute_workspace_state(&[])?;
    /// let graph = cache.build_dependency_graph(&workspace);
    ///
    /// if let Some(node) = graph.packages.get("my-package") {
    ///     println!("Has {} dependencies", node.dependencies.len());
    ///     println!("Has {} reverse dependencies", node.reverse_dependencies.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn build_dependency_graph(&self, workspace_state: &WorkspaceState) -> DependencyGraph {
        let mut packages = HashMap::new();

        for package in &workspace_state.packages {
            let reverse_deps: Vec<String> = workspace_state
                .packages
                .iter()
                .filter(|p| p.dependencies.contains(&package.name))
                .map(|p| p.name.clone())
                .collect();

            packages.insert(
                package.name.clone(),
                PackageNode {
                    name: package.name.clone(),
                    dependencies: package.dependencies.clone(),
                    reverse_dependencies: reverse_deps,
                },
            );
        }

        DependencyGraph { packages }
    }

    /// Computes a hash for a cargo command.
    ///
    /// This includes the subcommand, arguments, and current working directory.
    pub fn compute_command_hash(&self, subcommand: &str, args: &[String]) -> String {
        let mut hasher = Blake3Hasher::new();
        hasher.update(subcommand.as_bytes());
        hasher.update(args.join(" ").as_bytes());

        if let Ok(cwd) = std::env::current_dir() {
            hasher.update(cwd.to_string_lossy().as_bytes());
        }

        hasher.finalize().to_hex()[..HASH_DISPLAY_LEN].to_string()
    }

    /// Checks if the arguments indicate a release build.
    pub fn is_release_build(&self, args: &[String]) -> bool {
        args.iter()
            .any(|arg| arg == "--release" || arg.starts_with("--release"))
    }

    /// Gets the target directory from arguments or environment.
    pub fn get_target_dir(&self, args: &[String]) -> Option<PathBuf> {
        for (i, arg) in args.iter().enumerate() {
            if arg == "--target-dir" {
                return args.get(i + 1).map(PathBuf::from);
            }
            if arg.starts_with("--target-dir=") {
                return arg.split('=').nth(1).map(PathBuf::from);
            }
        }

        if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
            return Some(PathBuf::from(target_dir));
        }

        None
    }

    /// Generates a cache key for a package build.
    fn get_cache_key(
        &self,
        package: &PackageHash,
        command_hash: &str,
        env_hash: &str,
        is_release: bool,
        features_hash: &str,
    ) -> String {
        format!(
            "{}-{}-{}-{}-{}-{}",
            package.name,
            &package.source_hash[..HASH_DISPLAY_LEN],
            command_hash,
            env_hash,
            if is_release { "release" } else { "debug" },
            features_hash
        )
    }

    /// Checks if a valid incremental cache exists for a package.
    ///
    /// Returns `Some(IncrementalCache)` if a valid cache is found, `None` otherwise.
    /// A cache is valid if:
    /// - The Cargo.lock hash matches
    /// - The environment hash matches
    /// - The features hash matches
    /// - The source hash matches
    /// - All target files exist with correct sizes
    pub fn check_incremental_cache(
        &self,
        package: &PackageHash,
        workspace_state: &WorkspaceState,
        command_hash: &str,
        env_hash: &str,
        is_release: bool,
        args: &[String],
    ) -> Option<IncrementalCache> {
        let features_hash = self.compute_features_hash(args);

        let cache_key =
            self.get_cache_key(package, command_hash, env_hash, is_release, &features_hash);

        let cache_file = self.incremental_dir.join(format!("{}.json", cache_key));

        if cache_file.exists() {
            if let Ok(content) = fs::read_to_string(&cache_file) {
                if let Ok(cache) = serde_json::from_str::<IncrementalCache>(&content) {
                    // Check all invalidation conditions
                    if cache.cargo_lock_hash != workspace_state.cargo_lock_hash {
                        return None;
                    }

                    if cache.env_hash != env_hash {
                        return None;
                    }

                    if cache.features_hash != features_hash {
                        return None;
                    }

                    let all_valid = cache.target_files.iter().all(|(path, expected_size)| {
                        match fs::metadata(path) {
                            Ok(metadata) => metadata.len() == *expected_size,
                            Err(_) => false,
                        }
                    });

                    if cache.source_hash != package.source_hash {
                        return None;
                    }

                    if all_valid && cache.build_success {
                        return Some(cache);
                    }
                }
            }
        }

        None
    }

    /// Saves incremental cache for a package after a successful build.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache file cannot be written.
    #[allow(clippy::too_many_arguments)]
    pub fn save_incremental_cache(
        &self,
        package: &PackageHash,
        workspace_state: &WorkspaceState,
        command_hash: &str,
        env_hash: &str,
        is_release: bool,
        args: &[String],
        build_success: bool,
        duration_ms: u64,
    ) -> Result<()> {
        let features_hash = self.compute_features_hash(args);

        let target_dir = self
            .get_target_dir(args)
            .unwrap_or_else(|| workspace_state.root.join("target"));

        let profile = if is_release { "release" } else { "debug" };
        let deps_dir = target_dir.join(profile).join(".fingerprint");
        let deps_build_dir = target_dir.join(profile).join("deps");

        let mut target_files = Vec::new();
        let mut artifact_paths = Vec::new();

        if deps_dir.exists() {
            for entry in WalkDir::new(&deps_dir).max_depth(2).into_iter().flatten() {
                if entry.file_type().is_file() {
                    let path_str = entry.path().to_string_lossy();
                    if path_str.contains(&package.name) {
                        if let Ok(metadata) = fs::metadata(entry.path()) {
                            target_files.push((entry.path().to_path_buf(), metadata.len()));
                        }
                    }
                }
            }
        }

        if deps_build_dir.exists() {
            for entry in WalkDir::new(&deps_build_dir)
                .max_depth(1)
                .into_iter()
                .flatten()
            {
                if entry.file_type().is_file() {
                    let path_str = entry.path().to_string_lossy();
                    if path_str.contains(&package.name) {
                        if let Ok(metadata) = fs::metadata(entry.path()) {
                            target_files.push((entry.path().to_path_buf(), metadata.len()));
                            artifact_paths.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
        }

        let cache = IncrementalCache {
            package_name: package.name.clone(),
            package_version: package.version.clone(),
            source_hash: package.source_hash.clone(),
            cargo_lock_hash: workspace_state.cargo_lock_hash.clone(),
            command_hash: command_hash.to_string(),
            env_hash: env_hash.to_string(),
            is_release,
            features_hash: features_hash.clone(),
            target_files,
            artifact_paths,
            timestamp: chrono::Local::now().to_rfc3339(),
            build_success,
            duration_ms,
        };

        let cache_key =
            self.get_cache_key(package, command_hash, env_hash, is_release, &features_hash);

        let cache_file = self.incremental_dir.join(format!("{}.json", cache_key));
        fs::write(&cache_file, serde_json::to_string_pretty(&cache)?)?;

        Ok(())
    }

    /// Gets the list of packages that need rebuilding.
    ///
    /// This includes packages that:
    /// - Don't have a valid cache entry
    /// - Have transitive dependencies that need rebuilding
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cargo_save::CacheManager;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let cache = CacheManager::new()?;
    /// let workspace = cache.compute_workspace_state(&[])?;
    ///
    /// let changed = cache.get_changed_packages(&workspace, "cmd_hash", "env_hash", false, &[]);
    /// println!("Packages needing rebuild: {:?}", changed.iter().map(|p| &p.name).collect::<Vec<_>>());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_changed_packages(
        &self,
        workspace_state: &WorkspaceState,
        command_hash: &str,
        env_hash: &str,
        is_release: bool,
        args: &[String],
    ) -> Vec<PackageHash> {
        let mut changed = Vec::new();
        let mut checked: HashSet<String> = HashSet::new();

        // First pass: find packages without valid cache
        for package in &workspace_state.packages {
            if self
                .check_incremental_cache(
                    package,
                    workspace_state,
                    command_hash,
                    env_hash,
                    is_release,
                    args,
                )
                .is_none()
            {
                changed.push(package.clone());
                checked.insert(package.name.clone());
            }
        }

        // Build dependency graph for transitive invalidation
        let graph = self.build_dependency_graph(workspace_state);

        // Iteratively find all packages that depend on changed packages
        let mut iteration = 0;
        loop {
            let mut new_changed = Vec::new();

            for package in &workspace_state.packages {
                if checked.contains(&package.name) {
                    continue;
                }

                if let Some(node) = graph.packages.get(&package.name) {
                    for dep in &node.dependencies {
                        if changed.iter().any(|p| &p.name == dep) {
                            new_changed.push(package.clone());
                            checked.insert(package.name.clone());
                            break;
                        }
                    }
                }
            }

            if new_changed.is_empty() || iteration > workspace_state.packages.len() {
                break;
            }

            changed.extend(new_changed);
            iteration += 1;
        }

        changed
    }

    /// Generates a unique cache ID for a build.
    fn generate_cache_id(&self, cmd: &str, args: &[String]) -> String {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let hash = self.compute_command_hash(cmd, args);
        format!("{}-{}", timestamp, &hash[..8])
    }

    /// Runs a cargo command with caching.
    ///
    /// This is the main entry point for building with cargo-save. It:
    /// 1. Determines which packages need rebuilding
    /// 2. Runs cargo if needed
    /// 3. Captures and caches build output
    /// 4. Updates incremental caches for successful builds
    ///
    /// # Returns
    ///
    /// Returns a tuple of:
    /// - Cache ID
    /// - Exit code (None if process was killed)
    /// - Number of lines in build output
    /// - Build duration in milliseconds
    ///
    /// # Errors
    ///
    /// Returns an error if the cargo command cannot be executed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cargo_save::CacheManager;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let cache = CacheManager::new()?;
    /// let workspace = cache.compute_workspace_state(&[])?;
    ///
    /// let (cache_id, exit_code, lines, duration) = cache
    ///     .run_cargo_with_cache("build", &[], &workspace)?;
    ///
    /// println!("Build {} completed with exit code {:?}", cache_id, exit_code);
    /// println!("Output: {} lines in {}ms", lines, duration);
    /// # Ok(())
    /// # }
    /// ```
    pub fn run_cargo_with_cache(
        &self,
        subcommand: &str,
        args: &[String],
        workspace_state: &WorkspaceState,
    ) -> Result<(String, Option<i32>, usize, u64)> {
        let skip_incremental = matches!(subcommand, "clean" | "update" | "new" | "init");

        let cache_id = self.generate_cache_id(subcommand, args);
        let log_file = self.cache_dir.join(format!("{}.log", cache_id));
        let meta_file = self.metadata_dir.join(format!("{}.json", cache_id));

        let is_release = self.is_release_build(args);
        let command_hash = self.compute_command_hash(subcommand, args);
        let env_hash = self.compute_env_hash();

        let changed_packages = if skip_incremental {
            vec![]
        } else {
            self.get_changed_packages(workspace_state, &command_hash, &env_hash, is_release, args)
        };

        // Skip build if all packages are cached
        if changed_packages.is_empty()
            && matches!(subcommand, "build" | "check" | "clippy" | "test")
        {
            eprintln!(
                "{} All packages cached, skipping {}",
                LOG_PREFIX, subcommand
            );
            return Ok((cache_id, Some(0), 0, 0));
        }

        let total_packages = workspace_state.packages.len();
        let cached_count = total_packages - changed_packages.len();

        if !changed_packages.is_empty() && !skip_incremental {
            eprintln!(
                "{} Build plan: {}/{} packages cached, {} need rebuild",
                LOG_PREFIX,
                cached_count,
                total_packages,
                changed_packages.len()
            );
            eprintln!("{} Packages to rebuild:", LOG_PREFIX);
            for pkg in &changed_packages {
                eprintln!("{}   - {}", LOG_PREFIX, pkg.name);
            }
        }

        // Check for sccache integration and prompt if not configured
        match std::env::var("RUSTC_WRAPPER") {
            Ok(wrapper) if wrapper.contains("sccache") => {
                eprintln!("{} Using sccache for cross-project caching", LOG_PREFIX);
            }
            _ => {
                // Only prompt on actual builds, not on other commands
                if matches!(subcommand, "build" | "test") && !changed_packages.is_empty() {
                    // Check if we should prompt (only once per session)
                    static PROMPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                    if !PROMPTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                        let _ = Self::prompt_sccache_setup();
                    }
                }
            }
        }

        eprintln!(
            "{} Running: cargo {} {}",
            LOG_PREFIX,
            subcommand,
            args.join(" ")
        );
        eprintln!("{} Cache ID: {}", LOG_PREFIX, cache_id);

        let start_time = std::time::Instant::now();

        // Spawn cargo process
        let mut child = Command::new("cargo")
            .arg(subcommand)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn cargo process")?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let mut log = File::create(&log_file)?;
        let mut line_count = 0;
        let mut compiled_count = 0;

        // Set up channels for output capture
        let (tx, rx) = std::sync::mpsc::channel();
        let tx_stderr = tx.clone();

        // Spawn threads to read stdout and stderr
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                let _ = tx.send((line, false));
            }
        });

        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                let _ = tx_stderr.send((line, true));
            }
        });

        // Process output lines
        for (line, is_stderr) in rx {
            if line.trim().starts_with("Compiling ") || line.trim().starts_with("Building ") {
                compiled_count += 1;
                if !changed_packages.is_empty() {
                    let progress_info = format!(" [{}/{}]", compiled_count, changed_packages.len());
                    if is_stderr {
                        eprintln!("{}{}", line, progress_info);
                    } else {
                        println!("{}{}", line, progress_info);
                    }
                } else if is_stderr {
                    eprintln!("{}", line);
                } else {
                    println!("{}", line);
                }
            } else if is_stderr {
                eprintln!("{}", line);
            } else {
                println!("{}", line);
            }
            writeln!(log, "{}", line)?;
            line_count += 1;
        }

        let exit_code = child.wait()?.code();
        let duration = start_time.elapsed().as_millis() as u64;
        let build_success = exit_code == Some(0);

        // Copy log to workspace build-logs/ directory
        if let Ok(workspace_root) = workspace_state.root.canonicalize() {
            let build_logs_dir = workspace_root.join("build-logs");
            if let Ok(()) = fs::create_dir_all(&build_logs_dir) {
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let log_copy = build_logs_dir.join(format!("{}_{}.txt", timestamp, subcommand));
                let _ = fs::copy(&log_file, &log_copy);
            }
        }

        // Save build metadata
        let build_cache = BuildCache {
            cache_id: cache_id.clone(),
            command: format!("cargo {} {}", subcommand, args.join(" ")),
            subcommand: subcommand.to_string(),
            args: args.to_vec(),
            timestamp: chrono::Local::now().to_rfc3339(),
            exit_code,
            workspace_state: workspace_state.clone(),
            is_release,
            target_dir: self.get_target_dir(args),
            lines_count: line_count,
            duration_ms: duration,
            env_hash: env_hash.clone(),
        };

        fs::write(&meta_file, serde_json::to_string_pretty(&build_cache)?)?;

        // Save incremental caches for changed packages
        if !skip_incremental && build_success {
            for package in &changed_packages {
                let pkg_duration = duration / changed_packages.len().max(1) as u64;

                if let Err(e) = self.save_incremental_cache(
                    package,
                    workspace_state,
                    &command_hash,
                    &env_hash,
                    is_release,
                    args,
                    build_success,
                    pkg_duration,
                ) {
                    eprintln!(
                        "{} Failed to save cache for {}: {}",
                        LOG_PREFIX, package.name, e
                    );
                }
            }
        }

        eprintln!(
            "{} Cached {} lines to: {}",
            LOG_PREFIX, line_count, cache_id
        );
        eprintln!("{} Duration: {}ms", LOG_PREFIX, duration);

        Ok((cache_id, exit_code, line_count, duration))
    }

    /// Queries cached build logs.
    ///
    /// # Modes
    ///
    /// - `"head"`: First N lines (default 50)
    /// - `"tail"`: Last N lines (default 50)
    /// - `"grep"`: Lines matching pattern
    /// - `"range"`: Lines in range (e.g., "10-20")
    /// - `"errors"`: Lines containing errors
    /// - `"warnings"`: Lines containing warnings
    /// - `"all"`: All lines
    ///
    /// # Errors
    ///
    /// Returns an error if the log file cannot be read.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cargo_save::CacheManager;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let cache = CacheManager::new()?;
    ///
    /// // Show last 20 lines of most recent build
    /// cache.query_logs("tail", Some("20"), None, None)?;
    ///
    /// // Search for errors
    /// cache.query_logs("errors", None, None, None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn query_logs(
        &self,
        mode: &str,
        param: Option<&str>,
        cache_id: Option<&str>,
        last: Option<usize>,
    ) -> Result<()> {
        let log_file = if let Some(id) = cache_id {
            self.cache_dir.join(format!("{}.log", id))
        } else if let Some(n) = last {
            let entries = self.get_recent_logs(n)?;
            if let Some(entry) = entries.last() {
                self.cache_dir.join(format!("{}.log", entry.cache_id))
            } else {
                anyhow::bail!("No cached logs found");
            }
        } else {
            self.get_latest_log()?
        };

        if !log_file.exists() {
            anyhow::bail!("Log file not found: {}", log_file.display());
        }

        let content = fs::read_to_string(&log_file)?;
        let lines: Vec<&str> = content.lines().collect();

        match mode {
            "head" => {
                let n: usize = param.and_then(|p| p.parse().ok()).unwrap_or(50);
                for line in lines.iter().take(n) {
                    println!("{}", line);
                }
            }
            "tail" => {
                let n: usize = param.and_then(|p| p.parse().ok()).unwrap_or(50);
                let start = lines.len().saturating_sub(n);
                for line in lines.iter().skip(start) {
                    println!("{}", line);
                }
            }
            "grep" => {
                let pattern = param.unwrap_or("");
                let case_insensitive = pattern.to_lowercase() == pattern;

                for line in lines.iter() {
                    let matches = if case_insensitive {
                        line.to_lowercase().contains(pattern)
                    } else {
                        line.contains(pattern)
                    };

                    if matches {
                        println!("{}", line);
                    }
                }
            }
            "range" => {
                let range_str = param.unwrap_or("0-10");
                let parts: Vec<&str> = range_str.split(&['-', ':'][..]).collect();
                if parts.len() == 2 {
                    let start: usize = parts[0].parse().unwrap_or(0);
                    let end: usize = parts[1].parse().unwrap_or(lines.len());
                    for line in lines.iter().skip(start).take(end.saturating_sub(start)) {
                        println!("{}", line);
                    }
                }
            }
            "errors" | "error" => {
                for line in lines.iter() {
                    if line.contains("error[") || line.contains("error:") {
                        println!("{}", line);
                    }
                }
            }
            "warnings" | "warning" => {
                for line in lines.iter() {
                    if line.contains("warning:") {
                        println!("{}", line);
                    }
                }
            }
            "all" => {
                for line in lines {
                    println!("{}", line);
                }
            }
            _ => eprintln!("Unknown mode: {}", mode),
        }

        Ok(())
    }

    /// Gets the path to the most recent log file.
    fn get_latest_log(&self) -> Result<PathBuf> {
        let mut entries: Vec<_> = fs::read_dir(&self.cache_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "log"))
            .collect();

        entries.sort_by_key(|e| {
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        });

        entries
            .last()
            .map(|e| e.path())
            .context("No cached logs found")
    }

    /// Gets the N most recent build caches.
    fn get_recent_logs(&self, n: usize) -> Result<Vec<BuildCache>> {
        let mut entries: Vec<_> = fs::read_dir(&self.metadata_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
            .collect();

        entries.sort_by_key(|e| {
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        });

        let mut caches = Vec::new();
        for entry in entries.into_iter().rev().take(n) {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(cache) = serde_json::from_str::<BuildCache>(&content) {
                    caches.push(cache);
                }
            }
        }

        Ok(caches)
    }

    /// Lists all cached builds.
    ///
    /// # Arguments
    ///
    /// - `verbose`: Show detailed information
    /// - `workspace_only`: Only show caches for current workspace
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory cannot be read.
    pub fn list_caches(&self, verbose: bool, workspace_only: bool) -> Result<()> {
        let current_workspace: Option<PathBuf> = if workspace_only {
            Some(self.get_cargo_metadata()?.workspace_root.into())
        } else {
            None
        };

        let mut entries: Vec<_> = fs::read_dir(&self.metadata_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
            .collect();

        entries.sort_by_key(|e| {
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        });

        println!(
            "{:<25} {:<12} {:<8} {:<30}",
            "Cache ID", "Status", "Lines", "Command"
        );
        println!("{}", "-".repeat(80));

        for entry in entries {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(cache) = serde_json::from_str::<BuildCache>(&content) {
                    if let Some(ref ws) = current_workspace {
                        if cache.workspace_state.root != *ws {
                            continue;
                        }
                    }

                    let status = match cache.exit_code {
                        Some(0) => "✓ success",
                        Some(_) => "✗ failed",
                        None => "? unknown",
                    };

                    let cmd_short = if cache.command.len() > 30 {
                        format!("{}...", &cache.command[..27])
                    } else {
                        cache.command.clone()
                    };

                    println!(
                        "{:<25} {:<12} {:<8} {:<30}",
                        cache.cache_id, status, cache.lines_count, cmd_short
                    );

                    if verbose {
                        println!("  Timestamp: {}", cache.timestamp);
                        println!("  Duration: {}ms", cache.duration_ms);
                        println!("  Release: {}", cache.is_release);
                        println!("  Packages: {}", cache.workspace_state.packages.len());
                        println!();
                    }
                }
            }
        }

        Ok(())
    }

    /// Cleans old cache files.
    ///
    /// # Arguments
    ///
    /// - `days`: Remove caches older than this many days
    /// - `keep`: If specified, keep only this many most recent caches
    /// - `force`: Skip confirmation prompt
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory cannot be read.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cargo_save::CacheManager;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let cache = CacheManager::new()?;
    ///
    /// // Remove caches older than 7 days
    /// cache.clean_old_caches(7, None, false)?;
    ///
    /// // Keep only the 10 most recent caches
    /// cache.clean_old_caches(0, Some(10), true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn clean_old_caches(&self, days: u64, keep: Option<usize>, force: bool) -> Result<()> {
        let cutoff = SystemTime::now() - Duration::from_secs(days * 86400);

        let mut entries: Vec<_> = fs::read_dir(&self.cache_dir)?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let modified = e.metadata().and_then(|m| m.modified()).ok()?;
                Some((e, modified))
            })
            .collect();

        entries.sort_by_key(|(_, modified)| *modified);

        if let Some(keep_count) = keep {
            let to_remove = entries.len().saturating_sub(keep_count);
            if to_remove == 0 {
                println!(
                    "{} No caches to remove (keeping last {})",
                    LOG_PREFIX, keep_count
                );
                return Ok(());
            }

            if !force {
                print!(
                    "{} Remove {} old cache files? [y/N] ",
                    LOG_PREFIX, to_remove
                );
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("{} Aborted", LOG_PREFIX);
                    return Ok(());
                }
            }

            let mut removed = 0;
            for (entry, _) in entries.into_iter().take(to_remove) {
                if fs::remove_file(entry.path()).is_ok() {
                    removed += 1;
                }

                let meta_path = self.metadata_dir.join(
                    entry
                        .path()
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                        + ".json",
                );
                let _ = fs::remove_file(meta_path);
            }

            println!("{} Removed {} old cache files", LOG_PREFIX, removed);
        } else {
            let mut removed = 0;

            for (entry, modified) in entries {
                if modified < cutoff {
                    if fs::remove_file(entry.path()).is_ok() {
                        removed += 1;
                    }

                    let meta_path = self.metadata_dir.join(
                        entry
                            .path()
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                            + ".json",
                    );
                    let _ = fs::remove_file(meta_path);
                }
            }

            println!(
                "{} Removed {} cache files older than {} days",
                LOG_PREFIX, removed, days
            );
        }

        Ok(())
    }

    /// Shows cache statistics.
    ///
    /// Displays information about:
    /// - Total number of cached builds
    /// - Total cache size
    /// - Incremental cache count
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directories cannot be read.
    pub fn show_stats(&self) -> Result<()> {
        let mut total_size = 0u64;
        let mut log_count = 0u64;
        let mut meta_count = 0u64;
        for entry in fs::read_dir(&self.cache_dir)?.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if entry.path().extension().is_some_and(|e| e == "log") {
                    total_size += metadata.len();
                    log_count += 1;
                }
            }
        }

        for entry in fs::read_dir(&self.metadata_dir)?.flatten() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
                meta_count += 1;
            }
        }

        let incremental_count = fs::read_dir(&self.incremental_dir)?.count() as u64;
        for entry in fs::read_dir(&self.incremental_dir)?.flatten() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }

        let size_mb = total_size as f64 / 1024.0 / 1024.0;

        println!("{} Cache Statistics:", LOG_PREFIX);
        println!("  Build logs: {}", log_count);
        println!("  Metadata files: {}", meta_count);
        println!("  Incremental caches: {}", incremental_count);
        println!("  Total size: {:.2} MB", size_mb);
        println!();
        println!("  Cache directories:");
        println!("    - {}", self.cache_dir.display());
        println!("    - {}", self.metadata_dir.display());
        println!("    - {}", self.incremental_dir.display());

        Ok(())
    }

    /// Invalidates caches for specified packages or all packages.
    ///
    /// # Arguments
    ///
    /// - `packages`: Names of packages to invalidate (empty to invalidate all)
    /// - `all`: If true, invalidate all caches
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory cannot be read.
    pub fn invalidate_caches(&self, packages: Vec<String>, all: bool) -> Result<()> {
        if all {
            println!("{} Invalidating all caches...", LOG_PREFIX);
            let mut count = 0;

            for entry in fs::read_dir(&self.incremental_dir)?.flatten() {
                if fs::remove_file(entry.path()).is_ok() {
                    count += 1;
                }
            }

            println!("{} Removed {} incremental cache files", LOG_PREFIX, count);
        } else if !packages.is_empty() {
            println!("{} Invalidating caches for: {:?}", LOG_PREFIX, packages);
            let mut count = 0;

            for entry in fs::read_dir(&self.incremental_dir)?.flatten() {
                let filename = entry.file_name().to_string_lossy().to_string();
                for package in &packages {
                    if filename.starts_with(package) {
                        if fs::remove_file(entry.path()).is_ok() {
                            count += 1;
                        }
                        break;
                    }
                }
            }

            println!("{} Removed {} cache files", LOG_PREFIX, count);
        } else {
            println!(
                "{} Specify --all or package names to invalidate",
                LOG_PREFIX
            );
        }

        Ok(())
    }

    /// Shows the current workspace status.
    ///
    /// Displays information about:
    /// - Workspace root
    /// - Number of packages
    /// - Git features in use
    /// - Package hashes (if requested)
    ///
    /// # Arguments
    ///
    /// - `show_hashes`: If true, show package source hashes
    ///
    /// # Errors
    ///
    /// Returns an error if workspace state cannot be computed.
    pub fn show_status(&self, show_hashes: bool) -> Result<()> {
        let workspace = self.compute_workspace_state(&[])?;

        println!("{} Workspace Status:", LOG_PREFIX);
        println!("  Root: {}", workspace.root.display());
        println!("  Packages: {}", workspace.packages.len());
        println!("  Cargo.lock hash: {}", &workspace.cargo_lock_hash[..16]);
        println!("  Toolchain hash: {}", &workspace.toolchain_hash[..16]);
        println!();

        if let Some(ref git) = workspace.git_features {
            println!("  Git features:");
            println!(
                "    - Submodules: {}",
                if git.has_submodules { "yes" } else { "no" }
            );
            println!(
                "    - Sparse checkout: {}",
                if git.is_sparse { "yes" } else { "no" }
            );
            println!(
                "    - Worktree: {}",
                if git.is_worktree { "yes" } else { "no" }
            );
            println!("    - LFS: {}", if git.has_lfs { "yes" } else { "no" });
            println!(
                "    - Shallow: {}",
                if git.is_shallow { "yes" } else { "no" }
            );
            println!();
        }

        if show_hashes {
            println!("  Package hashes:");
            for pkg in &workspace.packages {
                println!(
                    "    {} {}: {}...",
                    pkg.name,
                    pkg.version,
                    &pkg.source_hash[..16]
                );
            }
        }

        Ok(())
    }

    /// Installs git hooks for automatic cache invalidation.
    ///
    /// Installs post-checkout and post-merge hooks that automatically
    /// invalidate caches when switching branches or merging.
    ///
    /// # Arguments
    ///
    /// - `workspace_root`: Root of the workspace (must be in a git repository)
    ///
    /// # Errors
    ///
    /// Returns an error if not in a git repository or if hooks cannot be written.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cargo_save::CacheManager;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let cache = CacheManager::new()?;
    /// let workspace = cache.compute_workspace_state(&[])?;
    ///
    /// cache.install_git_hooks(&workspace.root)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn install_git_hooks(&self, workspace_root: &Path) -> Result<()> {
        let git_dir = Command::new("git")
            .args(["rev-parse", "--git-common-dir"])
            .current_dir(workspace_root)
            .output()
            .context("Failed to get git directory")?;

        if !git_dir.status.success() {
            anyhow::bail!("Not in a git repository");
        }

        let git_dir_path = PathBuf::from(String::from_utf8_lossy(&git_dir.stdout).trim());
        let hooks_dir = git_dir_path.join("hooks");

        fs::create_dir_all(&hooks_dir)?;

        // Post-checkout hook
        let post_checkout_hook = hooks_dir.join("post-checkout");
        let hook_content = r#"#!/bin/sh
# cargo-save auto-invalidation hook
# This hook invalidates cargo-save cache when switching branches

if command -v cargo-save >/dev/null 2>&1; then
    # Only invalidate if HEAD changed (not just file checkouts)
    if [ "$3" = "1" ]; then
        echo "[cargo-save] Branch changed, invalidating cache..."
        cargo-save invalidate --all 2>/dev/null || true
    fi
fi
"#;

        fs::write(&post_checkout_hook, hook_content)
            .context("Failed to write post-checkout hook")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&post_checkout_hook)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&post_checkout_hook, perms)?;
        }

        // Post-merge hook
        let post_merge_hook = hooks_dir.join("post-merge");
        let merge_hook_content = r#"#!/bin/sh
# cargo-save auto-invalidation hook
# This hook invalidates cargo-save cache after merges

if command -v cargo-save >/dev/null 2>&1; then
    echo "[cargo-save] Merge completed, invalidating cache..."
    cargo-save invalidate --all 2>/dev/null || true
fi
"#;

        fs::write(&post_merge_hook, merge_hook_content)
            .context("Failed to write post-merge hook")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&post_merge_hook)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&post_merge_hook, perms)?;
        }

        eprintln!("{} Installed git hooks:", LOG_PREFIX);
        eprintln!("{}   - post-checkout", LOG_PREFIX);
        eprintln!("{}   - post-merge", LOG_PREFIX);
        eprintln!(
            "{} Hooks will auto-invalidate cache on branch changes",
            LOG_PREFIX
        );

        Ok(())
    }

    /// Checks if sccache is installed
    fn is_sccache_installed() -> bool {
        Command::new("sccache")
            .args(["--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Prompts user to setup sccache if not configured
    fn prompt_sccache_setup() -> Result<()> {
        use std::io::{self, Write};

        let sccache_installed = Self::is_sccache_installed();

        eprintln!("\nTip: sccache provides cross-project compilation caching");
        
        if sccache_installed {
            eprintln!("    sccache is installed but not configured.");
            eprint!("    Enable it now? [Y/n]: ");
            io::stderr().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if input.is_empty() || input == "y" || input == "yes" {
                Self::setup_sccache_env()?;
            } else {
                eprintln!("    To enable: export RUSTC_WRAPPER=sccache");
            }
        } else {
            eprint!("    Install sccache now? [Y/n]: ");
            io::stderr().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if input.is_empty() || input == "y" || input == "yes" {
                eprintln!("    Installing sccache...");
                let status = Command::new("cargo")
                    .args(["install", "sccache"])
                    .status()?;

                if status.success() {
                    eprintln!("    sccache installed successfully");
                    Self::setup_sccache_env()?;
                } else {
                    eprintln!("    Failed to install sccache");
                }
            } else {
                eprintln!("    To install: cargo install sccache");
                eprintln!("    Then enable: export RUSTC_WRAPPER=sccache");
            }
        }

        Ok(())
    }

    /// Sets up sccache environment variable
    fn setup_sccache_env() -> Result<()> {
        use std::io::{self, Write};

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let config_file = if shell.contains("zsh") {
            "~/.zshrc"
        } else if shell.contains("fish") {
            "~/.config/fish/config.fish"
        } else {
            "~/.bashrc"
        };

        eprintln!("\n    Add to {}:", config_file);
        eprintln!("    export RUSTC_WRAPPER=sccache");
        eprint!("\n    Add automatically? [Y/n]: ");
        io::stderr().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input.is_empty() || input == "y" || input == "yes" {
            let home = std::env::var("HOME")?;
            let config_path = config_file.replace("~", &home);
            
            let line = "\n# Enable sccache for cross-project caching\nexport RUSTC_WRAPPER=sccache\n";
            
            if let Ok(mut file) = fs::OpenOptions::new()
                .append(true)
                .open(&config_path)
            {
                file.write_all(line.as_bytes())?;
                eprintln!("    Added to {}", config_file);
                eprintln!("    Restart terminal or run: source {}", config_file);
            } else {
                eprintln!("    Could not write to {}", config_file);
                eprintln!("    Add manually: export RUSTC_WRAPPER=sccache");
            }
        } else {
            eprintln!("    Add manually to {}: export RUSTC_WRAPPER=sccache", config_file);
        }

        Ok(())
    }

    /// Interactive setup for sccache integration
    ///
    /// Guides the user through installing and configuring sccache
    /// for cross-project compilation caching.
    ///
    /// # Errors
    ///
    /// Returns an error if installation or configuration fails.
    pub fn setup_sccache(&self) -> Result<()> {
        println!("sccache Setup\n");

        // Check current status
        if let Ok(wrapper) = std::env::var("RUSTC_WRAPPER") {
            if wrapper.contains("sccache") {
                println!("sccache is already configured");
                println!("RUSTC_WRAPPER={}\n", wrapper);
                
                // Show stats if available
                if let Ok(output) = Command::new("sccache").args(["--show-stats"]).output() {
                    if output.status.success() {
                        println!("Statistics:");
                        println!("{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                return Ok(());
            }
        }

        // Check if installed
        if Self::is_sccache_installed() {
            println!("sccache is installed");
            println!("Configuring environment...\n");
            Self::setup_sccache_env()?;
        } else {
            println!("sccache is not installed");
            Self::prompt_sccache_setup()?;
        }

        println!("\nSetup complete");
        println!("\nNext steps:");
        println!("  1. Restart terminal or run: source ~/.bashrc (or ~/.zshrc)");
        println!("  2. Verify: cargo-save doctor");
        println!("  3. Use normally: cargo-save build");

        Ok(())
    }

    /// Checks environment and integration status.
    ///
    /// Displays diagnostic information about:
    /// - Git availability
    /// - sccache integration
    /// - Cache size and location
    /// - Recommendations for optimization
    ///
    /// # Errors
    ///
    /// Returns an error if cache statistics cannot be computed.
    pub fn doctor(&self) -> Result<()> {
        println!("cargo-save environment check\n");

        // Check git
        let git_available = Command::new("git")
            .args(["--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if git_available {
            let git_version = Command::new("git")
                .args(["--version"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default()
                .trim()
                .to_string();
            println!("Git: {}", git_version);
        } else {
            println!("Git: Not found");
            println!("  cargo-save will fall back to file hashing (slower)");
            println!("  Install git for optimal performance");
        }

        // Check sccache
        let rustc_wrapper = std::env::var("RUSTC_WRAPPER");
        match rustc_wrapper {
            Ok(wrapper) if !wrapper.is_empty() => {
                // Try to get sccache version
                let version_output = Command::new(&wrapper)
                    .args(["--version"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .unwrap_or_default();
                
                if version_output.contains("sccache") {
                    println!("RUSTC_WRAPPER: {} (cross-project caching enabled)", wrapper);
                    
                    // Try to get sccache stats
                    if let Ok(stats) = Command::new(&wrapper).args(["--show-stats"]).output() {
                        if stats.status.success() {
                            let stats_str = String::from_utf8_lossy(&stats.stdout);
                            if let Some(line) = stats_str.lines().find(|l| l.contains("Cache hits")) {
                                println!("  {}", line.trim());
                            }
                        }
                    }
                } else {
                    println!("RUSTC_WRAPPER: {} (custom wrapper)", wrapper);
                }
            }
            _ => {
                println!("RUSTC_WRAPPER: Not set");
                println!("  Run 'cargo-save setup-sccache' for cross-project caching");
            }
        }

        println!();

        // Check cache size
        let mut total_size = 0u64;
        let mut log_count = 0u64;
        let mut meta_count = 0u64;

        for entry in fs::read_dir(&self.cache_dir)?.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if entry.path().extension().is_some_and(|e| e == "log") {
                    total_size += metadata.len();
                    log_count += 1;
                }
            }
        }

        for entry in fs::read_dir(&self.metadata_dir)?.flatten() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
                meta_count += 1;
            }
        }

        let incremental_count = fs::read_dir(&self.incremental_dir)?.count() as u64;
        for entry in fs::read_dir(&self.incremental_dir)?.flatten() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }

        let size_mb = total_size as f64 / 1024.0 / 1024.0;

        println!("Cache Status:");
        println!("  Size: {:.2} MB", size_mb);
        println!("  Build logs: {}", log_count);
        println!("  Metadata files: {}", meta_count);
        println!("  Incremental caches: {}", incremental_count);
        println!("  Location: {}", self.cache_dir.display());

        if size_mb > 1000.0 {
            println!();
            println!("Cache is large (>{:.0} MB). Consider:", size_mb);
            println!("  cargo-save clean --days 30");
        }

        Ok(())
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new().expect("Failed to create CacheManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_features_hash() {
        let cache = CacheManager::new().unwrap();

        let hash1 = cache.compute_features_hash(&["--features".to_string(), "feat1".to_string()]);
        let hash2 = cache.compute_features_hash(&["--features=feat1".to_string()]);
        let hash3 = cache.compute_features_hash(&["--features".to_string(), "feat2".to_string()]);

        // Different features should produce different hashes
        assert_ne!(hash1, hash3);
        // Both syntaxes should produce the same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_is_release_build() {
        let cache = CacheManager::new().unwrap();

        assert!(cache.is_release_build(&["--release".to_string()]));
        assert!(!cache.is_release_build(&["--debug".to_string()]));
        assert!(!cache.is_release_build(&[]));
    }

    #[test]
    fn test_compute_command_hash() {
        let cache = CacheManager::new().unwrap();

        let hash1 = cache.compute_command_hash("build", &[]);
        let hash2 = cache.compute_command_hash("build", &[]);
        let hash3 = cache.compute_command_hash("test", &[]);

        // Same command should produce same hash
        assert_eq!(hash1, hash2);
        // Different commands should produce different hashes
        assert_ne!(hash1, hash3);
    }
}
