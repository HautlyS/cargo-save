//! Example: Git integration
//!
//! This example demonstrates git-related features of cargo-save.
//! It shows how to work with different git configurations.
//!
//! Run with: cargo run --example git_integration

use anyhow::Result;
use cargo_save::CacheManager;

fn main() -> Result<()> {
    let cache = CacheManager::new()?;

    println!("cargo-save example: Git Integration");
    println!("====================================\n");

    // Get git repository info
    let workspace = cache.compute_workspace_state(&[])?;

    if let Some(ref git) = workspace.git_features {
        println!("Git repository features:");

        if git.has_submodules {
            println!("  ✓ Submodules detected");
            println!("    Submodule changes will be tracked");
        }

        if git.is_sparse {
            println!("  ✓ Sparse checkout enabled");
            println!("    Only tracked files will be hashed");
        }

        if git.is_worktree {
            println!("  ✓ Git worktree");
            println!("    Worktree root will be used for hashing");
        }

        if git.has_lfs {
            println!("  ✓ Git LFS detected");
            println!("    LFS pointer files will be hashed instead of content");
        }

        if git.is_shallow {
            println!("  ✓ Shallow clone");
            println!("    Shallow file will be included in hash");
        }

        if !git.has_submodules
            && !git.is_sparse
            && !git.is_worktree
            && !git.has_lfs
            && !git.is_shallow
        {
            println!("  Standard git repository (no special features)");
        }
    } else {
        println!("Warning: Not in a git repository");
        println!("Falling back to file-based hashing (less accurate)");
    }

    // Show how source hashing works
    println!("\nSource hashing strategy:");
    if cache.get_git_repo_info(&workspace.root).is_some() {
        println!("  1. Git tree hash from 'git ls-tree'");
        println!("  2. Uncommitted changes from 'git status'");
        println!("  3. Modified files content hash");

        if workspace
            .git_features
            .as_ref()
            .map(|g| g.has_submodules)
            .unwrap_or(false)
        {
            println!("  4. Submodule status");
        }
        if workspace
            .git_features
            .as_ref()
            .map(|g| g.is_sparse)
            .unwrap_or(false)
        {
            println!("  5. Sparse-checkout patterns");
        }
        if workspace
            .git_features
            .as_ref()
            .map(|g| g.is_shallow)
            .unwrap_or(false)
        {
            println!("  6. Shallow clone info");
        }
    } else {
        println!("  File content hashing (fallback)");
        println!("  Only .rs and .toml files are considered");
    }

    // Install git hooks example
    println!("\nGit hooks:");
    println!("Run 'cargo save install-hooks' to install automatic cache invalidation");
    println!("  - post-checkout: Invalidates cache on branch switch");
    println!("  - post-merge: Invalidates cache after merge");

    Ok(())
}
