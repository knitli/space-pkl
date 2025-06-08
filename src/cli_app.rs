//! CLI Application module for Space Pklr
//!
//! This module defines the clap application structure and command dispatching

use clap::{Parser, Subcommand};
use miette::Result;

/// Space Pklr - A tool for configuration conversion, schema generation, and Pkl tooling integration
#[derive(Parser)]
#[command(name = "spklr")]
#[command(
    about = "A Rust CLI tool for Moon configuration conversion, schema generation, and Pkl tooling integration"
)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert Moon configuration files between formats
    Convert(crate::commands::convert::ConvertArgs),
    /// Generate schemas or template configurations
    #[command(subcommand)]
    Generate(crate::commands::generate::GenerateCommands),
    /// Install Pkl CLI tool
    #[command(subcommand)]
    PklMe(crate::commands::pklme::InstallCommands),
}

/// CLI application with error handling
pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert(args) => {
            tracing::info!("Starting configuration conversion");
            match crate::commands::convert::handle_convert(args).await {
                Ok(()) => Ok(()),
                Err(e) => {
                    tracing::error!("Conversion failed: {}", e);
                    Err(miette::Report::new(e))
                }
            }
        }
        Commands::Generate(commands) => {
            tracing::info!("Starting schema/template generation");
            match crate::commands::generate::handle_generate(commands).await {
                Ok(()) => Ok(()),
                Err(e) => {
                    tracing::error!("Generation failed: {}", e);
                    Err(e)
                }
            }
        }
        Commands::PklMe(commands) => {
            tracing::info!("Starting tool installation");
            match crate::commands::pklme::handle_install(commands).await {
                Ok(()) => Ok(()),
                Err(e) => {
                    tracing::error!("Installation failed: {}", e);
                    Err(e)
                }
            }
        }
    }
}
