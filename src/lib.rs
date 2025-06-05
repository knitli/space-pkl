//! Space Pklr Library
//!
//! This library provides the core functionality for the Space Pklr tool,
//! including configuration conversion, schema generation, and Pkl tooling integration.

pub mod cli_app;
pub mod commands;
pub mod config_processor;
pub mod error;
pub mod pkl_tooling;

// Re-export commonly used types for convenience
pub use config_processor::{ConfigFormat, MoonConfigType};
pub use error::{CliError, Result};
pub use pkl_tooling::{CompatibilityReport, PklCli, PklSource};
