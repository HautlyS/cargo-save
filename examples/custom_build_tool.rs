//! Example: Custom build tool integration
//!
//! This example shows how to integrate cargo-save into a custom build tool.
//! It demonstrates programmatic control over the caching behavior.
//!
//! Run with: cargo run --example custom_build_tool

use anyhow::Result;
use cargo_save::{CacheManager, ENV_VARS_THAT_AFFECT_BUILD};

fn main() -> Result<()> {
    let cache = CacheManager::new()?;

    println!("cargo-save example: Custom build tool");
    println!("======================================\n");

    // Configure custom environment
    std::env::set_var("RUSTFLAGS", "-C opt-level=2");

    // Get workspace state with custom args
    let args = vec!["--features".to_string(), "custom-feature".to_string()];
    let workspace = cache.compute_workspace_state(&args)?;

    println!("Custom build configuration:");
    println!("  Features: custom-feature");
    println!("  RUSTFLAGS: -C opt-level=2");

    // Check environment hash
    let env_hash = cache.compute_env_hash();
    println!("  Environment hash: {}...", &env_hash[..16]);

    // Show what affects the build
    println!("\nEnvironment variables tracked:");
    for var in ENV_VARS_THAT_AFFECT_BUILD {
        match std::env::var(var) {
            Ok(val) => println!("  {} = {}", var, val),
            Err(_) => {}
        }
    }

    // Demonstrate incremental build detection
    println!("\nIncremental build detection:");
    let command_hash = cache.compute_command_hash("build", &args);
    let features_hash = cache.compute_features_hash(&args);

    println!("  Command hash: {}...", &command_hash[..16]);
    println!("  Features hash: {}...", &features_hash[..16]);

    // Check cache status for each package
    let is_release = cache.is_release_build(&args);

    for pkg in &workspace.packages {
        match cache.check_incremental_cache(
            pkg,
            &workspace,
            &command_hash,
            &env_hash,
            is_release,
            &args,
        ) {
            Some(_) => println!("  ✓ {} - cached", pkg.name),
            None => println!("  ✗ {} - needs rebuild", pkg.name),
        }
    }

    Ok(())
}
