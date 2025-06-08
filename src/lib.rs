//! Space Pklr Library
//!
//! This library provides the core functionality for the Space Pklr tool,
//! including configuration conversion, schema generation, and Pkl tooling integration.

pub mod cli_app;
pub mod commands;
pub mod pkl_tooling;
pub mod types;

// Re-export commonly used types
pub use types::{CliError, InternalError, Result, SchemaFormat, LoadedConfig, MoonConfig, TypeMap, EnumTranslation, OpenStructs, ConfigTranslation, OptionalFormat, PropertyDefault, ensure_file_exists, ensure_output_writable, pkl_execution_error};
pub use pkl_tooling::{CompatibilityReport, PklCli, PklSource};
