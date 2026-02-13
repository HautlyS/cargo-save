use anyhow::{Context, Result};
use blake3::Hasher as Blake3Hasher;
use cargo_metadata::{Metadata, MetadataCommand, Package};
use clap::Parser;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
// Arc and Mutex available if needed for future concurrency
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

// ============ Configuration ============
const CACHE_VERSION: &str = "v3";
const LOG_PREFIX: &str = "[cargo-save]";

// Environment variables that affect builds
const ENV_VARS_THAT_AFFECT_BUILD: &[&str] = &[
    "RUSTFLAGS",
    "RUSTDOCFLAGS",
    "CARGO_TARGET_DIR",
    "CARGO_HOME",
    "CARGO_NET_OFFLINE",
    "CARGO_BUILD_JOBS",
    "CARGO_BUILD_TARGET",
    "CARGO_BUILD_RUSTFLAGS",
    "CARGO_PROFILE_DEV_DEBUG",
    "CARGO_PROFILE_RELEASE_DEBUG",
    "CARGO_PROFILE_RELEASE_OPT_LEVEL",
    "CARGO_PROFILE_RELEASE_LTO",
    "CC",
    "CXX",
    "AR",
    "LINKER",
];

// ============ CLI ============
#[derive(Parser)]
#[command(name = "cargo-save")]
#[command(
    about = "Smart caching cargo wrapper with git-based incremental builds",
    version
)]
enum Cli {
    /// Run cargo command with smart caching
    #[command(name = "save")]
    Save {
        /// Cargo subcommand (build, test, check, clippy, fmt, etc.)
        subcommand: String,

        /// Additional arguments for cargo
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Query cached logs
    #[command(name = "query")]
    Query {
        /// Query type: head, tail, grep, range, all
        mode: String,

        /// Parameter (e.g., line count, pattern)
        param: Option<String>,

        /// Optional: specific cache ID
        #[arg(short, long)]
        id: Option<String>,

        /// Show from last N runs
        #[arg(short, long)]
        last: Option<usize>,
    },

    /// List cached builds
    #[command(name = "list")]
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,

        /// Show only entries for current workspace
        #[arg(short, long)]
        workspace: bool,
    },

    /// Clear old caches
    #[command(name = "clean")]
    Clean {
        /// Remove caches older than N days
        #[arg(short, long, default_value = "7")]
        days: u64,

        /// Remove all caches except the last N
        #[arg(short, long)]
        keep: Option<usize>,

        /// Force remove without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Show cache statistics
    #[command(name = "stats")]
    Stats,

    /// Invalidate cache for specific packages or all
    #[command(name = "invalidate")]
    Invalidate {
        /// Package names to invalidate (empty = all)
        packages: Vec<String>,

        /// Invalidate all caches
        #[arg(short, long)]
        all: bool,
    },

    /// Show workspace structure and cache status
    #[command(name = "status")]
    Status {
        /// Show git hashes for each package
        #[arg(short, long)]
        hashes: bool,
    },

    /// Generate CI cache key (for GitHub Actions, etc.)
    #[command(name = "cache-key")]
    CacheKey {
        /// CI platform: github, gitlab, generic
        #[arg(short, long, default_value = "github")]
        platform: String,
    },

    /// Pre-compute hashes for all packages (warm cache)
    #[command(name = "warm")]
    Warm {
        /// Also warm release builds
        #[arg(long)]
        release: bool,
    },
}

// ============ Data Structures ============
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageHash {
    name: String,
    version: String,
    path: PathBuf,
    source_hash: String,
    dependencies: Vec<String>,
    features_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceState {
    root: PathBuf,
    packages: Vec<PackageHash>,
    cargo_lock_hash: String,
    toolchain_hash: String,
    timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BuildCache {
    cache_id: String,
    command: String,
    subcommand: String,
    args: Vec<String>,
    timestamp: String,
    exit_code: Option<i32>,
    workspace_state: WorkspaceState,
    is_release: bool,
    target_dir: Option<PathBuf>,
    lines_count: usize,
    duration_ms: u64,
    env_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IncrementalCache {
    package_name: String,
    package_version: String,
    source_hash: String,
    cargo_lock_hash: String,
    command_hash: String,
    env_hash: String,
    is_release: bool,
    features_hash: String,
    target_files: Vec<(PathBuf, u64)>, // (path, file_size)
    artifact_paths: Vec<PathBuf>,      // Actual build artifacts
    timestamp: String,
    build_success: bool,
    duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DependencyGraph {
    packages: HashMap<String, PackageNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageNode {
    name: String,
    dependencies: Vec<String>,
    reverse_dependencies: Vec<String>,
}

// ============ Cache Manager ============
struct CacheManager {
    cache_dir: PathBuf,
    incremental_dir: PathBuf,
    metadata_dir: PathBuf,
}

impl CacheManager {
    fn new() -> Result<Self> {
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

    fn get_cargo_metadata(&self) -> Result<Metadata> {
        let metadata = MetadataCommand::new()
            .exec()
            .context("Failed to get cargo metadata")?;
        Ok(metadata)
    }

    fn compute_toolchain_hash(&self) -> Result<String> {
        let mut hasher = Blake3Hasher::new();

        // Hash rustc version
        if let Ok(output) = Command::new("rustc").args(["--version"]).output() {
            if output.status.success() {
                hasher.update(&output.stdout);
            }
        }

        // Hash cargo version
        if let Ok(output) = Command::new("cargo").args(["--version"]).output() {
            if output.status.success() {
                hasher.update(&output.stdout);
            }
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    fn compute_cargo_lock_hash(&self, workspace_root: &Path) -> Result<String> {
        let lock_file = workspace_root.join("Cargo.lock");

        if lock_file.exists() {
            let content = fs::read(&lock_file)?;
            let mut hasher = Blake3Hasher::new();
            hasher.update(&content);
            Ok(hasher.finalize().to_hex().to_string())
        } else {
            // No lock file, use a default hash
            Ok("no-lock-file".to_string())
        }
    }

    fn compute_env_hash(&self) -> String {
        let mut hasher = Blake3Hasher::new();

        for var in ENV_VARS_THAT_AFFECT_BUILD {
            if let Ok(value) = std::env::var(var) {
                hasher.update(var.as_bytes());
                hasher.update(value.as_bytes());
            }
        }

        hasher.finalize().to_hex().to_string()
    }

    fn compute_features_hash(&self, args: &[String]) -> String {
        let mut hasher = Blake3Hasher::new();

        // Extract features from args
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

    fn compute_source_hash(&self, path: &Path, _args: &[String]) -> Result<String> {
        let mut hasher = Blake3Hasher::new();

        // First, try to use git for tracked files (most accurate)
        if let Ok(output) = Command::new("git")
            .args(["ls-tree", "-r", "HEAD", "--", path.to_str().unwrap_or(".")])
            .output()
        {
            if output.status.success() && !output.stdout.is_empty() {
                hasher.update(&output.stdout);

                // Also include git status for uncommitted changes
                if let Ok(status_output) = Command::new("git")
                    .args(["status", "--porcelain", "--", path.to_str().unwrap_or(".")])
                    .output()
                {
                    if status_output.status.success() && !status_output.stdout.is_empty() {
                        hasher.update(&status_output.stdout);

                        // Hash the actual content of modified files
                        let status_str = String::from_utf8_lossy(&status_output.stdout);
                        for line in status_str.lines() {
                            if line.len() > 3 {
                                let file_path = &line[3..];
                                let full_path = path.join(file_path);
                                if full_path.exists() && full_path.is_file() {
                                    if let Ok(content) = fs::read(&full_path) {
                                        hasher.update(full_path.to_string_lossy().as_bytes());
                                        hasher.update(&content);
                                    }
                                }
                            }
                        }
                    }
                }

                return Ok(hasher.finalize().to_hex().to_string());
            }
        }

        // Fallback: hash all source files manually
        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path_str = entry.path().to_string_lossy();

                // Skip common non-source directories
                if path_str.contains("/target/")
                    || path_str.contains("/.git/")
                    || path_str.contains("/node_modules/")
                {
                    continue;
                }

                // Only hash source files
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

    fn compute_package_hash(
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

        // Get workspace member dependencies
        let mut dependencies = Vec::new();

        for dep in &package.dependencies {
            // Only track workspace members as dependencies for caching purposes
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

    fn compute_workspace_state(&self, args: &[String]) -> Result<WorkspaceState> {
        let metadata = self.get_cargo_metadata()?;
        let root: PathBuf = metadata.workspace_root.clone().into();

        // Compute all package hashes in parallel
        let packages: Vec<PackageHash> = metadata
            .workspace_packages()
            .par_iter()
            .filter_map(|package| self.compute_package_hash(package, &metadata, args).ok())
            .collect();

        let cargo_lock_hash = self.compute_cargo_lock_hash(&root)?;
        let toolchain_hash = self.compute_toolchain_hash()?;

        Ok(WorkspaceState {
            root,
            packages,
            cargo_lock_hash,
            toolchain_hash,
            timestamp: chrono::Local::now().to_rfc3339(),
        })
    }

    fn build_dependency_graph(&self, workspace_state: &WorkspaceState) -> DependencyGraph {
        let mut packages = HashMap::new();

        for package in &workspace_state.packages {
            // Find reverse dependencies
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

    fn compute_command_hash(&self, subcommand: &str, args: &[String]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(subcommand.as_bytes());
        hasher.update(args.join(" ").as_bytes());

        if let Ok(cwd) = std::env::current_dir() {
            hasher.update(cwd.to_string_lossy().as_bytes());
        }

        format!("{:x}", &hasher.finalize())[..16].to_string()
    }

    fn is_release_build(&self, args: &[String]) -> bool {
        args.iter()
            .any(|arg| arg == "--release" || arg.starts_with("--release"))
    }

    fn get_target_dir(&self, args: &[String]) -> Option<PathBuf> {
        // Check args first
        for (i, arg) in args.iter().enumerate() {
            if arg == "--target-dir" {
                return args.get(i + 1).map(PathBuf::from);
            }
            if arg.starts_with("--target-dir=") {
                return arg.split('=').nth(1).map(PathBuf::from);
            }
        }

        // Check environment variable
        if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
            return Some(PathBuf::from(target_dir));
        }

        None
    }

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
            &package.source_hash[..16],
            command_hash,
            env_hash,
            if is_release { "release" } else { "debug" },
            features_hash
        )
    }

    fn check_incremental_cache(
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
                    // Validate cache by checking:
                    // 1. Cargo.lock hash matches
                    if cache.cargo_lock_hash != workspace_state.cargo_lock_hash {
                        return None;
                    }

                    // 2. Environment hash matches
                    if cache.env_hash != env_hash {
                        return None;
                    }

                    // 3. Features hash matches
                    if cache.features_hash != features_hash {
                        return None;
                    }

                    // 4. All target files still exist and have same size
                    let all_valid = cache.target_files.iter().all(|(path, expected_size)| {
                        match fs::metadata(path) {
                            Ok(metadata) => metadata.len() == *expected_size,
                            Err(_) => false,
                        }
                    });

                    // 5. Check if source hash is still the same
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

    fn save_incremental_cache(
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

        // Find fingerprint files and artifacts for this package
        if deps_dir.exists() {
            for entry in WalkDir::new(&deps_dir).max_depth(2) {
                if let Ok(entry) = entry {
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
        }

        // Find actual build artifacts (.rlib, .rmeta)
        if deps_build_dir.exists() {
            for entry in WalkDir::new(&deps_build_dir).max_depth(1) {
                if let Ok(entry) = entry {
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

    fn get_changed_packages(
        &self,
        workspace_state: &WorkspaceState,
        command_hash: &str,
        env_hash: &str,
        is_release: bool,
        args: &[String],
    ) -> Vec<PackageHash> {
        let mut changed = Vec::new();
        let mut checked: HashSet<String> = HashSet::new();

        // First pass: check direct changes
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

        // Build dependency graph for transitive calculation
        let graph = self.build_dependency_graph(workspace_state);

        // Second pass: check transitive dependencies using the graph
        let mut iteration = 0;
        loop {
            let mut new_changed = Vec::new();

            for package in &workspace_state.packages {
                if checked.contains(&package.name) {
                    continue;
                }

                // Check if any dependency changed using the graph
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

    fn generate_cache_id(&self, cmd: &str, args: &[String]) -> String {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let hash = self.compute_command_hash(cmd, args);
        format!("{}-{}", timestamp, &hash[..8])
    }

    fn run_cargo_with_cache(
        &self,
        subcommand: &str,
        args: &[String],
        workspace_state: &WorkspaceState,
    ) -> Result<(String, Option<i32>, usize, u64)> {
        // Handle commands that don't benefit from incremental caching
        let skip_incremental = matches!(subcommand, "clean" | "update" | "new" | "init");

        let cache_id = self.generate_cache_id(subcommand, args);
        let log_file = self.cache_dir.join(format!("{}.log", cache_id));
        let meta_file = self.metadata_dir.join(format!("{}.json", cache_id));

        let is_release = self.is_release_build(args);
        let command_hash = self.compute_command_hash(subcommand, args);
        let env_hash = self.compute_env_hash();

        // Check which packages need rebuild (only for build-related commands)
        let changed_packages = if skip_incremental {
            vec![]
        } else {
            self.get_changed_packages(workspace_state, &command_hash, &env_hash, is_release, args)
        };

        if changed_packages.is_empty()
            && matches!(subcommand, "build" | "check" | "clippy" | "test")
        {
            eprintln!(
                "{} All packages cached, skipping {}",
                LOG_PREFIX, subcommand
            );
            return Ok((cache_id, Some(0), 0, 0));
        }

        if !changed_packages.is_empty() && !skip_incremental {
            eprintln!(
                "{} {} packages need rebuild:",
                LOG_PREFIX,
                changed_packages.len()
            );
            for pkg in &changed_packages {
                eprintln!("{}   - {}", LOG_PREFIX, pkg.name);
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

        // Read stdout
        let stdout_reader = BufReader::new(stdout);
        for line in stdout_reader.lines() {
            let line = line?;
            println!("{}", line);
            writeln!(log, "{}", line)?;
            line_count += 1;
        }

        // Read stderr
        let stderr_reader = BufReader::new(stderr);
        for line in stderr_reader.lines() {
            let line = line?;
            eprintln!("{}", line);
            writeln!(log, "{}", line)?;
            line_count += 1;
        }

        let exit_code = child.wait()?.code();
        let duration = start_time.elapsed().as_millis() as u64;
        let build_success = exit_code == Some(0);

        // Save build cache metadata
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

        // Save incremental cache for each changed package
        if !skip_incremental && build_success {
            for package in &changed_packages {
                // Estimate duration per package (for simplicity, divide equally)
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

    fn query_logs(
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

    fn get_latest_log(&self) -> Result<PathBuf> {
        let mut entries: Vec<_> = fs::read_dir(&self.cache_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "log"))
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

    fn get_recent_logs(&self, n: usize) -> Result<Vec<BuildCache>> {
        let mut entries: Vec<_> = fs::read_dir(&self.metadata_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
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

    fn list_caches(&self, verbose: bool, workspace_only: bool) -> Result<()> {
        let current_workspace: Option<PathBuf> = if workspace_only {
            Some(self.get_cargo_metadata()?.workspace_root.into())
        } else {
            None
        };

        let mut entries: Vec<_> = fs::read_dir(&self.metadata_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
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
                    // Filter by workspace if requested
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

    fn clean_old_caches(&self, days: u64, keep: Option<usize>, force: bool) -> Result<()> {
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
            // Keep the last N entries, remove the rest
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

                // Also remove corresponding metadata file
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
            // Remove entries older than N days
            let mut removed = 0;

            for (entry, modified) in entries {
                if modified < cutoff {
                    if fs::remove_file(entry.path()).is_ok() {
                        removed += 1;
                    }

                    // Also remove corresponding metadata file
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

    fn show_stats(&self) -> Result<()> {
        let mut total_size = 0u64;
        let mut log_count = 0u64;
        let mut meta_count = 0u64;
        let mut incremental_count = 0u64;

        // Count logs
        for entry in fs::read_dir(&self.cache_dir)? {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if entry.path().extension().map_or(false, |e| e == "log") {
                        total_size += metadata.len();
                        log_count += 1;
                    }
                }
            }
        }

        // Count metadata
        for entry in fs::read_dir(&self.metadata_dir)? {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                    meta_count += 1;
                }
            }
        }

        // Count incremental cache
        for entry in fs::read_dir(&self.incremental_dir)? {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                    incremental_count += 1;
                }
            }
        }

        println!("{} Cache Statistics:", LOG_PREFIX);
        println!("  Location: {}", self.cache_dir.display());
        println!("  Version: {}", CACHE_VERSION);
        println!("  Total size: {} MB", total_size / 1_048_576);
        println!("  Log files: {}", log_count);
        println!("  Metadata files: {}", meta_count);
        println!("  Incremental caches: {}", incremental_count);

        Ok(())
    }

    fn invalidate_cache(&self, packages: Vec<String>, all: bool) -> Result<()> {
        if all {
            let mut removed = 0;

            for entry in fs::read_dir(&self.incremental_dir)? {
                if let Ok(entry) = entry {
                    if fs::remove_file(entry.path()).is_ok() {
                        removed += 1;
                    }
                }
            }

            println!(
                "{} Invalidated all {} incremental caches",
                LOG_PREFIX, removed
            );
        } else if packages.is_empty() {
            println!(
                "{} No packages specified. Use --all to invalidate all caches.",
                LOG_PREFIX
            );
        } else {
            let mut removed = 0;

            for entry in fs::read_dir(&self.incremental_dir)? {
                if let Ok(entry) = entry {
                    let filename = entry.file_name().to_string_lossy().to_string();

                    for package in &packages {
                        if filename.starts_with(package) {
                            if fs::remove_file(entry.path()).is_ok() {
                                removed += 1;
                                println!("{} Invalidated cache for {}", LOG_PREFIX, package);
                            }
                            break;
                        }
                    }
                }
            }

            println!("{} Invalidated {} cache entries", LOG_PREFIX, removed);
        }

        Ok(())
    }

    fn show_status(&self, show_hashes: bool) -> Result<()> {
        let workspace_state = self.compute_workspace_state(&[])?;

        println!(
            "{} Workspace: {}",
            LOG_PREFIX,
            workspace_state.root.display()
        );
        println!(
            "{} Packages: {}",
            LOG_PREFIX,
            workspace_state.packages.len()
        );
        println!(
            "{} Cargo.lock hash: {}...",
            LOG_PREFIX,
            &workspace_state.cargo_lock_hash[..16]
        );
        println!(
            "{} Toolchain hash: {}...",
            LOG_PREFIX,
            &workspace_state.toolchain_hash[..16]
        );
        println!();

        let command_hash = self.compute_command_hash("build", &[]);
        let env_hash = self.compute_env_hash();

        for package in &workspace_state.packages {
            let is_cached = self
                .check_incremental_cache(
                    package,
                    &workspace_state,
                    &command_hash,
                    &env_hash,
                    false,
                    &[],
                )
                .is_some();
            let release_cached = self
                .check_incremental_cache(
                    package,
                    &workspace_state,
                    &command_hash,
                    &env_hash,
                    true,
                    &[],
                )
                .is_some();

            let status = if is_cached && release_cached {
                "✓ cached (debug+release)"
            } else if is_cached {
                "✓ cached (debug)"
            } else if release_cached {
                "✓ cached (release)"
            } else {
                "✗ not cached"
            };

            if show_hashes {
                println!(
                    "  {:<30} {} (hash: {}...)",
                    package.name,
                    status,
                    &package.source_hash[..16]
                );
            } else {
                println!("  {:<30} {}", package.name, status);
            }
        }

        Ok(())
    }

    fn generate_cache_key(&self, platform: &str) -> Result<()> {
        let workspace_state = self.compute_workspace_state(&[])?;
        let env_hash = self.compute_env_hash();

        match platform {
            "github" => {
                // GitHub Actions cache key format
                println!(
                    "::set-output name=cache-key::cargo-save-{}-{}",
                    &workspace_state.cargo_lock_hash[..16],
                    &env_hash[..16]
                );
                println!(
                    "cargo-save-{}-{}",
                    &workspace_state.cargo_lock_hash[..16],
                    &env_hash[..16]
                );
            }
            "gitlab" => {
                // GitLab CI cache key
                println!(
                    "cargo-save-{}-{}",
                    &workspace_state.cargo_lock_hash[..16],
                    &env_hash[..16]
                );
            }
            _ => {
                // Generic format
                println!(
                    "cargo-save-{}-{}",
                    &workspace_state.cargo_lock_hash[..16],
                    &env_hash[..16]
                );
            }
        }

        Ok(())
    }

    fn warm_cache(&self, release: bool) -> Result<()> {
        eprintln!("{} Warming cache...", LOG_PREFIX);

        let args = if release {
            vec!["--release".to_string()]
        } else {
            vec![]
        };

        let workspace_state = self.compute_workspace_state(&args)?;

        eprintln!(
            "{} Computing hashes for {} packages...",
            LOG_PREFIX,
            workspace_state.packages.len()
        );

        // This pre-computes all hashes and stores them
        // Future builds will be faster because hashes are already computed

        let command_hash = self.compute_command_hash("build", &args);
        let env_hash = self.compute_env_hash();
        let is_release = release;

        let mut cached = 0;
        let mut needs_build = 0;

        for package in &workspace_state.packages {
            if self
                .check_incremental_cache(
                    package,
                    &workspace_state,
                    &command_hash,
                    &env_hash,
                    is_release,
                    &args,
                )
                .is_some()
            {
                cached += 1;
            } else {
                needs_build += 1;
            }
        }

        eprintln!("{} Cache status:", LOG_PREFIX);
        eprintln!("{}   Cached: {}", LOG_PREFIX, cached);
        eprintln!("{}   Needs build: {}", LOG_PREFIX, needs_build);

        if needs_build > 0 {
            eprintln!(
                "{} Run 'cargo-save save build{}' to build and cache",
                LOG_PREFIX,
                if release { " --release" } else { "" }
            );
        }

        Ok(())
    }
}

// ============ Main ============
fn main() -> Result<()> {
    let cli = Cli::parse();
    let cache = CacheManager::new()?;

    match cli {
        Cli::Save { subcommand, args } => {
            let workspace_state = cache.compute_workspace_state(&args)?;
            let (_cache_id, exit_code, _lines, _duration) =
                cache.run_cargo_with_cache(&subcommand, &args, &workspace_state)?;

            // Exit with the same code as cargo
            if let Some(code) = exit_code {
                if code != 0 {
                    std::process::exit(code);
                }
            }
        }
        Cli::Query {
            mode,
            param,
            id,
            last,
        } => {
            cache.query_logs(&mode, param.as_deref(), id.as_deref(), last)?;
        }
        Cli::List { verbose, workspace } => {
            cache.list_caches(verbose, workspace)?;
        }
        Cli::Clean { days, keep, force } => {
            cache.clean_old_caches(days, keep, force)?;
        }
        Cli::Stats => {
            cache.show_stats()?;
        }
        Cli::Invalidate { packages, all } => {
            cache.invalidate_cache(packages, all)?;
        }
        Cli::Status { hashes } => {
            cache.show_status(hashes)?;
        }
        Cli::CacheKey { platform } => {
            cache.generate_cache_key(&platform)?;
        }
        Cli::Warm { release } => {
            cache.warm_cache(release)?;
        }
    }

    Ok(())
}
