//! The reusable implementation behind the `jgrep` binary.

pub mod cli;
pub mod engine;
pub mod model;

use std::io;

use anyhow::Result;

use crate::{cli::Cli, engine::run};

/// Run the parsed command and return its grep-compatible process status.
///
/// # Errors
///
/// Returns an error for invalid options, model acquisition failures, or
/// unrecoverable search setup errors.
pub fn run_cli(cli: Cli) -> Result<u8> {
    let config = cli.into_config()?;

    // A standalone download always warms the cache. For a search, `-m 0`
    // deliberately promises no model work, even when a cache-warming flag is
    // also present.
    if config.download_model && (config.patterns.is_empty() || config.max_count != Some(0)) {
        let path = model::download_model(&config.semantic)?;
        eprintln!("jgrep: model is available at {}", path.display());
        if config.patterns.is_empty() {
            return Ok(0);
        }
    }

    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let summary = run(&config, &mut stdout, &mut stderr, model::create_scorer)?;
    Ok(summary.exit_code())
}
