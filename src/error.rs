//! Error Handling Module for Moon Config CLI
//!
//! This module provides comprehensive error handling using miette for rich, user-friendly
//! error reporting.

use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

/// Main CLI error type with rich diagnostics
#[derive(Error, Diagnostic, Debug)]
pub enum CliError {
    /// File not found error with helpful guidance
    #[error("File not found: {path}")]
    #[diagnostic(
        code(cli::file_not_found),
        help("Please check that the file path exists and is readable")
    )]
    FileNotFound { path: PathBuf },

    /// Output file already exists without --force flag
    #[error("Output file already exists: {path}")]
    #[diagnostic(
        code(cli::file_exists),
        help("Use --force flag to overwrite existing files, or choose a different output path")
    )]
    OutputFileExists { path: PathBuf },

    /// Unsupported format error with available options
    #[error("Unsupported format: {format}")]
    #[diagnostic(
        code(cli::unsupported_format),
        help("Available formats: {}", .available.join(", "))
    )]
    UnsupportedFormat {
        format: String,
        available: Vec<&'static str>,
    },

    /// Configuration rendering error
    #[error("Failed to render {config_type} configuration to {format:?} format")]
    #[diagnostic(
        code(cli::render_error),
        help("Check that the configuration is valid and the target format is supported")
    )]
    RenderError {
        config_type: String,
        format: crate::config_processor::ConfigFormat,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Proto tool manager not found
    #[error("Proto tool manager not found")]
    #[diagnostic(
        code(cli::proto_not_found),
        help("Install proto from https://moonrepo.dev/proto or use direct Pkl installation")
    )]
    ProtoNotFound { help: Option<String> },

    /// Pkl installation failed
    #[error("Failed to install Pkl CLI: {reason}")]
    #[diagnostic(
        code(cli::pkl_install_failed),
        help("{}", .help.as_deref().unwrap_or("Check network connectivity and try again, or install Pkl manually"))
    )]
    PklInstallFailed {
        reason: String,
        help: Option<String>,
    },

    /// Pkl execution failed
    #[error("Pkl CLI execution failed: {command}")]
    #[diagnostic(
        code(cli::pkl_execution_failed),
        help("{}", .help.as_deref().unwrap_or("Check Pkl syntax and file paths"))
    )]
    PklExecutionFailed {
        command: String,
        stderr: String,
        help: Option<String>,
    },

    /// Network/HTTP error during downloads
    #[error("Network error during download: {0}")]
    #[diagnostic(
        code(cli::network_error),
        help("Check internet connectivity and try again")
    )]
    NetworkError(String),

    /// I/O error with context
    #[error("I/O error: {context}")]
    #[diagnostic(code(cli::io_error), help("Check file permissions and disk space"))]
    IoError {
        context: String,
        #[source]
        source: std::io::Error,
    },

    /// Permission denied error
    #[error("Permission denied: {path}")]
    #[diagnostic(
        code(cli::permission_denied),
        help("Check file/directory permissions or run with appropriate privileges")
    )]
    PermissionDenied { path: PathBuf },

    /// Configuration validation error
    #[error("Configuration validation failed")]
    #[diagnostic(
        code(cli::validation_error),
        help("Check configuration syntax and required fields")
    )]
    ValidationError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Generic error wrapper
    #[error("Error: {0}")]
    #[diagnostic(code(cli::generic_error))]
    Generic(String),
}

/// Result type alias for CLI operations
pub type Result<T> = miette::Result<T, CliError>;

/// Helper function to create I/O errors with context
pub fn io_error_with_context<T>(
    context: impl Into<String>,
) -> impl FnOnce(std::io::Error) -> CliError {
    move |source| CliError::IoError {
        context: context.into(),
        source,
    }
}

/// Helper function to create render errors
pub fn render_error(
    config_type: impl Into<String>,
    format: crate::config_processor::ConfigFormat,
    source: impl std::error::Error + Send + Sync + 'static,
) -> CliError {
    CliError::RenderError {
        config_type: config_type.into(),
        format,
        source: Box::new(source),
    }
}

/// Helper function to create Pkl execution errors with context
pub fn pkl_execution_error(
    command: impl Into<String>,
    stderr: impl Into<String>,
    help: Option<String>,
) -> CliError {
    CliError::PklExecutionFailed {
        command: command.into(),
        stderr: stderr.into(),
        help,
    }
}

/// Helper function to create validation errors
pub fn validation_error(source: impl std::error::Error + Send + Sync + 'static) -> CliError {
    CliError::ValidationError {
        source: Box::new(source),
    }
}

/// Helper function to check if a path exists and is readable
pub fn ensure_file_exists(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        return Err(CliError::FileNotFound { path: path.clone() });
    }
    Ok(())
}

/// Helper function to check if output file can be written
pub fn ensure_output_writable(path: &PathBuf, force: bool) -> Result<()> {
    if path.exists() && !force {
        return Err(CliError::OutputFileExists { path: path.clone() });
    }
    Ok(())
}

/// Convert from reqwest::Error
impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> Self {
        CliError::NetworkError(err.to_string())
    }
}

/// Convert from anyhow::Error
impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> Self {
        CliError::Generic(err.to_string())
    }
}
