//! Schema Generator Module
//!
//! This module provides the core functionality for generating Pkl configuration schemas
//! from Moon configuration types. It converts Rust configuration structures into
//! strongly-typed Pkl modules with validation, examples, and comprehensive documentation.
//!
//! # Features
//!
//! - **Automated Schema Generation**: Converts Moon configuration types to Pkl schemas
//! - **Type-Safe Conversion**: Preserves type safety and validation rules
//! - **Comprehensive Examples**: Generates realistic examples for all schema types
//! - **Validation Support**: Includes constraints, patterns, and enum validations
//! - **Flexible Output**: Supports both single-file and multi-file generation
//!
//! # Core Components
//!
//! - [`SchemaGenerator`] - Main generator for converting configurations to Pkl
//! - Type conversion methods for different schema patterns (structs, enums, unions)
//! - Template integration for customizable output formatting
//! - Constraint extraction for validation rules
//!
//! # Usage Examples
//!
//! ## Basic Schema Generation
//!
//! ```rust
//! use space_pkl::prelude::*;
//!
//! # fn main() -> space_pkl::Result<()> {
//! let generator = SchemaGenerator::new(GeneratorConfig::default());
//!
//! // Generate all schemas at once
//! generator.generate_all()?;
//!
//! // Or generate individual schemas
//! let workspace_schema = generator.generate_workspace_schema()?;
//! let project_schema = generator.generate_project_schema()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Configuration
//!
//! ```rust
//! use space_pkl::prelude::*;
//! use std::path::PathBuf;
//!
//! # fn main() -> space_pkl::Result<()> {
//! let config = GeneratorConfig {
//!     include_examples: true,
//!     include_validation: true,
//!     output_dir: PathBuf::from("./custom-schemas"),
//!     module_name: "my_project".to_string(),
//!     header: Some("// Custom Pkl Schema\n".to_string()),
//!     ..Default::default()
//! };
//!
//! let generator = SchemaGenerator::new(config);
//! generator.generate_all()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Generated Pkl Structure
//!
//! The generator creates Pkl modules with:
//!
//! - **Type Definitions**: Classes and type aliases for all configuration types
//! - **Property Constraints**: Validation rules using Pkl's constraint system
//! - **Documentation**: Comprehensive doc comments from Rust source
//! - **Examples**: Usage examples for each property
//!
//! (c) 2025 Stash AI Inc (knitli)
//!   - Created by Adam Poulemanos ([@bashandbone](https://github.com/bashandbone))
//! Licensed under the [Plain MIT License](https://plainlicense.org/licenses/permissive/mit/)

use crate::config::{GeneratorConfig, SchemaType as ConfigSchemaType};
use crate::templates::TemplateEngine;
use crate::types::*;
use crate::Result;
use miette::{IntoDiagnostic, WrapErr};
use moon_config::*;
use schematic::schema::SchemaGenerator as SchematicGenerator;
use schematic::Config;
use schematic_types::{Schema, SchemaField, SchemaType};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

lazy_static::lazy_static! {
    static ref TOP_LEVEL_CONFIG_NAMES: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("WorkspaceConfig");
        set.insert("ProjectConfig");
        set.insert("TemplateConfig");
        set.insert("ToolchainConfig");
        set.insert("InheritedTasksConfig"); // Corresponds to TasksConfig
        set
    };
}

/// Core schema generator for Moon configurations.
///
/// The `SchemaGenerator` is the main entry point for converting Moon configuration
/// types into Pkl schemas. It handles the complete workflow from Rust type introspection
/// to Pkl module generation, including template rendering and file output.
///
/// # Architecture
///
/// The generator uses a multi-stage conversion process:
/// 1. **Type Analysis**: Uses `schematic` to introspect Rust configuration types
/// 2. **Schema Conversion**: Converts schematic schemas to Pkl type definitions
/// 3. **Template Rendering**: Applies Pkl template formatting with custom rules
/// 4. **File Generation**: Outputs formatted Pkl files with proper structure
///
/// # Thread Safety
///
/// `SchemaGenerator` is `Send` and `Sync`, making it safe to use across threads.
/// Each instance maintains its own configuration and template engine state.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use space_pkl::prelude::*;
///
/// # fn main() -> space_pkl::Result<()> {
/// let generator = SchemaGenerator::new(GeneratorConfig::default());
///
/// // Generate all Moon configuration schemas
/// generator.generate_all()?;
/// # Ok(())
/// # }
/// ```
///
/// ## Individual Schema Generation
///
/// ```rust
/// use space_pkl::prelude::*;
///
/// # fn main() -> space_pkl::Result<()> {
/// let generator = SchemaGenerator::new(GeneratorConfig::default());
///
/// let workspace_pkl = generator.generate_workspace_schema()?;
/// let project_pkl = generator.generate_project_schema()?;
/// let toolchain_pkl = generator.generate_toolchain_schema()?;
///
/// // Use the generated Pkl strings as needed
/// println!("Generated {} chars of workspace schema", workspace_pkl.len());
/// # Ok(())
/// # }
/// ```
///
/// ## Custom Configuration
///
/// ```rust
/// use space_pkl::prelude::*;
/// use std::path::PathBuf;
///
/// # fn main() -> space_pkl::Result<()> {
/// let config = GeneratorConfig {
///     include_examples: true,
///     include_validation: true,
///     include_deprecated: false, // Skip deprecated fields
///     output_dir: PathBuf::from("./my-schemas"),
///     module_name: "my_project".to_string(),
///     header: Some("// Generated Moon Pkl Schema\n".to_string()),
///     ..Default::default()
/// };
///
/// let generator = SchemaGenerator::new(config);
/// generator.generate_all()?;
/// # Ok(())
/// # }
/// ```
pub struct SchemaGenerator {
    config: GeneratorConfig,
    template_engine: TemplateEngine,
}

impl SchemaGenerator {
    /// Creates a new schema generator with the specified configuration.
    ///
    /// The generator initializes with a template engine configured for the given
    /// settings. All subsequent schema generation operations will use this configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration controlling schema generation behavior
    ///
    /// # Examples
    ///
    /// ```rust
    /// use space_pkl::prelude::*;
    /// use std::path::PathBuf;
    ///
    /// // Default configuration
    /// let generator = SchemaGenerator::new(GeneratorConfig::default());
    ///
    /// // Custom configuration
    /// let custom_config = GeneratorConfig {
    ///     include_examples: true,
    ///     include_validation: true,
    ///     output_dir: PathBuf::from("./schemas"),
    ///     module_name: "myapp".to_string(),
    ///     ..Default::default()
    /// };
    /// let custom_generator = SchemaGenerator::new(custom_config);
    /// ```
    pub fn new(config: GeneratorConfig) -> Self {
        let template_engine = TemplateEngine::new(&config);
        Self {
            config,
            template_engine,
        }
    }

    /// Generates all Moon configuration schemas and writes them to files.
    ///
    /// This is the primary method for batch generation. It creates all supported
    /// Moon configuration schemas (workspace, project, template, toolchain, and tasks)
    /// and writes them to the configured output directory.
    ///
    /// # File Structure
    ///
    /// When `split_types` is enabled (default), generates:
    /// - `Workspace.pkl` - Workspace configuration schema
    /// - `Project.pkl` - Project configuration schema
    /// - `Template.pkl` - Template configuration schema
    /// - `Toolchain.pkl` - Toolchain configuration schema
    /// - `Tasks.pkl` - Task configuration schema
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Output directory cannot be created
    /// - Schema generation fails for any configuration type
    /// - File writing permissions are insufficient
    /// - Template rendering encounters issues
    ///
    /// # Examples
    ///
    /// ```rust
    /// use space_pkl::prelude::*;
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> space_pkl::Result<()> {
    /// let config = GeneratorConfig {
    ///     output_dir: PathBuf::from("./pkl-schemas"),
    ///     include_examples: true,
    ///     ..Default::default()
    /// };
    ///
    /// let generator = SchemaGenerator::new(config);
    /// generator.generate_all()?;
    ///
    /// // Files are now available in ./pkl-schemas/
    /// assert!(PathBuf::from("./pkl-schemas/Workspace.pkl").exists());
    /// assert!(PathBuf::from("./pkl-schemas/Project.pkl").exists());
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_all(&self) -> Result<()> {
        info!("Generating all Moon configuration schemas");

        fs::create_dir_all(&self.config.output_dir)
            .into_diagnostic()
            .wrap_err("Failed to create output directory")?;

        // Generate individual schemas
        self.generate_workspace_schema_file()?;
        self.generate_project_schema_file()?;
        self.generate_template_schema_file()?;
        self.generate_toolchain_schema_file()?;
        self.generate_tasks_schema_file()?;

        info!(
            "Successfully generated all schemas in: {}",
            self.config.output_dir.display()
        );
        Ok(())
    }

    /// Generates a Pkl schema for Moon workspace configuration.
    ///
    /// Creates a comprehensive Pkl module for `WorkspaceConfig` including all
    /// workspace-level settings, project discovery rules, and tool configurations.
    ///
    /// # Generated Schema Includes
    ///
    /// - Workspace metadata (name, version, description)
    /// - Project discovery patterns and configuration
    /// - Global tool settings and version constraints
    /// - VCS and CI/CD integration settings
    /// - Dependency management configuration
    ///
    /// # Returns
    ///
    /// A `String` containing the complete Pkl module definition ready for use.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use space_pkl::prelude::*;
    ///
    /// # fn main() -> space_pkl::Result<()> {
    /// let generator = SchemaGenerator::new(GeneratorConfig::default());
    /// let workspace_pkl = generator.generate_workspace_schema()?;
    ///
    /// // The generated Pkl can be written to a file or used directly
    /// println!("Generated workspace schema:\n{}", workspace_pkl);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Sample Output
    ///
    /// ```pkl
    /// /// Moon workspace configuration schema
    /// module workspace
    ///
    /// /// Workspace configuration for Moon
    /// class WorkspaceConfig {
    ///   /// Workspace display name
    ///   name: String?
    ///
    ///   /// Project discovery patterns
    ///   projects: Listing<String>?
    ///
    ///   /// Global tool versions
    ///   version_constraint: String?
    /// }
    /// ```
    pub fn generate_workspace_schema(&self) -> Result<String> {
        debug!("Generating workspace schema");
        self.generate_schema_for_type::<WorkspaceConfig>("Workspace")
    }

    /// Generates a Pkl schema for Moon project configuration.
    ///
    /// Creates a Pkl module for `ProjectConfig` covering project-specific settings,
    /// build configuration, dependency management, and task definitions.
    ///
    /// # Generated Schema Includes
    ///
    /// - Project metadata and identification
    /// - Language and platform configuration
    /// - Build and output settings
    /// - Task and target definitions
    /// - Dependency and workspace relationships
    ///
    /// # Examples
    ///
    /// ```rust
    /// use space_pkl::prelude::*;
    ///
    /// # fn main() -> space_pkl::Result<()> {
    /// let generator = SchemaGenerator::new(GeneratorConfig::default());
    /// let project_pkl = generator.generate_project_schema()?;
    ///
    /// // The generated Pkl schema is ready to use
    /// assert!(!project_pkl.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_project_schema(&self) -> Result<String> {
        debug!("Generating project schema");
        self.generate_schema_for_type::<ProjectConfig>("Project")
    }

    /// Generates a Pkl schema for Moon template configuration.
    ///
    /// Creates a Pkl module for `TemplateConfig` used in project scaffolding
    /// and code generation workflows.
    ///
    /// # Returns
    ///
    /// A `String` containing the Pkl template schema with variable definitions,
    /// file patterns, and template composition rules.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use space_pkl::prelude::*;
    ///
    /// # fn main() -> space_pkl::Result<()> {
    /// let generator = SchemaGenerator::new(GeneratorConfig::default());
    /// let template_pkl = generator.generate_template_schema()?;
    /// println!("Template schema: {} characters", template_pkl.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_template_schema(&self) -> Result<String> {
        debug!("Generating template schema");
        self.generate_schema_for_type::<TemplateConfig>("Template")
    }

    /// Generates a Pkl schema for Moon toolchain configuration.
    ///
    /// Creates a Pkl module for `ToolchainConfig` defining tool versions,
    /// installation preferences, and environment setup.
    ///
    /// # Generated Schema Includes
    ///
    /// - Tool version specifications and constraints
    /// - Installation and download configuration
    /// - Environment variable setup
    /// - Tool-specific settings and preferences
    ///
    /// # Examples
    ///
    /// ```rust
    /// use space_pkl::prelude::*;
    ///
    /// # fn main() -> space_pkl::Result<()> {
    /// let generator = SchemaGenerator::new(GeneratorConfig::default());
    /// let toolchain_pkl = generator.generate_toolchain_schema()?;
    ///
    /// // Check if specific tools are configured
    /// assert!(toolchain_pkl.contains("ToolchainConfig"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_toolchain_schema(&self) -> Result<String> {
        debug!("Generating toolchain schema");
        self.generate_schema_for_type::<ToolchainConfig>("Toolchain")
    }

    /// Generates a Pkl schema for Moon task configuration.
    ///
    /// Creates a Pkl module for `InheritedTasksConfig` covering shared task
    /// definitions, inheritance patterns, and task execution settings.
    ///
    /// # Generated Schema Includes
    ///
    /// - Task definitions and command specifications
    /// - Input/output file patterns and dependencies
    /// - Environment variable configuration
    /// - Task inheritance and merging rules
    /// - Platform-specific task variations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use space_pkl::prelude::*;
    ///
    /// # fn main() -> space_pkl::Result<()> {
    /// let generator = SchemaGenerator::new(GeneratorConfig::default());
    /// let tasks_pkl = generator.generate_tasks_schema()?;
    ///
    /// // Verify task schema generation
    /// assert!(tasks_pkl.contains("InheritedTasksConfig"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_tasks_schema(&self) -> Result<String> {
        debug!("Generating tasks schema");
        self.generate_schema_for_type::<InheritedTasksConfig>("Tasks")
    }

    /// Internal method to generate a Pkl schema for a specific configuration type.
    ///
    /// This is the core conversion method that:
    /// 1. Uses `schematic` to introspect the Rust configuration type `T`
    /// 2. Converts the generated schemas to Pkl type representations
    /// 3. Renders the final Pkl module using the template engine
    ///
    /// # Type Parameters
    ///
    /// * `T` - A configuration type implementing `schematic::Config`
    ///
    /// # Arguments
    ///
    /// * `type_name` - Human-readable name for the schema (e.g., "Workspace", "Project")
    ///
    /// # Returns
    ///
    /// The complete Pkl module as a formatted string.
    ///
    /// # Implementation Details
    ///
    /// This method handles complex type conversions including:
    /// - Struct types with nested fields and validation
    /// - Enum types with string/numeric literals
    /// - Union types with nullable patterns
    /// - Array and mapping types with generic parameters
    /// - Reference types and cross-schema dependencies
    fn generate_schema_for_type<T: Config>(&self, type_name: &str) -> Result<String> {
        let mut generator = SchematicGenerator::default();
        generator.add::<T>();

        // Get the schemas from the generator
        let schema_map = generator.schemas;

        // Convert schematic schema to our Pkl representation
        let pkl_module = self.convert_schemas_to_pkl(schema_map, type_name)?;

        // Render using template engine
        self.template_engine
            .render_module(&pkl_module, &self.config)
    }

    /// Write schema to file
    fn generate_workspace_schema_file(&self) -> Result<()> {
        let schema = self.generate_workspace_schema()?;
        let file_path = self
            .config
            .output_dir
            .join(ConfigSchemaType::Workspace.filename());
        self.write_schema_file(&file_path, &schema, "Workspace")
    }

    fn generate_project_schema_file(&self) -> Result<()> {
        let schema = self.generate_project_schema()?;
        let file_path = self
            .config
            .output_dir
            .join(ConfigSchemaType::Project.filename());
        self.write_schema_file(&file_path, &schema, "Project")
    }

    fn generate_template_schema_file(&self) -> Result<()> {
        let schema = self.generate_template_schema()?;
        let file_path = self
            .config
            .output_dir
            .join(ConfigSchemaType::Template.filename());
        self.write_schema_file(&file_path, &schema, "Template")
    }

    fn generate_toolchain_schema_file(&self) -> Result<()> {
        let schema = self.generate_toolchain_schema()?;
        let file_path = self
            .config
            .output_dir
            .join(ConfigSchemaType::Toolchain.filename());
        self.write_schema_file(&file_path, &schema, "Toolchain")
    }

    fn generate_tasks_schema_file(&self) -> Result<()> {
        let schema = self.generate_tasks_schema()?;
        let file_path = self
            .config
            .output_dir
            .join(ConfigSchemaType::Tasks.filename());
        self.write_schema_file(&file_path, &schema, "Tasks")
    }

    fn write_schema_file(&self, path: &Path, content: &str, schema_name: &str) -> Result<()> {
        fs::write(path, content)
            .into_diagnostic()
            .wrap_err_with(|| {
                format!(
                    "Failed to write {} schema to {}",
                    schema_name,
                    path.display()
                )
            })?;

        info!("Generated {} schema: {}", schema_name, path.display());
        Ok(())
    }

    /// Converts a collection of schematic schemas into a complete Pkl module.
    ///
    /// This method orchestrates the conversion from raw schema data to a structured
    /// Pkl module representation. It handles type dependency resolution,
    /// and module organization.
    ///
    /// # Arguments
    ///
    /// * `schemas` - Map of schema names to their definitions from schematic introspection
    /// * `type_name` - Primary type name used for module naming
    ///
    /// # Returns
    ///
    /// A `PklModule` containing all converted types, and metadata.
    ///
    /// # Processing Steps
    ///
    /// 1. **Type Conversion**: Each schema is converted to a `PklType` with proper kind classification
    /// 2. **Export Resolution**: Main configuration types are automatically exported
    /// 3. **Documentation Generation**: Module-level documentation is created
    /// 4. **Dependency Analysis**: Cross-references between types are preserved
    ///
    /// # Type Classification
    ///
    /// - `Struct` → Pkl `Class` with properties and constraints
    /// - `Enum` → Pkl `TypeAlias` with union of literal values
    /// - `Union` → Pkl `TypeAlias` with type alternatives
    /// - `Reference` → Pkl `Class` referencing external types
    fn convert_schemas_to_pkl(
      &self,
      schemas: indexmap::IndexMap<String, Schema>,
      type_name: &str,
  ) -> Result<PklModule> {
      let mut module = PklModule {
          name: type_name.to_string(),
          documentation: Some(format!(
              "Moon {} configuration schema",
              type_name.to_lowercase()
          )),
          imports: vec![],
          types: vec![],
          properties: vec![],
      };

      let mut processed_types: HashSet<String> = HashSet::new();
      let mut collected_pkl_types: Vec<PklType> = Vec::new();

      // First pass: Identify top-level configs and process them directly
      // Also, collect all schemas that need recursive processing
      let mut schemas_to_process: indexmap::IndexMap<String, Schema> = indexmap::IndexMap::new();

      for (name, schema) in schemas {
          if TOP_LEVEL_CONFIG_NAMES.contains(name.as_str()) {
              if let SchemaType::Struct(struct_type) = &schema.ty {
                  debug!("Processing top-level config '{}' as module properties", name);
                  for (field_name, field) in &struct_type.fields {
                      let property = self.convert_field_to_property(field_name, field)?;

                      if property.deprecated.is_some() && !self.config.include_deprecated {
                          debug!(
                              "Skipping deprecated property '{}' in top-level config '{}'",
                              field_name, name
                          );
                          continue;
                      }

                      module.properties.push(property);
                  }
                  processed_types.insert(name); // Mark as processed to avoid re-processing
                  continue;
              } else {
                  warn!(
                      "Top-level config '{}' is not a struct, falling back to class generation.",
                      name
                  );
              }
          }
          schemas_to_process.insert(name, schema);
      }

      // Second pass: Recursively process all remaining schemas
      for (_name, schema) in schemas_to_process {
          self.process_schema_recursively(&schema, &mut processed_types, &mut collected_pkl_types)?;
      }

      // Filter out deprecated types if include_deprecated is false
      if !self.config.include_deprecated {
          module.types = collected_pkl_types
              .into_iter()
              .filter(|t| t.deprecated.is_none())
              .collect();
      } else {
          module.types = collected_pkl_types;
      }
      Ok(module)
    }


    /// Converts a single schematic schema into a Pkl type definition.
    ///
    /// This is the core type conversion method that handles the mapping from
    /// schematic's type system to Pkl's type system. It preserves semantic
    /// meaning while adapting to Pkl's syntax and capabilities.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schematic schema to convert
    /// * `name` - The name to use for the generated Pkl type
    ///
    /// # Returns
    ///
    /// A `PklType` representing the schema in Pkl's type system.
    ///
    /// # Type Conversion Rules
    ///
    /// ## Struct Types
    /// - Converted to Pkl `Class` types
    /// - Each field becomes a class property with appropriate type and constraints
    /// - Optional fields are marked with `?` nullable syntax
    /// - Deprecated fields are optionally included based on configuration
    ///
    /// ## Enum Types
    /// - String enums → `TypeAlias` with union of string literals (`"value1" | "value2"`)
    /// - Numeric enums → `TypeAlias` with union of numeric literals (`1 | 2 | 3`)
    /// - Empty enums → `Class` with documentation noting the empty state
    ///
    /// ## Union Types
    /// - Multiple alternatives → `TypeAlias` with union syntax (`Type1 | Type2`)
    /// - Nullable patterns → Simplified to `Type?` when possible
    /// - Complex nullable → `(Type1 | Type2)?` for multi-type nullables
    ///
    /// ## Reference Types
    /// - Preserved as `Class` types referencing external definitions
    /// - Properties resolved from the referenced schema when available
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Struct with validation becomes Pkl class
    /// struct Config {
    ///     name: String,      // → name: String
    ///     port: Option<u16>, // → port: Int?
    /// }
    ///
    /// // Enum becomes type alias
    /// enum Mode { Dev, Prod } // → "dev" | "prod"
    ///
    /// // Union becomes type alias
    /// String | i32            // → String | Int
    /// Option<String>          // → String?
    /// ```
    fn convert_schema_to_pkl_type(&self, schema: &Schema, name: &str) -> Result<PklType> {
        debug!("Converting schema '{}' of type: {:?}", name, schema.ty);
        let mut pkl_type = PklType {
            name: name.to_string(),
            documentation: schema.description.clone(),
            kind: PklTypeKind::Class,
            properties: vec![],
            abstract_type: false,
            open: true,
            extends: vec![],
            enum_values: None,
            deprecated: schema.deprecated.clone(),
        };

        let result = match &schema.ty {
            SchemaType::Struct(struct_type) => {
                for (field_name, field) in &struct_type.fields {
                    let property = self.convert_field_to_property(field_name, field)?;
                    pkl_type.properties.push(property);
                }
                debug!(
                    "Created struct class '{}' with {} properties",
                    name,
                    pkl_type.properties.len()
                );
                Ok(pkl_type)
            }
            SchemaType::Enum(enum_type) => {
                if !enum_type.values.is_empty() {
                    pkl_type.kind = PklTypeKind::TypeAlias;
                    let enum_values: Vec<String> = enum_type
                        .values
                        .iter()
                        .map(|v| match v {
                            schematic_types::LiteralValue::String(s) => format!("\"{}\"", s),
                            schematic_types::LiteralValue::Int(i) => i.to_string(),
                            schematic_types::LiteralValue::Bool(b) => b.to_string(),
                            _ => format!("{:?}", v),
                        })
                        .collect();

                    pkl_type.enum_values = Some(enum_values.join(" | "));
                    debug!(
                        "Created enum typealias '{}' with values: {}",
                        name,
                        pkl_type.enum_values.as_ref().unwrap()
                    );
                    Ok(pkl_type)
                } else {
                    pkl_type.documentation = Some(format!(
                        "{}{}This is an empty enum type.",
                        pkl_type.documentation.as_deref().unwrap_or(""),
                        if pkl_type.documentation.is_some() {
                            "\n\n"
                        } else {
                            ""
                        }
                    ));
                    debug!("Created empty enum class '{}'", name);
                    Ok(pkl_type)
                }
            }
            SchemaType::Union(union_type) => {
                pkl_type.kind = PklTypeKind::TypeAlias;
                let variant_types: Result<Vec<String>> = union_type
                    .variants_types
                    .iter()
                    .map(|v| self.get_pkl_type_name(v))
                    .collect();

                match variant_types {
                    Ok(types) => {
                        let union_str = types.join(" | ");
                        debug!(
                            "Created union typealias '{}' with types: {}",
                            name, union_str
                        );
                        pkl_type.enum_values = Some(union_str);
                        debug!(
                            "Union type for {}: {}",
                            name,
                            pkl_type.enum_values.as_ref().unwrap()
                        );
                        Ok(pkl_type)
                    }
                    Err(e) => {
                        warn!("Failed to resolve union types for {}: {}", name, e);
                        pkl_type.enum_values = Some("Any".to_string());
                        debug!("Failed to resolve union '{}', using Any", name);
                        Ok(pkl_type)
                    }
                }
            }
            SchemaType::Reference(_ref_name) => {
                debug!("Converting reference schema '{}' to PklType", name);
                pkl_type.kind = PklTypeKind::Class;
                Ok(pkl_type)
            }
            SchemaType::Object(_object_type) => {
                // Always treat as a mapping (TypeAlias) since ObjectType does not have named properties.
                pkl_type.kind = PklTypeKind::TypeAlias;
                debug!("Converted object schema '{}' to PklTypeKind::TypeAlias (Mapping)", name);
                Ok(pkl_type)
            }
            _ => {
                // Handle other schema types as needed
                debug!("Unhandled schema type for {}: {:?}", name, schema.ty);
                debug!("Created fallback class '{}' for unhandled type", name);
                Ok(pkl_type)
            }
        };
        result
  }

// Recursively processes a schema and its nested types to build a flat list of Pkl types.
//
// This function is crucial for handling complex schemas with nested definitions,
// ensuring that all referenced types are discovered and converted into `PklType`
// definitions within the module. It uses a `HashSet` to prevent infinite recursion
// for circular references.
//
// # Arguments
//
// * `schema` - The current schematic schema to process.
// * `processed_types` - A mutable `HashSet` to keep track of schema names that have already been processed.
// * `pkl_types` - A mutable vector to accumulate all discovered and converted `PklType` definitions.
//
// # Returns
//
// A `Result` indicating success or failure. Errors can occur during schema conversion.
//
// # Logic
//
// 1. **Base Case**: If the schema has already been processed (present in `processed_types`),
//    it's skipped to prevent infinite loops.
// 2. **Mark as Processed**: The current schema's name is added to `processed_types`.
// 3. **Convert to PklType**: The schema is converted into a `PklType`.
// 4. **Recursive Descent**: Based on the schema's type:
//    - **Struct/Object**: Iterates through fields/properties and recursively calls `process_nested_schema`
//      for each field's schema.
//    - **Array**: Recursively calls `process_nested_schema` for the `items_type`.
//    - **Union**: Recursively calls `process_nested_schema` for each `variants_type`.
//    - **Reference**: Recursively calls `process_nested_schema` for the referenced schema.
// 5. **Accumulate**: The converted `PklType` is added to the `pkl_types` vector.
fn process_schema_recursively(
        &self,
        schema: &Schema,
        processed_types: &mut HashSet<String>,
        pkl_types: &mut Vec<PklType>,
    ) -> Result<()> {
        let schema_name = schema.name.clone().unwrap_or_default();

        if schema_name.is_empty() {
            // Anonymous schema, process its children but don't add itself as a top-level type
            self.process_nested_schema(schema, processed_types, pkl_types)?;
            return Ok(());
        }

        if processed_types.contains(&schema_name) {
            debug!("Skipping already processed schema: {}", schema_name);
            return Ok(());
        }

        debug!("Processing schema recursively: {}", schema_name);
        processed_types.insert(schema_name.clone());

        // Process nested types first to ensure they are available when converting the parent
        self.process_nested_schema(schema, processed_types, pkl_types)?;

        // Convert the current schema to a PklType and add it
        let pkl_type = self.convert_schema_to_pkl_type(schema, &schema_name)?;
        pkl_types.push(pkl_type);

        Ok(())
    }

    /// Helper for `process_schema_recursively` to handle nested schemas within fields, arrays, unions, etc.
    fn process_nested_schema(
        &self,
        schema: &Schema,
        processed_types: &mut HashSet<String>,
        pkl_types: &mut Vec<PklType>,
    ) -> Result<()> {
        match &schema.ty {
            SchemaType::Struct(struct_type) => {
                for field in struct_type.fields.values() {
                    self.process_schema_recursively(&field.schema, processed_types, pkl_types)?;
                }
            }
            SchemaType::Object(object_type) => {
                self.process_schema_recursively(&object_type.key_type, processed_types, pkl_types)?;
                self.process_schema_recursively(&object_type.value_type, processed_types, pkl_types)?;
                // No named properties in ObjectType; only key_type and value_type are relevant.
            }
            SchemaType::Array(array_type) => {
                self.process_schema_recursively(&array_type.items_type, processed_types, pkl_types)?;
            }
            SchemaType::Union(union_type) => {
                for variant_schema in &union_type.variants_types {
                    self.process_schema_recursively(variant_schema, processed_types, pkl_types)?;
                }
            }
            SchemaType::Reference(ref_name) => {
                // For references, we need to find the actual schema definition
                // This assumes the schematic generator has already collected all schemas.
                // We don't have access to the full schema map here, so this needs to be
                // handled at the top level of convert_schemas_to_pkl or by ensuring
                // schematic_types::Schema::name is always populated for references.
                // For now, we'll rely on the top-level processing to pick up referenced types.
                debug!("Encountered reference type '{}' during recursive processing.", ref_name);
            }
            _ => {
                // Primitive types, enums, etc., do not have nested schemas to recurse into
            }
        }
        Ok(())
    }

    /// Converts a struct field from schematic into a Pkl property definition.
    ///
    /// This method handles the complete conversion of a field including its type,
    /// validation constraints, default values, examples, and metadata.
    ///
    /// # Arguments
    ///
    /// * `name` - The field name in the struct
    /// * `field` - The schematic field definition with type and metadata
    ///
    /// # Returns
    ///
    /// A `PklProperty` with complete type information and constraints.
    ///
    /// # Conversion Features
    ///
    /// - **Type Mapping**: Rust types mapped to appropriate Pkl types
    /// - **Constraint Extraction**: Validation rules become Pkl constraints
    /// - **Default Values**: Sensible defaults generated for different types
    /// - **Examples**: Realistic example values for documentation
    /// - **Deprecation**: Deprecated field information preserved
    ///
    /// # Example Conversions
    ///
    /// ```rust,ignore
    /// // String field with validation
    /// #[validate(length(min = 1, max = 50))]
    /// name: String
    /// // Becomes:
    /// // name: String(length >= 1)(length <= 50)
    ///
    /// // Optional numeric field
    /// port: Option<u16>
    /// // Becomes:
    /// // port: Int? = null
    ///
    /// // Array field
    /// tags: Vec<String>
    /// // Becomes:
    /// // tags: Listing<String> = new Listing {}
    /// ```
    fn convert_field_to_property(&self, name: &str, field: &SchemaField) -> Result<PklProperty> {
        let type_name = self.get_pkl_type_name(&field.schema)?;
        let default = self.extract_default_value(&field.schema)?;
        let constraints = self.extract_constraints(&field.schema)?;
        let examples = self.extract_examples(&field.schema)?;

        Ok(PklProperty {
            name: name.to_string(),
            type_name,
            documentation: field.schema.description.clone(),
            optional: field.optional,
            default,
            constraints,
            examples,
            deprecated: field
                .deprecated
                .clone()
                .or_else(|| field.schema.deprecated.clone()),
        })
    }

    /// Extracts sensible default values from schema type information.
    ///
    /// Generates appropriate Pkl default values based on the schema type and constraints.
    /// This helps provide meaningful starting points for configuration values.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema to extract defaults from
    ///
    /// # Returns
    ///
    /// An optional default value string in Pkl syntax, or `None` if no sensible default exists.
    ///
    /// # Default Generation Rules
    ///
    /// - **String Types**: First enum value if available, otherwise no default
    /// - **Boolean Types**: Always defaults to `false`
    /// - **Integer Types**: First enum value, minimum value, or no default
    /// - **Float Types**: First enum value, minimum value, or no default
    /// - **Array Types**: Empty `new Listing {}`
    /// - **Object Types**: Empty `new Mapping {}`
    /// - **Other Types**: No default generated
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Enum string → first value
    /// enum Mode { "dev", "prod" } → "dev"
    ///
    /// // Integer with min → min value
    /// port: u16 (min: 8000) → 8000
    ///
    /// // Boolean → false
    /// enabled: bool → false
    ///
    /// // Array → empty listing
    /// items: Vec<String> → new Listing {}
    /// ```
    fn extract_default_value(&self, schema: &Schema) -> Result<Option<String>> {
        let default_value = match &schema.ty {
            SchemaType::String(string_type) => {
                if let Some(enum_values) = &string_type.enum_values {
                    if !enum_values.is_empty() {
                        Some(format!("\"{}\"", enum_values[0]))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            SchemaType::Boolean(_) => Some("false".to_string()),
            SchemaType::Integer(int_type) => {
                if let Some(enum_values) = &int_type.enum_values {
                    if !enum_values.is_empty() {
                        Some(enum_values[0].to_string())
                    } else {
                        None
                    }
                } else if let Some(min) = int_type.min {
                    Some(min.to_string())
                } else {
                    None
                }
            }
            SchemaType::Float(float_type) => {
                if let Some(enum_values) = &float_type.enum_values {
                    if !enum_values.is_empty() {
                        Some(enum_values[0].to_string())
                    } else {
                        None
                    }
                } else if let Some(min) = float_type.min {
                    Some(min.to_string())
                } else {
                    None
                }
            }
            SchemaType::Array(_) => Some("new Listing {}".to_string()),
            SchemaType::Object(_) => Some("new Mapping {}".to_string()),
            _ => None,
        };

        Ok(default_value)
    }

    /// Extracts validation constraints from schema and converts them to Pkl constraint syntax.
    ///
    /// This method analyzes schema validation rules and converts them into Pkl's
    /// constraint system, preserving validation semantics while adapting syntax.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema containing validation rules
    ///
    /// # Returns
    ///
    /// A vector of `PklConstraint` objects representing all applicable validation rules.
    ///
    /// # Supported Constraint Types
    ///
    /// ## String Constraints
    /// - **Length**: `min_length`/`max_length` → `length >= N` / `length <= N`
    /// - **Pattern**: `pattern` → `matches(Regex(#"pattern"#))`
    /// - **Enum Values**: Multiple enum values → `oneOf("val1"|"val2"|"val3")`
    ///
    /// ## Numeric Constraints (Integer/Float)
    /// - **Range**: `min`/`max` → `this >= N` / `this <= N`
    /// - **Multiple**: `multiple_of` → `this % N == 0`
    /// - **Enum Values**: Multiple enum values → `oneOf(1|2|3)`
    ///
    /// ## Array Constraints
    /// - **Size**: `min_length`/`max_length` → `length >= N` / `length <= N`
    /// - **Uniqueness**: `unique` → `isDistinct`
    ///
    /// # Example Conversions
    ///
    /// ```rust,ignore
    /// // String with pattern and length
    /// #[validate(length(min = 3, max = 20), regex = "^[a-z]+$")]
    /// name: String
    /// // Becomes:
    /// // name: String(length >= 3)(length <= 20)(matches(Regex(#"^[a-z]+$"#)))
    ///
    /// // Integer with range
    /// #[validate(range(min = 1, max = 100))]
    /// count: i32
    /// // Becomes:
    /// // count: Int(this >= 1)(this <= 100)
    /// ```
    fn extract_constraints(&self, schema: &Schema) -> Result<Vec<PklConstraint>> {
        let mut constraints = Vec::new();

        match &schema.ty {
            SchemaType::String(string_type) => {
                if let Some(min_length) = string_type.min_length {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Length,
                        value: format!("length >= {}", min_length),
                        message: Some(format!("Must be at least {} characters long", min_length)),
                    });
                }

                if let Some(max_length) = string_type.max_length {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Length,
                        value: format!("length <= {}", max_length),
                        message: Some(format!("Must be at most {} characters long", max_length)),
                    });
                }

                if let Some(pattern) = &string_type.pattern {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Pattern,
                        value: format!("matches(Regex(#\"{}\"#))", pattern),
                        message: Some(format!("Must match pattern: {}", pattern)),
                    });
                }

                if let Some(enum_values) = &string_type.enum_values {
                    if enum_values.len() > 1 {
                        let values = enum_values
                            .iter()
                            .map(|v| format!("\"{}\"", v))
                            .collect::<Vec<_>>()
                            .join("|");
                        constraints.push(PklConstraint {
                            kind: PklConstraintKind::Custom,
                            value: format!("oneOf({})", values),
                            message: Some(format!("Must be one of: {}", enum_values.join(", "))),
                        });
                    }
                }
            }

            SchemaType::Integer(int_type) => {
                if let Some(min) = int_type.min {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Min,
                        value: format!("this >= {}", min),
                        message: Some(format!("Must be at least {}", min)),
                    });
                }

                if let Some(max) = int_type.max {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Max,
                        value: format!("this <= {}", max),
                        message: Some(format!("Must be at most {}", max)),
                    });
                }

                if let Some(multiple_of) = int_type.multiple_of {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Custom,
                        value: format!("this % {} == 0", multiple_of),
                        message: Some(format!("Must be a multiple of {}", multiple_of)),
                    });
                }

                if let Some(enum_values) = &int_type.enum_values {
                    if enum_values.len() > 1 {
                        let values = enum_values
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join("|");
                        constraints.push(PklConstraint {
                            kind: PklConstraintKind::Custom,
                            value: format!("oneOf({})", values),
                            message: Some(format!(
                                "Must be one of: {}",
                                enum_values
                                    .iter()
                                    .map(|v| v.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )),
                        });
                    }
                }
            }

            SchemaType::Float(float_type) => {
                if let Some(min) = float_type.min {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Min,
                        value: format!("this >= {}", min),
                        message: Some(format!("Must be at least {}", min)),
                    });
                }

                if let Some(max) = float_type.max {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Max,
                        value: format!("this <= {}", max),
                        message: Some(format!("Must be at most {}", max)),
                    });
                }
            }

            SchemaType::Array(array_type) => {
                if let Some(min_length) = array_type.min_length {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Length,
                        value: format!("length >= {}", min_length),
                        message: Some(format!("Must contain at least {} items", min_length)),
                    });
                }

                if let Some(max_length) = array_type.max_length {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Length,
                        value: format!("length <= {}", max_length),
                        message: Some(format!("Must contain at most {} items", max_length)),
                    });
                }

                if array_type.unique == Some(true) {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Custom,
                        value: "isDistinct".to_string(),
                        message: Some("All items must be unique".to_string()),
                    });
                }
            }

            _ => {}
        }

        Ok(constraints)
    }

    /// Generates realistic example values for schema types to enhance documentation.
    ///
    /// Creates meaningful, contextually appropriate examples that help users understand
    /// how to use configuration properties. Examples are formatted in valid Pkl syntax.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema to generate examples for
    ///
    /// # Returns
    ///
    /// A vector of example value strings in Pkl syntax.
    ///
    /// # Example Generation Strategy
    ///
    /// ## String Types
    /// - **Format-aware**: URLs, emails, UUIDs get realistic examples
    /// - **Enum values**: First 3 enum options as examples
    /// - **Patterns**: Generic examples for regex patterns
    /// - **Fallback**: `"example"` for unspecified strings
    ///
    /// ## Numeric Types
    /// - **Enum values**: All enum options as examples
    /// - **Range-based**: Uses minimum value or sensible defaults
    /// - **Integer**: `42` as fallback, respects min/max constraints
    /// - **Float**: `3.14` as fallback, respects min/max constraints
    ///
    /// ## Collection Types
    /// - **Arrays**: Empty listing + example with sample items
    /// - **Objects**: Empty mapping with key/value type information
    ///
    /// ## Boolean Types
    /// - Always provides both `true` and `false` examples
    ///
    /// # Example Outputs
    ///
    /// ```pkl
    /// // URL string format
    /// "https://example.com"
    ///
    /// // Email string format
    /// "user@example.com"
    ///
    /// // Array examples
    /// new Listing<String> {}
    /// new Listing { "item1"; "item2" }
    ///
    /// // Object examples
    /// new Mapping<String, Int> {}
    /// ```
    fn extract_examples(&self, schema: &Schema) -> Result<Vec<String>> {
        let mut examples = Vec::new();

        match &schema.ty {
            SchemaType::String(string_type) => {
                if let Some(enum_values) = &string_type.enum_values {
                    examples.extend(enum_values.iter().take(3).map(|v| format!("\"{}\"", v)));
                } else if let Some(format) = &string_type.format {
                    match format.as_str() {
                        "url" => examples.push("\"https://example.com\"".to_string()),
                        "email" => examples.push("\"user@example.com\"".to_string()),
                        "uri" => examples.push("\"https://api.example.com/v1\"".to_string()),
                        "uuid" => {
                            examples.push("\"550e8400-e29b-41d4-a716-446655440000\"".to_string())
                        }
                        "date" => examples.push("\"2023-12-25\"".to_string()),
                        "time" => examples.push("\"14:30:00\"".to_string()),
                        "datetime" => examples.push("\"2023-12-25T14:30:00Z\"".to_string()),
                        _ => examples.push(format!("\"example-{}\"", format)),
                    }
                } else if string_type.pattern.is_some() {
                    examples.push("\"example\"".to_string());
                } else {
                    examples.push("\"example\"".to_string());
                }
            }

            SchemaType::Integer(int_type) => {
                if let Some(enum_values) = &int_type.enum_values {
                    examples.extend(enum_values.iter().take(3).map(|v| v.to_string()));
                } else {
                    let example_value = if let Some(min) = int_type.min {
                        min
                    } else if let Some(max) = int_type.max {
                        std::cmp::max(0, max)
                    } else {
                        42
                    };
                    examples.push(example_value.to_string());
                }
            }

            SchemaType::Float(float_type) => {
                if let Some(enum_values) = &float_type.enum_values {
                    examples.extend(enum_values.iter().take(3).map(|v| v.to_string()));
                } else {
                    let example_value = if let Some(min) = float_type.min {
                        min
                    } else if let Some(max) = float_type.max {
                        f64::max(0.0, max)
                    } else {
                        3.14
                    };
                    examples.push(example_value.to_string());
                }
            }

            SchemaType::Boolean(_) => {
                examples.push("true".to_string());
                examples.push("false".to_string());
            }

            SchemaType::Array(array_type) => {
                let item_type = self.get_pkl_type_name(&array_type.items_type)?;
                examples.push(format!("new Listing<{}> {{}}", item_type));

                match &array_type.items_type.ty {
                    SchemaType::String(_) => {
                        examples.push("new Listing { \"item1\"; \"item2\" }".to_string())
                    }
                    SchemaType::Integer(_) => examples.push("new Listing { 1; 2; 3 }".to_string()),
                    _ => {}
                }
            }

            SchemaType::Object(object_type) => {
                let key_type = self.get_pkl_type_name(&object_type.key_type)?;
                let value_type = self.get_pkl_type_name(&object_type.value_type)?;
                examples.push(format!("new Mapping<{}, {}> {{}}", key_type, value_type));
            }

            SchemaType::Enum(enum_type) => {
                // For enum types, use the debug representation and clean it up for Pkl
                examples.extend(enum_type.values.iter().take(3).map(|v| {
                    let debug_str = format!("{:?}", v);
                    // Clean up the debug representation for Pkl
                    if debug_str.starts_with("String(") && debug_str.ends_with(")") {
                        // Extract string value: String("value") -> "value"
                        debug_str[7..debug_str.len() - 1].to_string()
                    } else if debug_str.starts_with("Int(") && debug_str.ends_with(")") {
                        // Extract int value: Int(42) -> 42
                        debug_str[4..debug_str.len() - 1].to_string()
                    } else if debug_str.starts_with("Bool(") && debug_str.ends_with(")") {
                        // Extract bool value: Bool(true) -> true
                        debug_str[5..debug_str.len() - 1].to_string()
                    } else if debug_str == "Null" {
                        "null".to_string()
                    } else {
                        // Default: wrap in quotes for safety
                        format!("\"{}\"", debug_str.replace("\"", "\\\""))
                    }
                }));
            }

            _ => {}
        }

        Ok(examples)
    }

    /// Resolves the appropriate Pkl type name for a given schema.
    ///
    /// This method handles the complex mapping from schematic's type system to Pkl's
    /// type names, including generic types, nullable patterns, and custom type mappings.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema to resolve the Pkl type name for
    ///
    /// # Returns
    ///
    /// The Pkl type name as a string, ready for use in Pkl syntax.
    ///
    /// # Type Resolution Rules
    ///
    /// ## Primitive Types
    /// - `String` → `"String"`
    /// - `Boolean` → `"Boolean"`
    /// - `Integer` → `"Int"`
    /// - `Float` → `"Float"`
    ///
    /// ## Generic Types
    /// - `Array<T>` → `"Listing<T>"`
    /// - `Object<K,V>` → `"Mapping<K, V>"`
    ///
    /// ## Complex Types
    /// - `Union<T1, T2>` → `"T1 | T2"`
    /// - `Option<T>` → `"T?"` (nullable shorthand)
    /// - `Complex Nullable` → `"(T1 | T2)?"` (complex nullable union)
    ///
    /// ## Custom Mappings
    /// - Applies configured type mappings from `GeneratorConfig::type_mappings`
    /// - Allows overriding default type names (e.g., `"String"` → `"Text"`)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Basic types
    /// String → "String"
    /// i32 → "Int"
    /// Vec<String> → "Listing<String>"
    /// HashMap<String, i32> → "Mapping<String, Int>"
    ///
    /// // Nullable patterns
    /// Option<String> → "String?"
    /// Union<String, i32, Null> → "(String | Int)?"
    /// Union<String, i32> → "String | Int"
    /// ```
    fn get_pkl_type_name(&self, schema: &Schema) -> Result<String> {
        let type_name = match &schema.ty {
            SchemaType::String(_) => "String".to_string(),
            SchemaType::Boolean(_) => "Boolean".to_string(),
            SchemaType::Integer(_) => "Int".to_string(),
            SchemaType::Float(_) => "Float".to_string(),
            SchemaType::Array(array_type) => {
                let item_type = self.get_pkl_type_name(&array_type.items_type)?;
                format!("Listing<{}>", item_type)
            }
            SchemaType::Object(object_type) => {
                let key_type = self.get_pkl_type_name(&object_type.key_type)?;
                let value_type = self.get_pkl_type_name(&object_type.value_type)?;
                format!("Mapping<{}, {}>", key_type, value_type)
            }
            SchemaType::Reference(ref_name) => ref_name.clone(),
            SchemaType::Struct(_) => {
                // For struct types, use the schema name if available, otherwise "Any"
                schema.name.clone().unwrap_or_else(|| "Any".to_string())
            }
            SchemaType::Enum(_) => {
                // For enum types, use the schema name if available, otherwise "Any"
                schema.name.clone().unwrap_or_else(|| "Any".to_string())
            }
            SchemaType::Union(union_type) => {
                // Handle union types properly, especially nullable patterns
                let variant_types: Result<Vec<String>> = union_type
                    .variants_types
                    .iter()
                    .map(|v| self.get_pkl_type_name(v))
                    .collect();

                match variant_types {
                    Ok(types) => {
                        // Check for nullable pattern (Type | Null)
                        let null_index = types.iter().position(|t| t == "Null");
                        let non_null_types: Vec<&String> =
                            types.iter().filter(|t| *t != "Null").collect();

                        if let Some(_) = null_index {
                            // This is a nullable union
                            if non_null_types.len() == 1 {
                                // Simple nullable: T | Null -> T?
                                format!("{}?", non_null_types[0])
                            } else if non_null_types.len() > 1 {
                                // Complex nullable: (T1 | T2) | Null -> (T1 | T2)?
                                format!(
                                    "({})?",
                                    non_null_types
                                        .iter()
                                        .map(|s| s.as_str())
                                        .collect::<Vec<_>>()
                                        .join(" | ")
                                )
                            } else {
                                // Only Null, shouldn't happen but handle gracefully
                                "Null".to_string()
                            }
                        } else {
                            // Non-nullable union: T1 | T2
                            if types.is_empty() {
                                "Any".to_string()
                            } else {
                                types.join(" | ")
                            }
                        }
                    }
                    Err(_) => {
                        // Fallback to Any if we can't resolve the union types
                        "Any".to_string()
                    }
                }
            }
            SchemaType::Null => "Null".to_string(),
            SchemaType::Unknown => "Any".to_string(),
            _ => "Any".to_string(),
        };

        Ok(self
            .config
            .type_mappings
            .get(&type_name)
            .cloned()
            .unwrap_or(type_name))
        }
}

/// Convenience Functions
///
/// These functions provide a simple API for generating individual schemas without
/// needing to create a `SchemaGenerator` instance. They use default configuration
/// and are ideal for quick schema generation or simple use cases.

/// Generates a workspace configuration schema using default settings.
///
/// This is a convenience function that creates a default `SchemaGenerator` and
/// generates the workspace schema. Equivalent to creating a generator with
/// `GeneratorConfig::default()` and calling `generate_workspace_schema()`.
///
/// # Returns
///
/// A `String` containing the complete Pkl workspace schema.
///
/// # Examples
///
/// ```rust
/// use space_pkl::generate_workspace_schema;
///
/// # fn main() -> space_pkl::Result<()> {
/// let workspace_pkl = generate_workspace_schema()?;
/// println!("Generated workspace schema: {} characters", workspace_pkl.len());
///
/// // The generated Pkl schema is ready to use
/// assert!(!workspace_pkl.is_empty());
/// # Ok(())
/// # }
/// ```
///
/// # See Also
///
/// - [`SchemaGenerator::generate_workspace_schema`] for custom configuration
/// - [`SchemaGenerator::generate_all`] for generating all schemas at once
pub fn generate_workspace_schema() -> Result<String> {
    SchemaGenerator::new(GeneratorConfig::default()).generate_workspace_schema()
}

/// Generates a project configuration schema using default settings.
///
/// This convenience function creates a Pkl schema for Moon project configuration
/// without requiring manual setup of a `SchemaGenerator` instance.
///
/// # Returns
///
/// A `String` containing the complete Pkl project schema with all project-level
/// configuration options including build settings, dependencies, and tasks.
///
/// # Examples
///
/// ```rust
/// use space_pkl::generate_project_schema;
///
/// # fn main() -> space_pkl::Result<()> {
/// let project_pkl = generate_project_schema()?;
///
/// // Check for specific project configuration elements
/// assert!(project_pkl.contains("ProjectConfig"));
///
/// // Use in your application
/// println!("Project schema ready: {} chars", project_pkl.len());
/// # Ok(())
/// # }
/// ```
///
/// # Generated Schema Features
///
/// - Project identification and metadata
/// - Language and platform configuration
/// - Build and compilation settings
/// - Task definitions and dependencies
/// - Tool-specific configurations
pub fn generate_project_schema() -> Result<String> {
    SchemaGenerator::new(GeneratorConfig::default()).generate_project_schema()
}

/// Generates a template configuration schema using default settings.
///
/// Creates a Pkl schema for Moon template configuration used in project
/// scaffolding and code generation workflows.
///
/// # Returns
///
/// A `String` containing the Pkl template schema with variable definitions,
/// file patterns, and template composition rules.
///
/// # Examples
///
/// ```rust
/// use space_pkl::generate_template_schema;
///
/// # fn main() -> space_pkl::Result<()> {
/// let template_pkl = generate_template_schema()?;
///
/// // Template schemas are typically smaller than project schemas
/// println!("Template schema: {} bytes", template_pkl.len());
///
/// // The generated Pkl schema is ready for use in template development
/// assert!(!template_pkl.is_empty());
/// # Ok(())
/// # }
/// ```
pub fn generate_template_schema() -> Result<String> {
    SchemaGenerator::new(GeneratorConfig::default()).generate_template_schema()
}

/// Generates a toolchain configuration schema using default settings.
///
/// Creates a Pkl schema for Moon toolchain configuration covering tool versions,
/// installation preferences, and environment setup.
///
/// # Returns
///
/// A `String` containing the Pkl toolchain schema with tool definitions,
/// version constraints, and environment configurations.
///
/// # Examples
///
/// ```rust
/// use space_pkl::generate_toolchain_schema;
///
/// # fn main() -> space_pkl::Result<()> {
/// let toolchain_pkl = generate_toolchain_schema()?;
///
/// // Toolchain schemas define tool versions and setup
/// assert!(toolchain_pkl.contains("ToolchainConfig"));
///
/// // Integrate with CI/CD pipelines
/// println!("Toolchain definition ready");
/// # Ok(())
/// # }
/// ```
///
/// # Schema Contents
///
/// - Tool version specifications and constraints
/// - Download and installation configuration
/// - Environment variable setup
/// - Platform-specific tool variations
pub fn generate_toolchain_schema() -> Result<String> {
    SchemaGenerator::new(GeneratorConfig::default()).generate_toolchain_schema()
}

/// Generates a tasks configuration schema using default settings.
///
/// Creates a Pkl schema for Moon task configuration including shared task
/// definitions, inheritance patterns, and execution settings.
///
/// # Returns
///
/// A `String` containing the Pkl tasks schema with task definitions,
/// inheritance rules, and execution configuration.
///
/// # Examples
///
/// ```rust
/// use space_pkl::generate_tasks_schema;
///
/// # fn main() -> space_pkl::Result<()> {
/// let tasks_pkl = generate_tasks_schema()?;
///
/// // Tasks schemas define reusable task configurations
/// assert!(tasks_pkl.contains("InheritedTasksConfig"));
///
/// // The generated Pkl schema is ready to use
/// assert!(!tasks_pkl.is_empty());
/// # Ok(())
/// # }
/// ```
///
/// # Schema Features
///
/// - Task command definitions and arguments
/// - Input/output file patterns and caching
/// - Environment variable configuration
/// - Task inheritance and merging strategies
/// - Platform-specific task variations
/// - Dependency management between tasks
pub fn generate_tasks_schema() -> Result<String> {
    SchemaGenerator::new(GeneratorConfig::default()).generate_tasks_schema()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GeneratorConfig, TemplateConfig};
    use schematic_types::{
        ArrayType, BooleanType, EnumType, FloatType, IntegerType, LiteralValue, ObjectType,
        StringType, StructType, UnionOperator, UnionType,
    };
    use std::collections::{BTreeMap, HashMap};

    fn create_test_config() -> GeneratorConfig {
        GeneratorConfig {
            include_comments: true,
            include_examples: true,
            include_validation: true,
            include_deprecated: false,
            no_extends: false,
            header: Some("Test header".to_string()),
            footer: None,
            output_dir: std::env::temp_dir().join("test_pkl"),
            module_name: "test".to_string(),
            split_types: true,
            type_mappings: HashMap::new(),
            template: TemplateConfig::default(),
        }
    }

    #[test]
    fn test_schema_generator_new() {
        let config = create_test_config();
        let generator = SchemaGenerator::new(config.clone());

        assert_eq!(generator.config.module_name, "test");
        assert!(generator.config.include_comments);
        assert_eq!(
            generator.config.output_dir,
            std::env::temp_dir().join("test_pkl")
        );
    }

    #[test]
    fn test_convenience_functions() {
        // Test that convenience functions don't panic and return some content
        let workspace_result = generate_workspace_schema();
        assert!(workspace_result.is_ok());

        let project_result = generate_project_schema();
        assert!(project_result.is_ok());

        let template_result = generate_template_schema();
        assert!(template_result.is_ok());

        let toolchain_result = generate_toolchain_schema();
        assert!(toolchain_result.is_ok());

        let tasks_result = generate_tasks_schema();
        assert!(tasks_result.is_ok());
    }

    #[test]
    fn test_get_pkl_type_name_basic_types() {
        let generator = SchemaGenerator::new(create_test_config());

        // Test string type
        let string_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::String(Box::new(StringType::default())),
        };
        assert_eq!(
            generator.get_pkl_type_name(&string_schema).unwrap(),
            "String"
        );

        // Test boolean type
        let bool_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Boolean(Box::new(BooleanType::default())),
        };
        assert_eq!(
            generator.get_pkl_type_name(&bool_schema).unwrap(),
            "Boolean"
        );

        // Test integer type
        let int_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Integer(Box::new(IntegerType::default())),
        };
        assert_eq!(generator.get_pkl_type_name(&int_schema).unwrap(), "Int");

        // Test float type
        let float_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Float(Box::new(FloatType::default())),
        };
        assert_eq!(generator.get_pkl_type_name(&float_schema).unwrap(), "Float");
    }

    #[test]
    fn test_get_pkl_type_name_array_type() {
        let generator = SchemaGenerator::new(create_test_config());

        let array_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Array(Box::new(ArrayType {
                items_type: Box::new(Schema {
                    name: None,
                    description: None,
                    deprecated: None,
                    nullable: false,
                    ty: SchemaType::String(Box::new(StringType::default())),
                }),
                min_length: None,
                max_length: None,
                unique: None,
                contains: None,
                max_contains: None,
                min_contains: None,
            })),
        };

        assert_eq!(
            generator.get_pkl_type_name(&array_schema).unwrap(),
            "Listing<String>"
        );
    }

    #[test]
    fn test_get_pkl_type_name_object_type() {
        let generator = SchemaGenerator::new(create_test_config());

        let object_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Object(Box::new(ObjectType {
                key_type: Box::new(Schema {
                    name: None,
                    description: None,
                    deprecated: None,
                    nullable: false,
                    ty: SchemaType::String(Box::new(StringType::default())),
                }),
                value_type: Box::new(Schema {
                    name: None,
                    description: None,
                    deprecated: None,
                    nullable: false,
                    ty: SchemaType::Integer(Box::new(IntegerType::default())),
                }),
                min_length: None,
                max_length: None,
                required: None,
            })),
        };

        assert_eq!(
            generator.get_pkl_type_name(&object_schema).unwrap(),
            "Mapping<String, Int>"
        );
    }

    #[test]
    fn test_get_pkl_type_name_reference_type() {
        let generator = SchemaGenerator::new(create_test_config());

        let ref_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Reference("CustomType".to_string()),
        };

        assert_eq!(
            generator.get_pkl_type_name(&ref_schema).unwrap(),
            "CustomType"
        );
    }

    #[test]
    fn test_get_pkl_type_name_nullable_union() {
        let generator = SchemaGenerator::new(create_test_config());

        let union_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Union(Box::new(UnionType {
                variants_types: vec![
                    Box::new(Schema {
                        name: None,
                        description: None,
                        deprecated: None,
                        nullable: false,
                        ty: SchemaType::String(Box::new(StringType::default())),
                    }),
                    Box::new(Schema {
                        name: None,
                        description: None,
                        deprecated: None,
                        nullable: false,
                        ty: SchemaType::Null,
                    }),
                ],
                default_index: None,
                operator: UnionOperator::AnyOf,
                partial: false,
            })),
        };

        assert_eq!(
            generator.get_pkl_type_name(&union_schema).unwrap(),
            "String?"
        );
    }

    #[test]
    fn test_extract_default_value_string_with_enum() {
        let generator = SchemaGenerator::new(create_test_config());

        let string_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::String(Box::new(StringType {
                enum_values: Some(vec!["option1".to_string(), "option2".to_string()]),
                ..Default::default()
            })),
        };

        let default = generator.extract_default_value(&string_schema).unwrap();
        assert_eq!(default, Some("\"option1\"".to_string()));
    }

    #[test]
    fn test_extract_default_value_boolean() {
        let generator = SchemaGenerator::new(create_test_config());

        let bool_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Boolean(Box::new(BooleanType::default())),
        };

        let default = generator.extract_default_value(&bool_schema).unwrap();
        assert_eq!(default, Some("false".to_string()));
    }

    #[test]
    fn test_extract_default_value_integer_with_min() {
        let generator = SchemaGenerator::new(create_test_config());

        let int_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Integer(Box::new(IntegerType {
                min: Some(10),
                ..Default::default()
            })),
        };

        let default = generator.extract_default_value(&int_schema).unwrap();
        assert_eq!(default, Some("10".to_string()));
    }

    #[test]
    fn test_extract_default_value_array() {
        let generator = SchemaGenerator::new(create_test_config());

        let array_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Array(Box::new(ArrayType {
                items_type: Box::new(Schema {
                    name: None,
                    description: None,
                    deprecated: None,
                    nullable: false,
                    ty: SchemaType::String(Box::new(StringType::default())),
                }),
                min_length: None,
                max_length: None,
                unique: None,
                contains: None,
                max_contains: None,
                min_contains: None,
            })),
        };

        let default = generator.extract_default_value(&array_schema).unwrap();
        assert_eq!(default, Some("new Listing {}".to_string()));
    }

    #[test]
    fn test_extract_constraints_string_length() {
        let generator = SchemaGenerator::new(create_test_config());

        let string_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::String(Box::new(StringType {
                min_length: Some(5),
                max_length: Some(20),
                ..Default::default()
            })),
        };

        let constraints = generator.extract_constraints(&string_schema).unwrap();
        assert_eq!(constraints.len(), 2);

        assert_eq!(constraints[0].kind, PklConstraintKind::Length);
        assert_eq!(constraints[0].value, "length >= 5");

        assert_eq!(constraints[1].kind, PklConstraintKind::Length);
        assert_eq!(constraints[1].value, "length <= 20");
    }

    #[test]
    fn test_extract_constraints_string_pattern() {
        let generator = SchemaGenerator::new(create_test_config());

        let string_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::String(Box::new(StringType {
                pattern: Some("^[a-z]+$".to_string()),
                ..Default::default()
            })),
        };

        let constraints = generator.extract_constraints(&string_schema).unwrap();
        assert_eq!(constraints.len(), 1);

        assert_eq!(constraints[0].kind, PklConstraintKind::Pattern);
        assert_eq!(constraints[0].value, "matches(Regex(#\"^[a-z]+$\"#))");
    }

    #[test]
    fn test_extract_constraints_integer_min_max() {
        let generator = SchemaGenerator::new(create_test_config());

        let int_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Integer(Box::new(IntegerType {
                min: Some(0),
                max: Some(100),
                ..Default::default()
            })),
        };

        let constraints = generator.extract_constraints(&int_schema).unwrap();
        assert_eq!(constraints.len(), 2);

        assert_eq!(constraints[0].kind, PklConstraintKind::Min);
        assert_eq!(constraints[0].value, "this >= 0");

        assert_eq!(constraints[1].kind, PklConstraintKind::Max);
        assert_eq!(constraints[1].value, "this <= 100");
    }

    #[test]
    fn test_extract_constraints_array_unique() {
        let generator = SchemaGenerator::new(create_test_config());

        let array_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Array(Box::new(ArrayType {
                items_type: Box::new(Schema {
                    name: None,
                    description: None,
                    deprecated: None,
                    nullable: false,
                    ty: SchemaType::String(Box::new(StringType::default())),
                }),
                unique: Some(true),
                min_length: None,
                max_length: None,
                contains: None,
                max_contains: None,
                min_contains: None,
            })),
        };

        let constraints = generator.extract_constraints(&array_schema).unwrap();
        assert_eq!(constraints.len(), 1);

        assert_eq!(constraints[0].kind, PklConstraintKind::Custom);
        assert_eq!(constraints[0].value, "isDistinct");
    }

    #[test]
    fn test_extract_examples_string_format() {
        let generator = SchemaGenerator::new(create_test_config());

        let url_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::String(Box::new(StringType {
                format: Some("url".to_string()),
                ..Default::default()
            })),
        };

        let examples = generator.extract_examples(&url_schema).unwrap();
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0], "\"https://example.com\"");

        let email_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::String(Box::new(StringType {
                format: Some("email".to_string()),
                ..Default::default()
            })),
        };

        let examples = generator.extract_examples(&email_schema).unwrap();
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0], "\"user@example.com\"");
    }

    #[test]
    fn test_extract_examples_integer() {
        let generator = SchemaGenerator::new(create_test_config());

        let int_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Integer(Box::new(IntegerType {
                min: Some(5),
                ..Default::default()
            })),
        };

        let examples = generator.extract_examples(&int_schema).unwrap();
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0], "5");
    }

    #[test]
    fn test_extract_examples_boolean() {
        let generator = SchemaGenerator::new(create_test_config());

        let bool_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Boolean(Box::new(BooleanType::default())),
        };

        let examples = generator.extract_examples(&bool_schema).unwrap();
        assert_eq!(examples.len(), 2);
        assert_eq!(examples[0], "true");
        assert_eq!(examples[1], "false");
    }

    #[test]
    fn test_extract_examples_enum() {
        let generator = SchemaGenerator::new(create_test_config());

        let enum_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Enum(Box::new(EnumType {
                values: vec![
                    LiteralValue::String("option1".to_string()),
                    LiteralValue::String("option2".to_string()),
                    LiteralValue::Int(42),
                ],
                default_index: None,
                variants: None,
            })),
        };

        let examples = generator.extract_examples(&enum_schema).unwrap();
        assert_eq!(examples.len(), 3);
        assert_eq!(examples[0], "\"option1\"");
        assert_eq!(examples[1], "\"option2\"");
        assert_eq!(examples[2], "42");
    }

    #[test]
    fn test_convert_schema_to_pkl_type_struct() {
        let generator = SchemaGenerator::new(create_test_config());

        let struct_schema = Schema {
            name: Some("TestStruct".to_string()),
            description: Some("A test struct".to_string()),
            deprecated: None,
            nullable: false,
            ty: SchemaType::Struct(Box::new(StructType {
                fields: BTreeMap::new(),
                partial: false,
                required: None,
            })),
        };

        let pkl_type = generator
            .convert_schema_to_pkl_type(&struct_schema, "TestStruct")
            .unwrap();
        assert_eq!(pkl_type.name, "TestStruct");
        assert_eq!(pkl_type.documentation, Some("A test struct".to_string()));
        assert!(matches!(pkl_type.kind, PklTypeKind::Class));
        assert!(!pkl_type.abstract_type);
    }

    #[test]
    fn test_convert_schema_to_pkl_type_enum() {
        let generator = SchemaGenerator::new(create_test_config());

        let enum_schema = Schema {
            name: Some("TestEnum".to_string()),
            description: Some("A test enum".to_string()),
            deprecated: None,
            nullable: false,
            ty: SchemaType::Enum(Box::new(EnumType {
                values: vec![
                    LiteralValue::String("option1".to_string()),
                    LiteralValue::String("option2".to_string()),
                ],
                default_index: None,
                variants: None,
            })),
        };

        let pkl_type = generator
            .convert_schema_to_pkl_type(&enum_schema, "TestEnum")
            .unwrap();
        assert_eq!(pkl_type.name, "TestEnum");
        assert!(matches!(pkl_type.kind, PklTypeKind::TypeAlias));
        assert_eq!(
            pkl_type.enum_values,
            Some("\"option1\" | \"option2\"".to_string())
        );
    }

    #[test]
    fn test_convert_field_to_property() {
        let generator = SchemaGenerator::new(create_test_config());

        let field = SchemaField {
            schema: Schema {
                name: None,
                description: Some("A test field".to_string()),
                deprecated: Some("Use newField instead".to_string()),
                nullable: false,
                ty: SchemaType::String(Box::new(StringType::default())),
            },
            optional: true,
            deprecated: None,
            comment: None,
            env_var: None,
            hidden: false,
            nullable: false,
            read_only: false,
            write_only: false,
        };

        let property = generator
            .convert_field_to_property("testField", &field)
            .unwrap();
        assert_eq!(property.name, "testField");
        assert_eq!(property.type_name, "String");
        assert_eq!(property.documentation, Some("A test field".to_string()));
        assert!(property.optional);
        assert_eq!(
            property.deprecated,
            Some("Use newField instead".to_string())
        );
    }

    #[test]
    fn test_type_mappings_custom() {
        let mut config = create_test_config();
        config
            .type_mappings
            .insert("String".to_string(), "Text".to_string());
        config
            .type_mappings
            .insert("Int".to_string(), "Number".to_string());

        let generator = SchemaGenerator::new(config);

        let string_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::String(Box::new(StringType::default())),
        };

        assert_eq!(generator.get_pkl_type_name(&string_schema).unwrap(), "Text");

        let int_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Integer(Box::new(IntegerType::default())),
        };

        assert_eq!(generator.get_pkl_type_name(&int_schema).unwrap(), "Number");
    }

    #[test]
    fn test_convert_schemas_to_pkl_module() {
        let generator = SchemaGenerator::new(create_test_config());

        let mut schemas = indexmap::IndexMap::new();
        schemas.insert(
            "TestConfig".to_string(),
            Schema {
                name: Some("TestConfig".to_string()),
                description: Some("Test configuration".to_string()),
                deprecated: None,
                nullable: false,
                ty: SchemaType::Struct(Box::new(StructType {
                    fields: BTreeMap::new(),
                    partial: false,
                    required: None,
                })),
            },
        );

        let module = generator.convert_schemas_to_pkl(schemas, "Test").unwrap();
        assert_eq!(module.name, "Test");
        assert_eq!(
            module.documentation,
            Some("Moon test configuration schema".to_string())
        );
        assert_eq!(module.types.len(), 1);
    }

    #[test]
    fn test_complex_union_type_handling() {
        let generator = SchemaGenerator::new(create_test_config());

        // Test complex union with multiple non-null types
        let complex_union_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Union(Box::new(UnionType {
                variants_types: vec![
                    Box::new(Schema {
                        name: None,
                        description: None,
                        deprecated: None,
                        nullable: false,
                        ty: SchemaType::String(Box::new(StringType::default())),
                    }),
                    Box::new(Schema {
                        name: None,
                        description: None,
                        deprecated: None,
                        nullable: false,
                        ty: SchemaType::Integer(Box::new(IntegerType::default())),
                    }),
                    Box::new(Schema {
                        name: None,
                        description: None,
                        deprecated: None,
                        nullable: false,
                        ty: SchemaType::Boolean(Box::new(BooleanType::default())),
                    }),
                ],
                default_index: None,
                operator: UnionOperator::AnyOf,
                partial: false,
            })),
        };

        let type_name = generator.get_pkl_type_name(&complex_union_schema).unwrap();
        assert_eq!(type_name, "String | Int | Boolean");
    }

    #[test]
    fn test_complex_nullable_union_handling() {
        let generator = SchemaGenerator::new(create_test_config());

        // Test complex nullable union: (String | Int) | Null
        let complex_nullable_union = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Union(Box::new(UnionType {
                variants_types: vec![
                    Box::new(Schema {
                        name: None,
                        description: None,
                        deprecated: None,
                        nullable: false,
                        ty: SchemaType::String(Box::new(StringType::default())),
                    }),
                    Box::new(Schema {
                        name: None,
                        description: None,
                        deprecated: None,
                        nullable: false,
                        ty: SchemaType::Integer(Box::new(IntegerType::default())),
                    }),
                    Box::new(Schema {
                        name: None,
                        description: None,
                        deprecated: None,
                        nullable: false,
                        ty: SchemaType::Null,
                    }),
                ],
                default_index: None,
                operator: UnionOperator::AnyOf,
                partial: false,
            })),
        };

        let type_name = generator
            .get_pkl_type_name(&complex_nullable_union)
            .unwrap();
        assert_eq!(type_name, "(String | Int)?");
    }

    #[test]
    fn test_extract_constraints_comprehensive() {
        let generator = SchemaGenerator::new(create_test_config());

        // Test string with all constraint types
        let comprehensive_string_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::String(Box::new(StringType {
                min_length: Some(3),
                max_length: Some(50),
                pattern: Some("^[a-zA-Z0-9_-]+$".to_string()),
                enum_values: Some(vec![
                    "dev".to_string(),
                    "prod".to_string(),
                    "test".to_string(),
                ]),
                ..Default::default()
            })),
        };

        let constraints = generator
            .extract_constraints(&comprehensive_string_schema)
            .unwrap();
        assert_eq!(constraints.len(), 4); // min_length, max_length, pattern, enum

        // Verify constraint kinds
        let constraint_kinds: Vec<&PklConstraintKind> =
            constraints.iter().map(|c| &c.kind).collect();
        assert!(constraint_kinds.contains(&&PklConstraintKind::Length));
        assert!(constraint_kinds.contains(&&PklConstraintKind::Pattern));
        assert!(constraint_kinds.contains(&&PklConstraintKind::Custom));
    }

    #[test]
    fn test_extract_constraints_integer_multiple_of() {
        let generator = SchemaGenerator::new(create_test_config());

        let int_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Integer(Box::new(IntegerType {
                min: Some(0),
                max: Some(1000),
                multiple_of: Some(5),
                enum_values: Some(vec![5, 10, 15, 20]),
                ..Default::default()
            })),
        };

        let constraints = generator.extract_constraints(&int_schema).unwrap();
        assert_eq!(constraints.len(), 4); // min, max, multiple_of, enum

        // Check multiple_of constraint
        let multiple_constraint = constraints.iter().find(|c| c.value.contains("% 5 == 0"));
        assert!(multiple_constraint.is_some());
        assert_eq!(multiple_constraint.unwrap().kind, PklConstraintKind::Custom);
    }

    #[test]
    fn test_extract_constraints_array_length_and_unique() {
        let generator = SchemaGenerator::new(create_test_config());

        let array_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Array(Box::new(ArrayType {
                items_type: Box::new(Schema {
                    name: None,
                    description: None,
                    deprecated: None,
                    nullable: false,
                    ty: SchemaType::String(Box::new(StringType::default())),
                }),
                min_length: Some(1),
                max_length: Some(10),
                unique: Some(true),
                contains: None,
                max_contains: None,
                min_contains: None,
            })),
        };

        let constraints = generator.extract_constraints(&array_schema).unwrap();
        assert_eq!(constraints.len(), 3); // min_length, max_length, unique

        // Verify all expected constraints are present
        let has_min_length = constraints.iter().any(|c| c.value == "length >= 1");
        let has_max_length = constraints.iter().any(|c| c.value == "length <= 10");
        let has_unique = constraints.iter().any(|c| c.value == "isDistinct");

        assert!(has_min_length);
        assert!(has_max_length);
        assert!(has_unique);
    }

    #[test]
    fn test_extract_examples_comprehensive_formats() {
        let generator = SchemaGenerator::new(create_test_config());

        // Test all supported string formats
        let formats_and_expected = vec![
            ("url", "\"https://example.com\""),
            ("email", "\"user@example.com\""),
            ("uri", "\"https://api.example.com/v1\""),
            ("uuid", "\"550e8400-e29b-41d4-a716-446655440000\""),
            ("date", "\"2023-12-25\""),
            ("time", "\"14:30:00\""),
            ("datetime", "\"2023-12-25T14:30:00Z\""),
            ("custom-format", "\"example-custom-format\""),
        ];

        for (format, expected) in formats_and_expected {
            let schema = Schema {
                name: None,
                description: None,
                deprecated: None,
                nullable: false,
                ty: SchemaType::String(Box::new(StringType {
                    format: Some(format.to_string()),
                    ..Default::default()
                })),
            };

            let examples = generator.extract_examples(&schema).unwrap();
            assert_eq!(examples.len(), 1);
            assert_eq!(examples[0], expected, "Failed for format: {}", format);
        }
    }

    #[test]
    fn test_extract_examples_numeric_types() {
        let generator = SchemaGenerator::new(create_test_config());

        // Test integer with constraints
        let int_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Integer(Box::new(IntegerType {
                min: Some(100),
                max: Some(200),
                enum_values: Some(vec![150, 175]),
                ..Default::default()
            })),
        };

        let examples = generator.extract_examples(&int_schema).unwrap();
        assert_eq!(examples.len(), 2);
        assert_eq!(examples[0], "150");
        assert_eq!(examples[1], "175");

        // Test float with constraints
        let float_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Float(Box::new(FloatType {
                min: Some(1.5),
                max: Some(9.9),
                enum_values: Some(vec![2.5, 5.0, 8.75]),
                ..Default::default()
            })),
        };

        let examples = generator.extract_examples(&float_schema).unwrap();
        assert_eq!(examples.len(), 3);
        assert_eq!(examples[0], "2.5");
        assert_eq!(examples[1], "5");
        assert_eq!(examples[2], "8.75");
    }

    #[test]
    fn test_extract_examples_complex_enum_types() {
        let generator = SchemaGenerator::new(create_test_config());

        let complex_enum_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Enum(Box::new(EnumType {
                values: vec![
                    LiteralValue::String("active".to_string()),
                    LiteralValue::String("inactive".to_string()),
                    LiteralValue::Int(1),
                    LiteralValue::Int(0),
                    LiteralValue::Bool(true),
                    LiteralValue::Bool(false),
                ],
                default_index: None,
                variants: None,
            })),
        };

        let examples = generator.extract_examples(&complex_enum_schema).unwrap();
        assert_eq!(examples.len(), 3); // Only take first 3

        // Verify the first few examples are correctly formatted
        assert_eq!(examples[0], "\"active\"");
        assert_eq!(examples[1], "\"inactive\"");
        assert_eq!(examples[2], "1");
    }

    #[test]
    fn test_extract_examples_nested_array_and_object() {
        let generator = SchemaGenerator::new(create_test_config());

        // Test array with complex item types
        let array_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Array(Box::new(ArrayType {
                items_type: Box::new(Schema {
                    name: None,
                    description: None,
                    deprecated: None,
                    nullable: false,
                    ty: SchemaType::Object(Box::new(ObjectType {
                        key_type: Box::new(Schema {
                            name: None,
                            description: None,
                            deprecated: None,
                            nullable: false,
                            ty: SchemaType::String(Box::new(StringType::default())),
                        }),
                        value_type: Box::new(Schema {
                            name: None,
                            description: None,
                            deprecated: None,
                            nullable: false,
                            ty: SchemaType::Integer(Box::new(IntegerType::default())),
                        }),
                        min_length: None,
                        max_length: None,
                        required: None,
                    })),
                }),
                min_length: None,
                max_length: None,
                unique: None,
                contains: None,
                max_contains: None,
                min_contains: None,
            })),
        };

        let examples = generator.extract_examples(&array_schema).unwrap();
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0], "new Listing<Mapping<String, Int>> {}");
    }

    #[test]
    fn test_convert_schema_to_pkl_type_empty_enum() {
        let generator = SchemaGenerator::new(create_test_config());

        let empty_enum_schema = Schema {
            name: Some("EmptyEnum".to_string()),
            description: Some("An empty enum".to_string()),
            deprecated: None,
            nullable: false,
            ty: SchemaType::Enum(Box::new(EnumType {
                values: vec![],
                default_index: None,
                variants: None,
            })),
        };

        let pkl_type = generator
            .convert_schema_to_pkl_type(&empty_enum_schema, "EmptyEnum")
            .unwrap();
        assert_eq!(pkl_type.name, "EmptyEnum");
        assert!(matches!(pkl_type.kind, PklTypeKind::Class));
        assert!(pkl_type
            .documentation
            .as_ref()
            .unwrap()
            .contains("This is an empty enum type"));
        assert!(pkl_type.enum_values.is_none());
    }

    #[test]
    fn test_convert_schema_to_pkl_type_union() {
        let generator = SchemaGenerator::new(create_test_config());

        let union_schema = Schema {
            name: Some("TestUnion".to_string()),
            description: Some("A test union".to_string()),
            deprecated: None,
            nullable: false,
            ty: SchemaType::Union(Box::new(UnionType {
                variants_types: vec![
                    Box::new(Schema {
                        name: None,
                        description: None,
                        deprecated: None,
                        nullable: false,
                        ty: SchemaType::String(Box::new(StringType::default())),
                    }),
                    Box::new(Schema {
                        name: None,
                        description: None,
                        deprecated: None,
                        nullable: false,
                        ty: SchemaType::Integer(Box::new(IntegerType::default())),
                    }),
                ],
                default_index: None,
                operator: UnionOperator::AnyOf,
                partial: false,
            })),
        };

        let pkl_type = generator
            .convert_schema_to_pkl_type(&union_schema, "TestUnion")
            .unwrap();
        assert_eq!(pkl_type.name, "TestUnion");
        assert!(matches!(pkl_type.kind, PklTypeKind::TypeAlias));
        assert_eq!(pkl_type.enum_values, Some("String | Int".to_string()));
    }

    #[test]
    fn test_convert_schema_to_pkl_type_reference() {
        let generator = SchemaGenerator::new(create_test_config());

        let reference_schema = Schema {
            name: Some("TestReference".to_string()),
            description: Some("A reference type".to_string()),
            deprecated: None,
            nullable: false,
            ty: SchemaType::Reference("ExternalType".to_string()),
        };

        let pkl_type = generator
            .convert_schema_to_pkl_type(&reference_schema, "TestReference")
            .unwrap();
        assert_eq!(pkl_type.name, "TestReference");
        assert!(matches!(pkl_type.kind, PklTypeKind::Class));
        assert_eq!(pkl_type.properties.len(), 0); // Reference types have no direct properties
    }

    #[test]
    fn test_convert_schema_to_pkl_type_unknown_type() {
        let generator = SchemaGenerator::new(create_test_config());

        let unknown_schema = Schema {
            name: Some("UnknownType".to_string()),
            description: Some("An unknown type".to_string()),
            deprecated: None,
            nullable: false,
            ty: SchemaType::Unknown,
        };

        let pkl_type = generator
            .convert_schema_to_pkl_type(&unknown_schema, "UnknownType")
            .unwrap();
        assert_eq!(pkl_type.name, "UnknownType");
        assert!(matches!(pkl_type.kind, PklTypeKind::Class));
        assert_eq!(pkl_type.properties.len(), 0);
    }

    #[test]
    fn test_convert_field_to_property_optional_deprecated() {
        let generator = SchemaGenerator::new(create_test_config());

        let field = SchemaField {
            schema: Schema {
                name: None,
                description: Some("A deprecated optional field".to_string()),
                deprecated: Some("Schema-level deprecation".to_string()),
                nullable: false,
                ty: SchemaType::String(Box::new(StringType {
                    min_length: Some(1),
                    max_length: Some(100),
                    ..Default::default()
                })),
            },
            optional: true,
            deprecated: Some("Field-level deprecation".to_string()),
            comment: Some("Additional comment".to_string()),
            env_var: Some("TEST_VAR".to_string()),
            hidden: false,
            nullable: true,
            read_only: true,
            write_only: false,
        };

        let property = generator
            .convert_field_to_property("deprecatedField", &field)
            .unwrap();
        assert_eq!(property.name, "deprecatedField");
        assert_eq!(property.type_name, "String");
        assert!(property.optional);
        // Field-level deprecation should take precedence
        assert_eq!(
            property.deprecated,
            Some("Field-level deprecation".to_string())
        );
        assert!(property.constraints.len() > 0); // Should have length constraints
        assert!(property.examples.len() > 0); // Should have examples
    }

    #[test]
    fn test_convert_schemas_to_pkl_module_multiple_schemas() {
        let generator = SchemaGenerator::new(create_test_config());

        let mut schemas = indexmap::IndexMap::new();

        // Add main config schema
        schemas.insert(
            "WorkspaceConfig".to_string(),
            Schema {
                name: Some("WorkspaceConfig".to_string()),
                description: Some("Main workspace configuration".to_string()),
                deprecated: None,
                nullable: false,
                ty: SchemaType::Struct(Box::new(StructType {
                    fields: BTreeMap::new(),
                    partial: false,
                    required: None,
                })),
            },
        );

        // Add helper type schema
        schemas.insert(
            "TaskType".to_string(),
            Schema {
                name: Some("TaskType".to_string()),
                description: Some("Task type enumeration".to_string()),
                deprecated: None,
                nullable: false,
                ty: SchemaType::Enum(Box::new(EnumType {
                    values: vec![
                        LiteralValue::String("build".to_string()),
                        LiteralValue::String("test".to_string()),
                    ],
                    default_index: None,
                    variants: None,
                })),
            },
        );

        // Add deprecated schema
        schemas.insert(
            "LegacyConfig".to_string(),
            Schema {
                name: Some("LegacyConfig".to_string()),
                description: Some("Legacy configuration".to_string()),
                deprecated: Some("Use WorkspaceConfig instead".to_string()),
                nullable: false,
                ty: SchemaType::Struct(Box::new(StructType {
                    fields: BTreeMap::new(),
                    partial: false,
                    required: None,
                })),
            },
        );

        let module = generator
            .convert_schemas_to_pkl(schemas, "Workspace")
            .unwrap();
        assert_eq!(module.name, "Workspace");
        assert_eq!(
            module.documentation,
            Some("Moon workspace configuration schema".to_string())
        );
        // Only non-deprecated, non-top-level types should be present
        assert_eq!(module.types.len(), 1);


        // Verify only non-deprecated types are present
        let type_names: Vec<&String> = module.types.iter().map(|t| &t.name).collect();
        assert!(type_names.contains(&&"TaskType".to_string()));
        // Deprecated type "LegacyConfig" should not be present
        assert!(!type_names.contains(&&"LegacyConfig".to_string()));
    }

    #[test]
    fn test_deprecated_property_filtering() {
        let mut config = create_test_config();
        config.include_deprecated = false;
        let generator = SchemaGenerator::new(config);

        let field = SchemaField {
            schema: Schema {
                name: None,
                description: Some("A deprecated field".to_string()),
                deprecated: Some("This field is deprecated".to_string()),
                nullable: false,
                ty: SchemaType::String(Box::new(StringType::default())),
            },
            optional: false,
            deprecated: None,
            comment: None,
            env_var: None,
            hidden: false,
            nullable: false,
            read_only: false,
            write_only: false,
        };

        let property = generator
            .convert_field_to_property("deprecatedField", &field)
            .unwrap();
        // Property should still be created but have deprecated flag
        assert_eq!(property.name, "deprecatedField");
        assert_eq!(
            property.deprecated,
            Some("This field is deprecated".to_string())
        );
    }

    #[test]
    fn test_edge_case_union_only_null() {
        let generator = SchemaGenerator::new(create_test_config());

        let only_null_union = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Union(Box::new(UnionType {
                variants_types: vec![Box::new(Schema {
                    name: None,
                    description: None,
                    deprecated: None,
                    nullable: false,
                    ty: SchemaType::Null,
                })],
                default_index: None,
                operator: UnionOperator::AnyOf,
                partial: false,
            })),
        };

        let type_name = generator.get_pkl_type_name(&only_null_union).unwrap();
        assert_eq!(type_name, "Null");
    }

    #[test]
    fn test_edge_case_empty_union() {
        let generator = SchemaGenerator::new(create_test_config());

        let empty_union = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Union(Box::new(UnionType {
                variants_types: vec![],
                default_index: None,
                operator: UnionOperator::AnyOf,
                partial: false,
            })),
        };

        let type_name = generator.get_pkl_type_name(&empty_union).unwrap();
        assert_eq!(type_name, "Any"); // Should fallback to Any for empty unions
    }

    #[test]
    fn test_edge_case_nested_arrays() {
        let generator = SchemaGenerator::new(create_test_config());

        // Array of arrays of strings
        let nested_array_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Array(Box::new(ArrayType {
                items_type: Box::new(Schema {
                    name: None,
                    description: None,
                    deprecated: None,
                    nullable: false,
                    ty: SchemaType::Array(Box::new(ArrayType {
                        items_type: Box::new(Schema {
                            name: None,
                            description: None,
                            deprecated: None,
                            nullable: false,
                            ty: SchemaType::String(Box::new(StringType::default())),
                        }),
                        min_length: None,
                        max_length: None,
                        unique: None,
                        contains: None,
                        max_contains: None,
                        min_contains: None,
                    })),
                }),
                min_length: None,
                max_length: None,
                unique: None,
                contains: None,
                max_contains: None,
                min_contains: None,
            })),
        };

        let type_name = generator.get_pkl_type_name(&nested_array_schema).unwrap();
        assert_eq!(type_name, "Listing<Listing<String>>");
    }

    #[test]
    fn test_edge_case_complex_nested_objects() {
        let generator = SchemaGenerator::new(create_test_config());

        // Mapping<String, Mapping<String, Int>>
        let nested_object_schema = Schema {
            name: None,
            description: None,
            deprecated: None,
            nullable: false,
            ty: SchemaType::Object(Box::new(ObjectType {
                key_type: Box::new(Schema {
                    name: None,
                    description: None,
                    deprecated: None,
                    nullable: false,
                    ty: SchemaType::String(Box::new(StringType::default())),
                }),
                value_type: Box::new(Schema {
                    name: None,
                    description: None,
                    deprecated: None,
                    nullable: false,
                    ty: SchemaType::Object(Box::new(ObjectType {
                        key_type: Box::new(Schema {
                            name: None,
                            description: None,
                            deprecated: None,
                            nullable: false,
                            ty: SchemaType::String(Box::new(StringType::default())),
                        }),
                        value_type: Box::new(Schema {
                            name: None,
                            description: None,
                            deprecated: None,
                            nullable: false,
                            ty: SchemaType::Integer(Box::new(IntegerType::default())),
                        }),
                        min_length: None,
                        max_length: None,
                        required: None,
                    })),
                }),
                min_length: None,
                max_length: None,
                required: None,
            })),
        };

        let type_name = generator.get_pkl_type_name(&nested_object_schema).unwrap();
        assert_eq!(type_name, "Mapping<String, Mapping<String, Int>>");
    }

    #[test]
    fn test_schema_conversion_with_struct_fields() {
        let generator = SchemaGenerator::new(create_test_config());

        let mut fields = BTreeMap::new();
        fields.insert(
            "name".to_string(),
            Box::new(SchemaField {
                schema: Schema {
                    name: None,
                    description: Some("The name field".to_string()),
                    deprecated: None,
                    nullable: false,
                    ty: SchemaType::String(Box::new(StringType {
                        min_length: Some(1),
                        max_length: Some(50),
                        ..Default::default()
                    })),
                },
                optional: false,
                deprecated: None,
                comment: None,
                env_var: None,
                hidden: false,
                nullable: false,
                read_only: false,
                write_only: false,
            }),
        );

        fields.insert(
            "age".to_string(),
            Box::new(SchemaField {
                schema: Schema {
                    name: None,
                    description: Some("The age field".to_string()),
                    deprecated: None,
                    nullable: false,
                    ty: SchemaType::Integer(Box::new(IntegerType {
                        min: Some(0),
                        max: Some(150),
                        ..Default::default()
                    })),
                },
                optional: true,
                deprecated: None,
                comment: None,
                env_var: None,
                hidden: false,
                nullable: false,
                read_only: false,
                write_only: false,
            }),
        );

        let struct_schema = Schema {
            name: Some("Person".to_string()),
            description: Some("A person entity".to_string()),
            deprecated: None,
            nullable: false,
            ty: SchemaType::Struct(Box::new(StructType {
                fields,
                partial: false,
                required: None,
            })),
        };

        let pkl_type = generator
            .convert_schema_to_pkl_type(&struct_schema, "Person")
            .unwrap();
        assert_eq!(pkl_type.name, "Person");
        assert_eq!(pkl_type.properties.len(), 2);

        // Check name property
        let name_prop = pkl_type
            .properties
            .iter()
            .find(|p| p.name == "name")
            .unwrap();
        assert_eq!(name_prop.type_name, "String");
        assert!(!name_prop.optional);
        assert!(name_prop.constraints.len() > 0); // Should have length constraints

        // Check age property
        let age_prop = pkl_type
            .properties
            .iter()
            .find(|p| p.name == "age")
            .unwrap();
        assert_eq!(age_prop.type_name, "Int");
        assert!(age_prop.optional);
        assert!(age_prop.constraints.len() > 0); // Should have min/max constraints
    }
}
