//! Convert command implementation for Moon Config CLI
//!
//! This module handles configuration file conversion between formats
//!.

use clap::Args;
use miette::Result;
use std::path::PathBuf;

use crate::config_processor::{ConfigFormat, MoonConfigType};
use crate::error::{CliError, ensure_file_exists, ensure_output_writable};

/// Convert command arguments.
#[derive(Args)]
pub struct ConvertArgs {
    /// Moon configuration type (required for type safety)
    #[arg(long, help = "Configuration type: project, workspace, template, toolchain, task")]
    pub config_type: MoonConfigType,

    /// Path to the input configuration file
    #[arg(short, long, help = "Input configuration file path")]
    pub input: PathBuf,

    /// Path to the output file (optional, defaults to stdout)
    #[arg(short, long, help = "Output file path (defaults to stdout)")]
    pub output: Option<PathBuf>,

    /// Input format (optional, auto-detected if not provided)
    #[arg(long, help = "Input format (auto-detected if not specified)")]
    pub from: Option<ConfigFormat>,

    /// Output format (intelligent defaults applied)
    #[arg(long, help = "Output format (defaults to json if input is yaml, otherwise yaml)")]
    pub to: Option<ConfigFormat>,

    /// Overwrite existing output file
    #[arg(short, long, help = "Force overwrite of existing output files")]
    pub force: bool,
}

/// Handle convert command execution
pub async fn handle_convert(args: ConvertArgs) -> Result<(), CliError> {
    use crate::config_processor::{load_config, convert_config, detect_format_from_path, ensure_pkl_available};
    use crate::error::{ensure_file_exists, ensure_output_writable};

    // Validate arguments
    validate_convert_args(&args)?;

    println!("🔄 Converting {} configuration...", args.config_type);
    println!("📁 Input: {}", args.input.display());

    // Load the configuration file
    let (content, detected_input_format) = load_config(&args.input, args.config_type, args.from).await?;

    // Apply format defaults with Pkl preferences
    let output_format = apply_format_defaults_with_pkl(Some(detected_input_format.clone()), args.to);

    println!("🔧 Converting from {} to {}", detected_input_format, output_format);

    // Check if Pkl CLI is needed and available
    if detected_input_format == ConfigFormat::Pkl || output_format == ConfigFormat::Pkl {
        match ensure_pkl_available().await {
            Ok(_) => {
                println!("✅ Pkl CLI is available");
            }
            Err(_) => {
                println!("⚠️  Pkl CLI not found. To use Pkl conversions, install it with:");
                println!("   moon-config-cli install pkl");

                // For now, proceed with placeholder conversion
                println!("🔄 Proceeding with basic conversion (full Pkl support requires Pkl CLI)");
            }
        }
    }

    // Convert the configuration
    let converted_content = convert_config(&content, detected_input_format, output_format.clone())?;

    // Write output
    if let Some(output_path) = &args.output {
        // Write to file
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| CliError::IoError {
                    context: format!("Creating output directory: {}", parent.display()),
                    source: e,
                })?;
        }

        tokio::fs::write(output_path, converted_content).await
            .map_err(|e| CliError::IoError {
                context: format!("Writing output file: {}", output_path.display()),
                source: e,
            })?;

        println!("✅ Successfully converted to {}", output_path.display());
    } else {
        // Write to stdout
        println!("--- Converted Configuration ---");
        println!("{}", converted_content);
    }

    Ok(())
}

/// Apply intelligent defaults for conversion formats
fn apply_format_defaults_with_pkl(from: Option<ConfigFormat>, to: Option<ConfigFormat>) -> ConfigFormat {
    to.unwrap_or_else(|| {
        match from {
            Some(ConfigFormat::Yaml) => ConfigFormat::Pkl, // Encourage Pkl adoption
            Some(ConfigFormat::Json) => ConfigFormat::Pkl, // Encourage Pkl adoption
            Some(ConfigFormat::Pkl) => ConfigFormat::Yaml, // Pkl to YAML for compatibility
            None => ConfigFormat::Json, // Default to JSON
        }
    })
}

/// Legacy function for backward compatibility
fn apply_format_defaults(from: Option<ConfigFormat>, to: Option<ConfigFormat>) -> ConfigFormat {
    apply_format_defaults_with_pkl(from, to)
}

/// Validate conversion arguments
fn validate_convert_args(args: &ConvertArgs) -> Result<(), CliError> {
    crate::error::ensure_file_exists(&args.input)?;

    if let Some(output) = &args.output {
        crate::error::ensure_output_writable(output, args.force)?;
    }

    Ok(())
}
