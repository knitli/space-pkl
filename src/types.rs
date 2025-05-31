//! PKL Type Definitions Module
//!
//! This module provides comprehensive type definitions for representing PKL (Pkl Configuration Language)
//! schema structures in Rust. It serves as the core type system for translating Moon configuration
//! types into PKL schema definitions with full support for documentation, validation, and templates.
//!
//! # Overview
//!
//! The type system is built around a hierarchical structure that mirrors PKL's type system:
//! - **Modules** contain collections of types, imports, and exports
//! - **Types** define PKL classes, type aliases, unions, and modules
//! - **Properties** represent fields within types with validation and documentation
//! - **Constraints** provide runtime validation rules for properties
//! - **Templates** enable customizable code generation contexts
//!
//! # Architecture
//!
//! ```text
//! PklModule
//! ├── imports: Vec<PklImport>        # Module dependencies
//! ├── exports: Vec<PklExport>        # Public API surface
//! ├── types: Vec<PklType>            # Type definitions
//! └── properties: Vec<PklProperty>   # Module-level properties
//!
//! PklType
//! ├── kind: PklTypeKind              # Class, TypeAlias, Union, Module
//! ├── properties: Vec<PklProperty>   # Type members
//! ├── extends: Vec<String>           # Inheritance chain
//! └── constraints: Vec<PklConstraint> # Validation rules
//!
//! PklProperty
//! ├── type_name: String              # PKL type reference
//! ├── constraints: Vec<PklConstraint> # Validation constraints
//! ├── examples: Vec<String>          # Usage examples
//! └── documentation: Option<String>  # Inline documentation
//! ```
//!
//! # Generated PKL Structure
//!
//! The types in this module generate PKL schemas with the following structure:
//!
//! ```pkl
//! /// Module documentation
//! module MyModule
//!
//! import "other.pkl"
//!
//! /// Class documentation
//! class MyClass {
//!   /// Property documentation
//!   /// Examples: "example1", "example2"
//!   name: String(length >= 1)?
//!
//!   /// Validated integer property
//!   count: Int(this >= 0 && this <= 100) = 0
//! }
//!
//! /// Type alias for configuration union
//! typealias ConfigValue = String | Int | Boolean
//!
//! /// Exported types
//! MyClass
//! ConfigValue
//! ```
//!
//! # Features
//!
//! - **Type Safety**: Full type information preserved from Rust to PKL
//! - **Documentation**: Rich documentation with examples and constraints
//! - **Validation**: Constraint-based validation with custom error messages
//! - **Modularity**: Import/export system for schema composition
//! - **Flexibility**: Support for inheritance, generics, and unions
//! - **Templates**: Customizable code generation with template contexts
//!
//! # Usage Examples
//!
//! ## Creating a Simple Type
//! ```rust
//! use space_pkl::types::*;
//!
//! let property = PklProperty {
//!     name: "username".to_string(),
//!     type_name: "String".to_string(),
//!     documentation: Some("User identifier".to_string()),
//!     optional: false,
//!     default: None,
//!     constraints: vec![
//!         PklConstraint {
//!             kind: PklConstraintKind::Length,
//!             value: "length >= 3".to_string(),
//!             message: Some("Username too short".to_string()),
//!         }
//!     ],
//!     examples: vec!["alice".to_string(), "bob_123".to_string()],
//!     deprecated: None,
//! };
//!
//! let user_type = PklType {
//!     name: "User".to_string(),
//!     documentation: Some("User account information".to_string()),
//!     kind: PklTypeKind::Class,
//!     properties: vec![property],
//!     abstract_type: false,
//!     extends: vec![],
//!     enum_values: None,
//!     deprecated: None,
//! };
//! ```
//!
//! ## Creating a Module with Exports
//! ```rust
//! use space_pkl::types::*;
//!
//! let module = PklModule {
//!     name: "UserModule".to_string(),
//!     documentation: Some("User management types".to_string()),
//!     imports: vec![
//!         PklImport {
//!             path: "base.pkl".to_string(),
//!             alias: Some("base".to_string()),
//!             glob: false,
//!         }
//!     ],
//!     exports: vec![
//!         PklExport {
//!             name: "User".to_string(),
//!             type_name: "User".to_string(),
//!         }
//!     ],
//!     types: vec![/* user_type from above */],
//!     properties: vec![],
//! };
//! ```
//!
//! # Type Mapping
//!
//! This module supports comprehensive type mapping from Rust to PKL:
//!
//! | Rust Type | PKL Type | Description |
//! |-----------|----------|-------------|
//! | `String` | `String` | Text values |
//! | `bool` | `Boolean` | True/false values |
//! | `i32`, `i64` | `Int` | Signed integers |
//! | `u32`, `u64` | `UInt` | Unsigned integers |
//! | `f32`, `f64` | `Float` | Floating-point numbers |
//! | `Vec<T>` | `Listing<T>` | Ordered collections |
//! | `HashMap<K,V>` | `Mapping<K,V>` | Key-value maps |
//! | `Option<T>` | `T?` | Optional values |
//! | `enum` | `"variant1" \| "variant2"` | Union types |
//! | `struct` | `class` | Object types |
//!
//! (c) 2025 Stash AI Inc (knitli)
//!   - Created by Adam Poulemanos ([@bashandbone](https://github.com/bashandbone)) for Stash AI Inc.
//! Licensed under the [Plain MIT License](https://plainlicense.org/licenses/permissive/mit/)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a complete PKL module definition.
///
/// `PklModule` is the top-level container for PKL schema definitions, encapsulating
/// all the components needed to generate a complete PKL module file. It serves as
/// the primary data structure for organizing types, managing dependencies, and
/// controlling the public API surface of generated schemas.
///
/// # Structure
///
/// A PKL module contains several key components:
/// - **Module Metadata**: Name and documentation
/// - **Dependency Management**: Import declarations for external modules
/// - **Public API**: Export declarations for types available to other modules
/// - **Type Definitions**: Classes, type aliases, unions, and nested modules
/// - **Module Properties**: Global configuration values and constants
///
/// # Generated PKL Output
///
/// A `PklModule` generates PKL code with the following structure:
/// ```pkl
/// /// Module documentation appears here
/// module ModuleName
///
/// import "dependency1.pkl" as dep1
/// import "dependency2.pkl"
/// import "utils/*"  // glob import
///
/// /// Type documentation
/// class MyType {
///   property: String
/// }
///
/// typealias Status = "active" | "inactive"
///
/// // Module-level properties
/// defaultTimeout: Duration = 30.s
///
/// // Exported types (public API)
/// MyType
/// Status
/// ```
///
/// # Usage Examples
///
/// ## Basic Module Creation
/// ```rust
/// use space_pkl::types::*;
///
/// let module = PklModule {
///     name: "Configuration".to_string(),
///     documentation: Some("Application configuration module".to_string()),
///     imports: vec![],
///     exports: vec![],
///     types: vec![],
///     properties: vec![],
/// };
/// ```
///
/// ## Module with Dependencies
/// ```rust
/// use space_pkl::types::*;
///
/// let module = PklModule {
///     name: "UserModule".to_string(),
///     documentation: Some("User management configuration".to_string()),
///     imports: vec![
///         PklImport {
///             path: "base.pkl".to_string(),
///             alias: Some("base".to_string()),
///             glob: false,
///         },
///         PklImport {
///             path: "utils/*".to_string(),
///             alias: None,
///             glob: true,  // Import all types from utils
///         },
///     ],
///     exports: vec![
///         PklExport {
///             name: "User".to_string(),
///             type_name: "User".to_string(),
///         }
///     ],
///     types: vec![/* type definitions */],
///     properties: vec![/* module-level constants */],
/// };
/// ```
///
/// ## Complete Configuration Module
/// ```rust
/// use space_pkl::types::*;
///
/// // Create a complete module for database configuration
/// let db_module = PklModule {
///     name: "DatabaseConfig".to_string(),
///     documentation: Some("Database connection and pool configuration".to_string()),
///     imports: vec![
///         PklImport {
///             path: "pkl:base".to_string(),
///             alias: None,
///             glob: false,
///         }
///     ],
///     exports: vec![
///         PklExport {
///             name: "DatabaseConfig".to_string(),
///             type_name: "DatabaseConfig".to_string(),
///         },
///         PklExport {
///             name: "PoolConfig".to_string(),
///             type_name: "PoolConfig".to_string(),
///         }
///     ],
///     types: vec![
///         // DatabaseConfig type definition
///         // PoolConfig type definition
///     ],
///     properties: vec![
///         // Default timeout constant
///         // Maximum connection limit
///     ],
/// };
/// ```
///
/// # Module Organization
///
/// Modules can be organized in several ways:
/// - **Single Module**: All types in one file (`moon.pkl`)
/// - **Split Modules**: Each configuration type in separate files (`workspace.pkl`, `project.pkl`, etc.)
/// - **Hierarchical**: Nested modules with imports between them
///
/// # Cross-Module References
///
/// Modules can reference types from other modules through imports:
/// ```pkl
/// module ProjectConfig
///
/// import "workspace.pkl" as workspace
///
/// class Project {
///   // Reference to type from workspace module
///   workspaceConfig: workspace.WorkspaceConfig?
/// }
/// ```
///
/// # Serialization
///
/// `PklModule` implements `Serialize` and `Deserialize` for JSON/YAML persistence:
/// ```rust
/// # use space_pkl::types::*;
/// let module = PklModule { /* ... */ };
/// let json = serde_json::to_string(&module)?;
/// let restored: PklModule = serde_json::from_str(&json)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PklModule {
    /// The name of the PKL module.
    ///
    /// This appears in the `module` declaration at the top of generated PKL files
    /// and affects how the module can be imported by other PKL files.
    ///
    /// # Naming Conventions
    /// - Use PascalCase for module names (e.g., "WorkspaceConfig", "DatabaseSettings")
    /// - Keep names descriptive but concise
    /// - Avoid special characters and spaces
    ///
    /// # Example
    /// ```pkl
    /// module WorkspaceConfig  // <- This comes from the name field
    /// ```
    pub name: String,

    /// Optional documentation for the module.
    ///
    /// When present, this generates PKL doc comments at the top of the module file,
    /// providing context and usage information for the entire module.
    ///
    /// # Format
    /// Documentation supports:
    /// - Multi-line descriptions
    /// - Markdown-style formatting
    /// - Usage examples
    /// - Links to related modules
    ///
    /// # Example PKL Output
    /// ```pkl
    /// /// Application workspace configuration
    /// ///
    /// /// This module defines types for configuring Moon workspaces,
    /// /// including project discovery, global settings, and constraints.
    /// module WorkspaceConfig
    /// ```
    pub documentation: Option<String>,

    /// Import declarations for external modules and dependencies.
    ///
    /// Defines the external modules that this module depends on. Each import
    /// makes types and values from other modules available within this module.
    ///
    /// # Import Types
    /// - **Named imports**: `import "module.pkl" as name`
    /// - **Direct imports**: `import "module.pkl"`
    /// - **Glob imports**: `import "package/*"`
    ///
    /// # Example PKL Output
    /// ```pkl
    /// import "pkl:base"
    /// import "workspace.pkl" as workspace
    /// import "utils/*"
    /// ```
    pub imports: Vec<PklImport>,

    /// Export declarations defining the module's public API.
    ///
    /// Specifies which types and values are publicly available when this
    /// module is imported by other modules. Only exported items are accessible
    /// from outside the module.
    ///
    /// # Export Strategy
    /// - Export main configuration types
    /// - Keep internal/helper types private
    /// - Use descriptive export names
    ///
    /// # Example PKL Output
    /// ```pkl
    /// // At the end of the module file
    /// WorkspaceConfig     // Exported type
    /// ProjectSettings     // Exported type
    /// ```
    pub exports: Vec<PklExport>,

    /// Type definitions contained within this module.
    ///
    /// The core content of the module, containing all class definitions,
    /// type aliases, unions, and nested module types that make up the
    /// module's functionality.
    ///
    /// # Type Organization
    /// Types are typically ordered by:
    /// 1. Core/main types first
    /// 2. Supporting/utility types second
    /// 3. Internal/private types last
    ///
    /// # Example PKL Output
    /// ```pkl
    /// class WorkspaceConfig {
    ///   projects: ProjectsConfig?
    ///   settings: WorkspaceSettings?
    /// }
    ///
    /// typealias LogLevel = "debug" | "info" | "warn" | "error"
    /// ```
    pub types: Vec<PklType>,

    /// Module-level properties and constants.
    ///
    /// Defines global values, constants, and default configurations that are
    /// available throughout the module. These become module-level properties
    /// in the generated PKL.
    ///
    /// # Use Cases
    /// - Default configuration values
    /// - Global constants and limits
    /// - Computed properties
    /// - Validation helpers
    ///
    /// # Example PKL Output
    /// ```pkl
    /// // Module-level properties
    /// defaultTimeout: Duration = 30.s
    /// maxRetries: Int = 3
    /// supportedVersions: Listing<String> = List("1.0", "2.0", "3.0")
    /// ```
    pub properties: Vec<PklProperty>,
}

/// Represents a PKL import statement for module dependencies.
///
/// `PklImport` defines how external modules and their types are made available
/// within the current module. It supports various import patterns including
/// named imports, direct imports, and glob imports for packages.
///
/// # Import Patterns
///
/// ## Named Import with Alias
/// Creates a namespace for the imported module:
/// ```pkl
/// import "workspace.pkl" as workspace
/// // Usage: workspace.WorkspaceConfig
/// ```
///
/// ## Direct Import
/// Imports the module without a namespace:
/// ```pkl
/// import "base.pkl"
/// // Usage: types are directly available
/// ```
///
/// ## Glob Import
/// Imports all modules from a package:
/// ```pkl
/// import "utils/*"
/// // Usage: all types from utils package are available
/// ```
///
/// ## Standard Library Import
/// Imports from PKL standard library:
/// ```pkl
/// import "pkl:base"
/// import "pkl:json"
/// ```
///
/// # Usage Examples
///
/// ## Basic Named Import
/// ```rust
/// use space_pkl::types::*;
///
/// let import = PklImport {
///     path: "workspace.pkl".to_string(),
///     alias: Some("ws".to_string()),
///     glob: false,
/// };
/// // Generates: import "workspace.pkl" as ws
/// ```
///
/// ## Standard Library Import
/// ```rust
/// use space_pkl::types::*;
///
/// let base_import = PklImport {
///     path: "pkl:base".to_string(),
///     alias: None,
///     glob: false,
/// };
/// // Generates: import "pkl:base"
/// ```
///
/// ## Package Glob Import
/// ```rust
/// use space_pkl::types::*;
///
/// let glob_import = PklImport {
///     path: "moon/configs/*".to_string(),
///     alias: None,
///     glob: true,
/// };
/// // Generates: import "moon/configs/*"
/// ```
///
/// ## Multiple Related Imports
/// ```rust
/// use space_pkl::types::*;
///
/// let imports = vec![
///     PklImport {
///         path: "pkl:base".to_string(),
///         alias: None,
///         glob: false,
///     },
///     PklImport {
///         path: "workspace.pkl".to_string(),
///         alias: Some("workspace".to_string()),
///         glob: false,
///     },
///     PklImport {
///         path: "utils/*".to_string(),
///         alias: None,
///         glob: true,
///     },
/// ];
/// ```
///
/// # Import Resolution
///
/// PKL resolves imports in the following order:
/// 1. **Standard Library**: `pkl:*` imports resolve to PKL built-in modules
/// 2. **Relative Paths**: `./module.pkl` or `../other.pkl` relative to current file
/// 3. **Absolute Paths**: `/path/to/module.pkl` from filesystem root
/// 4. **Package Paths**: `package/module.pkl` from configured package directories
///
/// # Best Practices
///
/// ## Import Organization
/// ```rust
/// // Order imports by type:
/// // 1. Standard library imports
/// // 2. External package imports
/// // 3. Local project imports
/// let organized_imports = vec![
///     // Standard library
///     PklImport { path: "pkl:base".to_string(), alias: None, glob: false },
///
///     // External packages
///     PklImport { path: "external/package.pkl".to_string(), alias: Some("ext".to_string()), glob: false },
///
///     // Local modules
///     PklImport { path: "workspace.pkl".to_string(), alias: Some("workspace".to_string()), glob: false },
/// ];
/// ```
///
/// ## Alias Naming
/// - Use short, descriptive aliases: `workspace`, `db`, `auth`
/// - Avoid single-letter aliases except for very common modules
/// - Use consistent naming across related modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PklImport {
    /// The path to the module being imported.
    ///
    /// Specifies the location of the module to import. Can be:
    /// - **Relative path**: `"./config.pkl"`, `"../shared/types.pkl"`
    /// - **Absolute path**: `"/usr/local/pkl/base.pkl"`
    /// - **Package path**: `"mypackage/module.pkl"`
    /// - **Standard library**: `"pkl:base"`, `"pkl:json"`
    /// - **Glob pattern**: `"utils/*"`, `"configs/**/*.pkl"`
    ///
    /// # Path Examples
    /// ```pkl
    /// import "pkl:base"                    // Standard library
    /// import "workspace.pkl"               // Relative to current module
    /// import "./configs/database.pkl"      // Explicit relative path
    /// import "/etc/pkl/system.pkl"         // Absolute path
    /// import "mypackage/types.pkl"         // Package-relative path
    /// import "utils/*"                     // Glob import
    /// ```
    pub path: String,

    /// Optional alias for the imported module.
    ///
    /// When provided, creates a namespace for accessing types from the imported
    /// module. Without an alias, imported types are available directly in the
    /// current namespace.
    ///
    /// # Usage
    /// - **With alias**: Types accessed as `alias.TypeName`
    /// - **Without alias**: Types accessed directly as `TypeName`
    ///
    /// # Examples
    /// ```pkl
    /// import "workspace.pkl" as ws    // With alias
    /// let config: ws.WorkspaceConfig  // Usage with alias
    ///
    /// import "types.pkl"              // Without alias
    /// let user: User                  // Direct usage
    /// ```
    pub alias: Option<String>,

    /// Whether this is a glob import that imports multiple modules.
    ///
    /// When `true`, the import path is treated as a glob pattern that matches
    /// multiple modules. All matching modules are imported and their types
    /// become available in the current namespace.
    ///
    /// # Glob Patterns
    /// - `"utils/*"` - All immediate children of utils directory
    /// - `"configs/**/*.pkl"` - All PKL files recursively under configs
    /// - `"types/*.pkl"` - All PKL files directly in types directory
    ///
    /// # Example
    /// ```rust
    /// # use space_pkl::types::*;
    /// let glob_import = PklImport {
    ///     path: "shared/types/*".to_string(),
    ///     alias: None,
    ///     glob: true,
    /// };
    /// // Imports all modules from shared/types/ directory
    /// ```
    ///
    /// # Note
    /// Glob imports cannot be combined with aliases. When `glob` is `true`,
    /// the `alias` field should be `None`.
    pub glob: bool,
}

/// Represents a PKL export declaration for module's public API.
///
/// `PklExport` defines which types and values are publicly accessible when
/// this module is imported by other modules. Only exported items can be
/// referenced from outside the module, providing encapsulation and API control.
///
/// # Export Mechanism
///
/// PKL exports work by listing type names at the end of a module file:
/// ```pkl
/// module MyModule
///
/// class InternalType { ... }     // Not exported (private)
/// class PublicType { ... }       // Will be exported
///
/// typealias Status = "ok" | "error"  // Will be exported
///
/// // Export declarations (at end of module)
/// PublicType    // Makes PublicType available to importers
/// Status        // Makes Status alias available to importers
/// ```
///
/// # Usage Examples
///
/// ## Basic Type Export
/// ```rust
/// use space_pkl::types::*;
///
/// let export = PklExport {
///     name: "DatabaseConfig".to_string(),
///     type_name: "DatabaseConfig".to_string(),
/// };
/// // Generates: DatabaseConfig
/// ```
///
/// ## Export with Different Name
/// ```rust
/// use space_pkl::types::*;
///
/// let export = PklExport {
///     name: "Config".to_string(),           // Public name
///     type_name: "DatabaseConfigImpl".to_string(),  // Internal type name
/// };
/// // Generates: Config = DatabaseConfigImpl
/// ```
///
/// ## Multiple Exports for a Module
/// ```rust
/// use space_pkl::types::*;
///
/// let exports = vec![
///     PklExport {
///         name: "WorkspaceConfig".to_string(),
///         type_name: "WorkspaceConfig".to_string(),
///     },
///     PklExport {
///         name: "ProjectConfig".to_string(),
///         type_name: "ProjectConfig".to_string(),
///     },
///     PklExport {
///         name: "LogLevel".to_string(),
///         type_name: "LogLevel".to_string(),
///     },
/// ];
/// ```
///
/// # Export Strategies
///
/// ## Explicit API Design
/// ```rust
/// // Export only essential types for clean API
/// let public_api = vec![
///     PklExport {
///         name: "Config".to_string(),        // Main configuration type
///         type_name: "DatabaseConfig".to_string(),
///     },
///     PklExport {
///         name: "ConnectionPool".to_string(), // Supporting type
///         type_name: "ConnectionPool".to_string(),
///     },
///     // Note: Internal types like DatabaseConnection are NOT exported
/// ];
/// ```
///
/// ## Re-exporting with Aliases
/// ```rust
/// // Provide simplified names for complex internal types
/// let simplified_exports = vec![
///     PklExport {
///         name: "DB".to_string(),            // Simple alias
///         type_name: "DatabaseConfiguration".to_string(),  // Complex internal name
///     },
///     PklExport {
///         name: "Pool".to_string(),          // Simple alias
///         type_name: "ConnectionPoolSettings".to_string(), // Complex internal name
///     },
/// ];
/// ```
///
/// # Access Control
///
/// Exports provide fine-grained control over module APIs:
///
/// ## Public vs Private Types
/// ```pkl
/// module DatabaseModule
///
/// // Private types (not exported)
/// class InternalConnectionManager { ... }
/// class DatabaseDriver { ... }
///
/// // Public types (exported)
/// class DatabaseConfig { ... }
/// class PoolSettings { ... }
///
/// // Only public types are accessible from outside
/// DatabaseConfig
/// PoolSettings
/// ```
///
/// ## Versioned APIs
/// ```rust
/// // Export different versions of the same concept
/// let versioned_exports = vec![
///     PklExport {
///         name: "ConfigV1".to_string(),
///         type_name: "DatabaseConfigV1".to_string(),
///     },
///     PklExport {
///         name: "Config".to_string(),        // Default to latest
///         type_name: "DatabaseConfigV2".to_string(),
///     },
/// ];
/// ```
///
/// # Best Practices
///
/// ## Naming Conventions
/// - Use clear, descriptive export names
/// - Follow consistent naming patterns across modules
/// - Prefer full names over abbreviations for clarity
/// - Use PascalCase for type names
///
/// ## API Design
/// - Export minimal necessary surface area
/// - Group related exports logically
/// - Provide aliases for complex internal names
/// - Document breaking changes in export names
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PklExport {
    /// The public name for this export.
    ///
    /// This is the name that other modules will use when importing and
    /// referencing this type. It becomes the identifier in import statements
    /// and type references.
    ///
    /// # Naming Guidelines
    /// - Use PascalCase for type names (e.g., "DatabaseConfig", "LogLevel")
    /// - Keep names descriptive but concise
    /// - Avoid abbreviations unless universally understood
    /// - Use consistent naming patterns across related modules
    ///
    /// # Example Usage
    /// ```pkl
    /// // In the exporting module
    /// DatabaseConfig    // <- This is the export name
    ///
    /// // In an importing module
    /// import "db.pkl"
    /// let config: DatabaseConfig  // <- Uses the export name
    /// ```
    pub name: String,

    /// The internal type name that this export refers to.
    ///
    /// This specifies the actual type definition within the module that
    /// should be made available under the public export name. Usually
    /// this matches the `name` field, but can differ when providing
    /// aliases or simplified names for complex internal types.
    ///
    /// # Use Cases
    ///
    /// ## Direct Export (Most Common)
    /// ```rust
    /// # use space_pkl::types::*;
    /// let export = PklExport {
    ///     name: "DatabaseConfig".to_string(),
    ///     type_name: "DatabaseConfig".to_string(),  // Same as name
    /// };
    /// ```
    ///
    /// ## Aliased Export
    /// ```rust
    /// # use space_pkl::types::*;
    /// let export = PklExport {
    ///     name: "Config".to_string(),                    // Simple public name
    ///     type_name: "DatabaseConfigurationImpl".to_string(),  // Complex internal name
    /// };
    /// ```
    ///
    /// ## Version-specific Export
    /// ```rust
    /// # use space_pkl::types::*;
    /// let export = PklExport {
    ///     name: "Config".to_string(),           // Generic public interface
    ///     type_name: "ConfigV2".to_string(),    // Specific implementation
    /// };
    /// ```
    ///
    /// # Type Resolution
    /// The `type_name` must refer to a type defined within the same module
    /// through the `types` field of the containing `PklModule`. Forward
    /// references are supported - the export can refer to types defined
    /// later in the module.
    pub type_name: String,
}

/// Represents a PKL type definition (class, type alias, union, or module).
///
/// `PklType` is the core abstraction for representing PKL type definitions,
/// supporting the full range of PKL's type system including classes, type aliases,
/// union types, and nested modules. It provides comprehensive support for
/// inheritance, validation, deprecation, and rich documentation.
///
/// # PKL Type System
///
/// PKL supports several kinds of type definitions:
/// - **Classes**: Object types with properties and methods
/// - **Type Aliases**: Named references to existing types or unions
/// - **Unions**: Types that can be one of several alternatives
/// - **Modules**: Nested module definitions
///
/// # Generated PKL Output
///
/// Different type kinds generate different PKL syntax:
///
/// ## Class Type
/// ```pkl
/// /// Class documentation
/// class DatabaseConfig {
///   host: String
///   port: Int(this >= 1 && this <= 65535) = 5432
///   ssl: Boolean = false
/// }
/// ```
///
/// ## Type Alias
/// ```pkl
/// /// Type alias documentation
/// typealias LogLevel = "debug" | "info" | "warn" | "error"
/// ```
///
/// ## Union Type
/// ```pkl
/// /// Union type documentation
/// typealias ConfigValue = String | Int | Boolean | Duration
/// ```
///
/// ## Abstract Class
/// ```pkl
/// /// Abstract base class
/// abstract class ConfigBase {
///   version: String
/// }
///
/// class DatabaseConfig extends ConfigBase {
///   host: String
/// }
/// ```
///
/// # Usage Examples
///
/// ## Simple Class Type
/// ```rust
/// use space_pkl::types::*;
///
/// let user_type = PklType {
///     name: "User".to_string(),
///     documentation: Some("User account information".to_string()),
///     kind: PklTypeKind::Class,
///     properties: vec![
///         PklProperty {
///             name: "username".to_string(),
///             type_name: "String".to_string(),
///             documentation: Some("Unique username".to_string()),
///             optional: false,
///             default: None,
///             constraints: vec![],
///             examples: vec!["alice".to_string()],
///             deprecated: None,
///         }
///     ],
///     abstract_type: false,
///     extends: vec![],
///     enum_values: None,
///     deprecated: None,
/// };
/// ```
///
/// ## Type Alias for Union
/// ```rust
/// use space_pkl::types::*;
///
/// let status_type = PklType {
///     name: "Status".to_string(),
///     documentation: Some("Application status values".to_string()),
///     kind: PklTypeKind::TypeAlias,
///     properties: vec![],
///     abstract_type: false,
///     extends: vec![],
///     enum_values: Some("\"active\" | \"inactive\" | \"maintenance\"".to_string()),
///     deprecated: None,
/// };
/// ```
///
/// ## Abstract Base Class
/// ```rust
/// use space_pkl::types::*;
///
/// let base_config = PklType {
///     name: "BaseConfig".to_string(),
///     documentation: Some("Base configuration class".to_string()),
///     kind: PklTypeKind::Class,
///     properties: vec![
///         PklProperty {
///             name: "version".to_string(),
///             type_name: "String".to_string(),
///             documentation: Some("Configuration version".to_string()),
///             optional: false,
///             default: Some("\"1.0\"".to_string()),
///             constraints: vec![],
///             examples: vec![],
///             deprecated: None,
///         }
///     ],
///     abstract_type: true,  // Makes this an abstract class
///     extends: vec![],
///     enum_values: None,
///     deprecated: None,
/// };
/// ```
///
/// ## Inherited Class
/// ```rust
/// use space_pkl::types::*;
///
/// let db_config = PklType {
///     name: "DatabaseConfig".to_string(),
///     documentation: Some("Database configuration extending BaseConfig".to_string()),
///     kind: PklTypeKind::Class,
///     properties: vec![
///         PklProperty {
///             name: "host".to_string(),
///             type_name: "String".to_string(),
///             documentation: Some("Database host".to_string()),
///             optional: false,
///             default: Some("\"localhost\"".to_string()),
///             constraints: vec![],
///             examples: vec![],
///             deprecated: None,
///         }
///     ],
///     abstract_type: false,
///     extends: vec!["BaseConfig".to_string()],  // Inherits from BaseConfig
///     enum_values: None,
///     deprecated: None,
/// };
/// ```
///
/// # Inheritance
///
/// PKL supports single inheritance through the `extends` field:
/// ```pkl
/// class BaseConfig {
///   version: String = "1.0"
/// }
///
/// class DatabaseConfig extends BaseConfig {
///   host: String = "localhost"
///   // Inherits version from BaseConfig
/// }
/// ```
///
/// # Validation and Constraints
///
/// Types can include validation constraints on their properties:
/// ```rust
/// # use space_pkl::types::*;
/// let validated_type = PklType {
///     name: "Port".to_string(),
///     kind: PklTypeKind::TypeAlias,
///     enum_values: Some("Int(this >= 1 && this <= 65535)".to_string()),
///     // ... other fields
/// #   documentation: None,
/// #   properties: vec![],
/// #   abstract_type: false,
/// #   extends: vec![],
/// #   deprecated: None,
/// };
/// ```
///
/// # Deprecation Support
///
/// Types can be marked as deprecated with migration guidance:
/// ```rust
/// # use space_pkl::types::*;
/// let deprecated_type = PklType {
///     name: "OldConfig".to_string(),
///     deprecated: Some("Use NewConfig instead".to_string()),
///     // ... other fields
/// #   documentation: None,
/// #   kind: PklTypeKind::Class,
/// #   properties: vec![],
/// #   abstract_type: false,
/// #   extends: vec![],
/// #   enum_values: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PklType {
    /// The name of the type.
    ///
    /// This becomes the type identifier in PKL and must be unique within
    /// the containing module. Used in type references, inheritance declarations,
    /// and export statements.
    ///
    /// # Naming Conventions
    /// - Use PascalCase for type names (e.g., "DatabaseConfig", "LogLevel")
    /// - Make names descriptive and unambiguous
    /// - Avoid generic names like "Config" or "Settings" without context
    /// - Use consistent naming patterns across related types
    ///
    /// # Examples
    /// - "WorkspaceConfig" - Clear and specific
    /// - "LogLevel" - Concise but descriptive
    /// - "DatabaseConnectionPool" - Detailed for complex types
    pub name: String,

    /// Optional documentation for the type.
    ///
    /// When present, generates PKL doc comments above the type definition.
    /// Should provide clear description of the type's purpose, usage examples,
    /// and any important constraints or relationships.
    ///
    /// # Documentation Format
    /// ```pkl
    /// /// Database connection configuration
    /// ///
    /// /// Defines settings for connecting to a database server including
    /// /// connection parameters, pooling options, and security settings.
    /// ///
    /// /// Example:
    /// /// ```
    /// /// config {
    /// ///   host = "localhost"
    /// ///   port = 5432
    /// ///   database = "myapp"
    /// /// }
    /// /// ```
    /// class DatabaseConfig {
    ///   // ...
    /// }
    /// ```
    pub documentation: Option<String>,

    /// The kind of PKL type this represents.
    ///
    /// Determines how the type is rendered in PKL syntax:
    /// - `Class` → `class TypeName { ... }`
    /// - `TypeAlias` → `typealias TypeName = ...`
    /// - `Union` → `typealias TypeName = "a" | "b" | "c"`
    /// - `Module` → `module TypeName { ... }`
    pub kind: PklTypeKind,

    /// Properties/fields contained within this type.
    ///
    /// For class types, these become the class properties. For other type kinds,
    /// this is typically empty unless the type needs to define nested structure.
    ///
    /// # Property Organization
    /// Properties are typically ordered by:
    /// 1. Required properties first
    /// 2. Optional properties second
    /// 3. Deprecated properties last (if included)
    pub properties: Vec<PklProperty>,

    /// Whether this type is abstract (classes only).
    ///
    /// When `true`, generates an `abstract class` that cannot be instantiated
    /// directly but can be extended by other classes. Only meaningful for
    /// `PklTypeKind::Class`.
    ///
    /// # Abstract Class Example
    /// ```pkl
    /// abstract class BaseConfig {
    ///   version: String = "1.0"
    /// }
    ///
    /// class DatabaseConfig extends BaseConfig {
    ///   host: String
    /// }
    /// ```
    pub abstract_type: bool,

    /// Base types that this type extends (inheritance).
    ///
    /// For class types, specifies the parent class(es) in the inheritance chain.
    /// PKL supports single inheritance, so this typically contains at most one
    /// element, but multiple entries are preserved for future extension.
    ///
    /// # Inheritance Example
    /// ```rust
    /// # use space_pkl::types::*;
    /// let child_type = PklType {
    ///     extends: vec!["BaseConfig".to_string(), "Timestamped".to_string()],
    ///     // ... other fields
    /// #   name: "ChildType".to_string(),
    /// #   documentation: None,
    /// #   kind: PklTypeKind::Class,
    /// #   properties: vec![],
    /// #   abstract_type: false,
    /// #   enum_values: None,
    /// #   deprecated: None,
    /// };
    /// ```
    pub extends: Vec<String>,

    /// For type aliases and unions, the type definition or union values.
    ///
    /// Contains the right-hand side of a type alias definition:
    /// - **Type Alias**: `"String"`, `"Int"`, `"SomeOtherType"`
    /// - **Union Type**: `"\"active\" | \"inactive\""`, `"String | Int"`
    /// - **Complex Type**: `"Listing<String>"`, `"Mapping<String, Int>"`
    ///
    /// # Examples
    /// ```rust
    /// # use space_pkl::types::*;
    /// // Simple type alias
    /// let alias = PklType {
    ///     enum_values: Some("String".to_string()),
    ///     // ...
    /// #   name: "Username".to_string(),
    /// #   documentation: None,
    /// #   kind: PklTypeKind::TypeAlias,
    /// #   properties: vec![],
    /// #   abstract_type: false,
    /// #   extends: vec![],
    /// #   deprecated: None,
    /// };
    ///
    /// // Union type
    /// let union = PklType {
    ///     enum_values: Some("\"debug\" | \"info\" | \"warn\" | \"error\"".to_string()),
    ///     // ...
    /// #   name: "LogLevel".to_string(),
    /// #   documentation: None,
    /// #   kind: PklTypeKind::Union,
    /// #   properties: vec![],
    /// #   abstract_type: false,
    /// #   extends: vec![],
    /// #   deprecated: None,
    /// };
    /// ```
    pub enum_values: Option<String>,

    /// Optional deprecation notice for this type.
    ///
    /// When present, marks the type as deprecated and provides guidance for
    /// migration. Generates deprecation warnings in PKL and documentation.
    ///
    /// # Deprecation Format
    /// Should include:
    /// - Reason for deprecation
    /// - Migration path or replacement
    /// - Timeline for removal (if applicable)
    ///
    /// # Example
    /// ```rust
    /// # use space_pkl::types::*;
    /// let deprecated_type = PklType {
    ///     deprecated: Some("Use DatabaseConfigV2 instead. This version lacks SSL support.".to_string()),
    ///     // ...
    /// #   name: "DatabaseConfig".to_string(),
    /// #   documentation: None,
    /// #   kind: PklTypeKind::Class,
    /// #   properties: vec![],
    /// #   abstract_type: false,
    /// #   extends: vec![],
    /// #   enum_values: None,
    /// };
    /// ```
    ///
    /// # Generated PKL Output
    /// ```pkl
    /// @Deprecated { "Use DatabaseConfigV2 instead. This version lacks SSL support." }
    /// class DatabaseConfig {
    ///   // ...
    /// }
    /// ```
    pub deprecated: Option<String>,
}

/// Represents the different kinds of type definitions in PKL.
///
/// `PklTypeKind` categorizes the various type definition syntaxes supported
/// by PKL, each with distinct semantics and generated code patterns. This
/// enum drives the code generation logic to produce appropriate PKL syntax
/// for each type category.
///
/// # Type Kind Overview
///
/// | Kind | PKL Syntax | Use Case | Example |
/// |------|------------|----------|---------|
/// | `Class` | `class Name { ... }` | Object types with properties | Configuration objects |
/// | `TypeAlias` | `typealias Name = Type` | Named type references | `UserId = String` |
/// | `Union` | `typealias Name = A \| B` | Multiple choice types | `Status = "ok" \| "error"` |
/// | `Module` | `module Name { ... }` | Nested modules | Sub-configuration namespaces |
///
/// # Generated PKL Examples
///
/// ## Class Type
/// ```pkl
/// class DatabaseConfig {
///   host: String = "localhost"
///   port: Int = 5432
///   ssl: Boolean = false
/// }
/// ```
///
/// ## Type Alias
/// ```pkl
/// typealias Username = String
/// typealias Port = Int(this >= 1 && this <= 65535)
/// ```
///
/// ## Union Type
/// ```pkl
/// typealias LogLevel = "debug" | "info" | "warn" | "error"
/// typealias ConfigValue = String | Int | Boolean | Duration
/// ```
///
/// ## Module Type
/// ```pkl
/// module DatabaseModule {
///   class ConnectionConfig { ... }
///   class PoolConfig { ... }
/// }
/// ```
///
/// # Usage Examples
///
/// ## Configuration Class
/// ```rust
/// use space_pkl::types::*;
///
/// let config_class = PklType {
///     name: "ServerConfig".to_string(),
///     kind: PklTypeKind::Class,  // Object type with properties
///     properties: vec![
///         // host, port, ssl properties...
///     ],
///     // ... other fields
/// #   documentation: None,
/// #   abstract_type: false,
/// #   extends: vec![],
/// #   enum_values: None,
/// #   deprecated: None,
/// };
/// ```
///
/// ## Simple Type Alias
/// ```rust
/// use space_pkl::types::*;
///
/// let user_id = PklType {
///     name: "UserId".to_string(),
///     kind: PklTypeKind::TypeAlias,  // Named reference to existing type
///     enum_values: Some("String".to_string()),  // Points to String type
///     // ... other fields
/// #   documentation: None,
/// #   properties: vec![],
/// #   abstract_type: false,
/// #   extends: vec![],
/// #   deprecated: None,
/// };
/// ```
///
/// ## Enumeration Union
/// ```rust
/// use space_pkl::types::*;
///
/// let status_enum = PklType {
///     name: "Status".to_string(),
///     kind: PklTypeKind::Union,  // Multiple choice type
///     enum_values: Some("\"active\" | \"inactive\" | \"pending\"".to_string()),
///     // ... other fields
/// #   documentation: None,
/// #   properties: vec![],
/// #   abstract_type: false,
/// #   extends: vec![],
/// #   deprecated: None,
/// };
/// ```
///
/// # Type Kind Selection Guide
///
/// ## When to Use `Class`
/// - Defining configuration objects with multiple properties
/// - Creating structured data types
/// - When you need inheritance or abstract base types
/// - For types that may evolve with new properties
///
/// ## When to Use `TypeAlias`
/// - Creating semantic aliases for primitive types
/// - Adding constraints to existing types
/// - Simplifying complex generic types
/// - For documentation and readability
///
/// ## When to Use `Union`
/// - Defining enumerations or tagged unions
/// - When a value can be one of several specific types
/// - For configuration options with fixed choices
/// - State machines and status values
///
/// ## When to Use `Module`
/// - Organizing related types into namespaces
/// - Creating hierarchical configuration structures
/// - When you need fine-grained import control
/// - For large schemas that benefit from modularity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum PklTypeKind {
    /// A PKL class type with properties and potential inheritance.
    ///
    /// Classes are the primary way to define structured object types in PKL.
    /// They can contain properties, extend other classes, and be marked as abstract.
    ///
    /// # Generated Syntax
    /// ```pkl
    /// class ClassName {
    ///   property1: String
    ///   property2: Int = 42
    /// }
    ///
    /// // With inheritance
    /// class ChildClass extends ParentClass {
    ///   additionalProperty: Boolean
    /// }
    ///
    /// // Abstract class
    /// abstract class BaseClass {
    ///   commonProperty: String
    /// }
    /// ```
    ///
    /// # Use Cases
    /// - Configuration objects (DatabaseConfig, ServerSettings)
    /// - Data transfer objects (User, Product, Order)
    /// - Complex structured types with validation
    /// - Types that benefit from inheritance hierarchies
    Class,

    /// A PKL type alias that provides a name for an existing type.
    ///
    /// Type aliases create named references to existing types, often with
    /// additional constraints or for semantic clarity. They don't create
    /// new types but provide alternative names.
    ///
    /// # Generated Syntax
    /// ```pkl
    /// // Simple alias
    /// typealias Username = String
    ///
    /// // Alias with constraints
    /// typealias Port = Int(this >= 1 && this <= 65535)
    ///
    /// // Alias for complex types
    /// typealias UserMap = Mapping<String, User>
    /// ```
    ///
    /// # Use Cases
    /// - Semantic naming (UserId, EmailAddress, Timestamp)
    /// - Adding constraints to primitive types (Port, PositiveInt)
    /// - Simplifying complex generic types
    /// - Creating domain-specific type vocabularies
    TypeAlias,

    /// A PKL union type that can be one of several alternatives.
    ///
    /// Union types allow values to be one of multiple specified types or values.
    /// They're commonly used for enumerations, tagged unions, and polymorphic data.
    ///
    /// # Generated Syntax
    /// ```pkl
    /// // String enumeration
    /// typealias LogLevel = "debug" | "info" | "warn" | "error"
    ///
    /// // Type union
    /// typealias ConfigValue = String | Int | Boolean
    ///
    /// // Mixed values and types
    /// typealias Status = "pending" | "completed" | Error
    /// ```
    ///
    /// # Use Cases
    /// - Enumerations (LogLevel, Status, Mode)
    /// - Configuration options with fixed choices
    /// - Polymorphic data types
    /// - State machine states
    Union,

    /// A PKL nested module definition.
    ///
    /// Modules provide namespacing and organization for related types and values.
    /// They can contain their own type definitions, imports, and exports.
    ///
    /// # Generated Syntax
    /// ```pkl
    /// module SubModule {
    ///   class LocalType { ... }
    ///   typealias LocalAlias = String
    ///
    ///   // Local exports
    ///   LocalType
    ///   LocalAlias
    /// }
    /// ```
    ///
    /// # Use Cases
    /// - Organizing related configuration types
    /// - Creating sub-namespaces within larger schemas
    /// - Grouping domain-specific types
    /// - Hierarchical configuration structures
    Module,
}

/// Represents a property/field within a PKL type or module.
///
/// `PklProperty` defines individual properties within PKL classes, modules, or other
/// container types. It provides comprehensive support for type information, validation
/// constraints, documentation, default values, and usage examples.
///
/// # Property Structure
///
/// PKL properties have rich metadata that controls their behavior:
/// - **Type Information**: PKL type name and optionality
/// - **Documentation**: Inline comments and usage examples
/// - **Validation**: Constraints and validation rules
/// - **Defaults**: Default values and initialization
/// - **Lifecycle**: Deprecation notices and migration paths
///
/// # Generated PKL Syntax
///
/// Properties generate different PKL syntax based on their configuration:
///
/// ## Required Property
/// ```pkl
/// propertyName: TypeName
/// ```
///
/// ## Optional Property
/// ```pkl
/// propertyName: TypeName?
/// ```
///
/// ## Property with Default
/// ```pkl
/// propertyName: TypeName = defaultValue
/// ```
///
/// ## Property with Constraints
/// ```pkl
/// propertyName: TypeName(constraint1)(constraint2)
/// ```
///
/// ## Property with Documentation
/// ```pkl
/// /// Property description
/// /// Examples: value1, value2
/// propertyName: TypeName
/// ```
///
/// ## Complete Property Example
/// ```pkl
/// /// Database connection timeout in seconds
/// ///
/// /// Controls how long to wait for database connections before timing out.
/// /// Should be set based on expected network latency and server response times.
/// ///
/// /// Examples: 30, 60, 120
/// timeout: Int(this >= 1 && this <= 300) = 30
/// ```
///
/// # Usage Examples
///
/// ## Simple Required Property
/// ```rust
/// use space_pkl::types::*;
///
/// let hostname = PklProperty {
///     name: "hostname".to_string(),
///     type_name: "String".to_string(),
///     documentation: Some("Server hostname or IP address".to_string()),
///     optional: false,
///     default: None,
///     constraints: vec![],
///     examples: vec!["localhost".to_string(), "db.example.com".to_string()],
///     deprecated: None,
/// };
/// ```
///
/// ## Optional Property with Default
/// ```rust
/// use space_pkl::types::*;
///
/// let port = PklProperty {
///     name: "port".to_string(),
///     type_name: "Int".to_string(),
///     documentation: Some("Server port number".to_string()),
///     optional: true,
///     default: Some("5432".to_string()),
///     constraints: vec![
///         PklConstraint {
///             kind: PklConstraintKind::Min,
///             value: "this >= 1".to_string(),
///             message: Some("Port must be positive".to_string()),
///         },
///         PklConstraint {
///             kind: PklConstraintKind::Max,
///             value: "this <= 65535".to_string(),
///             message: Some("Port must be valid".to_string()),
///         },
///     ],
///     examples: vec!["5432".to_string(), "3306".to_string(), "5984".to_string()],
///     deprecated: None,
/// };
/// ```
///
/// ## Complex Property with Validation
/// ```rust
/// use space_pkl::types::*;
///
/// let username = PklProperty {
///     name: "username".to_string(),
///     type_name: "String".to_string(),
///     documentation: Some("Database username for authentication".to_string()),
///     optional: false,
///     default: None,
///     constraints: vec![
///         PklConstraint {
///             kind: PklConstraintKind::Length,
///             value: "length >= 3".to_string(),
///             message: Some("Username too short".to_string()),
///         },
///         PklConstraint {
///             kind: PklConstraintKind::Pattern,
///             value: "matches(Regex(#\"^[a-zA-Z0-9_]+$\"#))".to_string(),
///             message: Some("Username contains invalid characters".to_string()),
///         },
///     ],
///     examples: vec!["postgres".to_string(), "admin".to_string(), "app_user".to_string()],
///     deprecated: None,
/// };
/// ```
///
/// ## Deprecated Property
/// ```rust
/// use space_pkl::types::*;
///
/// let old_setting = PklProperty {
///     name: "legacyTimeout".to_string(),
///     type_name: "Int".to_string(),
///     documentation: Some("Legacy timeout setting".to_string()),
///     optional: true,
///     default: Some("30".to_string()),
///     constraints: vec![],
///     examples: vec![],
///     deprecated: Some("Use 'timeout' property instead. Will be removed in v2.0.".to_string()),
/// };
/// ```
///
/// # Property Types
///
/// Properties can reference various PKL types:
///
/// ## Primitive Types
/// ```rust
/// # use space_pkl::types::*;
/// let primitives = vec![
///     PklProperty { type_name: "String".to_string(), /* ... */
/// #       name: "text".to_string(), documentation: None, optional: false,
/// #       default: None, constraints: vec![], examples: vec![], deprecated: None },
///     PklProperty { type_name: "Int".to_string(), /* ... */
/// #       name: "number".to_string(), documentation: None, optional: false,
/// #       default: None, constraints: vec![], examples: vec![], deprecated: None },
///     PklProperty { type_name: "Boolean".to_string(), /* ... */
/// #       name: "flag".to_string(), documentation: None, optional: false,
/// #       default: None, constraints: vec![], examples: vec![], deprecated: None },
///     PklProperty { type_name: "Duration".to_string(), /* ... */
/// #       name: "timeout".to_string(), documentation: None, optional: false,
/// #       default: None, constraints: vec![], examples: vec![], deprecated: None },
/// ];
/// ```
///
/// ## Collection Types
/// ```rust
/// # use space_pkl::types::*;
/// let collections = vec![
///     PklProperty { type_name: "Listing<String>".to_string(), /* ... */
/// #       name: "items".to_string(), documentation: None, optional: false,
/// #       default: None, constraints: vec![], examples: vec![], deprecated: None },
///     PklProperty { type_name: "Mapping<String, Int>".to_string(), /* ... */
/// #       name: "counts".to_string(), documentation: None, optional: false,
/// #       default: None, constraints: vec![], examples: vec![], deprecated: None },
///     PklProperty { type_name: "Set<String>".to_string(), /* ... */
/// #       name: "tags".to_string(), documentation: None, optional: false,
/// #       default: None, constraints: vec![], examples: vec![], deprecated: None },
/// ];
/// ```
///
/// ## Custom Types
/// ```rust
/// # use space_pkl::types::*;
/// let custom_types = vec![
///     PklProperty { type_name: "DatabaseConfig".to_string(), /* ... */
/// #       name: "database".to_string(), documentation: None, optional: false,
/// #       default: None, constraints: vec![], examples: vec![], deprecated: None },
///     PklProperty { type_name: "LogLevel".to_string(), /* ... */
/// #       name: "logLevel".to_string(), documentation: None, optional: false,
/// #       default: None, constraints: vec![], examples: vec![], deprecated: None },
/// ];
/// ```
///
/// # Best Practices
///
/// ## Property Naming
/// - Use camelCase for property names
/// - Be descriptive but concise
/// - Follow domain conventions
/// - Avoid abbreviations unless universal
///
/// ## Documentation
/// - Always provide documentation for public properties
/// - Include usage examples for complex properties
/// - Explain constraints and validation rules
/// - Document relationships to other properties
///
/// ## Defaults and Optionality
/// - Provide sensible defaults for optional properties
/// - Make properties optional only when truly optional
/// - Use constraints to enforce valid default values
/// - Consider the user experience of configuration authoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PklProperty {
    /// The name of the property.
    ///
    /// This becomes the property identifier in PKL and must be unique within
    /// the containing type. Should follow camelCase naming conventions for
    /// consistency with PKL style guidelines.
    ///
    /// # Naming Guidelines
    /// - Use camelCase (e.g., "databaseHost", "maxRetries", "isEnabled")
    /// - Be descriptive but concise
    /// - Avoid abbreviations unless universally understood
    /// - Use consistent naming patterns across related properties
    ///
    /// # Examples
    /// - "host" - Simple and clear
    /// - "connectionTimeout" - Descriptive compound name
    /// - "maxRetryAttempts" - Clear action and constraint
    /// - "isSSLEnabled" - Boolean property with "is" prefix
    pub name: String,

    /// The type name of the property.
    ///
    /// Specifies the PKL type for this property, which can be:
    /// - Primitive types: `String`, `Int`, `Boolean`, `Float`
    /// - Collection types: `List<T>`, `Set<T>`, `Map<K, V>`
    /// - Custom types: User-defined classes and type aliases
    /// - Union types: `A|B|C` for alternative types
    /// - Optional types: `T?` for nullable properties
    ///
    /// # Type Mapping
    /// Common Rust-to-PKL type mappings:
    /// - `String` → `String`
    /// - `i32`, `u32` → `Int`
    /// - `f32`, `f64` → `Float`
    /// - `bool` → `Boolean`
    /// - `Vec<T>` → `List<T>`
    /// - `HashMap<K, V>` → `Map<K, V>`
    /// - `Option<T>` → `T?`
    ///
    /// # Examples
    /// ```
    /// "String"              // Simple string property
    /// "Int"                 // Integer value
    /// "List<String>"        // Array of strings
    /// "DatabaseConfig"      // Custom type reference
    /// "String|Int"          // Union type (string or integer)
    /// "List<DatabaseConfig>?" // Optional list of custom objects
    /// ```
    pub type_name: String,

    /// Optional documentation for the property.
    ///
    /// Provides human-readable documentation that appears in generated PKL files
    /// as comments above the property declaration. Good documentation should
    /// explain the property's purpose, expected values, and any special behavior.
    ///
    /// # Documentation Guidelines
    /// - **Purpose**: Explain what the property controls or represents
    /// - **Values**: Describe expected values, ranges, or formats
    /// - **Behavior**: Document side effects or special handling
    /// - **Examples**: Include usage examples for complex properties
    /// - **Relationships**: Mention dependencies on other properties
    ///
    /// # Generated Output
    /// When documentation is provided, it generates PKL comments:
    /// ```pkl
    /// /// Database connection timeout in seconds.
    /// /// Must be between 1 and 300 seconds.
    /// /// Defaults to 30 seconds if not specified.
    /// connectionTimeout: Int = 30
    /// ```
    ///
    /// # Best Practices
    /// - Keep first line concise (appears in IDE tooltips)
    /// - Use proper grammar and punctuation
    /// - Include units for numeric values
    /// - Document validation constraints
    /// - Provide examples for complex formats
    ///
    /// # Examples
    /// ```
    /// Some("Database host address (hostname or IP)".to_string())
    /// Some("Maximum number of retry attempts (1-10)".to_string())
    /// Some("Enable SSL/TLS encryption for connections".to_string())
    /// None  // No documentation provided
    /// ```
    pub documentation: Option<String>,

    /// Whether the property is optional (nullable).
    ///
    /// When `true`, the property can be omitted from PKL configurations,
    /// and the property type is automatically made nullable (e.g., `String` becomes `String?`).
    /// Optional properties should generally have sensible defaults or be truly optional
    /// for the configuration to be valid.
    ///
    /// # Impact on Generated PKL
    /// ```pkl
    /// // Required property (optional = false)
    /// host: String
    ///
    /// // Optional property (optional = true)
    /// port: Int?
    ///
    /// // Optional with default (optional = true, default provided)
    /// timeout: Int? = 30
    /// ```
    ///
    /// # Design Considerations
    /// - **Required by default**: Only make properties optional when they truly are
    /// - **Provide defaults**: Optional properties should have reasonable defaults
    /// - **User experience**: Consider the configuration authoring experience
    /// - **Validation**: Ensure optional properties don't break validation logic
    ///
    /// # Examples
    /// ```
    /// true   // Property can be omitted
    /// false  // Property must be provided
    /// ```
    pub optional: bool,

    /// Default value for the property.
    ///
    /// Provides a default value that's used when the property is not specified
    /// in a PKL configuration. The default value must be compatible with the
    /// property's type and satisfy any validation constraints.
    ///
    /// # Format Requirements
    /// Default values must be valid PKL expressions:
    /// - **Strings**: Use quotes - `"localhost"`
    /// - **Numbers**: No quotes - `8080`, `3.14`
    /// - **Booleans**: No quotes - `true`, `false`
    /// - **Collections**: PKL syntax - `List("a", "b")`, `Map("key", "value")`
    /// - **Objects**: PKL object syntax - `new DatabaseConfig { host = "localhost" }`
    ///
    /// # Validation
    /// Default values are automatically validated against:
    /// - Type compatibility
    /// - Constraint rules (min/max, patterns, etc.)
    /// - Enum value restrictions
    ///
    /// # Generated Output
    /// ```pkl
    /// // With default
    /// port: Int = 8080
    ///
    /// // Without default
    /// host: String
    ///
    /// // Complex default
    /// database: DatabaseConfig = new DatabaseConfig {
    ///   host = "localhost"
    ///   port = 5432
    /// }
    /// ```
    ///
    /// # Examples
    /// ```
    /// Some("\"localhost\"".to_string())     // String default
    /// Some("8080".to_string())              // Integer default
    /// Some("true".to_string())              // Boolean default
    /// Some("List()".to_string())            // Empty list default
    /// None                                  // No default value
    /// ```
    pub default: Option<String>,

    /// Validation constraints applied to the property.
    ///
    /// Constraints define validation rules that property values must satisfy.
    /// They're converted to PKL constraint annotations that provide runtime
    /// validation and improve configuration authoring experience with better
    /// error messages and IDE support.
    ///
    /// # Constraint Types
    /// - **Range constraints**: `Min`, `Max` for numeric values
    /// - **Length constraints**: `MinLength`, `MaxLength` for strings/collections
    /// - **Pattern constraints**: `Pattern` for regex validation
    /// - **Enum constraints**: `OneOf` for restricting to specific values
    /// - **Custom constraints**: Complex validation logic
    ///
    /// # Generated PKL Annotations
    /// Constraints generate PKL annotations that provide validation:
    /// ```pkl
    /// @IntRange { min = 1; max = 65535 }
    /// port: Int
    ///
    /// @Length { min = 1; max = 255 }
    /// hostname: String
    ///
    /// @Regex("^[a-zA-Z0-9_-]+$")
    /// identifier: String
    /// ```
    ///
    /// # Constraint Composition
    /// Multiple constraints can be applied to a single property:
    /// ```pkl
    /// @IntRange { min = 1; max = 100 }
    /// @Matches("^(low|medium|high)$")
    /// priority: String
    /// ```
    ///
    /// # Error Messages
    /// Constraints can include custom error messages for better user experience:
    /// ```rust
    /// PklConstraint {
    ///     kind: PklConstraintKind::Min,
    ///     value: "1".to_string(),
    ///     message: Some("Port must be at least 1".to_string()),
    /// }
    /// ```
    ///
    /// # Examples
    /// ```
    /// vec![
    ///     PklConstraint { kind: PklConstraintKind::Min, value: "1".to_string(), message: None },
    ///     PklConstraint { kind: PklConstraintKind::Max, value: "65535".to_string(), message: None },
    /// ]
    /// ```
    pub constraints: Vec<PklConstraint>,

    /// Example values for the property.
    ///
    /// Provides concrete example values that demonstrate proper usage of the property.
    /// Examples are used in generated documentation, IDE tooltips, and can be
    /// included in template configurations to help users understand expected formats.
    ///
    /// # Format Requirements
    /// Examples must be valid PKL expressions compatible with the property type:
    /// - **Strings**: Use quotes - `"api.example.com"`
    /// - **Numbers**: No quotes - `443`, `1.5`
    /// - **Booleans**: No quotes - `true`, `false`
    /// - **Arrays**: PKL list syntax - `List("item1", "item2")`
    /// - **Objects**: PKL object syntax - `new Config { field = "value" }`
    ///
    /// # Usage in Documentation
    /// Examples appear in generated PKL comments:
    /// ```pkl
    /// /// Database host address.
    /// /// Examples: "localhost", "db.example.com", "192.168.1.100"
    /// host: String
    /// ```
    ///
    /// # Best Practices
    /// - **Representative**: Show realistic, production-like values
    /// - **Diverse**: Include different valid formats or patterns
    /// - **Progressive**: Start simple, show more complex examples
    /// - **Valid**: Ensure all examples satisfy constraints
    /// - **Contextual**: Examples should make sense for the property's domain
    ///
    /// # Multiple Examples
    /// Provide multiple examples to show variety:
    /// ```
    /// vec![
    ///     "\"localhost\"".to_string(),
    ///     "\"api.production.com\"".to_string(),
    ///     "\"192.168.1.100\"".to_string(),
    /// ]
    /// ```
    ///
    /// # Examples for Complex Types
    /// ```
    /// vec![
    ///     "new DatabaseConfig { host = \"localhost\"; port = 5432 }".to_string(),
    ///     "new DatabaseConfig { host = \"prod-db\"; port = 3306; ssl = true }".to_string(),
    /// ]
    /// ```
    pub examples: Vec<String>,

    /// Deprecation information for the property.
    ///
    /// When present, marks the property as deprecated and provides information
    /// about the deprecation. This generates appropriate PKL annotations and
    /// documentation to warn users about deprecated properties and guide them
    /// toward alternatives.
    ///
    /// # Deprecation Message Format
    /// The deprecation message should include:
    /// - **Reason**: Why the property is deprecated
    /// - **Alternative**: What to use instead
    /// - **Timeline**: When removal is planned (version/date)
    /// - **Migration**: How to migrate existing configurations
    ///
    /// # Generated PKL Output
    /// Deprecated properties generate warning annotations:
    /// ```pkl
    /// @Deprecated { message = "Use 'newProperty' instead. Will be removed in v2.0" }
    /// oldProperty: String?
    /// ```
    ///
    /// # Best Practices
    /// - **Clear guidance**: Always suggest alternatives
    /// - **Migration path**: Provide clear migration instructions
    /// - **Timeline**: Give users advance notice of removal
    /// - **Backward compatibility**: Keep deprecated properties functional
    ///
    /// # Deprecation Lifecycle
    /// 1. **Mark as deprecated**: Add deprecation message
    /// 2. **Update documentation**: Add migration guides
    /// 3. **Warn users**: Generate deprecation warnings
    /// 4. **Eventually remove**: After sufficient warning period
    ///
    /// # Examples
    /// ```
    /// Some("Use 'databaseUrl' instead. Will be removed in v2.0".to_string())
    /// Some("Replaced by 'connectionConfig'. Migrate by v1.5".to_string())
    /// None  // Property is not deprecated
    /// ```
    pub deprecated: Option<String>,
}

/// Represents a validation constraint for PKL properties.
///
/// Constraints define validation rules that are enforced at PKL evaluation time,
/// providing type safety, value validation, and better error messages. They map
/// directly to PKL annotation syntax and support both built-in and custom validation logic.
///
/// # Constraint Architecture
///
/// Each constraint consists of:
/// - **Kind**: The type of validation (range, pattern, length, etc.)
/// - **Value**: The constraint parameter (threshold, regex, enum values)
/// - **Message**: Optional custom error message for validation failures
///
/// # PKL Integration
///
/// Constraints generate PKL annotations that are enforced at runtime:
/// ```pkl
/// @IntRange { min = 1; max = 100 }
/// priority: Int
///
/// @Length { min = 5; max = 50 }
/// username: String
///
/// @Regex("^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$")
/// email: String
/// ```
///
/// # Error Handling
///
/// When constraints are violated, PKL generates helpful error messages:
/// ```
/// error: value out of range
///   --> config.pkl:10:15
///    |
/// 10 |     priority = 150
///    |                ^^^
///    | expected: value between 1 and 100 (inclusive)
///    | actual: 150
/// ```
///
/// # Custom Error Messages
///
/// Provide user-friendly error messages for better debugging:
/// ```rust
/// PklConstraint {
///     kind: PklConstraintKind::Min,
///     value: "1".to_string(),
///     message: Some("Priority must be at least 1 (lowest priority)".to_string()),
/// }
/// ```
///
/// # Constraint Composition
///
/// Multiple constraints can be applied to create complex validation:
/// ```rust
/// vec![
///     PklConstraint { kind: PklConstraintKind::MinLength, value: "8".to_string(), message: None },
///     PklConstraint { kind: PklConstraintKind::Pattern, value: ".*[A-Z].*".to_string(),
///                    message: Some("Must contain at least one uppercase letter".to_string()) },
///     PklConstraint { kind: PklConstraintKind::Pattern, value: ".*[0-9].*".to_string(),
///                    message: Some("Must contain at least one digit".to_string()) },
/// ]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PklConstraint {
    /// The type of constraint being applied.
    ///
    /// Determines which PKL annotation will be generated and what kind of
    /// validation logic will be applied. Each constraint kind has specific
    /// requirements for the `value` field format.
    ///
    /// # Constraint Categories
    /// - **Numeric**: `Min`, `Max` for value ranges
    /// - **Text**: `MinLength`, `MaxLength`, `Pattern` for string validation
    /// - **Collection**: `NonEmpty`, `Unique` for array/set validation
    /// - **Enumeration**: `OneOf` for restricting to specific values
    /// - **Custom**: `Custom` for complex validation logic
    pub kind: PklConstraintKind,

    /// The constraint parameter value.
    ///
    /// Format depends on the constraint kind:
    /// - **Min/Max**: Numeric string (`"42"`, `"3.14"`)
    /// - **MinLength/MaxLength**: Integer string (`"5"`, `"100"`)
    /// - **Pattern**: Regular expression string (`"^[a-z]+$"`)
    /// - **OneOf**: Comma-separated values (`"red,green,blue"`)
    /// - **Custom**: Custom expression string
    ///
    /// # Value Format Examples
    /// ```
    /// "42"                          // Numeric constraint
    /// "^[a-zA-Z0-9_-]+$"           // Regex pattern
    /// "production,staging,dev"      // Enum values
    /// "length > 0 && length < 100" // Custom expression
    /// ```
    pub value: String,

    /// Optional custom error message.
    ///
    /// When provided, this message replaces the default constraint violation
    /// message in PKL error output. Should be user-friendly and provide
    /// clear guidance on how to fix the validation error.
    ///
    /// # Message Guidelines
    /// - **Be specific**: Explain exactly what's wrong
    /// - **Be helpful**: Suggest how to fix the issue
    /// - **Be contextual**: Reference the property/field name
    /// - **Be actionable**: Provide concrete next steps
    ///
    /// # Examples
    /// ```
    /// Some("Port must be between 1 and 65535".to_string())
    /// Some("Username must contain only letters, numbers, and underscores".to_string())
    /// Some("Environment must be one of: production, staging, development".to_string())
    /// None  // Use default PKL error message
    /// ```
    pub message: Option<String>,
}

/// Types of validation constraints supported in PKL schemas.
///
/// Each constraint kind maps to specific PKL annotation syntax and provides
/// different types of validation logic. Constraints can be combined to create
/// comprehensive validation rules for properties.
///
/// # Constraint Categories
///
/// ## Numeric Constraints
/// - **Range validation**: Ensure values fall within acceptable bounds
/// - **Precision control**: Validate decimal places and numeric formats
/// - **Mathematical relationships**: Express constraints between related values
///
/// ## Text Constraints
/// - **Length validation**: Control string and identifier lengths
/// - **Format validation**: Ensure strings match expected patterns
/// - **Content validation**: Validate semantic meaning of text values
///
/// ## Collection Constraints
/// - **Size validation**: Control collection sizes and capacities
/// - **Content validation**: Ensure collection elements meet criteria
/// - **Uniqueness**: Prevent duplicate values in collections
///
/// # PKL Annotation Mapping
///
/// Each constraint kind generates specific PKL annotations:
///
/// | Constraint Kind | PKL Annotation | Example Usage |
/// |----------------|----------------|---------------|
/// | `Min` | `@IntRange { min = N }` | `@IntRange { min = 1 }` |
/// | `Max` | `@IntRange { max = N }` | `@IntRange { max = 100 }` |
/// | `Length` | `@Length { min = M; max = N }` | `@Length { min = 1; max = 50 }` |
/// | `Pattern` | `@Regex("pattern")` | `@Regex("^[a-z]+$")` |
/// | `Custom` | Custom annotation | `@Validate(expression)` |
///
/// # Usage Examples
///
/// ```rust
/// use space_pkl::types::*;
///
/// // Port number validation (1-65535)
/// let port_constraints = vec![
///     PklConstraint {
///         kind: PklConstraintKind::Min,
///         value: "1".to_string(),
///         message: Some("Port must be at least 1".to_string()),
///     },
///     PklConstraint {
///         kind: PklConstraintKind::Max,
///         value: "65535".to_string(),
///         message: Some("Port must be at most 65535".to_string()),
///     },
/// ];
///
/// // Username validation
/// let username_constraints = vec![
///     PklConstraint {
///         kind: PklConstraintKind::Length,
///         value: "3,20".to_string(), // min=3, max=20
///         message: Some("Username must be 3-20 characters".to_string()),
///     },
///     PklConstraint {
///         kind: PklConstraintKind::Pattern,
///         value: "^[a-zA-Z0-9_]+$".to_string(),
///         message: Some("Username can only contain letters, numbers, and underscores".to_string()),
///     },
/// ];
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PklConstraintKind {
    /// Minimum value constraint for numeric types.
    ///
    /// Ensures that numeric values (integers, floats) are greater than or equal
    /// to the specified minimum. Generates `@IntRange { min = N }` or
    /// `@FloatRange { min = N }` annotations in PKL.
    ///
    /// # Value Format
    /// - **Integer**: `"42"`
    /// - **Float**: `"3.14"`
    /// - **Negative**: `"-10"`
    ///
    /// # Generated PKL
    /// ```pkl
    /// @IntRange { min = 1 }
    /// port: Int
    ///
    /// @FloatRange { min = 0.0 }
    /// temperature: Float
    /// ```
    ///
    /// # Common Use Cases
    /// - Port numbers (`min = 1`)
    /// - Percentages (`min = 0.0`)
    /// - Counts and quantities (`min = 0`)
    /// - Ages and durations (`min = 1`)
    Min,

    /// Maximum value constraint for numeric types.
    ///
    /// Ensures that numeric values (integers, floats) are less than or equal
    /// to the specified maximum. Generates `@IntRange { max = N }` or
    /// `@FloatRange { max = N }` annotations in PKL.
    ///
    /// # Value Format
    /// - **Integer**: `"100"`
    /// - **Float**: `"99.99"`
    /// - **Large numbers**: `"2147483647"`
    ///
    /// # Generated PKL
    /// ```pkl
    /// @IntRange { max = 65535 }
    /// port: Int
    ///
    /// @FloatRange { max = 100.0 }
    /// percentage: Float
    /// ```
    ///
    /// # Common Use Cases
    /// - Port numbers (`max = 65535`)
    /// - Percentages (`max = 100.0`)
    /// - Buffer sizes (`max = 1048576`)
    /// - Priority levels (`max = 10`)
    Max,

    /// Length constraint for strings and collections.
    ///
    /// Controls the length of strings, lists, sets, and maps. Can specify
    /// minimum length, maximum length, or both. Generates `@Length` annotations
    /// with min/max parameters in PKL.
    ///
    /// # Value Formats
    /// - **Minimum only**: `"5"` (at least 5 characters/elements)
    /// - **Maximum only**: `",100"` (at most 100 characters/elements)
    /// - **Range**: `"5,100"` (between 5 and 100 characters/elements)
    /// - **Exact**: `"10,10"` (exactly 10 characters/elements)
    ///
    /// # Generated PKL
    /// ```pkl
    /// @Length { min = 1; max = 50 }
    /// username: String
    ///
    /// @Length { min = 1 }
    /// items: List<String>
    ///
    /// @Length { max = 100 }
    /// description: String
    /// ```
    ///
    /// # Common Use Cases
    /// - Usernames (`"3,20"`)
    /// - Passwords (`"8,128"`)
    /// - Descriptions (`",500"`)
    /// - Configuration arrays (`"1,"`)
    Length,

    /// Pattern/regex constraint for string validation.
    ///
    /// Validates strings against regular expression patterns. Useful for
    /// enforcing specific formats like emails, URLs, identifiers, or
    /// custom business rules. Generates `@Regex("pattern")` annotations in PKL.
    ///
    /// # Value Format
    /// The value should be a valid regular expression string:
    /// - **Basic patterns**: `"^[a-z]+$"` (lowercase letters only)
    /// - **Email validation**: `"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"`
    /// - **URL validation**: `"^https?://[^\\s/$.?#].[^\\s]*$"`
    /// - **Identifier validation**: `"^[a-zA-Z_][a-zA-Z0-9_]*$"`
    ///
    /// # Generated PKL
    /// ```pkl
    /// @Regex("^[a-zA-Z0-9_-]+$")
    /// identifier: String
    ///
    /// @Regex("^(development|staging|production)$")
    /// environment: String
    /// ```
    ///
    /// # Escaping Requirements
    /// Remember to escape backslashes in Rust strings:
    /// ```rust
    /// // Email pattern - note the double backslashes
    /// PklConstraint {
    ///     kind: PklConstraintKind::Pattern,
    ///     value: r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string(),
    ///     message: Some("Must be a valid email address".to_string()),
    /// }
    /// ```
    ///
    /// # Common Use Cases
    /// - Email addresses
    /// - URLs and URIs
    /// - Version strings (`"^\\d+\\.\\d+\\.\\d+$"`)
    /// - Environment names (`"^(dev|test|prod)$"`)
    /// - API keys and tokens
    Pattern,

    /// Custom validation constraint for complex rules.
    ///
    /// Allows defining custom validation logic that goes beyond simple
    /// range, length, or pattern constraints. Can reference multiple
    /// properties, perform calculations, or implement business-specific
    /// validation rules.
    ///
    /// # Value Format
    /// The value should be a PKL expression that evaluates to a boolean:
    /// - **Property references**: `"this.port > 0 && this.port < 65536"`
    /// - **Mathematical expressions**: `"this.min <= this.max"`
    /// - **String operations**: `"this.name.length > 0"`
    /// - **Collection operations**: `"this.items.length >= 1"`
    ///
    /// # Generated PKL
    /// ```pkl
    /// @Validate(this.startDate < this.endDate)
    /// class DateRange {
    ///   startDate: String
    ///   endDate: String
    /// }
    ///
    /// @Validate(this.retryCount >= 0 && this.retryCount <= this.maxRetries)
    /// class RetryConfig {
    ///   retryCount: Int
    ///   maxRetries: Int
    /// }
    /// ```
    ///
    /// # Advanced Examples
    /// ```rust
    /// // Validate that timeout is reasonable based on retry count
    /// PklConstraint {
    ///     kind: PklConstraintKind::Custom,
    ///     value: "this.timeout > this.retryCount * 1000".to_string(),
    ///     message: Some("Timeout must allow time for all retries".to_string()),
    /// }
    ///
    /// // Validate mutual exclusion of options
    /// PklConstraint {
    ///     kind: PklConstraintKind::Custom,
    ///     value: "!(this.useSSL && this.usePlaintext)".to_string(),
    ///     message: Some("Cannot enable both SSL and plaintext modes".to_string()),
    /// }
    /// ```
    ///
    /// # Common Use Cases
    /// - Cross-property validation
    /// - Business rule enforcement
    /// - Complex mathematical relationships
    /// - Conditional validation logic
    /// - Multi-field consistency checks
    Custom,
}

/// Context for template rendering in the PKL schema generation system.
///
/// Provides all the data and configuration needed to render PKL templates,
/// including the schema module definition, generator configuration, and
/// additional template variables for customization.
///
/// # Template Architecture
///
/// The template system uses a context-driven approach where:
/// 1. **Module data**: Provides the schema structure and types
/// 2. **Configuration**: Controls output format and features
/// 3. **Variables**: Enable dynamic customization and parameterization
///
/// # Template Variables
///
/// Template variables allow dynamic customization of generated output:
/// - **Metadata**: Version, timestamp, author information
/// - **Formatting**: Indentation, line endings, comment styles
/// - **Features**: Enable/disable optional sections
/// - **Customization**: Project-specific adaptations
///
/// # Usage in Templates
///
/// Templates access context data through template syntax:
/// ```handlebars
/// {{!-- Module information --}}
/// module {{module.name}}
///
/// {{#if config.include_examples}}
/// // Examples generated with {{variables.generator_version}}
/// {{/if}}
///
/// {{#each module.types}}
/// class {{name}} {
///   {{#each properties}}
///   {{#if documentation}}/// {{documentation}}{{/if}}
///   {{name}}: {{type_name}}{{#if optional}}?{{/if}}{{#if default}} = {{default}}{{/if}}
///   {{/each}}
/// }
/// {{/each}}
/// ```
///
/// # Context Construction
///
/// ```rust
/// use space_pkl::types::*;
/// use space_pkl::config::GeneratorConfig;
/// use std::collections::HashMap;
/// use serde_json::json;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let module = PklModule {
///     name: "AppConfig".to_string(),
///     // ... module definition
/// #   documentation: None, imports: vec![], exports: vec![], types: vec![], properties: vec![],
/// };
///
/// let config = GeneratorConfig {
///     include_examples: true,
///     include_validation: true,
///     // ... other config
/// #   output_dir: std::path::PathBuf::new(), template_config: Default::default(),
/// #   schema_type: crate::config::SchemaType::Workspace, type_mappings: HashMap::new(),
/// };
///
/// let mut variables = HashMap::new();
/// variables.insert("generator_version".to_string(), json!("1.0.0"));
/// variables.insert("timestamp".to_string(), json!("2025-05-31T10:00:00Z"));
/// variables.insert("author".to_string(), json!("space-pkl generator"));
///
/// let context = TemplateContext {
///     module,
///     config,
///     variables,
/// };
/// # Ok(())
/// # }
/// ```
///
/// # Common Template Variables
///
/// ## Metadata Variables
/// ```rust
/// variables.insert("version".to_string(), json!("1.2.3"));
/// variables.insert("description".to_string(), json!("Moon workspace configuration"));
/// variables.insert("generated_at".to_string(), json!("2025-05-31T10:00:00Z"));
/// variables.insert("generator".to_string(), json!("space-pkl v1.0.0"));
/// ```
///
/// ## Formatting Variables
/// ```rust
/// variables.insert("indent".to_string(), json!("  "));        // 2 spaces
/// variables.insert("line_ending".to_string(), json!("\n"));   // Unix line endings
/// variables.insert("comment_style".to_string(), json!("///"));// Doc comment style
/// ```
///
/// ## Feature Flags
/// ```rust
/// variables.insert("include_header".to_string(), json!(true));
/// variables.insert("include_imports".to_string(), json!(true));
/// variables.insert("include_examples".to_string(), json!(false));
/// variables.insert("verbose_docs".to_string(), json!(true));
/// ```
///
/// # Template Inheritance
///
/// Template contexts support inheritance for modular template systems:
/// ```rust
/// // Base context with common variables
/// let base_context = TemplateContext {
///     module: base_module,
///     config: base_config,
///     variables: base_variables,
/// };
///
/// // Extended context with additional variables
/// let mut extended_variables = base_context.variables.clone();
/// extended_variables.insert("custom_feature".to_string(), json!(true));
///
/// let extended_context = TemplateContext {
///     module: extended_module,
///     config: base_context.config,
///     variables: extended_variables,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    /// The PKL module being rendered.
    ///
    /// Contains the complete schema definition including types, properties,
    /// constraints, and documentation. This is the primary data source for
    /// template rendering and provides the structure for generated PKL files.
    ///
    /// # Module Contents
    /// - **Types**: Class definitions and enums
    /// - **Properties**: Configuration properties with validation
    /// - **Imports**: Dependencies on other modules
    /// - **Exports**: Public API surface
    /// - **Documentation**: Module and type-level documentation
    ///
    /// # Template Access
    /// Templates can access all module data:
    /// ```handlebars
    /// module {{module.name}}
    /// {{#if module.documentation}}
    /// /// {{module.documentation}}
    /// {{/if}}
    ///
    /// {{#each module.imports}}
    /// import "{{path}}"{{#if alias}} as {{alias}}{{/if}}
    /// {{/each}}
    /// ```
    pub module: PklModule,

    /// Generator configuration controlling output format and features.
    ///
    /// Provides settings that affect how templates are rendered, what
    /// features are included, and how the output is formatted. Templates
    /// can conditionally include content based on configuration flags.
    ///
    /// # Configuration Categories
    /// - **Feature flags**: Control what gets generated
    /// - **Output settings**: Formatting and structure preferences
    /// - **Template settings**: Template-specific customizations
    /// - **Type mappings**: Custom type conversion rules
    ///
    /// # Template Usage
    /// ```handlebars
    /// {{#if config.include_examples}}
    /// // Example configuration:
    /// example: String = "default value"
    /// {{/if}}
    ///
    /// {{#if config.include_validation}}
    /// @IntRange { min = 1; max = 100 }
    /// {{/if}}
    /// property: Int
    /// ```
    pub config: crate::config::GeneratorConfig,

    /// Additional template variables for customization.
    ///
    /// Provides a flexible mechanism for passing custom data to templates,
    /// enabling project-specific customizations, metadata injection, and
    /// dynamic template behavior without modifying core generator logic.
    ///
    /// # Variable Types
    /// Variables can be any JSON-serializable value:
    /// - **Strings**: Version numbers, descriptions, author names
    /// - **Numbers**: Timestamps, version codes, limits
    /// - **Booleans**: Feature flags, conditional toggles
    /// - **Objects**: Complex configuration data
    /// - **Arrays**: Lists of items, repeated sections
    ///
    /// # Template Access
    /// ```handlebars
    /// // String variables
    /// // Generated by {{variables.generator_name}} v{{variables.version}}
    ///
    /// // Boolean variables
    /// {{#if variables.include_debug}}
    /// debug: Boolean = true
    /// {{/if}}
    ///
    /// // Object variables
    /// {{#with variables.project_info}}
    /// /// Project: {{name}} ({{version}})
    /// /// Author: {{author}}
    /// {{/with}}
    /// ```
    ///
    /// # Common Variables
    /// - `generator_version`: Version of the generator
    /// - `timestamp`: Generation timestamp
    /// - `project_name`: Name of the project being configured
    /// - `environment`: Target environment (dev, prod, etc.)
    /// - `features`: List of enabled features
    pub variables: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GeneratorConfig;

    #[test]
    fn test_pkl_module_creation() {
        let module = PklModule {
            name: "TestModule".to_string(),
            documentation: Some("Test module documentation".to_string()),
            imports: vec![],
            exports: vec![],
            types: vec![],
            properties: vec![],
        };

        assert_eq!(module.name, "TestModule");
        assert_eq!(
            module.documentation,
            Some("Test module documentation".to_string())
        );
        assert!(module.imports.is_empty());
        assert!(module.exports.is_empty());
        assert!(module.types.is_empty());
        assert!(module.properties.is_empty());
    }

    #[test]
    fn test_pkl_import_creation() {
        let import = PklImport {
            path: "some/module.pkl".to_string(),
            alias: Some("mod".to_string()),
            glob: false,
        };

        assert_eq!(import.path, "some/module.pkl");
        assert_eq!(import.alias, Some("mod".to_string()));
        assert!(!import.glob);
    }

    #[test]
    fn test_pkl_import_glob() {
        let import = PklImport {
            path: "some/module/*".to_string(),
            alias: None,
            glob: true,
        };

        assert_eq!(import.path, "some/module/*");
        assert!(import.alias.is_none());
        assert!(import.glob);
    }

    #[test]
    fn test_pkl_export_creation() {
        let export = PklExport {
            name: "MyType".to_string(),
            type_name: "MyTypeImpl".to_string(),
        };

        assert_eq!(export.name, "MyType");
        assert_eq!(export.type_name, "MyTypeImpl");
    }

    #[test]
    fn test_pkl_type_class() {
        let pkl_type = PklType {
            name: "TestClass".to_string(),
            documentation: Some("A test class".to_string()),
            kind: PklTypeKind::Class,
            properties: vec![],
            abstract_type: false,
            extends: vec![],
            enum_values: None,
            deprecated: None,
        };

        assert_eq!(pkl_type.name, "TestClass");
        assert_eq!(pkl_type.documentation, Some("A test class".to_string()));
        assert!(matches!(pkl_type.kind, PklTypeKind::Class));
        assert!(!pkl_type.abstract_type);
        assert!(pkl_type.extends.is_empty());
        assert!(pkl_type.enum_values.is_none());
        assert!(pkl_type.deprecated.is_none());
    }

    #[test]
    fn test_pkl_type_abstract_class() {
        let pkl_type = PklType {
            name: "AbstractClass".to_string(),
            documentation: None,
            kind: PklTypeKind::Class,
            properties: vec![],
            abstract_type: true,
            extends: vec!["BaseClass".to_string()],
            enum_values: None,
            deprecated: None,
        };

        assert!(pkl_type.abstract_type);
        assert_eq!(pkl_type.extends, vec!["BaseClass"]);
    }

    #[test]
    fn test_pkl_type_deprecated() {
        let pkl_type = PklType {
            name: "DeprecatedType".to_string(),
            documentation: None,
            kind: PklTypeKind::Class,
            properties: vec![],
            abstract_type: false,
            extends: vec![],
            enum_values: None,
            deprecated: Some("Use NewType instead".to_string()),
        };

        assert_eq!(pkl_type.deprecated, Some("Use NewType instead".to_string()));
    }

    #[test]
    fn test_pkl_type_enum() {
        let pkl_type = PklType {
            name: "StatusEnum".to_string(),
            documentation: Some("Status enumeration".to_string()),
            kind: PklTypeKind::Union,
            properties: vec![],
            abstract_type: false,
            extends: vec![],
            enum_values: Some("\"active\" | \"inactive\" | \"pending\"".to_string()),
            deprecated: None,
        };

        assert!(matches!(pkl_type.kind, PklTypeKind::Union));
        assert_eq!(
            pkl_type.enum_values,
            Some("\"active\" | \"inactive\" | \"pending\"".to_string())
        );
    }

    #[test]
    fn test_pkl_type_alias() {
        let pkl_type = PklType {
            name: "StringAlias".to_string(),
            documentation: None,
            kind: PklTypeKind::TypeAlias,
            properties: vec![],
            abstract_type: false,
            extends: vec![],
            enum_values: Some("String".to_string()),
            deprecated: None,
        };

        assert!(matches!(pkl_type.kind, PklTypeKind::TypeAlias));
        assert_eq!(pkl_type.enum_values, Some("String".to_string()));
    }

    #[test]
    fn test_pkl_property_required() {
        let property = PklProperty {
            name: "requiredField".to_string(),
            type_name: "String".to_string(),
            documentation: Some("A required field".to_string()),
            optional: false,
            default: None,
            constraints: vec![],
            examples: vec![],
            deprecated: None,
        };

        assert_eq!(property.name, "requiredField");
        assert_eq!(property.type_name, "String");
        assert!(!property.optional);
        assert!(property.default.is_none());
        assert!(property.constraints.is_empty());
        assert!(property.examples.is_empty());
    }

    #[test]
    fn test_pkl_property_optional_with_default() {
        let property = PklProperty {
            name: "optionalField".to_string(),
            type_name: "Int".to_string(),
            documentation: None,
            optional: true,
            default: Some("42".to_string()),
            constraints: vec![],
            examples: vec!["0".to_string(), "100".to_string()],
            deprecated: None,
        };

        assert!(property.optional);
        assert_eq!(property.default, Some("42".to_string()));
        assert_eq!(property.examples, vec!["0", "100"]);
    }

    #[test]
    fn test_pkl_property_deprecated() {
        let property = PklProperty {
            name: "oldField".to_string(),
            type_name: "String".to_string(),
            documentation: None,
            optional: false,
            default: None,
            constraints: vec![],
            examples: vec![],
            deprecated: Some("Use newField instead".to_string()),
        };

        assert_eq!(
            property.deprecated,
            Some("Use newField instead".to_string())
        );
    }

    #[test]
    fn test_pkl_constraint_min() {
        let constraint = PklConstraint {
            kind: PklConstraintKind::Min,
            value: "this >= 0".to_string(),
            message: Some("Must be non-negative".to_string()),
        };

        assert!(matches!(constraint.kind, PklConstraintKind::Min));
        assert_eq!(constraint.value, "this >= 0");
        assert_eq!(constraint.message, Some("Must be non-negative".to_string()));
    }

    #[test]
    fn test_pkl_constraint_max() {
        let constraint = PklConstraint {
            kind: PklConstraintKind::Max,
            value: "this <= 100".to_string(),
            message: Some("Must not exceed 100".to_string()),
        };

        assert!(matches!(constraint.kind, PklConstraintKind::Max));
        assert_eq!(constraint.value, "this <= 100");
    }

    #[test]
    fn test_pkl_constraint_length() {
        let constraint = PklConstraint {
            kind: PklConstraintKind::Length,
            value: "length >= 1".to_string(),
            message: Some("Must not be empty".to_string()),
        };

        assert!(matches!(constraint.kind, PklConstraintKind::Length));
        assert_eq!(constraint.value, "length >= 1");
    }

    #[test]
    fn test_pkl_constraint_pattern() {
        let constraint = PklConstraint {
            kind: PklConstraintKind::Pattern,
            value: "matches(Regex(#\"^[a-z]+$\"#))".to_string(),
            message: Some("Must contain only lowercase letters".to_string()),
        };

        assert!(matches!(constraint.kind, PklConstraintKind::Pattern));
        assert!(constraint.value.contains("Regex"));
    }

    #[test]
    fn test_pkl_constraint_custom() {
        let constraint = PklConstraint {
            kind: PklConstraintKind::Custom,
            value: "customValidation(this)".to_string(),
            message: None,
        };

        assert!(matches!(constraint.kind, PklConstraintKind::Custom));
        assert!(constraint.message.is_none());
    }

    #[test]
    fn test_template_context_creation() {
        let module = PklModule {
            name: "Test".to_string(),
            documentation: None,
            imports: vec![],
            exports: vec![],
            types: vec![],
            properties: vec![],
        };

        let config = GeneratorConfig::default();
        let mut variables = HashMap::new();
        variables.insert(
            "key".to_string(),
            serde_json::Value::String("value".to_string()),
        );

        let context = TemplateContext {
            module: module.clone(),
            config: config.clone(),
            variables: variables.clone(),
        };

        assert_eq!(context.module.name, "Test");
        assert_eq!(context.config.module_name, config.module_name);
        assert_eq!(context.variables.len(), 1);
        assert_eq!(
            context.variables.get("key"),
            Some(&serde_json::Value::String("value".to_string()))
        );
    }

    #[test]
    fn test_pkl_type_kind_serialization() {
        let kinds = vec![
            PklTypeKind::Class,
            PklTypeKind::TypeAlias,
            PklTypeKind::Union,
            PklTypeKind::Module,
        ];

        for kind in kinds {
            let json = serde_json::to_string(&kind).expect("Failed to serialize PklTypeKind");
            let deserialized: PklTypeKind =
                serde_json::from_str(&json).expect("Failed to deserialize PklTypeKind");

            // Check that serialization/deserialization preserves the variant
            match (&kind, &deserialized) {
                (PklTypeKind::Class, PklTypeKind::Class) => {}
                (PklTypeKind::TypeAlias, PklTypeKind::TypeAlias) => {}
                (PklTypeKind::Union, PklTypeKind::Union) => {}
                (PklTypeKind::Module, PklTypeKind::Module) => {}
                _ => panic!("Serialization/deserialization mismatch"),
            }
        }
    }

    #[test]
    fn test_pkl_constraint_kind_serialization() {
        let kinds = vec![
            PklConstraintKind::Min,
            PklConstraintKind::Max,
            PklConstraintKind::Length,
            PklConstraintKind::Pattern,
            PklConstraintKind::Custom,
        ];

        for kind in kinds {
            let json = serde_json::to_string(&kind).expect("Failed to serialize PklConstraintKind");
            let deserialized: PklConstraintKind =
                serde_json::from_str(&json).expect("Failed to deserialize PklConstraintKind");

            // Check that serialization/deserialization preserves the variant
            match (&kind, &deserialized) {
                (PklConstraintKind::Min, PklConstraintKind::Min) => {}
                (PklConstraintKind::Max, PklConstraintKind::Max) => {}
                (PklConstraintKind::Length, PklConstraintKind::Length) => {}
                (PklConstraintKind::Pattern, PklConstraintKind::Pattern) => {}
                (PklConstraintKind::Custom, PklConstraintKind::Custom) => {}
                _ => panic!("Serialization/deserialization mismatch"),
            }
        }
    }

    #[test]
    fn test_complex_pkl_module_with_types() {
        let property1 = PklProperty {
            name: "name".to_string(),
            type_name: "String".to_string(),
            documentation: Some("Object name".to_string()),
            optional: false,
            default: None,
            constraints: vec![PklConstraint {
                kind: PklConstraintKind::Length,
                value: "length >= 1".to_string(),
                message: Some("Name cannot be empty".to_string()),
            }],
            examples: vec!["example".to_string()],
            deprecated: None,
        };

        let property2 = PklProperty {
            name: "count".to_string(),
            type_name: "Int".to_string(),
            documentation: None,
            optional: true,
            default: Some("0".to_string()),
            constraints: vec![PklConstraint {
                kind: PklConstraintKind::Min,
                value: "this >= 0".to_string(),
                message: Some("Count must be non-negative".to_string()),
            }],
            examples: vec![],
            deprecated: None,
        };

        let pkl_type = PklType {
            name: "TestObject".to_string(),
            documentation: Some("A test object type".to_string()),
            kind: PklTypeKind::Class,
            properties: vec![property1, property2],
            abstract_type: false,
            extends: vec![],
            enum_values: None,
            deprecated: None,
        };

        let import = PklImport {
            path: "base.pkl".to_string(),
            alias: Some("base".to_string()),
            glob: false,
        };

        let export = PklExport {
            name: "TestObject".to_string(),
            type_name: "TestObject".to_string(),
        };

        let module = PklModule {
            name: "ComplexModule".to_string(),
            documentation: Some("A complex test module".to_string()),
            imports: vec![import],
            exports: vec![export],
            types: vec![pkl_type],
            properties: vec![],
        };

        assert_eq!(module.name, "ComplexModule");
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.exports.len(), 1);
        assert_eq!(module.types.len(), 1);

        let test_type = &module.types[0];
        assert_eq!(test_type.name, "TestObject");
        assert_eq!(test_type.properties.len(), 2);

        let name_prop = &test_type.properties[0];
        assert_eq!(name_prop.name, "name");
        assert!(!name_prop.optional);
        assert_eq!(name_prop.constraints.len(), 1);

        let count_prop = &test_type.properties[1];
        assert_eq!(count_prop.name, "count");
        assert!(count_prop.optional);
        assert_eq!(count_prop.default, Some("0".to_string()));
    }

    #[test]
    fn test_pkl_module_with_deep_nesting() {
        let nested_constraint = PklConstraint {
            kind: PklConstraintKind::Min,
            value: "this > 0".to_string(),
            message: Some("Must be positive".to_string()),
        };

        let nested_property = PklProperty {
            name: "nestedLevel".to_string(),
            type_name: "Int".to_string(),
            documentation: Some("Nested level depth".to_string()),
            optional: false,
            default: None,
            deprecated: None,
            constraints: vec![nested_constraint],
            examples: vec!["1".to_string(), "2".to_string()],
        };

        let inner_type = PklType {
            name: "InnerType".to_string(),
            documentation: Some("Inner nested type".to_string()),
            kind: PklTypeKind::Class,
            properties: vec![nested_property],
            abstract_type: false,
            extends: vec![],
            enum_values: None,
            deprecated: None,
        };

        let outer_property = PklProperty {
            name: "inner".to_string(),
            type_name: "InnerType".to_string(),
            documentation: Some("Reference to inner type".to_string()),
            optional: true,
            default: Some("new InnerType {}".to_string()),
            deprecated: None,
            constraints: vec![],
            examples: vec![],
        };

        let outer_type = PklType {
            name: "OuterType".to_string(),
            documentation: Some("Outer container type".to_string()),
            kind: PklTypeKind::Class,
            properties: vec![outer_property],
            abstract_type: false,
            extends: vec!["BaseType".to_string()],
            enum_values: None,
            deprecated: None,
        };

        let module = PklModule {
            name: "NestedModule".to_string(),
            documentation: Some("Module with nested types".to_string()),
            imports: vec![PklImport {
                path: "base.pkl".to_string(),
                alias: Some("base".to_string()),
                glob: false,
            }],
            exports: vec![
                PklExport {
                    name: "InnerType".to_string(),
                    type_name: "InnerType".to_string(),
                },
                PklExport {
                    name: "OuterType".to_string(),
                    type_name: "OuterType".to_string(),
                },
            ],
            types: vec![inner_type, outer_type],
            properties: vec![],
        };

        assert_eq!(module.types.len(), 2);
        assert_eq!(module.exports.len(), 2);

        let inner = &module.types[0];
        assert_eq!(inner.name, "InnerType");
        assert!(!inner.abstract_type);
        assert_eq!(inner.properties[0].constraints.len(), 1);
        assert_eq!(inner.properties[0].examples.len(), 2);

        let outer = &module.types[1];
        assert_eq!(outer.name, "OuterType");
        assert_eq!(outer.extends.len(), 1);
        assert_eq!(outer.extends[0], "BaseType");
    }

    #[test]
    fn test_pkl_property_constraints_validation() {
        let min_constraint = PklConstraint {
            kind: PklConstraintKind::Min,
            value: "this >= 10".to_string(),
            message: Some("Must be at least 10".to_string()),
        };

        let max_constraint = PklConstraint {
            kind: PklConstraintKind::Max,
            value: "this <= 100".to_string(),
            message: Some("Must be at most 100".to_string()),
        };

        let length_constraint = PklConstraint {
            kind: PklConstraintKind::Length,
            value: "this.length >= 5".to_string(),
            message: Some("Must be at least 5 characters".to_string()),
        };

        let pattern_constraint = PklConstraint {
            kind: PklConstraintKind::Pattern,
            value: "this.matches(Regex(\"^[A-Za-z]+$\"))".to_string(),
            message: Some("Must contain only letters".to_string()),
        };

        let custom_constraint = PklConstraint {
            kind: PklConstraintKind::Custom,
            value: "this.isValid()".to_string(),
            message: Some("Must be valid".to_string()),
        };

        let property = PklProperty {
            name: "validatedField".to_string(),
            type_name: "String".to_string(),
            documentation: Some("A field with multiple constraints".to_string()),
            optional: false,
            default: None,
            deprecated: None,
            constraints: vec![
                min_constraint,
                max_constraint,
                length_constraint,
                pattern_constraint,
                custom_constraint,
            ],
            examples: vec!["ValidExample".to_string()],
        };

        assert_eq!(property.constraints.len(), 5);
        assert!(property
            .constraints
            .iter()
            .any(|c| matches!(c.kind, PklConstraintKind::Min)));
        assert!(property
            .constraints
            .iter()
            .any(|c| matches!(c.kind, PklConstraintKind::Max)));
        assert!(property
            .constraints
            .iter()
            .any(|c| matches!(c.kind, PklConstraintKind::Length)));
        assert!(property
            .constraints
            .iter()
            .any(|c| matches!(c.kind, PklConstraintKind::Pattern)));
        assert!(property
            .constraints
            .iter()
            .any(|c| matches!(c.kind, PklConstraintKind::Custom)));
    }

    #[test]
    fn test_pkl_type_serialization_edge_cases() {
        // Test TypeAlias serialization
        let typealias = PklType {
            name: "StringOrInt".to_string(),
            documentation: Some("Union type alias".to_string()),
            kind: PklTypeKind::TypeAlias,
            properties: vec![],
            abstract_type: false,
            extends: vec![],
            enum_values: Some("String | Int".to_string()),
            deprecated: Some("Use specific types instead".to_string()),
        };

        let serialized = serde_json::to_string(&typealias).expect("Failed to serialize");
        let deserialized: PklType =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(typealias.name, deserialized.name);
        assert_eq!(typealias.kind, deserialized.kind);
        assert_eq!(typealias.enum_values, deserialized.enum_values);
        assert_eq!(typealias.deprecated, deserialized.deprecated);

        // Test Enum serialization
        let enum_type = PklType {
            name: "Color".to_string(),
            documentation: Some("Color enumeration".to_string()),
            kind: PklTypeKind::Union,
            properties: vec![],
            abstract_type: false,
            extends: vec![],
            enum_values: Some("\"red\" | \"green\" | \"blue\"".to_string()),
            deprecated: None,
        };

        let enum_serialized = serde_json::to_string(&enum_type).expect("Failed to serialize enum");
        let enum_deserialized: PklType =
            serde_json::from_str(&enum_serialized).expect("Failed to deserialize enum");

        assert_eq!(enum_type.enum_values, enum_deserialized.enum_values);
        assert_eq!(enum_type.kind, enum_deserialized.kind);
    }

    #[test]
    fn test_template_context_with_complex_variables() {
        let module = PklModule {
            name: "ComplexTemplate".to_string(),
            documentation: Some("Template with complex variables".to_string()),
            imports: vec![],
            exports: vec![],
            types: vec![],
            properties: vec![],
        };

        let config = GeneratorConfig::default();
        let mut variables = HashMap::new();

        // Add complex nested variables
        variables.insert(
            "nested_object".to_string(),
            serde_json::json!({
                "level1": {
                    "level2": {
                        "value": "deep_value",
                        "array": [1, 2, 3]
                    }
                }
            }),
        );

        variables.insert(
            "array_of_objects".to_string(),
            serde_json::json!([
                {"name": "item1", "enabled": true},
                {"name": "item2", "enabled": false}
            ]),
        );

        variables.insert(
            "primitive_types".to_string(),
            serde_json::json!({
                "string": "test",
                "number": 42,
                "boolean": true,
                "null_value": null
            }),
        );

        let context = TemplateContext {
            module: module.clone(),
            config: config.clone(),
            variables: variables.clone(),
        };

        assert_eq!(context.variables.len(), 3);

        let nested = context.variables.get("nested_object").unwrap();
        assert!(nested.get("level1").is_some());

        let array = context.variables.get("array_of_objects").unwrap();
        assert!(array.as_array().is_some());
        assert_eq!(array.as_array().unwrap().len(), 2);

        let primitives = context.variables.get("primitive_types").unwrap();
        assert_eq!(primitives.get("string").unwrap().as_str().unwrap(), "test");
        assert_eq!(primitives.get("number").unwrap().as_i64().unwrap(), 42);
        assert_eq!(primitives.get("boolean").unwrap().as_bool().unwrap(), true);
        assert!(primitives.get("null_value").unwrap().is_null());
    }

    #[test]
    fn test_pkl_module_with_circular_dependencies() {
        // Test module A that imports B
        let module_a = PklModule {
            name: "ModuleA".to_string(),
            documentation: Some("Module A with dependency on B".to_string()),
            imports: vec![PklImport {
                path: "module_b.pkl".to_string(),
                alias: Some("B".to_string()),
                glob: false,
            }],
            exports: vec![PklExport {
                name: "TypeA".to_string(),
                type_name: "TypeA".to_string(),
            }],
            types: vec![PklType {
                name: "TypeA".to_string(),
                documentation: Some("Type that uses B.TypeB".to_string()),
                kind: PklTypeKind::Class,
                properties: vec![PklProperty {
                    name: "ref_to_b".to_string(),
                    type_name: "B.TypeB".to_string(),
                    documentation: Some("Reference to type in module B".to_string()),
                    optional: true,
                    default: None,
                    deprecated: None,
                    constraints: vec![],
                    examples: vec![],
                }],
                abstract_type: false,
                extends: vec![],
                enum_values: None,
                deprecated: None,
            }],
            properties: vec![],
        };

        // Test module B that imports A
        let module_b = PklModule {
            name: "ModuleB".to_string(),
            documentation: Some("Module B with dependency on A".to_string()),
            imports: vec![PklImport {
                path: "module_a.pkl".to_string(),
                alias: Some("A".to_string()),
                glob: false,
            }],
            exports: vec![PklExport {
                name: "TypeB".to_string(),
                type_name: "TypeB".to_string(),
            }],
            types: vec![PklType {
                name: "TypeB".to_string(),
                documentation: Some("Type that uses A.TypeA".to_string()),
                kind: PklTypeKind::Class,
                properties: vec![PklProperty {
                    name: "ref_to_a".to_string(),
                    type_name: "A.TypeA".to_string(),
                    documentation: Some("Reference to type in module A".to_string()),
                    optional: true,
                    default: None,
                    deprecated: None,
                    constraints: vec![],
                    examples: vec![],
                }],
                abstract_type: false,
                extends: vec![],
                enum_values: None,
                deprecated: None,
            }],
            properties: vec![],
        };

        // Verify that modules can reference each other
        assert_eq!(module_a.imports[0].path, "module_b.pkl");
        assert_eq!(module_b.imports[0].path, "module_a.pkl");

        let type_a_prop = &module_a.types[0].properties[0];
        assert_eq!(type_a_prop.type_name, "B.TypeB");

        let type_b_prop = &module_b.types[0].properties[0];
        assert_eq!(type_b_prop.type_name, "A.TypeA");
    }

    #[test]
    fn test_pkl_constraint_kind_all_variants() {
        let all_constraint_kinds = vec![
            PklConstraintKind::Min,
            PklConstraintKind::Max,
            PklConstraintKind::Length,
            PklConstraintKind::Pattern,
            PklConstraintKind::Custom,
        ];

        for kind in all_constraint_kinds {
            let constraint = PklConstraint {
                kind: kind.clone(),
                value: "test_value".to_string(),
                message: Some("Test message".to_string()),
            };

            // Test serialization
            let serialized =
                serde_json::to_string(&constraint).expect("Failed to serialize constraint");
            let deserialized: PklConstraint =
                serde_json::from_str(&serialized).expect("Failed to deserialize constraint");

            assert_eq!(constraint.kind, deserialized.kind);
            assert_eq!(constraint.value, deserialized.value);
        }
    }

    #[test]
    fn test_pkl_type_kind_all_variants() {
        let all_type_kinds = vec![
            PklTypeKind::Class,
            PklTypeKind::TypeAlias,
            PklTypeKind::Union,
        ];

        for kind in all_type_kinds {
            let pkl_type = PklType {
                name: format!("Test{:?}", kind),
                documentation: Some(format!("Test {:?} type", kind)),
                kind: kind.clone(),
                properties: vec![],
                abstract_type: false,
                extends: vec![],
                enum_values: None,
                deprecated: None,
            };

            // Test serialization
            let serialized = serde_json::to_string(&pkl_type).expect("Failed to serialize type");
            let deserialized: PklType =
                serde_json::from_str(&serialized).expect("Failed to deserialize type");

            assert_eq!(pkl_type.kind, deserialized.kind);
            assert_eq!(pkl_type.name, deserialized.name);
        }
    }

    #[test]
    fn test_pkl_property_edge_cases() {
        // Test property with empty documentation
        let prop_empty_doc = PklProperty {
            name: "empty_doc".to_string(),
            type_name: "String".to_string(),
            documentation: Some("".to_string()),
            optional: false,
            default: None,
            deprecated: None,
            constraints: vec![],
            examples: vec![],
        };

        assert_eq!(prop_empty_doc.documentation, Some("".to_string()));

        // Test property with very long documentation
        let long_doc = "A".repeat(1000);
        let prop_long_doc = PklProperty {
            name: "long_doc".to_string(),
            type_name: "String".to_string(),
            documentation: Some(long_doc.clone()),
            optional: false,
            default: None,
            deprecated: None,
            constraints: vec![],
            examples: vec![],
        };

        assert_eq!(prop_long_doc.documentation, Some(long_doc));

        // Test property with special characters in name
        let prop_special_chars = PklProperty {
            name: "property_with_underscores_and_123".to_string(),
            type_name: "String".to_string(),
            documentation: None,
            optional: true,
            default: Some("\"special \\\"quoted\\\" value\"".to_string()),
            deprecated: Some("Reason: contains special characters".to_string()),
            constraints: vec![],
            examples: vec!["\"example\"".to_string()],
        };

        assert!(prop_special_chars.name.contains("_"));
        assert!(prop_special_chars.name.contains("123"));
        assert!(prop_special_chars
            .default
            .as_ref()
            .unwrap()
            .contains("\\\""));
    }

    #[test]
    fn test_template_context_serialization_edge_cases() {
        let module = PklModule {
            name: "SerializationTest".to_string(),
            documentation: None,
            imports: vec![],
            exports: vec![],
            types: vec![],
            properties: vec![],
        };

        let config = GeneratorConfig::default();
        let mut variables = HashMap::new();

        // Test with empty variables
        let context_empty = TemplateContext {
            module: module.clone(),
            config: config.clone(),
            variables: HashMap::new(),
        };

        let serialized_empty = serde_json::to_value(&context_empty);
        assert!(serialized_empty.is_ok());

        // Test with complex variables containing edge cases
        variables.insert(
            "edge_cases".to_string(),
            serde_json::json!({
                "empty_string": "",
                "unicode": "🚀 Rust with Unicode",
                "newlines": "line1\nline2\nline3",
                "tabs": "column1\tcolumn2",
                "quotes": "He said \"Hello World!\"",
                "large_number": 9223372036854775807i64,
                "small_number": -9223372036854775808i64,
                "float": 3.14159265359,
                "scientific": 1.23e10
            }),
        );

        let context_complex = TemplateContext {
            module,
            config,
            variables,
        };

        let serialized_complex = serde_json::to_value(&context_complex);
        assert!(serialized_complex.is_ok());

        let json_value = serialized_complex.unwrap();
        let edge_cases = json_value
            .get("variables")
            .unwrap()
            .get("edge_cases")
            .unwrap();

        assert_eq!(
            edge_cases.get("empty_string").unwrap().as_str().unwrap(),
            ""
        );
        assert!(edge_cases
            .get("unicode")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("🚀"));
        assert!(edge_cases
            .get("newlines")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("\n"));
    }
}
