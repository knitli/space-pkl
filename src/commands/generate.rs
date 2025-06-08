//! Generate command implementation for Space Pklr
//!
//! This module handles schema and template generation
//!.

use std::str::FromStr;
use clap::{Args, Subcommand};
use miette::Result;
use std::path::PathBuf;

use crate::types::MoonConfig;

/// Generate command with subcommands.
#[derive(Subcommand)]
pub enum GenerateCommands {
    /// Generate schema for a Moon configuration type
    Schema(SchemaArgs),
    /// Generate template (default) configuration file
    Template(TemplateArgs),
}

/// Common arguments for generate subcommands
#[derive(Args)]
pub struct GenerateArgs {
    /// Moon configuration type (defaults to 'all')
    #[arg(long, default_value = "all", help = "Configuration type: project, workspace, template, toolchain, task, all (default)")]
    pub config_type: MoonConfig,

    /// Output directory for multiple files or file path for single output (optional, defaults to stdout)
    #[arg(short, long, help = "Output directory for multiple files or file path for single output (defaults to stdout)")]
    pub output: Option<PathBuf>,
}

/// Schema generation arguments
#[derive(Args)]
pub struct SchemaArgs {
    #[command(flatten)]
    pub common: GenerateArgs,

    #[arg(long, default_value = "all", help = "Schema format: json-schema, typescript, all (default)")]
    pub format: String,
}

/// Template generation arguments
#[derive(Args)]
pub struct TemplateArgs {
    #[command(flatten)]
    pub common: GenerateArgs,

    /// Output configuration format (defaults to 'all')
    #[arg(long, default_value = "all", help = "Configuration format: yaml, json, pkl, all (default)")]
    pub format: String,
}

/// Handle generate command execution
pub async fn handle_generate(commands: GenerateCommands) -> Result<()> {
    match commands {
        GenerateCommands::Schema(args) => handle_schema_generation(args).await,
        GenerateCommands::Template(args) => handle_template_generation(args).await,
    }
}

/// Handle schema generation using schematic's existing capabilities
pub async fn handle_schema_generation(args: SchemaArgs) -> Result<()> {
    use crate::_rewrite::{generate_schema, generate_all_schemas, generate_all_formats_schema, generate_all_schemas_all_formats};
    use crate::types::MoonConfig;

    match (&args.common.config_type, args.format.as_str()) {
        (MoonConfig::All, "all") => {
            println!("🔧 Generating schemas for all configuration types in all formats...");
            let results = generate_all_schemas_all_formats()
                .map_err(|e| miette::miette!("Failed to generate schemas: {}", e))?;

            if let Some(output_dir) = &args.common.output {
                tokio::fs::create_dir_all(output_dir).await
                    .map_err(|e| miette::miette!("Failed to create output directory {}: {}", output_dir.display(), e))?;

                for (filename, content) in results {
                    let file_path = output_dir.join(&filename);
                    tokio::fs::write(&file_path, &content).await
                        .map_err(|e| miette::miette!("Failed to write schema to {}: {}", file_path.display(), e))?;
                    println!("✅ Generated: {}", file_path.display());
                }
            } else {
                for (filename, content) in results {
                    println!("\n=== {} ===", filename);
                    println!("{}", content);
                }
            }
        }
        (MoonConfig::All, format) => {
            println!("🔧 Generating schemas for all configuration types in {} format...", format);
            let results = generate_all_schemas(format)
                .map_err(|e| miette::miette!("Failed to generate schemas: {}", e))?;

            if let Some(output_dir) = &args.common.output {
                tokio::fs::create_dir_all(output_dir).await
                    .map_err(|e| miette::miette!("Failed to create output directory {}: {}", output_dir.display(), e))?;

                for (filename, content) in results {
                    let file_path = output_dir.join(&filename);
                    tokio::fs::write(&file_path, &content).await
                        .map_err(|e| miette::miette!("Failed to write schema to {}: {}", file_path.display(), e))?;
                    println!("✅ Generated: {}", file_path.display());
                }
            } else {
                for (filename, content) in results {
                    println!("\n=== {} ===", filename);
                    println!("{}", content);
                }
            }
        }
        (config_type, "all") => {
            println!("🔧 Generating {} schemas in all formats...", config_type);
            let results = generate_all_formats_schema(*config_type)
                .map_err(|e| miette::miette!("Failed to generate schemas: {}", e))?;

            if let Some(output_dir) = &args.common.output {
                tokio::fs::create_dir_all(output_dir).await
                    .map_err(|e| miette::miette!("Failed to create output directory {}: {}", output_dir.display(), e))?;

                for (filename, content) in results {
                    let file_path = output_dir.join(&filename);
                    tokio::fs::write(&file_path, &content).await
                        .map_err(|e| miette::miette!("Failed to write schema to {}: {}", file_path.display(), e))?;
                    println!("✅ Generated: {}", file_path.display());
                }
            } else {
                for (filename, content) in results {
                    println!("\n=== {} ===", filename);
                    println!("{}", content);
                }
            }
        }
        (config_type, format) => {
            println!("🔧 Generating {} schema in {} format...", config_type, format);

            // Generate schema using schematic's existing renderers
            let schema_content = generate_schema(*config_type, format)
                .map_err(|e| miette::miette!("Failed to generate schema: {}", e))?;

            // Output to file or stdout
            if let Some(output_path) = &args.common.output {
                tokio::fs::write(output_path, &schema_content)
                    .await
                    .map_err(|e| miette::miette!("Failed to write schema to {}: {}",
                                               output_path.display(), e))?;

                println!("✅ Schema generated successfully: {}", output_path.display());
            } else {
                println!("{}", schema_content);
            }
        }
    }

    Ok(())
}

/// Handle template configuration generation using existing templates and defaults
pub async fn handle_template_generation(args: TemplateArgs) -> Result<()> {
    use crate::_rewrite::{generate_template, generate_all_templates, generate_all_formats_template, generate_all_templates_all_formats};
    use crate::types::{SchemaFormat, MoonConfig};

    match (&args.common.config_type, args.format.as_str()) {
        (MoonConfig::All, "all") => {
            println!("🔧 Generating template configurations for all types in all formats...");
            let results = generate_all_templates_all_formats()
                .map_err(|e| miette::miette!("Failed to generate templates: {}", e))?;

            if let Some(output_dir) = &args.common.output {
                tokio::fs::create_dir_all(output_dir).await
                    .map_err(|e| miette::miette!("Failed to create output directory {}: {}", output_dir.display(), e))?;

                for (filename, content) in results {
                    let file_path = output_dir.join(&filename);
                    tokio::fs::write(&file_path, &content).await
                        .map_err(|e| miette::miette!("Failed to write template to {}: {}", file_path.display(), e))?;
                    println!("✅ Generated: {}", file_path.display());
                }
            } else {
                for (filename, content) in results {
                    println!("\n=== {} ===", filename);
                    println!("{}", content);
                }
            }
        }
        (MoonConfig::All, format_str) => {
            let format = SchemaFormat::from_str(format_str)
                .map_err(|e| miette::miette!("Invalid format '{}': {}", format_str, e))?;

            println!("🔧 Generating template configurations for all types in {} format...", format);
            let results = generate_all_templates(format)
                .map_err(|e| miette::miette!("Failed to generate templates: {}", e))?;

            if let Some(output_dir) = &args.common.output {
                tokio::fs::create_dir_all(output_dir).await
                    .map_err(|e| miette::miette!("Failed to create output directory {}: {}", output_dir.display(), e))?;

                for (filename, content) in results {
                    let file_path = output_dir.join(&filename);
                    tokio::fs::write(&file_path, &content).await
                        .map_err(|e| miette::miette!("Failed to write template to {}: {}", file_path.display(), e))?;
                    println!("✅ Generated: {}", file_path.display());
                }
            } else {
                for (filename, content) in results {
                    println!("\n=== {} ===", filename);
                    println!("{}", content);
                }
            }
        }
        (config_type, "all") => {
            println!("🔧 Generating {} template configurations in all formats...", config_type);
            let results = generate_all_formats_template(*config_type)
                .map_err(|e| miette::miette!("Failed to generate templates: {}", e))?;

            if let Some(output_dir) = &args.common.output {
                tokio::fs::create_dir_all(output_dir).await
                    .map_err(|e| miette::miette!("Failed to create output directory {}: {}", output_dir.display(), e))?;

                for (filename, content) in results {
                    let file_path = output_dir.join(&filename);
                    tokio::fs::write(&file_path, &content).await
                        .map_err(|e| miette::miette!("Failed to write template to {}: {}", file_path.display(), e))?;
                    println!("✅ Generated: {}", file_path.display());
                }
            } else {
                for (filename, content) in results {
                    println!("\n=== {} ===", filename);
                    println!("{}", content);
                }
            }
        }
        (config_type, format_str) => {
            let format = SchemaFormat::from_str(format_str)
                .map_err(|e| miette::miette!("Invalid format '{}': {}", format_str, e))?;

            println!("🔧 Generating {} template configuration in {} format...", config_type, format);

            // Generate template using existing templates and defaults
            let template_content = generate_template(*config_type, format)
                .map_err(|e| miette::miette!("Failed to generate template: {}", e))?;

            // Output to file or stdout
            if let Some(output_path) = &args.common.output {
                tokio::fs::write(output_path, &template_content)
                    .await
                    .map_err(|e| miette::miette!("Failed to write template to {}: {}",
                                               output_path.display(), e))?;

                println!("✅ Template configuration generated successfully: {}", output_path.display());
            } else {
                println!("{}", template_content);
            }
        }
    }

    Ok(())
}
