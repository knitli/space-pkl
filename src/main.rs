//! Space Pklr - A tool for configuration conversion, schema generation, and Pkl tooling integration
//!
//! This is the main entry point for the Space Pklr tool.

mod cli_app;
mod config_processor;
mod pkl_tooling;
mod error;
mod commands;

use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize comprehensive logging/tracing
    init_tracing()?;

    // Global error handling with rich context
    if let Err(error) = run_cli().await {
        // Use miette for rich error reporting
        eprintln!("{:?}", error);
        std::process::exit(1);
    }

    Ok(())
}

/// Initialize enhanced tracing with structured logging
fn init_tracing() -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("spklr=info"));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_timer(tracing_subscriber::fmt::time::uptime())
                .with_level(true)
                .with_thread_ids(false)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(true)
        )
        .with(filter)
        .init();

    Ok(())
}

/// Run CLI with comprehensive error handling and logging
async fn run_cli() -> Result<()> {
    tracing::info!("Starting Space Pklr");
    tracing::debug!("Recommended Pkl version: {}", crate::pkl_tooling::get_recommended_pkl_version());
    tracing::debug!("Compatible Pkl versions: {:?}", crate::pkl_tooling::get_compatible_pkl_versions());

    let result = cli_app::run().await;

    if let Err(ref error) = result {
        tracing::error!("CLI execution failed: {}", error);
        // Log additional context for debugging
        tracing::debug!("Error chain: {:?}", error);
    } else {
        tracing::info!("CLI execution completed successfully");
    }

    result
}
