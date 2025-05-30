//! `types.rs`
//! Type definitions for Pkl schema generation.
//!
//! (c) 2025 Stash AI Inc (knitli)
//!   - Created by Adam Poulemanos ([@bashandbone](https://github.com/bashandbone))
//! Licensed under the [Plain MIT License](https://plainlicense.org/licenses/permissive/mit/)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a Pkl module definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PklModule {
    /// Module name
    pub name: String,

    /// Module documentation
    pub documentation: Option<String>,

    /// Module imports
    pub imports: Vec<PklImport>,

    /// Module exports
    pub exports: Vec<PklExport>,

    /// Type definitions in this module
    pub types: Vec<PklType>,

    /// Module-level properties
    pub properties: Vec<PklProperty>,
}

/// Represents a Pkl import statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PklImport {
    /// Import path
    pub path: String,

    /// Import alias
    pub alias: Option<String>,

    /// Whether this is a glob import
    pub glob: bool,
}

/// Represents a Pkl export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PklExport {
    /// Export name
    pub name: String,

    /// Export type
    pub type_name: String,
}

/// Represents a Pkl type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PklType {
    /// Type name
    pub name: String,

    /// Type documentation
    pub documentation: Option<String>,

    /// Type kind (class, typealias, etc.)
    pub kind: PklTypeKind,

    /// Type properties/fields
    pub properties: Vec<PklProperty>,

    /// Whether this type is abstract
    pub abstract_type: bool,

    /// Base types this extends
    pub extends: Vec<String>,

    /// For enums/typealias, the union values
    pub enum_values: Option<String>,
}

/// Kinds of Pkl types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PklTypeKind {
    /// A class type
    Class,
    /// A type alias
    TypeAlias,
    /// An enum/union type
    Union,
    /// A module type
    Module,
}

/// Represents a property in a Pkl type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PklProperty {
    /// Property name
    pub name: String,

    /// Property type
    pub type_name: String,

    /// Property documentation
    pub documentation: Option<String>,

    /// Whether the property is optional
    pub optional: bool,

    /// Default value
    pub default: Option<String>,

    /// Validation constraints
    pub constraints: Vec<PklConstraint>,

    /// Example values
    pub examples: Vec<String>,
}

/// Represents a validation constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PklConstraint {
    /// Constraint type
    pub kind: PklConstraintKind,

    /// Constraint value
    pub value: String,

    /// Constraint message
    pub message: Option<String>,
}

/// Types of validation constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PklConstraintKind {
    /// Minimum value constraint
    Min,
    /// Maximum value constraint
    Max,
    /// Length constraint
    Length,
    /// Pattern/regex constraint
    Pattern,
    /// Custom validation
    Custom,
}

/// Context for template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    /// The module being rendered
    pub module: PklModule,

    /// Generator configuration
    pub config: crate::config::GeneratorConfig,

    /// Additional template variables
    pub variables: HashMap<String, serde_json::Value>,
}
