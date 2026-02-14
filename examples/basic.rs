//! Example: Basic caching workflow
//!
//! This example demonstrates the basic usage of cargo-save as a library.
//!
//! Run with: cargo run --example basic

use anyhow::Result;
use cargo_save::CacheManager;

fn main() -> Result<()> {
    // Initialize the cache manager
    let cache = CacheManager::new()?;

    println!("cargo-save example: Basic caching");
    println!("=====================================\n");

    // Compute workspace state
    println!("Computing workspace state...");
    let workspace = cache.compute_workspace_state(&[])?;

    println!("Found {} packages in workspace:", workspace.packages.len());
    for pkg in &workspace.packages {
        println!("  - {} v{}", pkg.name, pkg.version);
    }

    // Build dependency graph
    println!("\nBuilding dependency graph...");
    let graph = cache.build_dependency_graph(&workspace);

    for (name, node) in &graph.packages {
        if !node.reverse_dependencies.is_empty() {
            println!(
                "  {} is depended on by: {:?}",
                name, node.reverse_dependencies
            );
        }
    }

    // Check which packages would need rebuilding
    let command_hash = cache.compute_command_hash("build", &[]);
    let env_hash = cache.compute_env_hash();
    let is_release = false;

    let changed = cache.get_changed_packages(&workspace, &command_hash, &env_hash, is_release, &[]);

    println!("\nPackages needing rebuild: {}", changed.len());
    for pkg in &changed {
        println!(
            "  - {} (source hash: {}...)",
            pkg.name,
            &pkg.source_hash[..16]
        );
    }

    // Show cache statistics
    println!("\nCache statistics:");
    cache.show_stats()?;

    Ok(())
}
