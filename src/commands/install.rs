//! Install command implementation for Moon Config CLI
//!
//! This module handles installation of external tools like Pkl CLI
//!.

use clap::{Args, Subcommand};
use miette::Result;

/// Install command with subcommands.
#[derive(Subcommand)]
pub enum InstallCommands {
    /// Install Pkl CLI
    Pkl(PklInstallArgs),
}

/// Pkl installation arguments
#[derive(Args)]
pub struct PklInstallArgs {
    /// Specific version to install (defaults to recommended version)
    #[arg(long, help = "Pkl version to install (defaults to tested compatible version)")]
    pub version: Option<String>,

    /// Force reinstallation even if already installed
    #[arg(short, long, help = "Force reinstallation")]
    pub force: bool,
}

/// Handle install command execution
///
/// - Dispatch to appropriate tool installation handler
/// - Currently only supports Pkl CLI installation
pub async fn handle_install(commands: InstallCommands) -> Result<()> {
    match commands {
        InstallCommands::Pkl(args) => handle_pkl_installation(args).await,
    }
}

/// Handle Pkl CLI installation
///
/// - Use pkl_tooling module for installation logic
/// - Apply version defaults (pinned compatible version)
/// - Handle force reinstallation
/// - Provide progress indicators and clear feedback
pub async fn handle_pkl_installation(args: PklInstallArgs) -> Result<()> {
    let version = args.version.unwrap_or_else(|| {
        crate::pkl_tooling::get_recommended_pkl_version().to_string()
    });

    display_installation_progress(&format!("Starting Pkl CLI installation (version: {})", version));

    if args.force {
        println!("🔄 Force flag enabled - will reinstall if already present");
    }

    // Check existing installation if not forcing
    if !args.force {
        display_installation_progress("Checking for existing Pkl installation...");
        if let Ok(Some(existing_pkl)) = crate::pkl_tooling::find_pkl_executable().await {
            if let Some(existing_version) = &existing_pkl.version {
                if existing_version == &version {
                    println!("✅ Pkl CLI version {} already installed at: {}", existing_version, existing_pkl.path.display());
                    println!("   Source: {:?}", existing_pkl.source);
                    println!("   Use --force to reinstall");
                    return Ok(());
                } else {
                    println!("⚠️  Found Pkl CLI version {}, but requested version {}", existing_version, version);
                    println!("   Proceeding with installation of requested version...");
                }
            } else {
                println!("⚠️  Found Pkl CLI but could not determine version");
                println!("   Proceeding with installation...");
            }
        }
    }

    // Perform installation
    display_installation_progress(&format!("Installing Pkl CLI version {}...", version));
    let pkl_cli = crate::pkl_tooling::install_pkl(Some(version.clone())).await?;

    // Validate installation
    display_installation_progress("Validating installation...");
    let is_valid = crate::pkl_tooling::validate_pkl_installation(&pkl_cli).await?;

    if is_valid {
        display_installation_success("Pkl CLI", &pkl_cli.path, Some(&version));
        println!("   Source: {:?}", pkl_cli.source);
        println!("   You can now use Pkl conversions in the convert command");
    } else {
        return Err(miette::Report::new(crate::error::CliError::PklInstallFailed {
            reason: "Installation validation failed".to_string(),
            help: Some("Try reinstalling or check installation manually".to_string()),
        }));
    }

    Ok(())
}

/// Check if tool is already installed
///
/// - Check if tool is already available and functional
/// - Return version information if available
async fn check_existing_installation() -> Result<Option<String>> {

    if let Ok(Some(pkl_cli)) = crate::pkl_tooling::find_pkl_executable().await {
        Ok(pkl_cli.version)
    } else {
        Ok(None)
    }
}

/// Display installation progress
fn display_installation_progress(step: &str) {
    println!("⏳ {}", step);
}

/// Display installation success
fn display_installation_success(tool: &str, path: &std::path::Path, version: Option<&str>) {
    println!("✅ Successfully installed {} at {}", tool, path.display());
    if let Some(v) = version {
        println!("   Version: {}", v);
    }
}
