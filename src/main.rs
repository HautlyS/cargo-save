//! Main entry point for cargo-save CLI
//!
//! This binary provides the command-line interface for cargo-save.
//! It can be invoked in two ways:
//!
//! 1. As a cargo subcommand: `cargo save <subcommand>`
//! 2. Directly: `cargo-save <subcommand>`
//!
//! # Examples
//!
//! Build with caching:
//! ```bash
//! cargo save build
//! ```
//!
//! Run tests with caching:
//! ```bash
//! cargo save test
//! ```
//!
//! Show cache statistics:
//! ```bash
//! cargo save stats
//! ```
//!
//! Query build logs:
//! ```bash
//! cargo save query tail
//! ```

use cargo_save::{CacheManager, Cli};
use clap::Parser;

/// Main entry point for the cargo-save CLI.
///
/// Parses command-line arguments and dispatches to the appropriate
/// handler based on the subcommand.
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cache = CacheManager::new()?;

    // Dispatch to the appropriate handler based on the CLI subcommand
    match cli {
        // Handle both "cargo save <cmd>" and "cargo-save <cmd>" syntax
        Cli::Save { subcommand, args } | Cli::Direct { subcommand, args } => {
            let workspace = cache.compute_workspace_state(&args)?;
            let (_, exit_code, _, _) =
                cache.run_cargo_with_cache(&subcommand, &args, &workspace)?;
            std::process::exit(exit_code.unwrap_or(1));
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
            cache.invalidate_caches(packages, all)?;
        }

        Cli::Status { hashes } => {
            cache.show_status(hashes)?;
        }

        Cli::CacheKey { platform } => {
            let workspace = cache.compute_workspace_state(&[])?;
            let key = format!(
                "cargo-save-{}-{}",
                platform,
                &workspace.toolchain_hash[..16]
            );
            println!("{}", key);
        }

        Cli::Warm { release } => {
            let mut args = vec![];
            if release {
                args.push("--release".to_string());
            }
            let workspace = cache.compute_workspace_state(&args)?;
            let command_hash = cache.compute_command_hash("warm", &args);
            let env_hash = cache.compute_env_hash();
            let is_release = cache.is_release_build(&args);

            let changed =
                cache.get_changed_packages(&workspace, &command_hash, &env_hash, is_release, &args);

            if changed.is_empty() {
                println!("[cargo-save] All packages already cached");
            } else {
                println!(
                    "[cargo-save] Pre-computing hashes for {} packages",
                    changed.len()
                );
                for pkg in &changed {
                    println!("  - {}", pkg.name);
                }
            }
        }

        Cli::InstallHooks => {
            let workspace = cache.compute_workspace_state(&[])?;
            cache.install_git_hooks(&workspace.root)?;
        }

        Cli::Doctor => {
            cache.doctor()?;
        }

        Cli::SetupSccache => {
            cache.setup_sccache()?;
        }
    }

    Ok(())
}
