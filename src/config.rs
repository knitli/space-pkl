//! Configuration Module
//!
//! This module provides configuration types that control how Pkl schemas are generated
//! from Moon configuration types. It offers fine-grained control over output formatting,
//! content inclusion, and template customization.
//!
//! # Overview
//!
//! The configuration system is built around two main types:
//! - [`GeneratorConfig`] - Controls overall schema generation behavior
//! - [`TemplateConfig`] - Manages template engine settings and customization
//!
//! # Features
//!
//! - **Content Control**: Choose what to include (comments, examples, validation)
//! - **Output Customization**: Custom headers, footers, and module organization
//! - **Type Mapping**: Override default Rust-to-Pkl type mappings
//! - **Template System**: Customize Pkl output format and structure
//! - **File Organization**: Single-file or multi-file output strategies
//!
//! # Quick Start
//!
//! ```rust
//! use space_pkl::prelude::*;
//! use std::path::PathBuf;
//!
//! // Use default configuration
//! let config = GeneratorConfig::default();
//!
//! // Or customize as needed
//! let custom_config = GeneratorConfig {
//!     include_examples: true,
//!     include_validation: true,
//!     output_dir: PathBuf::from("./my-schemas"),
//!     module_name: "my_project".to_string(),
//!     ..Default::default()
//! };
//! ```
//!
//! # Configuration Patterns
//!
//! ## Development Mode
//! ```rust
//! # use space_pkl::prelude::*;
//! # use std::path::PathBuf;
//! let dev_config = GeneratorConfig {
//!     include_comments: true,
//!     include_examples: true,
//!     include_validation: true,
//!     include_deprecated: true, // Include everything for development
//!     split_types: true,        // Separate files for easier editing
//!     ..Default::default()
//! };
//! ```
//!
//! ## Production Mode
//! ```rust
//! # use space_pkl::prelude::*;
//! # use std::path::PathBuf;
//! let prod_config = GeneratorConfig {
//!     include_comments: false,
//!     include_examples: false,
//!     include_deprecated: false, // Clean output for production
//!     split_types: false,        // Single file for deployment
//!     header: Some("// Production Pkl Schema\n".to_string()),
//!     ..Default::default()
//! };
//! ```
//!
//! (c) 2025 Stash AI Inc (knitli)
//!   - Created by Adam Poulemanos ([@bashandbone](https://github.com/bashandbone)) for Stash AI Inc.
//! Licensed under the [Plain MIT License](https://plainlicense.org/licenses/permissive/mit/)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for Pkl schema generation.
///
/// `GeneratorConfig` controls all aspects of how Moon configuration types are
/// converted to Pkl schemas. It provides comprehensive control over content
/// inclusion, output formatting, and file organization.
///
/// # Content Control
///
/// Fine-tune what gets included in the generated schemas:
/// - **Comments**: Include documentation from Rust source code
/// - **Examples**: Generate realistic usage examples for properties
/// - **Validation**: Include Pkl constraints for type validation
/// - **Deprecated Items**: Choose whether to include deprecated fields/types
///
/// # Output Customization
///
/// Control the structure and formatting of generated files:
/// - **Headers/Footers**: Add custom content to all generated files
/// - **Module Names**: Customize the Pkl module naming scheme
/// - **File Organization**: Single-file vs. multi-file output strategies
/// - **Type Mappings**: Override default Rust-to-Pkl type conversions
///
/// # Examples
///
/// ## Minimal Configuration
/// ```rust
/// use space_pkl::prelude::*;
///
/// let minimal = GeneratorConfig {
///     include_comments: false,
///     include_examples: false,
///     include_validation: false,
///     ..Default::default()
/// };
/// ```
///
/// ## Documentation-Rich Configuration
/// ```rust
/// use space_pkl::prelude::*;
/// use std::path::PathBuf;
///
/// let rich_docs = GeneratorConfig {
///     include_comments: true,
///     include_examples: true,
///     include_validation: true,
///     include_deprecated: true,
///     header: Some("/// Comprehensive Moon Configuration Schema\n".to_string()),
///     output_dir: PathBuf::from("./docs/schemas"),
///     ..Default::default()
/// };
/// ```
///
/// ## Custom Type Mapping
/// ```rust
/// use space_pkl::prelude::*;
/// use std::collections::HashMap;
///
/// let mut type_mappings = HashMap::new();
/// type_mappings.insert("String".to_string(), "Text".to_string());
/// type_mappings.insert("Int".to_string(), "Number".to_string());
///
/// let custom_types = GeneratorConfig {
///     type_mappings,
///     ..Default::default()
/// };
/// ```
///
/// # Default Behavior
///
/// The default configuration provides a balanced setup suitable for most use cases:
/// - Includes comments, examples, and validation
/// - Excludes deprecated items to keep output clean
/// - Uses multi-file output for better organization
/// - Outputs to `./pkl-schemas` directory
/// - Uses `"moon"` as the default module name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    /// Whether to include comments in generated schemas.
    ///
    /// When `true`, documentation comments from Rust source code are converted
    /// to Pkl doc comments, providing rich inline documentation.
    ///
    /// # Example Output Difference
    /// ```pkl
    /// // With include_comments: true
    /// /// The workspace display name
    /// name: String?
    ///
    /// // With include_comments: false
    /// name: String?
    /// ```
    pub include_comments: bool,

    /// Whether to include example values in property documentation.
    ///
    /// When `true`, generates realistic example values for each property
    /// to help users understand expected formats and values.
    ///
    /// # Example Output
    /// ```pkl
    /// /// The server port number
    /// ///
    /// /// Examples: 8080, 3000, 8000
    /// port: Int?
    /// ```
    pub include_examples: bool,

    /// Whether to include validation constraints in type definitions.
    ///
    /// When `true`, converts Rust validation rules (like `#[validate]`)
    /// into Pkl constraint syntax for runtime validation.
    ///
    /// # Example Output
    /// ```pkl
    /// /// Username with validation
    /// username: String(length >= 3)(length <= 20)(matches(Regex(#"^[a-zA-Z0-9_]+$"#)))
    /// ```
    pub include_validation: bool,

    /// Whether to include deprecated fields and types.
    ///
    /// When `false` (default), deprecated items are filtered out to keep
    /// schemas clean. Set to `true` for comprehensive documentation or
    /// migration scenarios.
    pub include_deprecated: bool,

    /// Custom header content prepended to all generated files.
    ///
    /// Useful for adding copyright notices, generation timestamps,
    /// or custom documentation headers.
    ///
    /// # Example
    /// ```rust
    /// # use space_pkl::prelude::*;
    /// let config = GeneratorConfig {
    ///     header: Some("// Generated Pkl Schema\n// DO NOT EDIT MANUALLY\n".to_string()),
    ///     ..Default::default()
    /// };
    /// ```
    pub header: Option<String>,

    /// Custom footer content appended to all generated files.
    ///
    /// Can be used for additional documentation, validation rules,
    /// or custom Pkl code that should appear in every schema file.
    pub footer: Option<String>,

    /// Output directory for generated schema files.
    ///
    /// All Pkl files will be written to this directory. The directory
    /// will be created if it doesn't exist.
    ///
    /// # Default
    /// `./pkl-schemas`
    pub output_dir: PathBuf,

    /// Module name used in Pkl module declarations.
    ///
    /// This name appears in Pkl `module` declarations and affects
    /// how schemas can be imported by other Pkl files.
    ///
    /// # Default
    /// `"moon"`
    pub module_name: String,

    /// Whether to generate individual files for each schema type.
    ///
    /// - `true` (default): Creates separate `.pkl` files for each configuration type
    /// - `false`: Combines all schemas into a single module file
    ///
    /// # Multi-file Output (split_types: true)
    /// ```text
    /// pkl-schemas/
    /// ├── workspace.pkl
    /// ├── project.pkl
    /// ├── template.pkl
    /// ├── toolchain.pkl
    /// ├── tasks.pkl
    /// └── mod.pkl
    /// ```
    ///
    /// # Single-file Output (split_types: false)
    /// ```text
    /// pkl-schemas/
    /// └── moon.pkl
    /// ```
    pub split_types: bool,

    /// Custom mappings from Rust types to Pkl type names.
    ///
    /// Allows overriding the default type conversion rules. Keys are the
    /// default Pkl type names, values are the replacement names.
    ///
    /// # Example
    /// ```rust
    /// # use space_pkl::prelude::*;
    /// use std::collections::HashMap;
    ///
    /// let mut mappings = HashMap::new();
    /// mappings.insert("String".to_string(), "Text".to_string());
    /// mappings.insert("Boolean".to_string(), "Bool".to_string());
    ///
    /// let config = GeneratorConfig {
    ///     type_mappings: mappings,
    ///     ..Default::default()
    /// };
    /// // Now String fields become Text fields in Pkl output
    /// ```
    pub type_mappings: HashMap<String, String>,

    /// Template engine configuration for customizing Pkl output format.
    ///
    /// Controls how the generated Pkl types are formatted and rendered
    /// into the final schema files.
    pub template: TemplateConfig,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            include_comments: true,
            include_examples: true,
            include_validation: true,
            include_deprecated: false,
            header: Some(default_header()),
            footer: None,
            output_dir: PathBuf::from("./pkl-schemas"),
            module_name: "moon".to_string(),
            split_types: true,
            type_mappings: default_type_mappings(),
            template: TemplateConfig::default(),
        }
    }
}

/// Template configuration for customizing Pkl schema output format.
///
/// `TemplateConfig` controls the template engine that formats and renders
/// Pkl type definitions into final schema files. It provides extensive
/// customization options for output formatting, file organization, and
/// template-driven code generation.
///
/// # Template System Overview
///
/// The template system uses a configurable engine to transform internal
/// type representations into Pkl syntax. You can:
/// - Override default templates for specific type patterns
/// - Provide custom formatting for complex types
/// - Control file naming and organization
/// - Add custom template files alongside generated schemas
///
/// # Template Types
///
/// - **Type Templates**: Control how individual types are rendered
/// - **Module Templates**: Control overall file structure and organization
/// - **Property Templates**: Control how object properties are formatted
/// - **Comment Templates**: Control documentation formatting
///
/// # Examples
///
/// ## Basic Template Configuration
/// ```rust
/// use space_pkl::prelude::*;
/// use std::path::PathBuf;
///
/// let basic_template = TemplateConfig {
///     generate_templates: true,
///     template_extension: "pkl".to_string(),
///     ..Default::default()
/// };
/// ```
///
/// ## Custom Template Directory
/// ```rust
/// use space_pkl::prelude::*;
/// use std::path::PathBuf;
///
/// let custom_dir = TemplateConfig {
///     template_dir: Some(PathBuf::from("./my-templates")),
///     generate_templates: true,
///     template_extension: "pkl.template".to_string(),
///     ..Default::default()
/// };
/// ```
///
/// ## Custom Type Templates
/// ```rust
/// use space_pkl::prelude::*;
/// use std::collections::HashMap;
///
/// let mut custom_templates = HashMap::new();
/// custom_templates.insert("String".to_string(), "Text".to_string());
/// custom_templates.insert("ConfigObject".to_string(), "Configuration".to_string());
///
/// let custom_types = TemplateConfig {
///     custom_templates,
///     generate_templates: true,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Base directory for template files.
    ///
    /// When specified, the template engine will look for custom template
    /// files in this directory. Templates found here override the built-in
    /// default templates.
    ///
    /// # Template File Structure
    /// ```text
    /// template_dir/
    /// ├── type.pkl.hbs        # Type definition template
    /// ├── module.pkl.hbs      # Module structure template
    /// ├── property.pkl.hbs    # Property definition template
    /// └── comment.pkl.hbs     # Comment formatting template
    /// ```
    ///
    /// # Example
    /// ```rust
    /// # use space_pkl::prelude::*;
    /// use std::path::PathBuf;
    ///
    /// let config = TemplateConfig {
    ///     template_dir: Some(PathBuf::from("./my-pkl-templates")),
    ///     ..Default::default()
    /// };
    /// ```
    pub template_dir: Option<PathBuf>,

    /// Custom template mappings for specific type names.
    ///
    /// Maps type names to custom template identifiers or template content.
    /// This allows fine-grained control over how specific types are rendered
    /// in the generated Pkl schemas.
    ///
    /// # Key-Value Format
    /// - **Key**: The type name to match (e.g., "String", "ConfigObject")
    /// - **Value**: The template identifier or custom template content
    ///
    /// # Example
    /// ```rust
    /// # use space_pkl::prelude::*;
    /// use std::collections::HashMap;
    ///
    /// let mut templates = HashMap::new();
    /// templates.insert("DatabaseConfig".to_string(), "db_template".to_string());
    /// templates.insert("ServerConfig".to_string(), "server_template".to_string());
    ///
    /// let config = TemplateConfig {
    ///     custom_templates: templates,
    ///     ..Default::default()
    /// };
    /// ```
    pub custom_templates: HashMap<String, String>,

    /// Whether to generate template files alongside Pkl schemas.
    ///
    /// When `true`, creates template files that can be used as starting
    /// points for configuration authoring. These templates contain
    /// realistic example values and comprehensive documentation.
    ///
    /// # Generated Template Structure
    /// ```text
    /// output_dir/
    /// ├── workspace.pkl          # Schema definitions
    /// ├── workspace.template.pkl # Usage template with examples
    /// ├── project.pkl
    /// ├── project.template.pkl
    /// └── ...
    /// ```
    ///
    /// # Default
    /// `true` - Templates are generated by default
    pub generate_templates: bool,

    /// File extension for generated template files.
    ///
    /// Controls the extension used for both schema files and template files.
    /// The actual filename format depends on the file type:
    /// - Schema files: `{type}.{extension}`
    /// - Template files: `{type}.template.{extension}`
    ///
    /// # Common Extensions
    /// - `"pkl"` (default) - Standard Pkl files
    /// - `"pkl.template"` - Template-specific extension
    /// - `"config"` - Generic configuration files
    ///
    /// # Example
    /// ```rust
    /// # use space_pkl::prelude::*;
    /// let config = TemplateConfig {
    ///     template_extension: "config.pkl".to_string(),
    ///     ..Default::default()
    /// };
    /// // Generates: workspace.config.pkl, project.config.pkl, etc.
    /// ```
    pub template_extension: String,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            template_dir: None,
            custom_templates: HashMap::new(),
            generate_templates: true,
            template_extension: "pkl".to_string(),
        }
    }
}

/// Schema type to generate.
///
/// `SchemaType` represents the different Moon configuration schemas that
/// can be generated. Each type corresponds to a specific configuration
/// domain in the Moon workspace management system.
///
/// # Schema Types
///
/// Each schema type generates Pkl definitions for a specific configuration area:
/// - **Workspace**: Top-level workspace settings and global configuration
/// - **Project**: Individual project configuration and metadata
/// - **Template**: Project template definitions and scaffolding rules
/// - **Toolchain**: Development tool configuration and version management
/// - **Tasks**: Task definitions, dependencies, and execution configuration
/// - **All**: Meta-type representing all schema types (used for bulk operations)
///
/// # File Organization
///
/// Each schema type maps to specific output files:
/// ```text
/// pkl-schemas/
/// ├── workspace.pkl   # Workspace configuration
/// ├── project.pkl     # Project configuration
/// ├── template.pkl    # Template definitions
/// ├── toolchain.pkl   # Toolchain management
/// ├── tasks.pkl       # Task configuration
/// └── mod.pkl         # Module index (when split_types: true)
/// ```
///
/// # Usage Examples
///
/// ## Generate Single Schema Type
/// ```rust
/// use space_pkl::prelude::*;
///
/// let generator = SchemaGenerator::new(GeneratorConfig::default());
///
/// // Generate only workspace schema
/// generator.generate_workspace_schema()?;
///
/// // Or using the enum directly
/// let schema_type = SchemaType::Workspace;
/// println!("Generating: {}", schema_type.filename());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Generate Multiple Schema Types
/// ```rust
/// use space_pkl::prelude::*;
///
/// let generator = SchemaGenerator::new(GeneratorConfig::default());
///
/// // Generate all schemas at once
/// generator.generate_all()?;
///
/// // Or generate specific types
/// let types = vec![SchemaType::Workspace, SchemaType::Project];
/// for schema_type in types {
///     println!("Processing: {}", schema_type.module_name());
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Schema Content
///
/// Each schema type includes relevant Pkl type definitions:
///
/// ## Workspace Schema
/// - Global workspace settings
/// - Project discovery rules
/// - Default configurations
/// - Workspace-level constraints
///
/// ## Project Schema
/// - Project metadata and identification
/// - Build and test configurations
/// - Dependency management
/// - Project-specific overrides
///
/// ## Template Schema
/// - Template metadata and variables
/// - File generation rules
/// - Substitution patterns
/// - Template validation constraints
///
/// ## Toolchain Schema
/// - Tool version specifications
/// - Installation and configuration
/// - Platform-specific settings
/// - Tool integration options
///
/// ## Tasks Schema
/// - Task definitions and metadata
/// - Execution environment setup
/// - Input/output specifications
/// - Dependency relationships
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SchemaType {
    /// Workspace configuration schema.
    ///
    /// Generates Pkl types for top-level workspace configuration including
    /// global settings, project discovery rules, and workspace-wide defaults.
    ///
    /// # Generated Types
    /// - `WorkspaceConfig` - Main workspace configuration
    /// - `ProjectsConfig` - Project discovery and organization
    /// - `GeneratorConfig` - Code generation settings
    /// - `ConstraintsConfig` - Workspace-level validation rules
    ///
    /// # Output File
    /// `workspace.pkl`
    Workspace,

    /// Project configuration schema.
    ///
    /// Generates Pkl types for individual project configuration including
    /// metadata, build settings, dependencies, and project-specific overrides.
    ///
    /// # Generated Types
    /// - `ProjectConfig` - Main project configuration
    /// - `DependencyConfig` - Dependency specifications
    /// - `LanguageConfig` - Language-specific settings
    /// - `PlatformConfig` - Platform and environment configuration
    ///
    /// # Output File
    /// `project.pkl`
    Project,

    /// Template configuration schema.
    ///
    /// Generates Pkl types for project template definitions including
    /// template metadata, variable substitution, and file generation rules.
    ///
    /// # Generated Types
    /// - `TemplateConfig` - Main template configuration
    /// - `VariableConfig` - Template variable definitions
    /// - `FileConfig` - File generation and transformation rules
    /// - `SubstitutionConfig` - Content replacement patterns
    ///
    /// # Output File
    /// `template.pkl`
    Template,

    /// Toolchain configuration schema.
    ///
    /// Generates Pkl types for development toolchain management including
    /// tool versions, installation settings, and integration configuration.
    ///
    /// # Generated Types
    /// - `ToolchainConfig` - Main toolchain configuration
    /// - `ToolConfig` - Individual tool specifications
    /// - `VersionConfig` - Version constraints and resolution
    /// - `PlatformToolConfig` - Platform-specific tool settings
    ///
    /// # Output File
    /// `toolchain.pkl`
    Toolchain,

    /// Tasks configuration schema.
    ///
    /// Generates Pkl types for task definitions including execution settings,
    /// dependencies, input/output specifications, and scheduling configuration.
    ///
    /// # Generated Types
    /// - `TasksConfig` - Main tasks configuration
    /// - `TaskConfig` - Individual task definitions
    /// - `DependencyConfig` - Task dependency relationships
    /// - `ExecutionConfig` - Task execution environment and settings
    ///
    /// # Output File
    /// `tasks.pkl`
    Tasks,

    /// All schema types.
    ///
    /// Meta-type representing all available schema types. Used for bulk
    /// operations like generating all schemas at once. Cannot be used
    /// directly for file operations.
    ///
    /// # Usage
    /// ```rust
    /// use space_pkl::prelude::*;
    ///
    /// let generator = SchemaGenerator::new(GeneratorConfig::default());
    /// generator.generate_all()?; // Generates all schema types
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Note
    /// Calling `filename()` or `module_name()` on `SchemaType::All` will panic
    /// since it doesn't represent a single file or module.
    All,
}

impl SchemaType {
    /// Get the output filename for this schema type.
    ///
    /// Returns the standard filename used when generating Pkl schema files
    /// for this configuration type. The filename follows the pattern
    /// `{type}.pkl` where `{type}` is the lowercase schema type name.
    ///
    /// # Returns
    /// A static string containing the filename with `.pkl` extension.
    ///
    /// # Examples
    /// ```rust
    /// use space_pkl::prelude::*;
    ///
    /// assert_eq!(SchemaType::Workspace.filename(), "workspace.pkl");
    /// assert_eq!(SchemaType::Project.filename(), "project.pkl");
    /// assert_eq!(SchemaType::Template.filename(), "template.pkl");
    /// assert_eq!(SchemaType::Toolchain.filename(), "toolchain.pkl");
    /// assert_eq!(SchemaType::Tasks.filename(), "tasks.pkl");
    /// ```
    ///
    /// # Panics
    /// Panics if called on `SchemaType::All` since it doesn't represent
    /// a single file.
    ///
    /// ```should_panic
    /// # use space_pkl::prelude::*;
    /// SchemaType::All.filename(); // This will panic
    /// ```
    pub fn filename(&self) -> &'static str {
        match self {
            Self::Workspace => "workspace.pkl",
            Self::Project => "project.pkl",
            Self::Template => "template.pkl",
            Self::Toolchain => "toolchain.pkl",
            Self::Tasks => "tasks.pkl",
            Self::All => unreachable!("All is not a single file"),
        }
    }

    /// Get the Pkl module name for this schema type.
    ///
    /// Returns the standard module name used in Pkl `module` declarations
    /// for this configuration type. The module name follows PascalCase
    /// convention for Pkl modules.
    ///
    /// # Returns
    /// A static string containing the PascalCase module name.
    ///
    /// # Examples
    /// ```rust
    /// use space_pkl::prelude::*;
    ///
    /// assert_eq!(SchemaType::Workspace.module_name(), "Workspace");
    /// assert_eq!(SchemaType::Project.module_name(), "Project");
    /// assert_eq!(SchemaType::Template.module_name(), "Template");
    /// assert_eq!(SchemaType::Toolchain.module_name(), "Toolchain");
    /// assert_eq!(SchemaType::Tasks.module_name(), "Tasks");
    /// ```
    ///
    /// # Pkl Module Declaration
    /// The module name is used in generated Pkl files like this:
    /// ```pkl
    /// module Workspace
    ///
    /// // Schema definitions...
    /// ```
    ///
    /// # Panics
    /// Panics if called on `SchemaType::All` since it doesn't represent
    /// a single module.
    ///
    /// ```should_panic
    /// # use space_pkl::prelude::*;
    /// SchemaType::All.module_name(); // This will panic
    /// ```
    pub fn module_name(&self) -> &'static str {
        match self {
            Self::Workspace => "Workspace",
            Self::Project => "Project",
            Self::Template => "Template",
            Self::Toolchain => "Toolchain",
            Self::Tasks => "Tasks",
            Self::All => unreachable!("All is not a single module"),
        }
    }
}

/// Generate the default header content for Pkl schema files.
///
/// Creates a comprehensive header that includes version information,
/// project links, and usage documentation. The header provides context
/// about the generated schemas and their relationship to Moon and Pkl.
///
/// # Header Content
/// - Project and tool identification
/// - Version information from Cargo.toml
/// - Links to relevant GitHub repositories
/// - Brief description of schema purpose
/// - License and attribution information
///
/// # Example Output
/// ```text
/// //! Moon Configuration Schema for Pkl
/// //!
/// //! Generated by space-pkl v0.1.0
/// //! Source: https://github.com/knitli/space-pkl
/// //! Moon: https://github.com/moonrepo/moon
/// //!
/// //! This schema provides type-safe configuration authoring for Moon workspace management.
/// ```
///
/// # Usage
/// This function is automatically called by `GeneratorConfig::default()`
/// to provide a consistent header across all generated files. You can
/// override this by setting a custom header in your configuration.
///
/// # Returns
/// A formatted string containing the complete header with trailing newline.
fn default_header() -> String {
    format!(
        r#"//! Moon Configuration Schema for Pkl
//!
//! Generated by space-pkl v{}
//! Source: https://github.com/knitli/space-pkl
//! Moon: https://github.com/moonrepo/moon
//!
//! This schema provides type-safe configuration authoring for Moon workspace management.

"#,
        env!("CARGO_PKG_VERSION")
    )
}

/// Generate default Rust-to-Pkl type mappings.
///
/// Creates a comprehensive mapping table that translates common Rust types
/// to their equivalent Pkl type representations. This ensures consistent
/// and idiomatic Pkl type usage across all generated schemas.
///
/// # Mapping Categories
///
/// ## Primitive Types
/// - `String` → `String` - Text values
/// - `bool` → `Boolean` - True/false values
/// - `i32`, `i64` → `Int` - Signed integers
/// - `u32`, `u64` → `UInt` - Unsigned integers
/// - `f32`, `f64` → `Float` - Floating-point numbers
///
/// ## Collection Types
/// - `Vec<T>` → `Listing<T>` - Ordered sequences
/// - `HashMap<K,V>` → `Mapping<K,V>` - Key-value maps
/// - `BTreeMap<K,V>` → `Mapping<K,V>` - Ordered key-value maps
/// - `Option<T>` → `T?` - Optional values (represented as empty string for special handling)
///
/// # Customization
/// These default mappings can be overridden by providing custom mappings
/// in `GeneratorConfig.type_mappings`. Custom mappings take precedence
/// over defaults.
///
/// # Example Usage
/// ```rust
/// use space_pkl::prelude::*;
/// use std::collections::HashMap;
///
/// let mappings = default_type_mappings();
/// assert_eq!(mappings.get("String"), Some(&"String".to_string()));
/// assert_eq!(mappings.get("Vec"), Some(&"Listing".to_string()));
/// ```
///
/// # Returns
/// A `HashMap` containing default type mappings from Rust type names
/// to Pkl type names.
pub fn default_type_mappings() -> HashMap<String, String> {
    let mut mappings = HashMap::new();

    // Common Rust -> Pkl type mappings
    mappings.insert("String".to_string(), "String".to_string());
    mappings.insert("bool".to_string(), "Boolean".to_string());
    mappings.insert("i32".to_string(), "Int".to_string());
    mappings.insert("i64".to_string(), "Int".to_string());
    mappings.insert("u32".to_string(), "UInt".to_string());
    mappings.insert("u64".to_string(), "UInt".to_string());
    mappings.insert("f32".to_string(), "Float".to_string());
    mappings.insert("f64".to_string(), "Float".to_string());
    mappings.insert("Vec".to_string(), "Listing".to_string());
    mappings.insert("HashMap".to_string(), "Mapping".to_string());
    mappings.insert("BTreeMap".to_string(), "Mapping".to_string());
    mappings.insert("Option".to_string(), "".to_string()); // Optional in Pkl

    mappings
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_generator_config_default() {
        let config = GeneratorConfig::default();

        assert!(config.include_comments);
        assert!(config.include_examples);
        assert!(config.include_validation);
        assert!(!config.include_deprecated);
        assert!(config.header.is_some());
        assert!(config.footer.is_none());
        assert_eq!(config.output_dir, PathBuf::from("./pkl-schemas"));
        assert_eq!(config.module_name, "moon");
        assert!(config.split_types);
        assert!(!config.type_mappings.is_empty());
        assert_eq!(config.template.template_extension, "pkl");
    }

    #[test]
    fn test_template_config_default() {
        let config = TemplateConfig::default();

        assert!(config.template_dir.is_none());
        assert!(config.custom_templates.is_empty());
        assert!(config.generate_templates);
        assert_eq!(config.template_extension, "pkl");
    }

    #[test]
    fn test_schema_type_filename() {
        assert_eq!(SchemaType::Workspace.filename(), "workspace.pkl");
        assert_eq!(SchemaType::Project.filename(), "project.pkl");
        assert_eq!(SchemaType::Template.filename(), "template.pkl");
        assert_eq!(SchemaType::Toolchain.filename(), "toolchain.pkl");
        assert_eq!(SchemaType::Tasks.filename(), "tasks.pkl");
    }

    #[test]
    fn test_schema_type_module_name() {
        assert_eq!(SchemaType::Workspace.module_name(), "Workspace");
        assert_eq!(SchemaType::Project.module_name(), "Project");
        assert_eq!(SchemaType::Template.module_name(), "Template");
        assert_eq!(SchemaType::Toolchain.module_name(), "Toolchain");
        assert_eq!(SchemaType::Tasks.module_name(), "Tasks");
    }

    #[test]
    #[should_panic(expected = "All is not a single file")]
    fn test_schema_type_all_filename_panics() {
        SchemaType::All.filename();
    }

    #[test]
    #[should_panic(expected = "All is not a single module")]
    fn test_schema_type_all_module_name_panics() {
        SchemaType::All.module_name();
    }

    #[test]
    fn test_default_header_contains_version() {
        let header = default_header();
        assert!(header.contains(env!("CARGO_PKG_VERSION")));
        assert!(header.contains("Moon Configuration Schema"));
        assert!(header.contains("space-pkl"));
        assert!(header.contains("https://github.com/knitli/space-pkl"));
    }

    #[test]
    fn test_default_type_mappings() {
        let mappings = default_type_mappings();

        // Check common type mappings
        assert_eq!(mappings.get("String"), Some(&"String".to_string()));
        assert_eq!(mappings.get("bool"), Some(&"Boolean".to_string()));
        assert_eq!(mappings.get("i32"), Some(&"Int".to_string()));
        assert_eq!(mappings.get("i64"), Some(&"Int".to_string()));
        assert_eq!(mappings.get("u32"), Some(&"UInt".to_string()));
        assert_eq!(mappings.get("u64"), Some(&"UInt".to_string()));
        assert_eq!(mappings.get("f32"), Some(&"Float".to_string()));
        assert_eq!(mappings.get("f64"), Some(&"Float".to_string()));
        assert_eq!(mappings.get("Vec"), Some(&"Listing".to_string()));
        assert_eq!(mappings.get("HashMap"), Some(&"Mapping".to_string()));
        assert_eq!(mappings.get("BTreeMap"), Some(&"Mapping".to_string()));
        assert_eq!(mappings.get("Option"), Some(&"".to_string()));

        // Ensure we have a reasonable number of mappings
        assert!(mappings.len() >= 10);
    }

    #[test]
    fn test_generator_config_custom_values() {
        let custom_mappings = {
            let mut map = HashMap::new();
            map.insert("CustomType".to_string(), "PklType".to_string());
            map
        };

        let config = GeneratorConfig {
            include_comments: false,
            include_examples: false,
            include_validation: false,
            include_deprecated: true,
            header: Some("Custom header".to_string()),
            footer: Some("Custom footer".to_string()),
            output_dir: PathBuf::from("/custom/path"),
            module_name: "custom".to_string(),
            split_types: false,
            type_mappings: custom_mappings.clone(),
            template: TemplateConfig {
                template_dir: Some(PathBuf::from("/templates")),
                custom_templates: HashMap::new(),
                generate_templates: false,
                template_extension: "template".to_string(),
            },
        };

        assert!(!config.include_comments);
        assert!(!config.include_examples);
        assert!(!config.include_validation);
        assert!(config.include_deprecated);
        assert_eq!(config.header, Some("Custom header".to_string()));
        assert_eq!(config.footer, Some("Custom footer".to_string()));
        assert_eq!(config.output_dir, PathBuf::from("/custom/path"));
        assert_eq!(config.module_name, "custom");
        assert!(!config.split_types);
        assert_eq!(config.type_mappings, custom_mappings);
        assert_eq!(config.template.template_extension, "template");
    }

    #[test]
    fn test_template_config_custom_values() {
        let mut custom_templates = HashMap::new();
        custom_templates.insert("TypeA".to_string(), "template_a".to_string());

        let config = TemplateConfig {
            template_dir: Some(PathBuf::from("/custom/templates")),
            custom_templates: custom_templates.clone(),
            generate_templates: false,
            template_extension: "custom".to_string(),
        };

        assert_eq!(
            config.template_dir,
            Some(PathBuf::from("/custom/templates"))
        );
        assert_eq!(config.custom_templates, custom_templates);
        assert!(!config.generate_templates);
        assert_eq!(config.template_extension, "custom");
    }

    #[test]
    fn test_schema_type_serialization() {
        // Test that SchemaType can be serialized/deserialized
        let original = SchemaType::Workspace;
        let json = serde_json::to_string(&original).expect("Failed to serialize");
        let deserialized: SchemaType = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(original.filename(), deserialized.filename());
        assert_eq!(original.module_name(), deserialized.module_name());
    }

    #[test]
    fn test_generator_config_serialization() {
        let config = GeneratorConfig::default();
        let json = serde_json::to_string(&config).expect("Failed to serialize GeneratorConfig");
        let deserialized: GeneratorConfig =
            serde_json::from_str(&json).expect("Failed to deserialize GeneratorConfig");

        assert_eq!(config.include_comments, deserialized.include_comments);
        assert_eq!(config.module_name, deserialized.module_name);
        assert_eq!(config.split_types, deserialized.split_types);
    }

    #[test]
    fn test_template_config_serialization() {
        let config = TemplateConfig::default();
        let json = serde_json::to_string(&config).expect("Failed to serialize TemplateConfig");
        let deserialized: TemplateConfig =
            serde_json::from_str(&json).expect("Failed to deserialize TemplateConfig");

        assert_eq!(config.generate_templates, deserialized.generate_templates);
        assert_eq!(config.template_extension, deserialized.template_extension);
    }

    // Additional comprehensive tests

    #[test]
    fn test_generator_config_builder_pattern() {
        let config = GeneratorConfig {
            include_comments: true,
            include_examples: false,
            include_validation: true,
            include_deprecated: false,
            header: None,
            footer: Some("Custom footer".to_string()),
            output_dir: PathBuf::from("./test-output"),
            module_name: "test_module".to_string(),
            split_types: false,
            type_mappings: HashMap::new(),
            template: TemplateConfig::default(),
        };

        assert!(config.include_comments);
        assert!(!config.include_examples);
        assert!(config.include_validation);
        assert!(!config.include_deprecated);
        assert!(config.header.is_none());
        assert_eq!(config.footer, Some("Custom footer".to_string()));
        assert_eq!(config.output_dir, PathBuf::from("./test-output"));
        assert_eq!(config.module_name, "test_module");
        assert!(!config.split_types);
    }

    #[test]
    fn test_template_config_with_complex_mappings() {
        let mut custom_templates = HashMap::new();
        custom_templates.insert("ClassTemplate".to_string(), "class_custom.hbs".to_string());
        custom_templates.insert(
            "ModuleTemplate".to_string(),
            "module_custom.hbs".to_string(),
        );
        custom_templates.insert(
            "PropertyTemplate".to_string(),
            "property_custom.hbs".to_string(),
        );

        let config = TemplateConfig {
            template_dir: Some(PathBuf::from("/usr/local/templates")),
            custom_templates: custom_templates.clone(),
            generate_templates: true,
            template_extension: "handlebars".to_string(),
        };

        assert_eq!(config.custom_templates.len(), 3);
        assert_eq!(
            config.custom_templates.get("ClassTemplate"),
            Some(&"class_custom.hbs".to_string())
        );
        assert_eq!(config.template_extension, "handlebars");
        assert!(config.generate_templates);
    }

    #[test]
    fn test_empty_type_mappings() {
        let config = GeneratorConfig {
            type_mappings: HashMap::new(),
            ..GeneratorConfig::default()
        };

        assert!(config.type_mappings.is_empty());
    }

    #[test]
    fn test_large_type_mappings() {
        let mut large_mappings = HashMap::new();
        for i in 0..1000 {
            large_mappings.insert(format!("Type{}", i), format!("PklType{}", i));
        }

        let config = GeneratorConfig {
            type_mappings: large_mappings.clone(),
            ..GeneratorConfig::default()
        };

        assert_eq!(config.type_mappings.len(), 1000);
        assert_eq!(
            config.type_mappings.get("Type500"),
            Some(&"PklType500".to_string())
        );
    }

    #[test]
    fn test_schema_type_all_variants() {
        let all_types = vec![
            SchemaType::Workspace,
            SchemaType::Project,
            SchemaType::Template,
            SchemaType::Toolchain,
            SchemaType::Tasks,
            SchemaType::All,
        ];

        assert_eq!(all_types.len(), 6);

        // Test that only non-All types can produce filenames
        for schema_type in &all_types[..5] {
            assert!(!schema_type.filename().is_empty());
            assert!(!schema_type.module_name().is_empty());
        }
    }

    #[test]
    fn test_config_with_absolute_paths() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let absolute_path = temp_dir.path().to_path_buf();

        let config = GeneratorConfig {
            output_dir: absolute_path.clone(),
            template: TemplateConfig {
                template_dir: Some(absolute_path.clone()),
                ..TemplateConfig::default()
            },
            ..GeneratorConfig::default()
        };

        assert!(config.output_dir.is_absolute());
        assert!(config.template.template_dir.as_ref().unwrap().is_absolute());
    }

    #[test]
    fn test_config_with_relative_paths() {
        let config = GeneratorConfig {
            output_dir: PathBuf::from("./relative/path"),
            template: TemplateConfig {
                template_dir: Some(PathBuf::from("../templates")),
                ..TemplateConfig::default()
            },
            ..GeneratorConfig::default()
        };

        assert!(!config.output_dir.is_absolute());
        assert!(!config.template.template_dir.as_ref().unwrap().is_absolute());
    }

    #[test]
    fn test_config_edge_cases() {
        // Test with empty strings
        let config = GeneratorConfig {
            module_name: String::new(),
            header: Some(String::new()),
            footer: Some(String::new()),
            ..GeneratorConfig::default()
        };

        assert!(config.module_name.is_empty());
        assert_eq!(config.header, Some(String::new()));
        assert_eq!(config.footer, Some(String::new()));

        // Test with very long strings
        let long_string = "a".repeat(10000);
        let config2 = GeneratorConfig {
            module_name: long_string.clone(),
            header: Some(long_string.clone()),
            ..GeneratorConfig::default()
        };

        assert_eq!(config2.module_name.len(), 10000);
        assert_eq!(config2.header.as_ref().unwrap().len(), 10000);
    }

    #[test]
    fn test_template_config_edge_cases() {
        // Test with empty extension
        let config = TemplateConfig {
            template_extension: String::new(),
            ..TemplateConfig::default()
        };

        assert!(config.template_extension.is_empty());

        // Test with extension with dot
        let config2 = TemplateConfig {
            template_extension: ".pkl".to_string(),
            ..TemplateConfig::default()
        };

        assert_eq!(config2.template_extension, ".pkl");
    }

    #[test]
    fn test_configuration_clone() {
        let original = GeneratorConfig::default();
        let cloned = original.clone();

        assert_eq!(original.module_name, cloned.module_name);
        assert_eq!(original.include_comments, cloned.include_comments);
        assert_eq!(original.output_dir, cloned.output_dir);
    }

    #[test]
    fn test_configuration_debug() {
        let config = GeneratorConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("GeneratorConfig"));
        assert!(debug_str.contains("include_comments"));
        assert!(debug_str.contains("moon"));
    }

    #[test]
    fn test_default_header_structure() {
        let header = default_header();

        // Check for key components
        assert!(header.contains("Moon Configuration Schema"));
        assert!(header.contains("Generated by space-pkl"));
        assert!(header.contains("github.com/knitli/space-pkl"));
        assert!(header.contains("github.com/moonrepo/moon"));
        assert!(header.contains("type-safe configuration"));
        assert!(header.starts_with("//!"));
        assert!(header.ends_with("\n"));
    }

    #[test]
    fn test_type_mappings_completeness() {
        let mappings = default_type_mappings();

        // Check that all basic Rust types are mapped
        let expected_rust_types = ["String", "bool", "i32", "i64", "u32", "u64", "f32", "f64"];
        for rust_type in &expected_rust_types {
            assert!(
                mappings.contains_key(*rust_type),
                "Missing mapping for {}",
                rust_type
            );
        }

        // Check that all basic collection types are mapped
        let expected_collections = ["Vec", "HashMap", "BTreeMap", "Option"];
        for collection in &expected_collections {
            assert!(
                mappings.contains_key(*collection),
                "Missing mapping for {}",
                collection
            );
        }
    }

    #[test]
    fn test_schema_type_equality() {
        assert_eq!(SchemaType::Workspace, SchemaType::Workspace);
        assert_ne!(SchemaType::Workspace, SchemaType::Project);

        let mut set = std::collections::HashSet::new();
        set.insert(SchemaType::Workspace);
        set.insert(SchemaType::Project);
        set.insert(SchemaType::Workspace); // duplicate

        assert_eq!(set.len(), 2); // Should only have 2 unique items
    }

    #[test]
    fn test_config_partial_updates() {
        let mut config = GeneratorConfig::default();

        // Test individual field updates
        config.include_comments = false;
        assert!(!config.include_comments);
        assert!(config.include_examples); // Others should remain unchanged

        config.module_name = "updated".to_string();
        assert_eq!(config.module_name, "updated");

        config
            .type_mappings
            .insert("NewType".to_string(), "PklNewType".to_string());
        assert!(config.type_mappings.contains_key("NewType"));
    }
}
