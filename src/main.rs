#![warn(missing_docs)]
#![doc(html_root_url = "https://docs.rs/space-pkl")]
//! `main.rs` - CLI entry point for space-pkl
//! CLI for space-pkl - Pkl schema generation for Moon configurations.
//!
//! (c) 2025 Stash AI Inc (knitli)
//!   - Created by Adam Poulemanos ([@bashandbone](https://github.com/bashandbone)) for Stash AI Inc.
//! Licensed under the [Plain MIT License](https://plainlicense.org/licenses/permissive/mit/)

use clap::{Parser, Subcommand, ValueEnum};
use miette::{IntoDiagnostic, Result, WrapErr};
use space_pkl::config::{GeneratorConfig, SchemaType};
use space_pkl::generator::SchemaGenerator;
use std::path::PathBuf;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(
    name = "space-pkl",
    about = "Generate Pkl schemas and templates for Moon workspace configurations",
    version = env!("CARGO_PKG_VERSION"),
    author = "Adam Poulemanos <adam@knit.li>"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Set the logging level
    #[arg(short, long, value_enum, default_value = "info")]
    log_level: LogLevel,
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

        /// Output directory for generated files
        #[arg(short, long, default_value = "./pkl-schemas")]
        output: PathBuf,

        /// Overwrite existing files without prompting
        #[arg(long)]
        overwrite: bool,

        /// Don't include comments in generated schemas
        #[arg(long)]
        no_comments: bool,

        /// Don't include examples in generated schemas
        #[arg(long)]
        no_examples: bool,

        /// Include deprecated fields/types in generated schemas
        #[arg(long)]
        include_deprecated: bool,

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
            output,
            overwrite,
            no_comments,
            no_examples,
            include_deprecated,
            header,
            footer,
            module_name,
            single_file,
        } => {
            let config = GeneratorConfig {
                include_comments: !no_comments,
                include_examples: !no_examples,
                include_deprecated,
                header: header.or_else(|| Some(default_header())),
                footer,
                output_dir: output.clone(),
                module_name,
                split_types: !single_file,
                ..Default::default()
            };

            let generator = SchemaGenerator::new(config);
            let schema_type_enum = SchemaType::from(schema_type);

            // Check for existing files and handle overwrite logic
            let files_to_generate = match schema_type_enum {
                SchemaType::All => vec![
                    SchemaType::Workspace,
                    SchemaType::Project,
                    SchemaType::Template,
                    SchemaType::Toolchain,
                    SchemaType::Tasks,
                ],
                single_type => vec![single_type],
            };

            if !overwrite {
                let mut existing_files = Vec::new();
                for file_type in &files_to_generate {
                    let file_path = output.join(file_type.filename());
                    if file_path.exists() {
                        existing_files.push(file_path);
                    }
                }

                if !existing_files.is_empty() {
                    eprintln!("Error: The following files already exist:");
                    for file in &existing_files {
                        eprintln!("  {}", file.display());
                    }
                    eprintln!("Use --overwrite to overwrite existing files, or specify a different output directory.");
                    return Ok(());
                }
            }

            // Create output directory
            std::fs::create_dir_all(&output)
                .into_diagnostic()
                .wrap_err("Failed to create output directory")?;

            match schema_type_enum {
                SchemaType::All => {
                    info!("Generating all Moon configuration schemas");
                    generator.generate_all()?;

                    // Verify all files were created
                    for file_type in &files_to_generate {
                        let file_path = output.join(file_type.filename());
                        if !file_path.exists() {
                            warn!("Failed to create file: {}", file_path.display());
                        } else {
                            info!("Generated: {}", file_path.display());
                        }
                    }

                    // Check for module index if split_types is enabled
                    if !single_file {
                        let mod_file = output.join("mod.pkl");
                        if !mod_file.exists() {
                            warn!("Failed to create module index: {}", mod_file.display());
                        } else {
                            info!("Generated module index: {}", mod_file.display());
                        }
                    }
                }
                SchemaType::Workspace => {
                    info!("Generating workspace schema");
                    let schema = generator.generate_workspace_schema()?;
                    let file_path = output.join(schema_type_enum.filename());
                    std::fs::write(&file_path, schema).into_diagnostic()?;

                    if !file_path.exists() {
                        warn!("Failed to verify file creation: {}", file_path.display());
                    } else {
                        info!("Generated: {}", file_path.display());
                    }
                }
                SchemaType::Project => {
                    info!("Generating project schema");
                    let schema = generator.generate_project_schema()?;
                    let file_path = output.join(schema_type_enum.filename());
                    std::fs::write(&file_path, schema).into_diagnostic()?;

                    if !file_path.exists() {
                        warn!("Failed to verify file creation: {}", file_path.display());
                    } else {
                        info!("Generated: {}", file_path.display());
                    }
                }
                SchemaType::Template => {
                    info!("Generating template schema");
                    let schema = generator.generate_template_schema()?;
                    let file_path = output.join(schema_type_enum.filename());
                    std::fs::write(&file_path, schema).into_diagnostic()?;

                    if !file_path.exists() {
                        warn!("Failed to verify file creation: {}", file_path.display());
                    } else {
                        info!("Generated: {}", file_path.display());
                    }
                }
                SchemaType::Toolchain => {
                    info!("Generating toolchain schema");
                    let schema = generator.generate_toolchain_schema()?;
                    let file_path = output.join(schema_type_enum.filename());
                    std::fs::write(&file_path, schema).into_diagnostic()?;

                    if !file_path.exists() {
                        warn!("Failed to verify file creation: {}", file_path.display());
                    } else {
                        info!("Generated: {}", file_path.display());
                    }
                }
                SchemaType::Tasks => {
                    info!("Generating tasks schema");
                    let schema = generator.generate_tasks_schema()?;
                    let file_path = output.join(schema_type_enum.filename());
                    std::fs::write(&file_path, schema).into_diagnostic()?;

                    if !file_path.exists() {
                        warn!("Failed to verify file creation: {}", file_path.display());
                    } else {
                        info!("Generated: {}", file_path.display());
                    }
                }
            }
        }

        Commands::Init {
            config_type,
            output: _,
            with_examples: _,
        } => {
            let schema_type_enum = SchemaType::from(config_type);
            info!(
                "Initializing {} configuration",
                schema_type_enum.module_name()
            );
            // TODO: Implement template initialization
            println!("Template initialization not yet implemented");
        }

        Commands::Validate {
            file,
            config_type: _,
        } => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_default_values() {
        let cli = Cli::try_parse_from(&["space-pkl", "generate"]).unwrap();

        match cli.command {
            Commands::Generate {
                schema_type,
                output,
                overwrite,
                no_comments,
                no_examples,
                include_deprecated,
                header,
                footer,
                module_name,
                single_file,
            } => {
                assert!(matches!(schema_type, CliSchemaType::All));
                assert_eq!(output, PathBuf::from("./pkl-schemas"));
                assert!(!overwrite);
                assert!(!no_comments);
                assert!(!no_examples);
                assert!(!include_deprecated);
                assert!(header.is_none());
                assert!(footer.is_none());
                assert_eq!(module_name, "moon");
                assert!(!single_file);
            }
            _ => panic!("Expected Generate command"),
        }

        assert!(matches!(cli.log_level, LogLevel::Info));
    }

    #[test]
    fn test_cli_schema_type_conversion() {
        let workspace = SchemaType::from(CliSchemaType::Workspace);
        assert!(matches!(workspace, SchemaType::Workspace));

        let project = SchemaType::from(CliSchemaType::Project);
        assert!(matches!(project, SchemaType::Project));

        let template = SchemaType::from(CliSchemaType::Template);
        assert!(matches!(template, SchemaType::Template));

        let toolchain = SchemaType::from(CliSchemaType::Toolchain);
        assert!(matches!(toolchain, SchemaType::Toolchain));

        let tasks = SchemaType::from(CliSchemaType::Tasks);
        assert!(matches!(tasks, SchemaType::Tasks));

        let all = SchemaType::from(CliSchemaType::All);
        assert!(matches!(all, SchemaType::All));
    }

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(Level::from(LogLevel::Trace), Level::TRACE);
        assert_eq!(Level::from(LogLevel::Debug), Level::DEBUG);
        assert_eq!(Level::from(LogLevel::Info), Level::INFO);
        assert_eq!(Level::from(LogLevel::Warn), Level::WARN);
        assert_eq!(Level::from(LogLevel::Error), Level::ERROR);
    }

    #[test]
    fn test_generate_command_all_options() {
        let cli = Cli::try_parse_from(&[
            "space-pkl",
            "generate",
            "workspace",
            "--output",
            "/tmp/test-output",
            "--overwrite",
            "--no-comments",
            "--no-examples",
            "--include-deprecated",
            "--header",
            "Custom header",
            "--footer",
            "Custom footer",
            "--module-name",
            "test-module",
            "--single-file",
        ])
        .unwrap();

        match cli.command {
            Commands::Generate {
                schema_type,
                output,
                overwrite,
                no_comments,
                no_examples,
                include_deprecated,
                header,
                footer,
                module_name,
                single_file,
            } => {
                assert!(matches!(schema_type, CliSchemaType::Workspace));
                assert_eq!(output, PathBuf::from("/tmp/test-output"));
                assert!(overwrite);
                assert!(no_comments);
                assert!(no_examples);
                assert!(include_deprecated);
                assert_eq!(header.as_ref().unwrap(), "Custom header");
                assert_eq!(footer.as_ref().unwrap(), "Custom footer");
                assert_eq!(module_name, "test-module");
                assert!(single_file);
            }
            _ => panic!("Expected Generate command"),
        }
    }

    #[test]
    fn test_init_command_parsing() {
        let cli = Cli::try_parse_from(&[
            "space-pkl",
            "init",
            "workspace",
            "--output",
            "/tmp/test.pkl",
            "--with-examples",
        ])
        .unwrap();

        match cli.command {
            Commands::Init {
                config_type,
                output,
                with_examples,
            } => {
                assert!(matches!(config_type, CliSchemaType::Workspace));
                assert_eq!(output.as_ref().unwrap(), &PathBuf::from("/tmp/test.pkl"));
                assert!(with_examples);
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_validate_command_parsing() {
        let cli = Cli::try_parse_from(&[
            "space-pkl",
            "validate",
            "/path/to/config.pkl",
            "--config-type",
            "project",
        ])
        .unwrap();

        match cli.command {
            Commands::Validate { file, config_type } => {
                assert_eq!(file, PathBuf::from("/path/to/config.pkl"));
                assert!(matches!(config_type.unwrap(), CliSchemaType::Project));
            }
            _ => panic!("Expected Validate command"),
        }
    }

    #[test]
    fn test_validate_command_without_config_type() {
        let cli = Cli::try_parse_from(&["space-pkl", "validate", "/path/to/config.pkl"]).unwrap();

        match cli.command {
            Commands::Validate { file, config_type } => {
                assert_eq!(file, PathBuf::from("/path/to/config.pkl"));
                assert!(config_type.is_none());
            }
            _ => panic!("Expected Validate command"),
        }
    }

    #[test]
    fn test_log_level_parsing() {
        let cli = Cli::try_parse_from(&["space-pkl", "--log-level", "debug", "generate"]).unwrap();

        assert!(matches!(cli.log_level, LogLevel::Debug));
    }

    #[test]
    fn test_log_level_short_option() {
        let cli = Cli::try_parse_from(&["space-pkl", "-l", "error", "generate"]).unwrap();

        assert!(matches!(cli.log_level, LogLevel::Error));
    }

    #[test]
    fn test_invalid_schema_type() {
        let result = Cli::try_parse_from(&["space-pkl", "generate", "--schema-type", "invalid"]);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_log_level() {
        let result = Cli::try_parse_from(&["space-pkl", "--log-level", "invalid", "generate"]);

        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_args() {
        // Init command requires config_type
        let result = Cli::try_parse_from(&["space-pkl", "init"]);
        assert!(result.is_err());

        // Validate command requires file
        let result = Cli::try_parse_from(&["space-pkl", "validate"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_help_flag() {
        let result = Cli::try_parse_from(&["space-pkl", "--help"]);

        // Help flag should cause an error with exit code 0
        assert!(result.is_err());
    }

    #[test]
    fn test_version_flag() {
        let result = Cli::try_parse_from(&["space-pkl", "--version"]);

        // Version flag should cause an error with exit code 0
        assert!(result.is_err());
    }

    #[test]
    fn test_default_header_generation() {
        let header = default_header();

        assert!(header.contains("Moon Configuration Schema for Pkl"));
        assert!(header.contains(env!("CARGO_PKG_VERSION")));
        assert!(header.contains("https://github.com/knitli/space-pkl"));
        assert!(header.contains("https://github.com/moonrepo/moon"));
        assert!(header.contains("Stash AI Inc"));
        assert!(header.contains("Adam Poulemanos"));
        assert!(header.contains("Plain MIT License"));
    }

    #[test]
    fn test_cli_schema_type_debug() {
        let schema_type = CliSchemaType::Workspace;
        let debug_str = format!("{:?}", schema_type);
        assert_eq!(debug_str, "Workspace");
    }

    #[test]
    fn test_cli_schema_type_clone() {
        let original = CliSchemaType::Project;
        let cloned = original.clone();
        assert!(matches!(cloned, CliSchemaType::Project));
    }

    #[test]
    fn test_log_level_debug() {
        let level = LogLevel::Info;
        let debug_str = format!("{:?}", level);
        assert_eq!(debug_str, "Info");
    }

    #[test]
    fn test_log_level_clone() {
        let original = LogLevel::Warn;
        let cloned = original.clone();
        assert!(matches!(cloned, LogLevel::Warn));
    }

    #[test]
    fn test_complex_argument_combinations() {
        // Test complex combination of arguments
        let cli = Cli::try_parse_from(&[
            "space-pkl",
            "--log-level",
            "trace",
            "generate",
            "all",
            "--output",
            "./custom-output",
            "--overwrite",
            "--no-comments",
            "--include-deprecated",
            "--module-name",
            "custom-moon",
            "--single-file",
        ])
        .unwrap();

        assert!(matches!(cli.log_level, LogLevel::Trace));

        match cli.command {
            Commands::Generate {
                schema_type,
                output,
                overwrite,
                no_comments,
                include_deprecated,
                module_name,
                single_file,
                ..
            } => {
                assert!(matches!(schema_type, CliSchemaType::All));
                assert_eq!(output, PathBuf::from("./custom-output"));
                assert!(overwrite);
                assert!(no_comments);
                assert!(include_deprecated);
                assert_eq!(module_name, "custom-moon");
                assert!(single_file);
            }
            _ => panic!("Expected Generate command"),
        }
    }

    #[test]
    fn test_generate_command_with_unicode_paths() {
        let cli = Cli::try_parse_from(&[
            "space-pkl",
            "generate",
            "--output",
            "./测试/输出/🚀",
            "--header",
            "Unicode header: 测试 🎉",
            "--footer",
            "Unicode footer: ⭐ 完成",
            "--module-name",
            "测试模块",
        ])
        .unwrap();

        match cli.command {
            Commands::Generate {
                output,
                header,
                footer,
                module_name,
                ..
            } => {
                assert_eq!(output, PathBuf::from("./测试/输出/🚀"));
                assert_eq!(header.as_ref().unwrap(), "Unicode header: 测试 🎉");
                assert_eq!(footer.as_ref().unwrap(), "Unicode footer: ⭐ 完成");
                assert_eq!(module_name, "测试模块");
            }
            _ => panic!("Expected Generate command"),
        }
    }

    #[test]
    fn test_init_command_minimal() {
        let cli = Cli::try_parse_from(&["space-pkl", "init", "template"]).unwrap();

        match cli.command {
            Commands::Init {
                config_type,
                output,
                with_examples,
            } => {
                assert!(matches!(config_type, CliSchemaType::Template));
                assert!(output.is_none());
                assert!(!with_examples);
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_validate_command_with_special_characters() {
        let cli =
            Cli::try_parse_from(&["space-pkl", "validate", "./config files/test[1].pkl"]).unwrap();

        match cli.command {
            Commands::Validate { file, .. } => {
                assert_eq!(file, PathBuf::from("./config files/test[1].pkl"));
            }
            _ => panic!("Expected Validate command"),
        }
    }

    #[test]
    fn test_all_schema_types_parsing() {
        let schema_types = vec![
            ("workspace", CliSchemaType::Workspace),
            ("project", CliSchemaType::Project),
            ("template", CliSchemaType::Template),
            ("toolchain", CliSchemaType::Toolchain),
            ("tasks", CliSchemaType::Tasks),
            ("all", CliSchemaType::All),
        ];

        for (type_str, _expected_type) in schema_types {
            let cli = Cli::try_parse_from(&["space-pkl", "generate", type_str]).unwrap();

            match cli.command {
                Commands::Generate { schema_type, .. } => {
                    // Convert the expected type to check against
                    let expected_schema = SchemaType::from(_expected_type);
                    let actual_schema = SchemaType::from(schema_type);

                    // Use discriminant comparison for enum variants
                    assert_eq!(
                        std::mem::discriminant(&expected_schema),
                        std::mem::discriminant(&actual_schema)
                    );
                }
                _ => panic!("Expected Generate command"),
            }
        }
    }

    #[test]
    fn test_all_log_levels_parsing() {
        let log_levels = vec![
            ("trace", LogLevel::Trace),
            ("debug", LogLevel::Debug),
            ("info", LogLevel::Info),
            ("warn", LogLevel::Warn),
            ("error", LogLevel::Error),
        ];

        for (level_str, expected_level) in log_levels {
            let cli =
                Cli::try_parse_from(&["space-pkl", "--log-level", level_str, "generate"]).unwrap();

            // Compare the LogLevel enums directly
            assert_eq!(
                std::mem::discriminant(&expected_level),
                std::mem::discriminant(&cli.log_level)
            );
        }
    }

    // Integration tests for command execution would require async test framework
    // These test the actual command logic without async execution

    #[test]
    fn test_generate_files_list_logic() {
        use space_pkl::config::SchemaType;

        // Test the logic for determining which files to generate
        let all_types = vec![
            SchemaType::Workspace,
            SchemaType::Project,
            SchemaType::Template,
            SchemaType::Toolchain,
            SchemaType::Tasks,
        ];

        // When SchemaType::All is used, should generate all types
        let files_for_all = match SchemaType::All {
            SchemaType::All => all_types.clone(),
            single_type => vec![single_type],
        };

        assert_eq!(files_for_all.len(), 5);
        assert!(files_for_all.contains(&SchemaType::Workspace));
        assert!(files_for_all.contains(&SchemaType::Project));
        assert!(files_for_all.contains(&SchemaType::Template));
        assert!(files_for_all.contains(&SchemaType::Toolchain));
        assert!(files_for_all.contains(&SchemaType::Tasks));

        // When single type is used, should generate only that type
        let files_for_single = match SchemaType::Workspace {
            SchemaType::All => all_types,
            single_type => vec![single_type],
        };

        assert_eq!(files_for_single.len(), 1);
        assert!(files_for_single.contains(&SchemaType::Workspace));
    }

    #[test]
    fn test_config_building_from_cli_args() {
        // Test the configuration building logic from CLI arguments
        let no_comments = true;
        let no_examples = false;
        let include_deprecated = true;
        let header = Some("Custom header".to_string());
        let footer = None;
        let output = PathBuf::from("/tmp/test");
        let module_name = "test_module".to_string();
        let single_file = true;

        let config = GeneratorConfig {
            include_comments: !no_comments,
            include_examples: !no_examples,
            include_deprecated,
            header: header.or_else(|| Some(default_header())),
            footer,
            output_dir: output.clone(),
            module_name: module_name.clone(),
            split_types: !single_file,
            ..Default::default()
        };

        assert!(!config.include_comments); // no_comments was true
        assert!(config.include_examples); // no_examples was false
        assert!(config.include_deprecated);
        assert_eq!(config.header.as_ref().unwrap(), "Custom header");
        assert!(config.footer.is_none());
        assert_eq!(config.output_dir, output);
        assert_eq!(config.module_name, module_name);
        assert!(!config.split_types); // single_file was true
    }

    #[test]
    fn test_config_building_with_defaults() {
        // Test configuration building with default values
        let no_comments = false;
        let no_examples = false;
        let include_deprecated = false;
        let header: Option<String> = None;
        let footer = None;
        let output = PathBuf::from("./pkl-schemas");
        let module_name = "moon".to_string();
        let single_file = false;

        let config = GeneratorConfig {
            include_comments: !no_comments,
            include_examples: !no_examples,
            include_deprecated,
            header: header.or_else(|| Some(default_header())),
            footer,
            output_dir: output.clone(),
            module_name: module_name.clone(),
            split_types: !single_file,
            ..Default::default()
        };

        assert!(config.include_comments);
        assert!(config.include_examples);
        assert!(!config.include_deprecated);
        assert!(config.header.is_some()); // Should use default header
        assert!(config.footer.is_none());
        assert_eq!(config.output_dir, output);
        assert_eq!(config.module_name, module_name);
        assert!(config.split_types);
    }

    // Mock tests for error handling scenarios
    #[test]
    fn test_error_handling_patterns() {
        use miette::{IntoDiagnostic, WrapErr};

        // Test the error wrapping patterns used in main
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ));

        let wrapped = result.into_diagnostic().wrap_err("Failed to read file");
        assert!(wrapped.is_err());

        // Test directory creation error pattern
        let result = std::fs::create_dir_all("/invalid/path/that/cannot/be/created")
            .into_diagnostic()
            .wrap_err("Failed to create output directory");

        // This should fail on most systems due to permissions
        assert!(result.is_err());
    }

    #[test]
    fn test_path_handling() {
        // Test various path formats that might be passed via CLI
        let paths = vec![
            "./relative/path",
            "/absolute/path",
            "~/home/path",
            "../parent/path",
            "./path with spaces/file.pkl",
            "./path_with_underscores/file.pkl",
            "./path-with-hyphens/file.pkl",
        ];

        for path_str in paths {
            let path = PathBuf::from(path_str);
            // Just verify that PathBuf can handle these formats
            assert!(!path.as_os_str().is_empty());
        }
    }

    #[test]
    fn test_filename_generation() {
        // Test the filename() method from SchemaType (assuming it exists)
        // This would test the logic for generating output filenames
        let types = vec![
            SchemaType::Workspace,
            SchemaType::Project,
            SchemaType::Template,
            SchemaType::Toolchain,
            SchemaType::Tasks,
        ];

        for schema_type in types {
            let filename = schema_type.filename();
            assert!(!filename.is_empty());
            assert!(filename.ends_with(".pkl"));
        }
    }
}
