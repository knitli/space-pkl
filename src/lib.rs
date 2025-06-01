#![warn(missing_docs)]
#![doc(html_root_url = "https://docs.rs/space-pkl")]
//!
//! # space-pkl
//!
//! Space-pkl is a **library** and **CLI tool** that provides Pkl schema generation, and template generation for Moon workspace configurations.
//!
//! This documentation focuses on using space-pkl as a library.
//! For CLI usage, refer to the README at [space-pkl](https://github.com/knitli/space-pkl).
//!
//! It provides utilities to generate Pkl schemas and templates from Moon's
//! configuration types, enabling type-safe configuration authoring in Pkl format.
//! It bridges the gap between Moon's Rust-based configuration system and Pkl's
//! configuration language, allowing teams to author configurations with full
//! type safety and IDE support.
//!
//! ## Features
//!
//! - **Schema Generation**: Convert Moon configuration types to Pkl schemas
//! - **Template Creation**: Generate Pkl templates with examples and documentation
//! - **Type Safety**: Ensure configuration consistency across environments
//! - **CLI Tool**: Command-line interface for easy schema generation
//! - **Programmatic API**: Integrate schema generation into build processes
//! - **Customization**: Configurable templates, type mappings, and output formats
//!
//! ## Quick Start
//!
//! ### Basic Usage
//!
//! ```rust
//! use space_pkl::prelude::*;
//!
//! # fn main() -> space_pkl::Result<()> {
//! // Generate all schemas with default settings
//! let generator = SchemaGenerator::new(GeneratorConfig::default());
//! generator.generate_all()?;
//!
//! // Or generate a specific schema
//! let workspace_schema = generator.generate_workspace_schema()?;
//! println!("Generated schema:\n{}", workspace_schema);
//! # Ok(())
//! # }
//! ```
//!
//! ### Custom Configuration
//!
//! ```rust
//! use space_pkl::prelude::*;
//! use std::path::PathBuf;
//!
//! # fn main() -> space_pkl::Result<()> {
//! let config = GeneratorConfig {
//!     include_comments: true,
//!     include_examples: true,
//!     include_validation: true,
//!     output_dir: PathBuf::from("./pkl-schemas"),
//!     module_name: "myproject".to_string(),
//!     header: Some("// Custom Moon Configuration Schema\n".to_string()),
//!     ..Default::default()
//! };
//!
//! let generator = SchemaGenerator::new(config);
//! generator.generate_all()?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Convenience Functions
//!
//! ```rust
//! # fn main() -> space_pkl::Result<()> {
//! // Use convenience functions for quick schema generation
//! let workspace = space_pkl::generate_workspace_schema()?;
//! let project = space_pkl::generate_project_schema()?;
//! let toolchain = space_pkl::generate_toolchain_schema()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Generated Output
//!
//! The crate generates Pkl schemas that look like this:
//!
//! ```pkl
//! /// Moon workspace configuration schema
//! module Workspace
//!
//! /// Workspace configuration for Moon projects
//! class WorkspaceConfig {
//!   /// Project discovery and organization settings
//!   projects: ProjectsConfig?
//!
//!   /// Version control system configuration
//!   vcs: VcsConfig?
//!
//!   /// File hashing and caching configuration
//!   hasher: HasherConfig?
//! }
//! ```
//!
//! (c) 2025 Stash AI Inc (knitli)
//!   - Created by Adam Poulemanos ([@bashandbone](https://github.com/bashandbone))
//! Licensed under the [Plain MIT License](https://plainlicense.org/licenses/permissive/mit/)

pub mod config;
pub mod generator;
pub mod templates;
pub mod types;
pub mod utils;

// Re-export main types for easier access
pub use config::{GeneratorConfig, TemplateConfig};
pub use generator::SchemaGenerator;

// Re-export convenience functions
pub use generator::{
    generate_project_schema, generate_tasks_schema, generate_template_schema,
    generate_toolchain_schema, generate_workspace_schema,
};

/// Prelude module containing the most commonly used types and functions.
///
/// This module re-exports the essential items needed for most use cases,
/// allowing users to import everything they need with a single `use` statement.
///
/// # Examples
///
/// ```rust
/// use space_pkl::prelude::*;
///
/// # fn main() -> space_pkl::Result<()> {
/// let config = GeneratorConfig::default();
/// let generator = SchemaGenerator::new(config);
/// let schema = generator.generate_workspace_schema()?;
/// # Ok(())
/// # }
/// ```
pub mod prelude {
    //! Prelude module for convenient imports.
    //!
    //! This module re-exports the most commonly used types and functions
    //! from space-pkl, providing a convenient way to import everything
    //! needed for typical usage.

    /// Configuration types for schema generation.
    pub use crate::config::*;

    /// Main schema generator and convenience functions.
    pub use crate::generator::*;

    /// Pkl type definitions for advanced usage.
    pub use crate::types::*;
}

/// Result type used throughout the crate.
///
/// This is a type alias for [`miette::Result`] with the default error type,
/// providing rich error messages with context and suggestions.
///
/// # Examples
///
/// ```rust
/// use space_pkl::Result;
///
/// fn generate_schema() -> Result<String> {
///     // Function that might fail with a detailed error message
///     Ok("schema content".to_string())
/// }
/// ```
pub type Result<T> = miette::Result<T>;

/// Error type for space-pkl operations.
///
/// This is a type alias for [`miette::Report`], which provides rich error
/// messages with source code context, suggestions, and help text.
///
/// # Examples
///
/// ```rust
/// use space_pkl::Error;
/// use miette::{IntoDiagnostic, WrapErr};
///
/// fn example() -> Result<(), Error> {
///     std::fs::read_to_string("nonexistent.file")
///         .into_diagnostic()
///         .wrap_err("Failed to read configuration file")?;
///     Ok(())
/// }
/// ```
pub type Error = miette::Report;
