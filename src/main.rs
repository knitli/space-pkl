//! `main.rs` - CLI entry point for space-pkl
//! CLI for space-pkl - Pkl schema generation for Moon configurations.
//!
//! (c) 2025 Stash AI Inc (knitli)
//!   - Created by Adam Poulemanos ([@bashandbone](https://github.com/bashandbone))
//! Licensed under the [Plain MIT License](https://plainlicense.org/licenses/permissive/mit/)

use clap::{Parser, Subcommand, ValueEnum};
use miette::{IntoDiagnostic, Result, WrapErr};
use space_pkl::config::{GeneratorConfig, SchemaType};
use space_pkl::generator::SchemaGenerator;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(
    name = "space-pkl",
    about = "Generate Pkl schemas and templates for Moon workspace configurations",
    version = env!("CARGO_PKG_VERSION"),
    author = "bashandbone <bashandbone@users.noreply.github.com>"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Set the logging level
    #[arg(short, long, value_enum, default_value = "info")]
    log_level: LogLevel,

    /// Output directory for generated files
    #[arg(short, long, default_value = "./pkl-schemas")]
    output: PathBuf,
}

/// CLI wrapper for SchemaType to implement ValueEnum
#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliSchemaType {
    Workspace,
    Project,
    Template,
    Toolchain,
    Tasks,
    All,
}

impl From<CliSchemaType> for SchemaType {
    fn from(cli_type: CliSchemaType) -> Self {
        match cli_type {
            CliSchemaType::Workspace => SchemaType::Workspace,
            CliSchemaType::Project => SchemaType::Project,
            CliSchemaType::Template => SchemaType::Template,
            CliSchemaType::Toolchain => SchemaType::Toolchain,
            CliSchemaType::Tasks => SchemaType::Tasks,
            CliSchemaType::All => SchemaType::All,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Pkl schemas
    Generate {
        /// Type of schema to generate
        #[arg(value_enum, default_value = "all")]
        schema_type: CliSchemaType,

        /// Don't include comments in generated schemas
        #[arg(long)]
        no_comments: bool,

        /// Don't include examples in generated schemas
        #[arg(long)]
        no_examples: bool,

        /// Custom header for generated files
        #[arg(long)]
        header: Option<String>,

        /// Custom footer for generated files
        #[arg(long)]
        footer: Option<String>,

        /// Module name for generated schemas
        #[arg(long, default_value = "moon")]
        module_name: String,

        /// Generate as single file instead of split types
        #[arg(long)]
        single_file: bool,
    },

    /// Initialize a new Pkl configuration from templates
    Init {
        /// Type of configuration to initialize
        #[arg(value_enum)]
        config_type: CliSchemaType,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include example values
        #[arg(long)]
        with_examples: bool,
    },

    /// Validate an existing Pkl configuration
    Validate {
        /// Path to the configuration file to validate
        file: PathBuf,

        /// Type of configuration (auto-detect if not specified)
        #[arg(short, long, value_enum)]
        config_type: Option<CliSchemaType>,
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::from(cli.log_level))
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .into_diagnostic()
        .wrap_err("Failed to initialize logging")?;

    match cli.command {
        Commands::Generate {
            schema_type,
            no_comments,
            no_examples,
            header,
            footer,
            module_name,
            single_file,
        } => {
            let output_dir = cli.output.clone(); // Clone to avoid move
            let config = GeneratorConfig {
                include_comments: !no_comments,
                include_examples: !no_examples,
                header: header.or_else(|| Some(default_header())),
                footer,
                output_dir,
                module_name,
                split_types: !single_file,
                ..Default::default()
            };

            let generator = SchemaGenerator::new(config);
            let schema_type_enum = SchemaType::from(schema_type);

            match schema_type_enum {
                SchemaType::All => {
                    info!("Generating all Moon configuration schemas");
                    generator.generate_all()?;
                }
                SchemaType::Workspace => {
                    info!("Generating workspace schema");
                    let schema = generator.generate_workspace_schema()?;
                    let file_path = cli.output.join(schema_type_enum.filename());
                    std::fs::create_dir_all(&cli.output).into_diagnostic()?;
                    std::fs::write(&file_path, schema).into_diagnostic()?;
                    info!("Generated: {}", file_path.display());
                }
                SchemaType::Project => {
                    info!("Generating project schema");
                    let schema = generator.generate_project_schema()?;
                    let file_path = cli.output.join(schema_type_enum.filename());
                    std::fs::create_dir_all(&cli.output).into_diagnostic()?;
                    std::fs::write(&file_path, schema).into_diagnostic()?;
                    info!("Generated: {}", file_path.display());
                }
                SchemaType::Template => {
                    info!("Generating template schema");
                    let schema = generator.generate_template_schema()?;
                    let file_path = cli.output.join(schema_type_enum.filename());
                    std::fs::create_dir_all(&cli.output).into_diagnostic()?;
                    std::fs::write(&file_path, schema).into_diagnostic()?;
                    info!("Generated: {}", file_path.display());
                }
                SchemaType::Toolchain => {
                    info!("Generating toolchain schema");
                    let schema = generator.generate_toolchain_schema()?;
                    let file_path = cli.output.join(schema_type_enum.filename());
                    std::fs::create_dir_all(&cli.output).into_diagnostic()?;
                    std::fs::write(&file_path, schema).into_diagnostic()?;
                    info!("Generated: {}", file_path.display());
                }
                SchemaType::Tasks => {
                    info!("Generating tasks schema");
                    let schema = generator.generate_tasks_schema()?;
                    let file_path = cli.output.join(schema_type_enum.filename());
                    std::fs::create_dir_all(&cli.output).into_diagnostic()?;
                    std::fs::write(&file_path, schema).into_diagnostic()?;
                    info!("Generated: {}", file_path.display());
                }
            }
        }

        Commands::Init {
            config_type,
            output: _,
            with_examples: _,
        } => {
            let schema_type_enum = SchemaType::from(config_type);
            info!("Initializing {} configuration", schema_type_enum.module_name());
            // TODO: Implement template initialization
            println!("Template initialization not yet implemented");
        }

        Commands::Validate { file, config_type: _ } => {
            info!("Validating configuration file: {}", file.display());
            // TODO: Implement configuration validation
            println!("Configuration validation not yet implemented");
        }
    }

    Ok(())
}

fn default_header() -> String {
    format!(
        r#"//! Moon Configuration Schema for Pkl
//!
//! Generated by space-pkl v{}
//! Source: https://github.com/knitli/space-pkl
//! Moon: https://github.com/moonrepo/moon
//!
//! This schema provides type-safe configuration templates for Moon workspace management.
//
//! (c) 2025 Stash AI Inc (knitli)
//!   - Created by Adam Poulemanos ([@bashandbone](https://github.com/bashandbone))
//! Licensed under the [Plain MIT License](https://plainlicense.org/licenses/permissive/mit/)
//!

"#,
        env!("CARGO_PKG_VERSION")
    )
}
