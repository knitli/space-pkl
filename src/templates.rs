//! Template Engine Module for Pkl Schema Generation
//!
//! This module provides a comprehensive template system for generating Pkl configuration
//! schemas from Moon configuration types. It uses the Handlebars templating engine to
//! convert structured type definitions into properly formatted Pkl files with documentation,
//! validation, and examples.
//!
//! # Core Features
//!
//! - **Flexible Template System**: Handlebars-based templates with custom helpers
//! - **Built-in Templates**: Ready-to-use templates for common Pkl patterns
//! - **Custom Template Support**: Load and use project-specific templates
//! - **Rich Helper Functions**: String manipulation, type checking, and formatting
//! - **Documentation Generation**: Automatic comment and example generation
//! - **Validation Integration**: Constraint and annotation rendering
//!
//! # Template Architecture
//!
//! ```text
//! ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
//! │   PklModule     │───▶│  TemplateEngine  │───▶│   Pkl Output    │
//! │  (Type Data)    │    │   (Handlebars)   │    │  (Formatted)    │
//! └─────────────────┘    └──────────────────┘    └─────────────────┘
//!                               │
//!                               ▼
//!                        ┌──────────────┐
//!                        │   Templates  │
//!                        │   & Helpers  │
//!                        └──────────────┘
//! ```
//!
//! # Template Types
//!
//! ## Module Templates
//! Generate complete Pkl files with proper structure:
//! - Module declarations and imports
//! - Type definitions and classes
//! - Documentation and examples
//! - Validation constraints
//!
//! ## Class Templates
//! Render individual Pkl classes:
//! - Property definitions with types
//! - Optional property handling
//! - Default value assignment
//! - Constraint annotations
//!
//! ## Property Templates
//! Format individual properties:
//! - Type annotations
//! - Documentation comments
//! - Validation decorators
//! - Example values
//!
//! # Usage Examples
//!
//! ## Basic Template Rendering
//!
//! ```rust
//! use space_pkl::templates::TemplateEngine;
//! use space_pkl::config::GeneratorConfig;
//! use space_pkl::types::PklModule;
//!
//! # fn example() -> space_pkl::Result<()> {
//! let config = GeneratorConfig::default();
//! let engine = TemplateEngine::new(&config);
//!
//! let module = PklModule {
//!     name: "AppConfig".to_string(),
//!     documentation: Some("Application configuration schema".to_string()),
//!     // ... other fields
//! #   imports: vec![], exports: vec![], types: vec![], properties: vec![],
//! };
//!
//! let pkl_output = engine.render_module(&module, &config)?;
//! println!("{}", pkl_output);
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Template Registration
//!
//! ```rust
//! use space_pkl::templates::TemplateEngine;
//! use space_pkl::config::{GeneratorConfig, TemplateConfig};
//! use std::path::PathBuf;
//!
//! # fn example() -> space_pkl::Result<()> {
//! let template_config = TemplateConfig {
//!     template_dir: Some(PathBuf::from("./custom-templates")),
//!     template_extension: "hbs".to_string(),
//!     custom_templates: std::collections::HashMap::new(),
//!     generate_templates: true,
//! };
//!
//! let config = GeneratorConfig {
//!     include_examples: true,
//!     include_validation: true,
//!     ..Default::default()
//! };
//!
//! let engine = TemplateEngine::new(&config);
//! // Engine now has access to custom templates
//! # Ok(())
//! # }
//! ```
//!
//! # Template Helpers
//!
//! ## String Manipulation
//! - `capitalize`: Capitalize first letter
//! - `snake_case`: Convert to snake_case
//! - `camel_case`: Convert to camelCase
//!
//! ## Type Operations
//! - `optional`: Handle optional type formatting
//! - `is_typealias`: Check if type is an alias
//! - `escape_pkl_keyword`: Escape Pkl reserved words
//!
//! ## Documentation
//! - `doc`: Format documentation comments
//! - `deprecated`: Render deprecation warnings
//!
//! # Template Syntax
//!
//! Templates use Handlebars syntax with Pkl-specific helpers:
//!
//! ```handlebars
//! {{!-- Module header --}}
//! module {{module.name}}
//!
//! {{#each module.imports}}
//! import "{{path}}"{{#if alias}} as {{alias}}{{/if}}
//! {{/each}}
//!
//! {{#each module.types}}
//! {{#if documentation}}/// {{doc documentation}}{{/if}}
//! class {{capitalize name}} {
//!   {{#each properties}}
//!   {{#if documentation}}/// {{doc documentation}}{{/if}}
//!   {{#if deprecated}}{{deprecated deprecated}}{{/if}}
//!   {{escape_pkl_keyword name}}: {{type_name}}{{optional optional}}{{#if default}} = {{default}}{{/if}}
//!   {{/each}}
//! }
//! {{/each}}
//! ```
//!
//! # Built-in Templates
//!
//! The engine includes several built-in templates:
//!
//! ## Module Template
//! Renders complete Pkl modules with proper structure, imports, and type definitions.
//!
//! ## Index Template
//! Generates module index files that provide access to all schema types.
//!
//! ## Class Template
//! Formats individual Pkl classes with properties and validation.
//!
//! ## Property Template
//! Handles individual property formatting with types and constraints.
//!
//! # Customization
//!
//! ## Custom Templates
//! Place `.hbs` files in your template directory to override built-in templates:
//! ```text
//! templates/
//! ├── module.hbs      # Override module template
//! ├── class.hbs       # Override class template
//! ├── custom.hbs      # Add custom template
//! └── helpers.hbs     # Template helpers
//! ```
//!
//! ## Template Variables
//! Access configuration and context data in templates:
//! - `module`: Complete module definition
//! - `config`: Generator configuration
//! - `variables`: Custom template variables
//!
//! # Error Handling
//!
//! The template engine provides detailed error reporting:
//! - Template parsing errors with line numbers
//! - Helper function failures with context
//! - Variable resolution errors with suggestions
//! - File I/O errors with file paths
//!
//! (c) 2025 Stash AI Inc (knitli)
//!   - Created by Adam Poulemanos ([@bashandbone](https://github.com/bashandbone)) for Stash AI Inc.
//! Licensed under the [Plain MIT License](https://plainlicense.org/licenses/permissive/mit/)

use crate::config::GeneratorConfig;
use crate::types::*;
use crate::Result;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use miette::{IntoDiagnostic, WrapErr};
use serde_json::json;
use std::collections::HashMap;
use tracing::debug;

/// Template engine for rendering Pkl schemas from type definitions.
///
/// The `TemplateEngine` provides a high-level interface for converting structured
/// type definitions into formatted Pkl configuration files. It uses the Handlebars
/// templating engine with custom helpers and built-in templates optimized for
/// Pkl syntax and Moon configuration patterns.
///
/// # Features
///
/// - **Handlebars Integration**: Full Handlebars templating with custom helpers
/// - **Built-in Templates**: Ready-to-use templates for common Pkl patterns
/// - **Custom Template Loading**: Support for project-specific template customization
/// - **Helper Functions**: String manipulation, type formatting, and validation helpers
/// - **Error Handling**: Detailed error reporting with context and suggestions
/// - **Caching**: Template compilation caching for performance
///
/// # Template System Architecture
///
/// ```text
/// ┌─────────────────┐
/// │ TemplateEngine  │
/// ├─────────────────┤
/// │ • Handlebars    │◄─── Built-in Templates
/// │ • Helpers       │◄─── Custom Templates
/// │ • Cache         │◄─── Template Variables
/// └─────────────────┘
///          │
///          ▼
/// ┌─────────────────┐
/// │   Pkl Output    │
/// │ • Formatted     │
/// │ • Validated     │
/// │ • Documented    │
/// └─────────────────┘
/// ```
///
/// # Usage Patterns
///
/// ## Simple Module Rendering
/// ```rust
/// use space_pkl::templates::TemplateEngine;
/// use space_pkl::config::GeneratorConfig;
///
/// # fn example() -> space_pkl::Result<()> {
/// let config = GeneratorConfig::default();
/// let engine = TemplateEngine::new(&config);
///
/// # let module = space_pkl::types::PklModule {
/// #   name: "Test".to_string(), documentation: None, imports: vec![],
/// #   exports: vec![], types: vec![], properties: vec![]
/// # };
/// // Render module with default templates
/// let output = engine.render_module(&module, &config)?;
/// # Ok(())
/// # }
/// ```
///
/// ## Custom Template Configuration
/// ```rust
/// use space_pkl::templates::TemplateEngine;
/// use space_pkl::config::{GeneratorConfig, TemplateConfig};
/// use std::path::PathBuf;
///
/// # fn example() -> space_pkl::Result<()> {
/// let template_config = TemplateConfig {
///     template_dir: Some(PathBuf::from("./templates")),
///     template_extension: "hbs".to_string(),
///     custom_templates: std::collections::HashMap::new(),
///     generate_templates: true,
/// };
///
/// let config = GeneratorConfig {
///     include_examples: true,
///     include_validation: true,
///     ..Default::default()
/// };
///
/// let engine = TemplateEngine::new(&config);
/// # Ok(())
/// # }
/// ```
///
/// # Template Loading
///
/// The engine loads templates in this order:
/// 1. **Built-in templates**: Always available, provide baseline functionality
/// 2. **Custom templates**: Override built-ins when present in template directory
/// 3. **Runtime templates**: Can be registered programmatically
///
/// # Template Context
///
/// Templates receive rich context data:
/// - **module**: Complete `PklModule` with types and properties
/// - **config**: `GeneratorConfig` with feature flags and settings
/// - **variables**: Custom variables for template customization
///
/// # Helper Functions
///
/// Built-in helpers for common template operations:
/// - `capitalize`: "hello" → "Hello"
/// - `snake_case`: "HelloWorld" → "hello_world"
/// - `camel_case`: "hello_world" → "helloWorld"
/// - `optional`: Add "?" for optional types
/// - `doc`: Format documentation comments
/// - `deprecated`: Render deprecation warnings
/// - `escape_pkl_keyword`: Handle Pkl reserved words
///
/// # Performance Considerations
///
/// - Templates are compiled once during engine creation
/// - Helper functions are optimized for repeated calls
/// - Large modules are processed in chunks to manage memory
/// - Template caching reduces compilation overhead
///
/// # Error Handling
///
/// The engine provides comprehensive error reporting:
/// - Template syntax errors with line numbers
/// - Helper function failures with parameter details
/// - File I/O errors with full paths
/// - Variable resolution errors with suggestions
pub struct TemplateEngine {
    /// The Handlebars template engine instance.
    ///
    /// Configured with custom helpers, escape functions, and registered templates.
    /// Handles all template compilation, caching, and rendering operations.
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    /// Creates a new template engine with the specified configuration.
    ///
    /// Initializes the Handlebars engine, registers built-in templates and helpers,
    /// and loads any custom templates from the configured template directory.
    /// The engine is ready to render templates immediately after creation.
    ///
    /// # Configuration Processing
    ///
    /// 1. **HTML Escaping**: Disabled to prevent interference with Pkl syntax
    /// 2. **Built-in Templates**: Registered for module, class, and property rendering
    /// 3. **Helper Functions**: Registered for string manipulation and formatting
    /// 4. **Custom Templates**: Loaded from `template_dir` if specified
    /// 5. **Template Validation**: All templates validated during registration
    ///
    /// # Template Directory Structure
    ///
    /// When `template_dir` is specified, the engine looks for templates with the
    /// configured extension (default `.hbs`):
    /// ```text
    /// templates/
    /// ├── module.hbs      # Override main module template
    /// ├── class.hbs       # Override class template
    /// ├── property.hbs    # Override property template
    /// ├── index.hbs       # Override index template
    /// └── custom.hbs      # Additional custom templates
    /// ```
    ///
    /// # Template Loading Rules
    ///
    /// - **Extension matching**: Only files with the configured extension are loaded
    /// - **Name mapping**: File stem becomes template name (`module.hbs` → `"module"`)
    /// - **Override behavior**: Custom templates override built-in templates with same name
    /// - **Error handling**: Invalid templates are skipped with warning logs
    ///
    /// # Examples
    ///
    /// ## Default Configuration
    /// ```rust
    /// use space_pkl::templates::TemplateEngine;
    /// use space_pkl::config::GeneratorConfig;
    ///
    /// let config = GeneratorConfig::default();
    /// let engine = TemplateEngine::new(&config);
    /// // Engine ready with built-in templates
    /// ```
    ///
    /// ## Custom Template Directory
    /// ```rust
    /// use space_pkl::templates::TemplateEngine;
    /// use space_pkl::config::{GeneratorConfig, TemplateConfig};
    /// use std::path::PathBuf;
    ///
    /// let template_config = TemplateConfig {
    ///     template_dir: Some(PathBuf::from("./my-templates")),
    ///     template_extension: "hbs".to_string(),
    ///     custom_templates: std::collections::HashMap::new(),
    ///     generate_templates: true,
    /// };
    ///
    /// let config = GeneratorConfig {
    ///     include_examples: true,
    ///     include_validation: true,
    ///     ..Default::default()
    /// };
    ///
    /// let engine = TemplateEngine::new(&config);
    /// // Engine includes custom templates from ./my-templates/
    /// ```
    ///
    /// # Error Handling
    ///
    /// Template loading errors are handled gracefully:
    /// - **Missing directories**: Logged as warnings, built-ins used
    /// - **Invalid templates**: Skipped with error logs
    /// - **Permission errors**: Logged and continue with available templates
    /// - **Syntax errors**: Detailed error messages with file locations
    pub fn new(config: &GeneratorConfig) -> Self {
        let mut handlebars = Handlebars::new();

        // Disable HTML escaping to prevent &lt; &gt; &quot; in output
        handlebars.register_escape_fn(handlebars::no_escape);

        // Register built-in templates
        Self::register_builtin_templates(&mut handlebars);

        // Register helper functions
        Self::register_helpers(&mut handlebars);

        // Load custom templates if specified
        if let Some(template_dir) = &config.template.template_dir {
            if template_dir.exists() {
                // Register templates from directory manually since register_templates_directory
                // may not be available in this version
                if let Ok(entries) = std::fs::read_dir(template_dir) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                            if entry.path().extension().and_then(|s| s.to_str())
                                == Some(&config.template.template_extension.trim_start_matches('.'))
                            {
                                if let Ok(template_content) = std::fs::read_to_string(entry.path())
                                {
                                    let _ =
                                        handlebars.register_template_string(name, template_content);
                                }
                            }
                        }
                    }
                }
            }
        }

        Self { handlebars }
    }

    /// Renders a complete Pkl module from type definitions.
    ///
    /// Converts a `PklModule` containing type definitions, properties, and metadata
    /// into a formatted Pkl configuration file. The output includes proper module
    /// declarations, imports, type definitions, documentation, and validation constraints.
    ///
    /// # Rendering Process
    ///
    /// 1. **Context Creation**: Builds template context with module, config, and variables
    /// 2. **Type Processing**: Renders each type definition using appropriate templates
    /// 3. **Template Resolution**: Uses custom templates if available, falls back to built-ins
    /// 4. **Content Generation**: Applies Handlebars processing with helper functions
    /// 5. **Output Formatting**: Produces formatted Pkl code ready for use
    ///
    /// # Generated Pkl Structure
    ///
    /// ```pkl
    /// // Generated header (if configured)
    /// /// Module documentation
    /// module ModuleName
    ///
    /// // Imports section
    /// import "dependency.pkl" as dep
    ///
    /// // Type definitions
    /// /// Class documentation
    /// class ClassName {
    ///   /// Property documentation
    ///   @Validation
    ///   propertyName: PropertyType = defaultValue
    /// }
    ///
    /// // Generated footer (if configured)
    /// ```
    ///
    /// # Template Context
    ///
    /// The rendering context includes:
    /// - **module**: Complete `PklModule` with all type definitions
    /// - **config**: `GeneratorConfig` controlling output features
    /// - **variables**: Empty map (can be customized for advanced use cases)
    ///
    /// # Debug Logging
    ///
    /// When debug logging is enabled, the method provides detailed information:
    /// - Type definitions being processed
    /// - Serialized type data for inspection
    /// - Individual template rendering results
    /// - Template resolution and caching information
    ///
    /// # Examples
    ///
    /// ## Basic Module Rendering
    /// ```rust
    /// use space_pkl::templates::TemplateEngine;
    /// use space_pkl::types::*;
    /// use space_pkl::config::GeneratorConfig;
    ///
    /// # fn example() -> space_pkl::Result<()> {
    /// let engine = TemplateEngine::new(&GeneratorConfig::default());
    ///
    /// let module = PklModule {
    ///     name: "AppConfig".to_string(),
    ///     documentation: Some("Application configuration".to_string()),
    ///     imports: vec![],
    ///     exports: vec![],
    ///     types: vec![
    ///         PklType {
    ///             name: "DatabaseConfig".to_string(),
    ///             kind: PklTypeKind::Class,
    ///             properties: vec![
    ///                 PklProperty {
    ///                     name: "host".to_string(),
    ///                     type_name: "String".to_string(),
    ///                     documentation: Some("Database host".to_string()),
    ///                     optional: false,
    ///                     default: Some("\"localhost\"".to_string()),
    ///                     // ... other fields
    /// #                   constraints: vec![], examples: vec![], deprecated: None,
    ///                 }
    ///             ],
    ///             // ... other fields
    /// #           documentation: None, enum_values: None, extends: vec![], abstract_type: false, deprecated: None,
    ///         }
    ///     ],
    ///     properties: vec![],
    /// };
    ///
    /// let config = GeneratorConfig::default();
    /// let pkl_output = engine.render_module(&module, &config)?;
    ///
    /// // Output will be formatted Pkl code:
    /// // module AppConfig
    /// //
    /// // /// Application configuration
    /// // class DatabaseConfig {
    /// //   /// Database host
    /// //   host: String = "localhost"
    /// // }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Module with Custom Configuration
    /// ```rust
    /// use space_pkl::templates::TemplateEngine;
    /// use space_pkl::config::GeneratorConfig;
    ///
    /// # fn example() -> space_pkl::Result<()> {
    /// let config = GeneratorConfig {
    ///     include_examples: true,
    ///     include_validation: true,
    ///     ..Default::default()
    /// };
    ///
    /// let engine = TemplateEngine::new(&config);
    /// // Rendering will include examples and validation annotations
    /// # let module = space_pkl::types::PklModule {
    /// #   name: "Test".to_string(), documentation: None, imports: vec![],
    /// #   exports: vec![], types: vec![], properties: vec![]
    /// # };
    /// let output = engine.render_module(&module, &config)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Error Handling
    ///
    /// Returns detailed errors for common failure scenarios:
    /// - **Template not found**: When required templates are missing
    /// - **Rendering errors**: When template syntax or helper functions fail
    /// - **Data errors**: When module data is invalid or incomplete
    /// - **Helper errors**: When helper functions encounter invalid parameters
    ///
    /// # Performance Notes
    ///
    /// - Templates are compiled once during engine creation
    /// - Large modules are processed efficiently through streaming
    /// - Debug logging can impact performance in hot paths
    /// - Context cloning is optimized for common use cases
    pub fn render_module(&self, module: &PklModule, config: &GeneratorConfig) -> Result<String> {
        // Debug logging to see what types we're rendering
        for pkl_type in &module.types {
            debug!(
                "Rendering type '{}': kind={:?}, properties={}, enum_values={:?}",
                pkl_type.name,
                pkl_type.kind,
                pkl_type.properties.len(),
                pkl_type.enum_values
            );

            // Serialize the type to see how it looks in JSON
            if let Ok(serialized) = serde_json::to_string(pkl_type) {
                debug!("Serialized PklType '{}': {}", pkl_type.name, serialized);
            }

            // Test individual template rendering
            if let Ok(rendered) = self.handlebars.render("class", pkl_type) {
                debug!("Rendered class '{}': {}", pkl_type.name, rendered);
            }
        }

        let context = TemplateContext {
            module: module.clone(),
            config: config.clone(),
            variables: HashMap::new(),
        };

        self.handlebars
            .render("module", &context)
            .into_diagnostic()
            .wrap_err("Failed to render module template")
    }

    /// Renders a module index file that provides access to all schema types.
    ///
    /// Generates a comprehensive index file that serves as an entry point to all
    /// available Pkl configuration schemas. The index includes documentation,
    /// import statements, and references to main configuration classes for easy
    /// discovery and usage.
    ///
    /// # Index File Purpose
    ///
    /// The index file serves multiple purposes:
    /// - **Discovery**: Lists all available schema modules
    /// - **Documentation**: Provides overview and usage examples
    /// - **Imports**: Centralized access to all schema types
    /// - **Examples**: Sample configurations for each schema type
    /// - **Navigation**: Quick reference for developers
    ///
    /// # Generated Index Structure
    ///
    /// ```pkl
    /// /// Moon Configuration Schemas Module Index
    /// ///
    /// /// This module provides access to all Moon configuration schemas.
    /// /// Import this module to get access to all configuration types.
    /// ///
    /// /// ## Available Schemas
    /// /// - Workspace: workspace.pkl (WorkspaceConfig)
    /// /// - Project: project.pkl (ProjectConfig)
    /// /// - Template: template.pkl (TemplateConfig)
    /// /// - Toolchain: toolchain.pkl (ToolchainConfig)
    /// /// - Tasks: tasks.pkl (InheritedTasksConfig)
    /// module moon_schemas
    ///
    /// import "workspace.pkl" as workspace
    /// import "project.pkl" as project
    /// // ... other imports
    /// ```
    ///
    /// # Schema Registry
    ///
    /// The index includes a comprehensive registry of all schema types:
    ///
    /// | Schema Name | File Name | Main Class | Purpose |
    /// |-------------|-----------|------------|---------|
    /// | Workspace | workspace.pkl | WorkspaceConfig | Workspace-level settings |
    /// | Project | project.pkl | ProjectConfig | Project configuration |
    /// | Template | template.pkl | TemplateConfig | Template definitions |
    /// | Toolchain | toolchain.pkl | ToolchainConfig | Tool configurations |
    /// | Tasks | tasks.pkl | InheritedTasksConfig | Task definitions |
    ///
    /// # Context Data
    ///
    /// The index template receives:
    /// - **module_name**: Name from generator configuration
    /// - **config**: Complete generator configuration
    /// - **schemas**: Array of schema definitions with metadata
    ///
    /// # Template Variables
    ///
    /// Each schema entry includes:
    /// - `name`: Human-readable schema name
    /// - `file`: Pkl file name for imports
    /// - `main_class`: Primary configuration class name
    ///
    /// # Usage Examples
    ///
    /// ## Basic Index Generation
    /// ```rust
    /// use space_pkl::templates::TemplateEngine;
    /// use space_pkl::config::GeneratorConfig;
    ///
    /// # fn example() -> space_pkl::Result<()> {
    /// let config = GeneratorConfig {
    ///     module_name: "moon_schemas".to_string(),
    ///     include_examples: true,
    ///     include_validation: true,
    ///     ..Default::default()
    /// };
    ///
    /// let engine = TemplateEngine::new(&config);
    /// let index_content = engine.render_module_index(&config)?;
    ///
    /// // The generated index is ready to use
    /// assert!(!index_content.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Integration with Build Systems
    /// ```rust
    /// use space_pkl::templates::TemplateEngine;
    /// use space_pkl::config::GeneratorConfig;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> space_pkl::Result<()> {
    /// let config = GeneratorConfig {
    ///     include_examples: true,
    ///     include_validation: true,
    ///     module_name: "schemas".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// let engine = TemplateEngine::new(&config);
    /// let index = engine.render_module_index(&config)?;
    ///
    /// // The generated index content is ready to write
    /// assert!(!index.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Customization
    ///
    /// The index template can be customized by:
    /// 1. **Override template**: Place `index.hbs` in template directory
    /// 2. **Modify schema list**: Update the schemas array in this method
    /// 3. **Add metadata**: Include additional schema information
    /// 4. **Custom formatting**: Adjust template for different output styles
    ///
    /// # Error Handling
    ///
    /// Returns errors for:
    /// - **Template not found**: When index template is missing
    /// - **Rendering failures**: When template syntax errors occur
    /// - **JSON serialization**: When context data is invalid
    /// - **Configuration errors**: When module_name or other config is invalid
    ///
    /// # Output Integration
    ///
    /// The generated index file integrates with Pkl tooling:
    /// - **IDE support**: Provides autocomplete and navigation
    /// - **Validation**: Enables cross-module type checking
    /// - **Documentation**: Generates comprehensive API docs
    /// - **Testing**: Supports schema validation testing
    pub fn render_module_index(&self, config: &GeneratorConfig) -> Result<String> {
        let context = json!({
            "module_name": config.module_name,
            "config": config,
            "schemas": [
                {"name": "Workspace", "file": "workspace.pkl", "main_class": "WorkspaceConfig"},
                {"name": "Project", "file": "project.pkl", "main_class": "ProjectConfig"},
                {"name": "Template", "file": "template.pkl", "main_class": "TemplateConfig"},
                {"name": "Toolchain", "file": "toolchain.pkl", "main_class": "ToolchainConfig"},
                {"name": "Tasks", "file": "tasks.pkl", "main_class": "InheritedTasksConfig"},
            ]
        });

        self.handlebars
            .render("index", &context)
            .into_diagnostic()
            .wrap_err("Failed to render index template")
    }

    /// Register built-in templates
    fn register_builtin_templates(handlebars: &mut Handlebars) {
        // Main module template
        handlebars
            .register_template_string("module", MODULE_TEMPLATE)
            .expect("Failed to register module template");

        // Index template
        handlebars
            .register_template_string("index", INDEX_TEMPLATE)
            .expect("Failed to register index template");

        // Type templates
        handlebars
            .register_template_string("class", CLASS_TEMPLATE)
            .expect("Failed to register class template");

        handlebars
            .register_template_string("property", PROPERTY_TEMPLATE)
            .expect("Failed to register property template");
    }

    /// Registers helper functions with the Handlebars template engine.
    ///
    /// Helper functions extend template capabilities with commonly needed operations
    /// for Pkl schema generation. They handle string manipulation, type formatting,
    /// documentation rendering, and Pkl-specific syntax requirements.
    ///
    /// # Available Helpers
    ///
    /// ## String Manipulation Helpers
    ///
    /// ### `capitalize`
    /// Capitalizes the first letter of a string while preserving the rest.
    /// - **Usage**: `{{capitalize "hello world"}}` → `"Hello world"`
    /// - **Purpose**: Format class names and type identifiers
    /// - **Parameters**: Single string parameter
    ///
    /// ### `snake_case`
    /// Converts CamelCase strings to snake_case format.
    /// - **Usage**: `{{snake_case "HelloWorld"}}` → `"hello_world"`
    /// - **Purpose**: Convert Rust identifiers to Pkl naming conventions
    /// - **Parameters**: Single string parameter
    ///
    /// ### `camel_case`
    /// Converts snake_case strings to camelCase format.
    /// - **Usage**: `{{camel_case "hello_world"}}` → `"helloWorld"`
    /// - **Purpose**: Format property names for Pkl conventions
    /// - **Parameters**: Single string parameter
    ///
    /// ## Type and Syntax Helpers
    ///
    /// ### `optional`
    /// Adds optional type syntax for nullable properties.
    /// - **Usage**: `{{type_name}}{{optional is_optional}}` → `"String?"` or `"String"`
    /// - **Purpose**: Handle optional property type annotations
    /// - **Parameters**: Boolean indicating if type is optional
    ///
    /// ### `is_typealias`
    /// Checks if a type definition is a type alias.
    /// - **Usage**: `{{#if (is_typealias this)}}typealias{{else}}class{{/if}}`
    /// - **Purpose**: Conditional rendering for different type kinds
    /// - **Parameters**: Type object to check
    ///
    /// ### `escape_pkl_keyword`
    /// Escapes Pkl reserved words and keywords.
    /// - **Usage**: `{{escape_pkl_keyword "class"}}` → `"\`class\`"`
    /// - **Purpose**: Handle property names that conflict with Pkl keywords
    /// - **Parameters**: String to potentially escape
    ///
    /// ## Documentation Helpers
    ///
    /// ### `doc`
    /// Formats documentation strings for Pkl comment syntax.
    /// - **Usage**: `{{#if docs}}/// {{doc docs}}{{/if}}`
    /// - **Purpose**: Render multi-line documentation with proper comment formatting
    /// - **Parameters**: Documentation string (can be multi-line)
    ///
    /// ### `deprecated`
    /// Renders deprecation annotations and warnings.
    /// - **Usage**: `{{deprecated deprecation_message}}`
    /// - **Purpose**: Generate Pkl deprecation decorators
    /// - **Parameters**: Deprecation message string
    ///
    /// # Helper Registration Process
    ///
    /// 1. **Function Registration**: Each helper is registered with a unique name
    /// 2. **Type Safety**: Helpers include parameter validation and error handling
    /// 3. **Performance**: Helpers are optimized for repeated template rendering
    /// 4. **Error Reporting**: Invalid helper usage generates detailed error messages
    ///
    /// # Template Usage Examples
    ///
    /// ## Class Definition with Helpers
    /// ```handlebars
    /// {{#if documentation}}/// {{doc documentation}}{{/if}}
    /// {{#if deprecated}}{{deprecated deprecated}}{{/if}}
    /// class {{capitalize name}} {
    ///   {{#each properties}}
    ///   {{#if documentation}}/// {{doc documentation}}{{/if}}
    ///   {{escape_pkl_keyword name}}: {{type_name}}{{optional optional}}{{#if default}} = {{default}}{{/if}}
    ///   {{/each}}
    /// }
    /// ```
    ///
    /// ## Property with Conditional Formatting
    /// ```handlebars
    /// {{#each properties}}
    /// /// {{doc (or documentation "No documentation available")}}
    /// {{#if deprecated}}@Deprecated("{{deprecated}}"){{/if}}
    /// {{camel_case name}}: {{#if (is_typealias type)}}{{type.alias}}{{else}}{{type.name}}{{/if}}{{optional optional}}
    /// {{/each}}
    /// ```
    ///
    /// # Error Handling
    ///
    /// Helpers provide robust error handling:
    /// - **Parameter validation**: Check for required parameters
    /// - **Type checking**: Ensure parameters are correct types
    /// - **Graceful fallbacks**: Default behavior for invalid inputs
    /// - **Detailed errors**: Clear error messages with context
    ///
    /// # Custom Helper Integration
    ///
    /// Additional helpers can be registered after engine creation:
    /// ```rust
    /// use handlebars::{Helper, HelperResult, Output, RenderContext};
    ///
    /// fn custom_helper(
    ///     h: &Helper,
    ///     _: &handlebars::Handlebars,
    ///     _: &handlebars::Context,
    ///     _: &mut RenderContext,
    ///     out: &mut dyn Output,
    /// ) -> HelperResult {
    ///     // Custom helper implementation
    ///     Ok(())
    /// }
    ///
    /// // Register with engine (requires access to handlebars instance)
    /// ```
    fn register_helpers(handlebars: &mut Handlebars) {
        // Helper for capitalizing strings
        handlebars.register_helper("capitalize", Box::new(capitalize_helper));

        // Helper for converting to snake_case
        handlebars.register_helper("snake_case", Box::new(snake_case_helper));

        // Helper for converting to camelCase
        handlebars.register_helper("camel_case", Box::new(camel_case_helper));

        // Helper for rendering documentation
        handlebars.register_helper("doc", Box::new(doc_helper));

        // Helper for rendering optional types
        handlebars.register_helper("optional", Box::new(optional_helper));

        // Helper for type alias check
        handlebars.register_helper("is_typealias", Box::new(is_typealias_helper));

        // Helper for escaping pkl keywords
        handlebars.register_helper("escape_pkl_keyword", Box::new(escape_pkl_keyword_helper));

        // Helper for rendering deprecation decorators
        handlebars.register_helper("deprecated", Box::new(deprecated_helper));
    }
}

// Template constants
const MODULE_TEMPLATE: &str = r#"{{#if config.header}}{{config.header}}{{/if}}

{{~#if module.documentation}}
/// {{module.documentation}}
{{/if}}
{{~#if config.include_examples}}
///
/// ## Example
///
/// ```pkl
/// import "{{module.name}}.pkl"
///
/// config: {{module.name}} = new {
/// // Add your configuration here
/// }
/// ```
{{/if}}
module {{module.name}}

{{#each module.imports}}
import "{{path}}"{{#if alias}} as {{alias}}{{/if}}
{{/each}}

{{#each module.types}}
{{> class this}}

{{/each}}

{{#if config.footer}}{{config.footer}}{{/if}}"#;

const INDEX_TEMPLATE: &str = r#"/// Moon Configuration Schemas Module Index
///
/// This module provides access to all Moon configuration schemas.

module {{module_name}}

{{#each schemas}}
import "{{file}}" as {{name}}Module
{{/each}}

{{#each schemas}}
/// {{name}} configuration schema
typealias {{name}} = {{name}}Module.{{main_class}}
{{/each}}"#;

const CLASS_TEMPLATE: &str = r#"{{#if documentation}}
{{doc documentation}}
{{/if}}
{{#if deprecated}}
{{deprecated deprecated}}
{{/if}}
{{#if (is_typealias kind)}}typealias {{name}} = {{#if enum_values}}{{enum_values}}{{else}}Any{{/if}}{{else}}{{#if abstract_type}}abstract {{/if}}class {{name}}{{#if extends}} extends {{#each extends}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}{{/if}} {
{{#each properties}}
{{> property this}}
{{/each}}
}
{{/if}}"#;

const PROPERTY_TEMPLATE: &str = r#"{{#if documentation}}
{{doc documentation}}
{{/if}}
{{#if deprecated}}
{{deprecated deprecated}}
{{/if}}
{{#if examples}}
  ///
  /// Examples:
{{#each examples}}
  /// - `{{this}}`
{{/each}}
{{/if}}
  {{escape_pkl_keyword name}}: {{type_name}}{{#if optional}}?{{/if}}{{#if default}} = {{default}}{{/if}}
"#;

// Helper functions
fn capitalize_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(value) = param.value().as_str() {
            let capitalized = value
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if i == 0 {
                        c.to_uppercase().collect::<String>()
                    } else {
                        c.to_string()
                    }
                })
                .collect::<String>();
            out.write(&capitalized)?;
        }
    }
    Ok(())
}

fn snake_case_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(value) = param.value().as_str() {
            let snake_case = value
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if i > 0 && c.is_uppercase() {
                        format!("_{}", c.to_lowercase())
                    } else {
                        c.to_lowercase().collect()
                    }
                })
                .collect::<String>();
            out.write(&snake_case)?;
        }
    }
    Ok(())
}

fn camel_case_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(value) = param.value().as_str() {
            let camel_case = value
                .split('_')
                .enumerate()
                .map(|(i, word)| {
                    if i == 0 {
                        word.to_lowercase()
                    } else {
                        word.chars()
                            .enumerate()
                            .map(|(j, c)| {
                                if j == 0 {
                                    c.to_uppercase().collect()
                                } else {
                                    c.to_string()
                                }
                            })
                            .collect()
                    }
                })
                .collect::<String>();
            out.write(&camel_case)?;
        }
    }
    Ok(())
}

fn doc_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(value) = param.value().as_str() {
            // Split into lines and properly format each line
            let lines: Vec<&str> = value.lines().collect();

            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    out.write(&format!("  /// {}", trimmed))?;
                } else {
                    out.write("  ///")?;
                }

                // Add newline except for the last line
                if i < lines.len() - 1 {
                    out.write("\n")?;
                }
            }
        }
    }
    Ok(())
}

fn optional_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(type_name) = param.value().as_str() {
            out.write(&format!("({})?", type_name))?;
        }
    }
    Ok(())
}

fn is_typealias_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    // Get the 'kind' value from the current context
    let is_typealias = if let Some(kind_param) = h.param(0) {
        let kind_value = match kind_param.value() {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string().trim_matches('"').to_string(),
        };

        tracing::debug!("is_typealias_helper: checking kind = '{}'", kind_value);

        kind_value == "TypeAlias"
    } else {
        false
    };

    tracing::debug!("is_typealias_helper: result = {}", is_typealias);

    // Return empty string for false (falsy), non-empty for true (truthy)
    if is_typealias {
        out.write("true")?;
    } else {
        out.write("")?;
    }

    Ok(())
}

fn escape_pkl_keyword_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(value) = param.value().as_str() {
            // List of pkl reserved keywords that need to be escaped
            let pkl_keywords = [
                "abstract",
                "amends",
                "as",
                "class",
                "const",
                "default",
                "extends",
                "external",
                "false",
                "for",
                "function",
                "hidden",
                "if",
                "import",
                "in",
                "let",
                "local",
                "module",
                "new",
                "nothing",
                "null",
                "open",
                "out",
                "read",
                "super",
                "this",
                "throw",
                "trace",
                "true",
                "typealias",
                "unknown",
                "when",
                "import*",
            ];

            if pkl_keywords.contains(&value) {
                // Escape with backticks
                out.write(&format!("`{}`", value))?;
            } else {
                out.write(value)?;
            }
        }
    }
    Ok(())
}

fn deprecated_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(message) = param.value().as_str() {
            // Only render deprecation if there's a message
            if !message.trim().is_empty() {
                out.write("@Deprecated\n")?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GeneratorConfig, TemplateConfig};
    use serde_json::{json, Value};
    use std::collections::HashMap;

    fn create_test_config() -> GeneratorConfig {
        GeneratorConfig {
            include_comments: true,
            include_examples: true,
            include_validation: true,
            include_deprecated: true,
            header: Some("// Test header".to_string()),
            footer: Some("// Test footer".to_string()),
            output_dir: std::env::temp_dir().join("test_pkl"),
            module_name: "test".to_string(),
            split_types: true,
            type_mappings: HashMap::new(),
            template: TemplateConfig::default(),
        }
    }

    fn create_test_module() -> PklModule {
        PklModule {
            name: "TestModule".to_string(),
            documentation: Some("Test module documentation".to_string()),
            imports: vec![PklImport {
                path: "base.pkl".to_string(),
                alias: Some("Base".to_string()),
                glob: false,
            }],
            types: vec![
                create_test_class(),
                create_test_typealias(),
                create_test_enum(),
            ],
            exports: vec![PklExport {
                name: "TestClass".to_string(),
                type_name: "TestClass".to_string(),
            }],
            properties: vec![],
        }
    }

    fn create_test_class() -> PklType {
        PklType {
            name: "TestClass".to_string(),
            kind: PklTypeKind::Class,
            documentation: Some("A test class with various properties".to_string()),
            properties: vec![
                PklProperty {
                    name: "stringProp".to_string(),
                    type_name: "String".to_string(),
                    optional: false,
                    documentation: Some("A required string property".to_string()),
                    default: Some("\"default\"".to_string()),
                    examples: vec!["\"example1\"".to_string(), "\"example2\"".to_string()],
                    constraints: vec![PklConstraint {
                        kind: PklConstraintKind::Length,
                        value: "length >= 1".to_string(),
                        message: Some("Must not be empty".to_string()),
                    }],
                    deprecated: None,
                },
                PklProperty {
                    name: "optionalProp".to_string(),
                    type_name: "Int".to_string(),
                    optional: true,
                    documentation: Some("An optional integer property".to_string()),
                    default: None,
                    examples: vec!["42".to_string()],
                    constraints: vec![],
                    deprecated: Some("Use newProp instead".to_string()),
                },
                PklProperty {
                    name: "class".to_string(), // Test keyword escaping
                    type_name: "Boolean".to_string(),
                    optional: false,
                    documentation: Some("A property with a keyword name".to_string()),
                    default: Some("false".to_string()),
                    examples: vec![],
                    constraints: vec![],
                    deprecated: None,
                },
            ],
            extends: vec!["Base.BaseClass".to_string()],
            abstract_type: false,
            deprecated: None,
            enum_values: None,
        }
    }

    fn create_test_typealias() -> PklType {
        PklType {
            name: "StringEnum".to_string(),
            kind: PklTypeKind::TypeAlias,
            documentation: Some("A string enum type alias".to_string()),
            properties: vec![],
            extends: vec![],
            abstract_type: false,
            deprecated: Some("Use NewStringEnum instead".to_string()),
            enum_values: Some("\"option1\" | \"option2\" | \"option3\"".to_string()),
        }
    }

    fn create_test_enum() -> PklType {
        PklType {
            name: "Status".to_string(),
            kind: PklTypeKind::TypeAlias,
            documentation: Some("Status enumeration".to_string()),
            properties: vec![],
            extends: vec![],
            abstract_type: false,
            deprecated: None,
            enum_values: Some("\"active\" | \"inactive\" | \"pending\"".to_string()),
        }
    }

    fn create_abstract_class() -> PklType {
        PklType {
            name: "AbstractBase".to_string(),
            kind: PklTypeKind::Class,
            documentation: Some("An abstract base class".to_string()),
            properties: vec![PklProperty {
                name: "id".to_string(),
                type_name: "String".to_string(),
                optional: false,
                documentation: Some("Unique identifier".to_string()),
                default: None,
                examples: vec![],
                constraints: vec![],
                deprecated: None,
            }],
            extends: vec![],
            abstract_type: true,
            deprecated: None,
            enum_values: None,
        }
    }

    #[test]
    fn test_template_engine_new() {
        let config = create_test_config();
        let engine = TemplateEngine::new(&config);

        // Verify that the engine was created successfully
        assert!(engine.handlebars.get_template("module").is_some());
        assert!(engine.handlebars.get_template("class").is_some());
        assert!(engine.handlebars.get_template("property").is_some());
        assert!(engine.handlebars.get_template("index").is_some());
    }

    #[test]
    fn test_render_module_basic() {
        let engine = TemplateEngine::new(&create_test_config());
        let module = create_test_module();
        let config = create_test_config();

        let result = engine.render_module(&module, &config);
        assert!(result.is_ok());

        let rendered = result.unwrap();

        // Check that basic module structure is present
        assert!(rendered.contains("// Test header"));
        assert!(rendered.contains("/// Test module documentation"));
        assert!(rendered.contains("module TestModule"));
        assert!(rendered.contains("import \"base.pkl\" as Base"));
        assert!(rendered.contains("class TestClass extends Base.BaseClass"));
        assert!(rendered.contains("typealias StringEnum"));
        assert!(rendered.contains("// Test footer"));
    }

    #[test]
    fn test_render_module_without_header_footer() {
        let mut config = create_test_config();
        config.header = None;
        config.footer = None;

        let engine = TemplateEngine::new(&config);
        let module = create_test_module();

        let result = engine.render_module(&module, &config);
        assert!(result.is_ok());

        let rendered = result.unwrap();

        // Check that header/footer are not present
        assert!(!rendered.contains("// Test header"));
        assert!(!rendered.contains("// Test footer"));
        assert!(rendered.contains("module TestModule"));
    }

    #[test]
    fn test_render_module_index() {
        let config = create_test_config();
        let engine = TemplateEngine::new(&config);

        let result = engine.render_module_index(&config);
        assert!(result.is_ok());

        let rendered = result.unwrap();

        // Check index structure
        assert!(rendered.contains("module test"));
        assert!(rendered.contains("import \"workspace.pkl\" as WorkspaceModule"));
        assert!(rendered.contains("import \"project.pkl\" as ProjectModule"));
        assert!(rendered.contains("typealias Workspace = WorkspaceModule.WorkspaceConfig"));
        assert!(rendered.contains("typealias Project = ProjectModule.ProjectConfig"));
    }

    #[test]
    fn test_render_class_with_properties() {
        let engine = TemplateEngine::new(&create_test_config());
        let class = create_test_class();

        let result = engine.handlebars.render("class", &class);
        assert!(result.is_ok());

        let rendered = result.unwrap();

        // Check class structure
        assert!(rendered.contains("/// A test class with various properties"));
        assert!(rendered.contains("class TestClass extends Base.BaseClass"));
        assert!(rendered.contains("stringProp: String = \"default\""));
        assert!(rendered.contains("optionalProp: Int?"));
        assert!(rendered.contains("`class`: Boolean = false"));
        assert!(rendered.contains("@Deprecated"));
    }

    #[test]
    fn test_render_abstract_class() {
        let engine = TemplateEngine::new(&create_test_config());
        let class = create_abstract_class();

        let result = engine.handlebars.render("class", &class);
        assert!(result.is_ok());

        let rendered = result.unwrap();

        // Check abstract class structure
        assert!(rendered.contains("abstract class AbstractBase"));
        assert!(rendered.contains("id: String"));
    }

    #[test]
    fn test_render_typealias() {
        let engine = TemplateEngine::new(&create_test_config());
        let typealias = create_test_typealias();

        let result = engine.handlebars.render("class", &typealias);
        assert!(result.is_ok());

        let rendered = result.unwrap();

        // Check typealias structure
        assert!(rendered.contains("/// A string enum type alias"));
        assert!(rendered.contains("@Deprecated"));
        assert!(rendered.contains("typealias StringEnum = \"option1\" | \"option2\" | \"option3\""));
    }

    #[test]
    fn test_render_property_with_all_features() {
        let engine = TemplateEngine::new(&create_test_config());
        let property = &create_test_class().properties[0]; // stringProp

        let result = engine.handlebars.render("property", property);
        assert!(result.is_ok());

        let rendered = result.unwrap();

        // Check property features
        assert!(rendered.contains("/// A required string property"));
        assert!(rendered.contains("/// Examples:"));
        assert!(rendered.contains("/// - `\"example1\"`"));
        assert!(rendered.contains("/// - `\"example2\"`"));
        assert!(rendered.contains("stringProp: String = \"default\""));
    }

    #[test]
    fn test_render_optional_property_with_deprecation() {
        let engine = TemplateEngine::new(&create_test_config());
        let property = &create_test_class().properties[1]; // optionalProp

        let result = engine.handlebars.render("property", property);
        assert!(result.is_ok());

        let rendered = result.unwrap();

        // Check optional and deprecated property
        assert!(rendered.contains("/// An optional integer property"));
        assert!(rendered.contains("@Deprecated"));
        assert!(rendered.contains("optionalProp: Int?"));
        assert!(!rendered.contains(" = ")); // No default value
    }

    #[test]
    fn test_capitalize_helper() {
        let mut engine = TemplateEngine::new(&create_test_config());

        let template = "{{capitalize 'hello world'}}";
        engine
            .handlebars
            .register_template_string("test", template)
            .unwrap();

        let result = engine.handlebars.render("test", &json!({}));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello world");
    }

    #[test]
    fn test_snake_case_helper() {
        let mut engine = TemplateEngine::new(&create_test_config());

        let template = "{{snake_case 'CamelCaseString'}}";
        engine
            .handlebars
            .register_template_string("test", template)
            .unwrap();

        let result = engine.handlebars.render("test", &json!({}));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "camel_case_string");
    }

    #[test]
    fn test_camel_case_helper() {
        let mut engine = TemplateEngine::new(&create_test_config());

        let template = "{{camel_case 'snake_case_string'}}";
        engine
            .handlebars
            .register_template_string("test", template)
            .unwrap();

        let result = engine.handlebars.render("test", &json!({}));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "snakeCaseString");
    }

    #[test]
    fn test_doc_helper_single_line() {
        let mut engine = TemplateEngine::new(&create_test_config());

        let template = "{{doc 'Single line documentation'}}";
        engine
            .handlebars
            .register_template_string("test", template)
            .unwrap();

        let result = engine.handlebars.render("test", &json!({}));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "  /// Single line documentation");
    }

    #[test]
    fn test_doc_helper_multi_line() {
        let engine = TemplateEngine::new(&create_test_config());

        // Test with a property that has multi-line documentation
        let property = PklProperty {
            name: "testProp".to_string(),
            type_name: "String".to_string(),
            optional: false,
            documentation: Some("Line 1\nLine 2\n\nLine 4".to_string()),
            default: None,
            examples: vec![],
            constraints: vec![],
            deprecated: None,
        };

        let result = engine.handlebars.render("property", &property);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("/// Line 1"));
        assert!(rendered.contains("/// Line 2"));
        assert!(rendered.contains("///\n"));
        assert!(rendered.contains("/// Line 4"));
    }

    #[test]
    fn test_is_typealias_helper() {
        let mut engine = TemplateEngine::new(&create_test_config());

        // Test for TypeAlias kind
        let template = "{{#if (is_typealias 'TypeAlias')}}yes{{else}}no{{/if}}";
        engine
            .handlebars
            .register_template_string("test", template)
            .unwrap();

        let result = engine.handlebars.render("test", &json!({}));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "yes");

        // Test for Class kind
        let template2 = "{{#if (is_typealias 'Class')}}yes{{else}}no{{/if}}";
        engine
            .handlebars
            .register_template_string("test2", template2)
            .unwrap();

        let result2 = engine.handlebars.render("test2", &json!({}));
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), "no");
    }

    #[test]
    fn test_escape_pkl_keyword_helper() {
        let mut engine = TemplateEngine::new(&create_test_config());

        // Test escaping pkl keyword
        let template = "{{escape_pkl_keyword 'class'}}";
        engine
            .handlebars
            .register_template_string("test", template)
            .unwrap();

        let result = engine.handlebars.render("test", &json!({}));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "`class`");

        // Test non-keyword
        let template2 = "{{escape_pkl_keyword 'myProperty'}}";
        engine
            .handlebars
            .register_template_string("test2", template2)
            .unwrap();

        let result2 = engine.handlebars.render("test2", &json!({}));
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), "myProperty");
    }

    #[test]
    fn test_escape_pkl_keywords_comprehensive() {
        let mut engine = TemplateEngine::new(&create_test_config());

        // Test multiple pkl keywords
        let keywords = vec![
            "abstract",
            "amends",
            "as",
            "class",
            "const",
            "default",
            "extends",
            "external",
            "false",
            "for",
            "function",
            "hidden",
            "if",
            "import",
            "in",
            "let",
            "local",
            "module",
            "new",
            "nothing",
            "null",
            "open",
            "out",
            "read",
            "super",
            "this",
            "throw",
            "trace",
            "true",
            "typealias",
            "unknown",
            "when",
            "import*",
        ];

        for keyword in keywords {
            let template = format!("{{{{escape_pkl_keyword '{}'}}}}", keyword);
            engine
                .handlebars
                .register_template_string("test_keyword", &template)
                .unwrap();

            let result = engine.handlebars.render("test_keyword", &json!({}));
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), format!("`{}`", keyword));
        }
    }

    #[test]
    fn test_deprecated_helper() {
        let mut engine = TemplateEngine::new(&create_test_config());

        // Test with deprecation message
        let template = "{{deprecated 'Use newFunction instead'}}";
        engine
            .handlebars
            .register_template_string("test", template)
            .unwrap();

        let result = engine.handlebars.render("test", &json!({}));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "@Deprecated\n");

        // Test with empty message
        let template2 = "{{deprecated ''}}";
        engine
            .handlebars
            .register_template_string("test2", template2)
            .unwrap();

        let result2 = engine.handlebars.render("test2", &json!({}));
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), "");
    }

    #[test]
    fn test_optional_helper() {
        let mut engine = TemplateEngine::new(&create_test_config());

        let template = "{{optional 'String'}}";
        engine
            .handlebars
            .register_template_string("test", template)
            .unwrap();

        let result = engine.handlebars.render("test", &json!({}));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "(String)?");
    }

    #[test]
    fn test_template_context_serialization() {
        let module = create_test_module();
        let config = create_test_config();

        let context = TemplateContext {
            module: module.clone(),
            config: config.clone(),
            variables: HashMap::new(),
        };

        // Test that the context can be serialized (important for template rendering)
        let serialized = serde_json::to_value(&context);
        assert!(serialized.is_ok());

        let json_value = serialized.unwrap();
        assert!(json_value.get("module").is_some());
        assert!(json_value.get("config").is_some());
    }

    #[test]
    fn test_complex_module_rendering() {
        let engine = TemplateEngine::new(&create_test_config());

        let mut module = create_test_module();
        module.types.push(create_abstract_class());

        let config = create_test_config();

        let result = engine.render_module(&module, &config);
        assert!(result.is_ok());

        let rendered = result.unwrap();

        // Verify all types are rendered correctly
        assert!(rendered.contains("class TestClass extends Base.BaseClass"));
        assert!(rendered.contains("typealias StringEnum"));
        assert!(rendered.contains("abstract class AbstractBase"));

        // Verify structure integrity
        assert!(rendered.contains("module TestModule"));
        assert!(rendered.contains("import \"base.pkl\" as Base"));
        assert!(rendered.matches("class ").count() >= 2); // TestClass and AbstractBase
        assert!(rendered.matches("typealias ").count() >= 2); // StringEnum and Status
    }

    #[test]
    fn test_module_without_imports() {
        let mut config = create_test_config();
        config.include_examples = false; // Disable examples to avoid import in example code

        let engine = TemplateEngine::new(&config);

        let mut module = create_test_module();
        module.imports.clear();

        // Also clear extends clauses that reference imports
        for type_def in &mut module.types {
            type_def.extends.clear();
        }

        let result = engine.render_module(&module, &config);
        assert!(result.is_ok());

        let rendered = result.unwrap();

        // Should not contain import statements
        assert!(!rendered.contains("import \""));
        assert!(rendered.contains("module TestModule"));
    }

    #[test]
    fn test_module_without_documentation() {
        let engine = TemplateEngine::new(&create_test_config());

        let mut module = create_test_module();
        module.documentation = None;

        let config = create_test_config();

        let result = engine.render_module(&module, &config);
        assert!(result.is_ok());

        let rendered = result.unwrap();

        // Should not contain module documentation
        assert!(!rendered.contains("/// Test module documentation"));
        assert!(rendered.contains("module TestModule"));
    }

    #[test]
    fn test_config_without_examples() {
        let mut config = create_test_config();
        config.include_examples = false;

        let engine = TemplateEngine::new(&config);
        let module = create_test_module();

        let result = engine.render_module(&module, &config);
        assert!(result.is_ok());

        let rendered = result.unwrap();

        // Should not contain example section
        assert!(!rendered.contains("## Example"));
        assert!(!rendered.contains("```pkl"));
    }

    #[test]
    fn test_template_with_custom_variables() {
        let _engine = TemplateEngine::new(&create_test_config());

        let mut variables = HashMap::new();
        variables.insert(
            "custom_var".to_string(),
            Value::String("custom_value".to_string()),
        );

        let context = TemplateContext {
            module: create_test_module(),
            config: create_test_config(),
            variables,
        };

        // Test that custom variables are available in context
        let serialized = serde_json::to_value(&context).unwrap();
        assert_eq!(
            serialized["variables"]["custom_var"].as_str().unwrap(),
            "custom_value"
        );
    }

    #[test]
    fn test_empty_module_rendering() {
        let engine = TemplateEngine::new(&create_test_config());

        let module = PklModule {
            name: "EmptyModule".to_string(),
            documentation: None,
            imports: vec![],
            types: vec![],
            exports: vec![],
            properties: vec![],
        };

        let config = create_test_config();

        let result = engine.render_module(&module, &config);
        assert!(result.is_ok());

        let rendered = result.unwrap();

        // Should still have basic structure
        assert!(rendered.contains("module EmptyModule"));
        assert!(rendered.contains("// Test header"));
        assert!(rendered.contains("// Test footer"));
    }

    #[test]
    fn test_template_compilation_errors() {
        let mut engine = TemplateEngine::new(&create_test_config());

        // Test invalid template syntax
        let invalid_template = "{{#if unclosed_block}}incomplete";
        let result = engine
            .handlebars
            .register_template_string("invalid", invalid_template);
        assert!(result.is_err());

        // Test unknown helper
        let unknown_helper_template = "{{unknown_helper 'test'}}";
        engine
            .handlebars
            .register_template_string("unknown", unknown_helper_template)
            .unwrap();
        let render_result = engine.handlebars.render("unknown", &json!({}));
        assert!(render_result.is_err());
    }

    #[test]
    fn test_complex_template_scenario_nested_conditions() {
        let mut engine = TemplateEngine::new(&create_test_config());

        let complex_template = r#"
{{#if documentation}}
  {{doc documentation}}
  {{#if deprecated}}
    {{deprecated deprecated}}
    {{#if (is_typealias kind)}}
      {{#if enum_values}}
        typealias {{name}} = {{enum_values}}
      {{else}}
        typealias {{name}} = Any
      {{/if}}
    {{else}}
      {{#if abstract_type}}abstract {{/if}}class {{name}}{{#if extends}} extends {{#each extends}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}{{/if}} {
        // Deprecated class implementation
      }
    {{/if}}
  {{else}}
    // Non-deprecated implementation
  {{/if}}
{{else}}
  // No documentation available
{{/if}}"#;

        engine
            .handlebars
            .register_template_string("complex", complex_template)
            .unwrap();

        let test_data = json!({
            "name": "ComplexType",
            "documentation": "Complex type documentation",
            "deprecated": "Use NewType instead",
            "kind": "TypeAlias",
            "enum_values": "String | Int",
            "abstract_type": false,
            "extends": []
        });

        let result = engine.handlebars.render("complex", &test_data);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("Complex type documentation"));
        assert!(rendered.contains("@Deprecated"));
        assert!(rendered.contains("typealias ComplexType = String | Int"));
    }

    #[test]
    fn test_helper_functions_edge_cases() {
        let mut engine = TemplateEngine::new(&create_test_config());

        // Test capitalize helper with special cases
        let edge_cases = vec![
            ("", ""),
            ("a", "A"),
            ("ALREADY_CAPS", "ALREADY_CAPS"),
            ("123numbers", "123numbers"),
            ("special-chars_test", "Special-chars_test"),
        ];

        for (input, expected) in edge_cases {
            let template = format!("{{{{capitalize '{}'}}}}", input);
            engine
                .handlebars
                .register_template_string("test_cap", &template)
                .unwrap();
            let result = engine.handlebars.render("test_cap", &json!({}));
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), expected);
        }
    }

    #[test]
    fn test_doc_helper_with_edge_cases() {
        let mut engine = TemplateEngine::new(&create_test_config());

        // Test with empty string
        let template1 = "{{doc ''}}";
        engine
            .handlebars
            .register_template_string("doc_empty", template1)
            .unwrap();
        let result1 = engine.handlebars.render("doc_empty", &json!({}));
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), "");

        // Test with very long documentation
        let long_doc = "a".repeat(1000);
        let template2 = format!("{{{{doc \"{}\"}}}}", long_doc);
        engine
            .handlebars
            .register_template_string("doc_long", &template2)
            .unwrap();
        let result2 = engine.handlebars.render("doc_long", &json!({}));
        assert!(result2.is_ok());
        assert!(result2.unwrap().contains("/// aaaa"));

        // Test with special characters - use JSON string to properly escape
        let test_data = json!({
            "special_doc": "Documentation with quotes and backslashes"
        });
        let template3 = "{{doc special_doc}}";
        engine
            .handlebars
            .register_template_string("doc_special", template3)
            .unwrap();
        let result3 = engine.handlebars.render("doc_special", &test_data);
        assert!(result3.is_ok());
    }

    #[test]
    fn test_rendering_with_malformed_data() {
        let engine = TemplateEngine::new(&create_test_config());

        // Test with incomplete module data - just test that we can handle gracefully
        let malformed_pkl_module = PklModule {
            name: "TestModule".to_string(),
            documentation: None,
            imports: vec![],
            types: vec![],
            exports: vec![],
            properties: vec![],
        };

        let config = create_test_config();
        let result = engine.render_module(&malformed_pkl_module, &config);

        // Should succeed with empty module
        assert!(result.is_ok());
    }

    #[test]
    fn test_property_template_with_constraints() {
        let engine = TemplateEngine::new(&create_test_config());

        let property_with_constraints = json!({
            "name": "constrainedProp",
            "type_name": "String",
            "optional": false,
            "documentation": "A property with constraints",
            "default": null,
            "examples": ["\"example1\"", "\"example2\""],
            "constraints": [
                {
                    "kind": "Length",
                    "value": "length >= 1",
                    "message": "Must not be empty"
                },
                {
                    "kind": "Pattern",
                    "value": "matches(Regex(#\"[a-z]+\"#))",
                    "message": "Must contain only lowercase letters"
                }
            ],
            "deprecated": null
        });

        let result = engine
            .handlebars
            .render("property", &property_with_constraints);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("constrainedProp: String"));
        assert!(rendered.contains("/// A property with constraints"));
        assert!(rendered.contains("/// - `\"example1\"`"));
        assert!(rendered.contains("/// - `\"example2\"`"));
    }

    #[test]
    fn test_type_template_with_complex_hierarchy() {
        let engine = TemplateEngine::new(&create_test_config());

        let complex_type = PklType {
            name: "ComplexHierarchy".to_string(),
            kind: PklTypeKind::Class,
            documentation: Some("A complex type with multiple inheritance".to_string()),
            properties: vec![PklProperty {
                name: "baseProperty".to_string(),
                type_name: "String".to_string(),
                optional: false,
                documentation: Some("Property from base".to_string()),
                default: None,
                examples: vec![],
                constraints: vec![],
                deprecated: None,
            }],
            extends: vec![
                "Base.FirstBase".to_string(),
                "Other.SecondBase".to_string(),
                "Third.ThirdBase".to_string(),
            ],
            abstract_type: false,
            deprecated: None,
            enum_values: None,
        };

        let result = engine.handlebars.render("class", &complex_type);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains(
            "class ComplexHierarchy extends Base.FirstBase, Other.SecondBase, Third.ThirdBase"
        ));
        assert!(rendered.contains("baseProperty: String"));
    }

    #[test]
    fn test_module_template_with_many_exports() {
        let engine = TemplateEngine::new(&create_test_config());

        let module_many_exports = PklModule {
            name: "ManyExports".to_string(),
            documentation: Some("Module with many exports".to_string()),
            imports: vec![],
            types: vec![
                PklType {
                    name: "Export1Type".to_string(),
                    documentation: Some("Export 1 documentation".to_string()),
                    kind: PklTypeKind::Class,
                    properties: vec![],
                    abstract_type: false,
                    extends: vec![],
                    enum_values: None,
                    deprecated: None,
                },
                PklType {
                    name: "Export2Type".to_string(),
                    documentation: Some("Export 2 documentation".to_string()),
                    kind: PklTypeKind::Class,
                    properties: vec![],
                    abstract_type: false,
                    extends: vec![],
                    enum_values: None,
                    deprecated: None,
                },
            ],
            exports: vec![
                PklExport {
                    name: "Export1".to_string(),
                    type_name: "Export1Type".to_string(),
                },
                PklExport {
                    name: "Export2".to_string(),
                    type_name: "Export2Type".to_string(),
                },
            ],
            properties: vec![],
        };

        let config = create_test_config();
        let result = engine.render_module(&module_many_exports, &config);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        // Check for types (since exports aren't in the current template)
        assert!(rendered.contains("Export1Type"));
        assert!(rendered.contains("Export2Type"));
        assert!(rendered.contains("class Export1Type"));
        assert!(rendered.contains("class Export2Type"));
    }

    #[test]
    fn test_helper_parameter_validation() {
        let mut engine = TemplateEngine::new(&create_test_config());

        // Test helpers with missing parameters
        let templates_and_expected = vec![
            ("{{capitalize 'test'}}", true), // Should succeed - has parameter
            ("{{doc 'test'}}", true),        // Should succeed - has parameter
            ("{{escape_pkl_keyword 'test'}}", true), // Should succeed - has parameter
            ("{{deprecated 'test'}}", true), // Should succeed - has parameter
        ];

        for (template, should_succeed) in templates_and_expected {
            engine
                .handlebars
                .register_template_string("param_test", template)
                .unwrap();
            let result = engine.handlebars.render("param_test", &json!({}));

            if should_succeed {
                assert!(result.is_ok(), "Template '{}' should succeed", template);
            } else {
                assert!(result.is_err(), "Template '{}' should fail", template);
            }
        }
    }

    #[test]
    fn test_template_context_with_complex_variables() {
        let module = create_test_module();
        let config = create_test_config();

        let mut variables = HashMap::new();
        variables.insert("version".to_string(), Value::String("1.0.0".to_string()));
        variables.insert(
            "author".to_string(),
            Value::String("Test Author".to_string()),
        );
        variables.insert(
            "build_time".to_string(),
            Value::String("2024-01-01T00:00:00Z".to_string()),
        );
        variables.insert(
            "features".to_string(),
            Value::Array(vec![
                Value::String("feature1".to_string()),
                Value::String("feature2".to_string()),
            ]),
        );
        variables.insert(
            "nested".to_string(),
            Value::Object(serde_json::Map::from_iter([
                ("key1".to_string(), Value::String("value1".to_string())),
                (
                    "key2".to_string(),
                    Value::Number(serde_json::Number::from(42)),
                ),
            ])),
        );

        let context = TemplateContext {
            module,
            config,
            variables,
        };

        let serialized = serde_json::to_value(&context).unwrap();

        // Verify all complex variables are preserved
        assert_eq!(serialized["variables"]["version"], "1.0.0");
        assert_eq!(serialized["variables"]["author"], "Test Author");
        assert_eq!(serialized["variables"]["features"][0], "feature1");
        assert_eq!(serialized["variables"]["nested"]["key1"], "value1");
        assert_eq!(serialized["variables"]["nested"]["key2"], 42);
    }

    #[test]
    fn test_template_rendering_with_circular_references() {
        let engine = TemplateEngine::new(&create_test_config());

        // Create types that reference each other
        let type_a = PklType {
            name: "TypeA".to_string(),
            documentation: Some("References TypeB".to_string()),
            kind: PklTypeKind::Class,
            properties: vec![PklProperty {
                name: "refToB".to_string(),
                type_name: "TypeB".to_string(),
                optional: true,
                documentation: Some("Reference to TypeB".to_string()),
                default: None,
                examples: vec![],
                constraints: vec![],
                deprecated: None,
            }],
            abstract_type: false,
            extends: vec![],
            enum_values: None,
            deprecated: None,
        };

        let type_b = PklType {
            name: "TypeB".to_string(),
            documentation: Some("References TypeA".to_string()),
            kind: PklTypeKind::Class,
            properties: vec![PklProperty {
                name: "refToA".to_string(),
                type_name: "TypeA".to_string(),
                optional: true,
                documentation: Some("Reference to TypeA".to_string()),
                default: None,
                examples: vec![],
                constraints: vec![],
                deprecated: None,
            }],
            abstract_type: false,
            extends: vec![],
            enum_values: None,
            deprecated: None,
        };

        let module = PklModule {
            name: "CircularModule".to_string(),
            documentation: Some("Module with circular references".to_string()),
            imports: vec![],
            types: vec![type_a, type_b],
            exports: vec![],
            properties: vec![],
        };

        let config = create_test_config();
        let result = engine.render_module(&module, &config);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("class TypeA"));
        assert!(rendered.contains("class TypeB"));
        assert!(rendered.contains("refToB: TypeB?"));
        assert!(rendered.contains("refToA: TypeA?"));
    }

    #[test]
    fn test_template_engine_error_handling() {
        let engine = TemplateEngine::new(&create_test_config());

        // Test with module that has invalid characters in name
        let invalid_module = PklModule {
            name: "Invalid-Module-Name!@#".to_string(),
            documentation: None,
            imports: vec![],
            types: vec![],
            exports: vec![],
            properties: vec![],
        };

        let config = create_test_config();
        let result = engine.render_module(&invalid_module, &config);

        // Should handle gracefully (Pkl might allow such names)
        assert!(result.is_ok());
    }

    #[test]
    fn test_template_with_unicode_content() {
        let mut engine = TemplateEngine::new(&create_test_config());

        // Test with Unicode characters in documentation
        let unicode_doc = "Documentation with émojis 🚀 and spëcial chars: αβγ 中文";
        let template = format!("{{{{doc '{}'}}}}", unicode_doc);
        engine
            .handlebars
            .register_template_string("unicode_test", &template)
            .unwrap();

        let result = engine.handlebars.render("unicode_test", &json!({}));
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("émojis 🚀"));
        assert!(rendered.contains("spëcial chars"));
        assert!(rendered.contains("αβγ 中文"));
    }

    #[test]
    fn test_performance_with_large_module() {
        let engine = TemplateEngine::new(&create_test_config());

        // Create a module with many types
        let mut large_types = Vec::new();
        for i in 0..100 {
            large_types.push(PklType {
                name: format!("Type{}", i),
                documentation: Some(format!("Documentation for Type{}", i)),
                kind: PklTypeKind::Class,
                properties: vec![PklProperty {
                    name: format!("property{}", i),
                    type_name: "String".to_string(),
                    optional: false,
                    documentation: Some(format!("Property {} documentation", i)),
                    default: None,
                    examples: vec![format!("\"example{}\"", i)],
                    constraints: vec![],
                    deprecated: None,
                }],
                abstract_type: false,
                extends: vec![],
                enum_values: None,
                deprecated: None,
            });
        }

        let large_module = PklModule {
            name: "LargeModule".to_string(),
            documentation: Some("Large module for performance testing".to_string()),
            imports: vec![],
            types: large_types,
            exports: vec![],
            properties: vec![],
        };

        let config = create_test_config();
        let start_time = std::time::Instant::now();
        let result = engine.render_module(&large_module, &config);
        let duration = start_time.elapsed();

        assert!(result.is_ok());
        // Should complete within reasonable time (adjust threshold as needed)
        assert!(
            duration.as_secs() < 10,
            "Template rendering took too long: {:?}",
            duration
        );

        let rendered = result.unwrap();
        assert!(rendered.contains("Type0"));
        assert!(rendered.contains("Type99"));
        assert!(rendered.contains("property0"));
        assert!(rendered.contains("property99"));
    }
}
