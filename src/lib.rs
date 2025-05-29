//! # space-pkl
//! 
//! Pkl schema and template generation for Moon workspace configurations.
//! 
//! This crate provides utilities to generate Pkl schemas and templates from Moon's
//! configuration types, enabling type-safe configuration authoring in Pkl format.
//! 
//! ## Features
//! 
//! - Generate Pkl schemas from Moon configuration types
//! - Create Pkl templates with examples and documentation
//! - CLI tool for easy schema generation
//! - Programmatic API for integration
//! 
//! ## Quick Start
//! 
//! ```rust
//! use space_pkl::generator::SchemaGenerator;
//! use space_pkl::config::GeneratorConfig;
//! 
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = GeneratorConfig::default();
//! let generator = SchemaGenerator::new(config);
//! 
//! // Generate workspace schema
//! let workspace_schema = generator.generate_workspace_schema()?;
//! println!("{}", workspace_schema);
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod generator;
pub mod templates;
pub mod types;
pub mod utils;

pub use config::*;
pub use generator::*;

/// Re-export common types for convenience
pub mod prelude {
    pub use crate::config::*;
    pub use crate::generator::*;
    pub use crate::types::*;
}

/// Result type used throughout the crate
pub type Result<T> = miette::Result<T>;

/// Error type for space-pkl operations
pub type Error = miette::Report;
