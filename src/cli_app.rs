//! CLI Application module for Moon Config CLI
//!
//! This module defines the clap application structure and command dispatching

use clap::{Parser, Subcommand};
use miette::Result;
use std::path::PathBuf;

/// Moon Config CLI - A tool for configuration conversion, schema generation, and Pkl tooling integration
#[derive(Parser)]
#[command(name = "moon-config-cli")]
#[command(about = "A Rust CLI tool for Moon configuration conversion, schema generation, and Pkl tooling integration")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert Moon configuration files between formats
    Convert(crate::commands::convert::ConvertArgs),
    /// Generate schemas or skeleton configurations
    #[command(subcommand)]
    Generate(crate::commands::generate::GenerateCommands),
    /// Install external tools like Pkl CLI
    #[command(subcommand)]
    Install(crate::commands::install::InstallCommands),
}

/// Enhanced CLI application with comprehensive error handling
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
            tracing::info!("Starting schema/skeleton generation");
            match crate::commands::generate::handle_generate(commands).await {
                Ok(()) => Ok(()),
                Err(e) => {
                    tracing::error!("Generation failed: {}", e);
                    Err(e)
                }
            }
        }
        Commands::Install(commands) => {
            tracing::info!("Starting tool installation");
            match crate::commands::install::handle_install(commands).await {
                Ok(()) => Ok(()),
                Err(e) => {
                    tracing::error!("Installation failed: {}", e);
                    Err(e)
                }
            }
        }
    }
}
