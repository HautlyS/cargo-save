//! Example: CI/CD integration
//!
//! This example demonstrates how to use cargo-save in CI/CD pipelines.
//! It shows how to generate cache keys and query build logs.
//!
//! Run with: cargo run --example ci_integration

use anyhow::Result;
use cargo_save::CacheManager;

fn main() -> Result<()> {
    let cache = CacheManager::new()?;

    println!("cargo-save example: CI/CD Integration");
    println!("======================================\n");

    // Generate cache key for CI
    let workspace = cache.compute_workspace_state(&[])?;

    println!("Cache keys for CI platforms:");
    println!(
        "  GitHub: cargo-save-github-{}",
        &workspace.toolchain_hash[..16]
    );
    println!(
        "  GitLab: cargo-save-gitlab-{}",
        &workspace.toolchain_hash[..16]
    );
    println!("  Generic: cargo-save-{}", &workspace.toolchain_hash[..16]);

    // Show workspace info that affects caching
    println!("\nWorkspace configuration:");
    println!("  Root: {}", workspace.root.display());
    println!("  Packages: {}", workspace.packages.len());
    println!("  Cargo.lock hash: {}...", &workspace.cargo_lock_hash[..16]);
    println!("  Toolchain hash: {}...", &workspace.toolchain_hash[..16]);

    if let Some(ref git) = workspace.git_features {
        println!("\nGit features:");
        println!(
            "  Submodules: {}",
            if git.has_submodules { "yes" } else { "no" }
        );
        println!(
            "  Sparse checkout: {}",
            if git.is_sparse { "yes" } else { "no" }
        );
        println!("  Worktree: {}", if git.is_worktree { "yes" } else { "no" });
        println!("  LFS: {}", if git.has_lfs { "yes" } else { "no" });
        println!(
            "  Shallow clone: {}",
            if git.is_shallow { "yes" } else { "no" }
        );
    }

    // Example: List recent builds
    println!("\nRecent builds:");
    println!("(Use 'cargo run -- list' to see actual cached builds)");

    // Example: Query build logs
    println!("\nQuerying build logs:");
    println!("(Use 'cargo run -- query tail' to see actual logs)");

    Ok(())
}
