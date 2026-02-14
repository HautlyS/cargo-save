//! Example: Using cargo-save with sccache
//!
//! Demonstrates checking sccache integration status.
//!
//! # Setup
//!
//! ```bash
//! cargo install sccache
//! export RUSTC_WRAPPER=sccache
//! cargo run --example sccache_integration
//! ```

use cargo_save::CacheManager;
use std::env;

fn main() -> anyhow::Result<()> {
    println!("cargo-save + sccache Integration\n");

    // Check if sccache is configured
    match env::var("RUSTC_WRAPPER") {
        Ok(wrapper) if wrapper.contains("sccache") => {
            println!("sccache is configured: RUSTC_WRAPPER={}\n", wrapper);
        }
        Ok(wrapper) => {
            println!("Custom RUSTC_WRAPPER: {}\n", wrapper);
        }
        Err(_) => {
            println!("sccache is not configured");
            println!("To enable: export RUSTC_WRAPPER=sccache\n");
        }
    }

    // Run doctor command
    let cache = CacheManager::new()?;
    cache.doctor()?;

    Ok(())
}
