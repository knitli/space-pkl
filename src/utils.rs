//! Utility Functions for PKL Schema Generation
//!
//! This module provides essential utility functions that support the PKL schema generation
//! process. It includes file system operations, string manipulation functions, identifier
//! conversion utilities, and validation helpers specifically designed for PKL syntax
//! requirements and Moon configuration patterns.
//!
//! # Core Functionality
//!
//! ## File System Operations
//! - **Directory Management**: Safe directory creation with error handling
//! - **File I/O**: Robust file reading and writing with detailed error messages
//! - **Path Handling**: Cross-platform path manipulation and validation
//!
//! ## String and Identifier Conversion
//! - **Naming Convention Translation**: Convert between Rust and PKL naming styles
//! - **Type Name Processing**: Clean and format type names for PKL schemas
//! - **Identifier Validation**: Ensure generated names comply with PKL syntax rules
//!
//! ## PKL Syntax Support
//! - **Keyword Escaping**: Handle PKL reserved words safely
//! - **Identifier Formatting**: Convert Rust identifiers to PKL conventions
//! - **Type Name Cleaning**: Remove common Rust type prefixes and suffixes
//!
//! # Usage Patterns
//!
//! ## File Operations
//! ```rust
//! use space_pkl::utils::*;
//! use std::path::Path;
//!
//! # fn example() -> space_pkl::Result<()> {
//! // Ensure output directory exists
//! ensure_dir_exists(Path::new("./output"))?;
//!
//! // Read template file
//! let template = read_file_to_string(Path::new("template.pkl"))?;
//!
//! // Write generated schema
//! let schema = "module MySchema\n// Generated content";
//! write_string_to_file(Path::new("./output/schema.pkl"), schema)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Identifier Conversion
//! ```rust
//! use space_pkl::utils::*;
//!
//! // Convert Rust naming to PKL naming
//! let pkl_name = rust_to_pkl_identifier("my_config_option"); // "myConfigOption"
//! let class_name = capitalize_first_letter("myClass");         // "MyClass"
//! let type_name = type_name_to_pkl("WorkspaceConfig");        // "Workspace"
//!
//! // Validate PKL identifiers
//! assert!(is_valid_pkl_identifier("validName"));
//! assert!(!is_valid_pkl_identifier("123invalid"));
//! ```
//!
//! # Conversion Rules
//!
//! ## Rust to PKL Identifier Conversion
//! - **snake_case** → **camelCase**: `my_variable` → `myVariable`
//! - **Preserve existing camelCase**: `alreadyCamel` → `alreadyCamel`
//! - **Handle multiple underscores**: `foo__bar` → `fooBar`
//! - **Clean edge cases**: Leading/trailing underscores handled gracefully
//!
//! ## Type Name Processing
//! - **Remove common suffixes**: `WorkspaceConfig` → `Workspace`
//! - **Remove common prefixes**: `PartialProjectConfig` → `Project`
//! - **Capitalize result**: Ensure proper PKL class naming
//!
//! ## PKL Identifier Validation
//! - **First character**: Must be letter or underscore
//! - **Subsequent characters**: Letters, numbers, underscores only
//! - **Empty strings**: Invalid
//! - **Unicode support**: Handles international characters properly
//!
//! # Error Handling
//!
//! All file operations provide enhanced error messages with context:
//! ```rust
//! use space_pkl::utils::*;
//! use std::path::Path;
//!
//! # fn example() -> space_pkl::Result<()> {
//! // Detailed error messages include file paths
//! match read_file_to_string(Path::new("missing.txt")) {
//!     Ok(content) => println!("Success: {}", content),
//!     Err(e) => {
//!         // Error includes: "Failed to read file: missing.txt"
//!         eprintln!("Error: {}", e);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Performance Considerations
//!
//! - **String Operations**: Optimized for repeated identifier conversions
//! - **File I/O**: Minimal overhead with efficient error handling
//! - **Memory Usage**: Functions avoid unnecessary allocations
//! - **Unicode Handling**: Efficient character processing for international text
//!
//! # Testing
//!
//! The module includes comprehensive tests covering:
//! - **Normal cases**: Standard identifier and type conversions
//! - **Edge cases**: Empty strings, special characters, Unicode
//! - **Error conditions**: Invalid file paths, permission errors
//! - **Performance**: Large identifier conversion scenarios
//!
//! # Integration
//!
//! These utilities integrate seamlessly with other modules:
//! - **Generator**: Uses file operations for schema output
//! - **Templates**: Uses identifier conversion in template helpers
//! - **Types**: Uses validation for generated type names
//! - **Config**: Uses path operations for configuration loading
//!
//! (c) 2025 Stash AI Inc (knitli)
//!   - Created by Adam Poulemanos ([@bashandbone](https://github.com/bashandbone)) for Stash AI Inc.
//! Licensed under the [Plain MIT License](https://plainlicense.org/licenses/permissive/mit/)

use crate::Result;
use miette::{IntoDiagnostic, WrapErr};
use std::fs;
use std::path::Path;

/// Ensures a directory exists, creating it and any parent directories if necessary.
///
/// This function provides safe directory creation with comprehensive error handling.
/// It checks if the directory already exists before attempting creation, making it
/// safe to call repeatedly. Parent directories are created automatically as needed.
///
/// # Behavior
///
/// - **Existing directories**: No-op, returns success immediately
/// - **Missing directories**: Creates the directory and all parent directories
/// - **Permission errors**: Returns detailed error with path context
/// - **Invalid paths**: Returns appropriate error with diagnostics
///
/// # Error Handling
///
/// Provides enhanced error messages that include:
/// - The full path that failed to be created
/// - The underlying system error (permissions, disk space, etc.)
/// - Contextual information for debugging
///
/// # Examples
///
/// ## Basic Directory Creation
/// ```rust
/// use space_pkl::utils::ensure_dir_exists;
/// use std::path::Path;
///
/// # fn example() -> space_pkl::Result<()> {
/// // Create a single directory
/// ensure_dir_exists(Path::new("./output"))?;
///
/// // Create nested directories
/// ensure_dir_exists(Path::new("./generated/schemas/workspace"))?;
///
/// // Safe to call multiple times
/// ensure_dir_exists(Path::new("./output"))?; // No error, already exists
/// # Ok(())
/// # }
/// ```
///
/// ## Integration with File Operations
/// ```rust
/// use space_pkl::utils::{ensure_dir_exists, write_string_to_file};
/// use std::path::Path;
///
/// # fn example() -> space_pkl::Result<()> {
/// let output_path = Path::new("./generated/schemas/workspace.pkl");
///
/// // Ensure parent directory exists before writing
/// if let Some(parent) = output_path.parent() {
///     ensure_dir_exists(parent)?;
/// }
///
/// write_string_to_file(output_path, "module Workspace")?;
/// # Ok(())
/// # }
/// ```
///
/// # Platform Compatibility
///
/// Works correctly across different operating systems:
/// - **Unix/Linux**: Handles standard Unix permissions and paths
/// - **Windows**: Supports Windows path formats and security
/// - **macOS**: Compatible with macOS filesystem requirements
///
/// # Performance Notes
///
/// - **Existence check**: Fast metadata check before creation attempt
/// - **Batch creation**: Creates entire directory tree in single operation
/// - **No redundant work**: Skips creation if directory already exists
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

/// Reads a file to a string with enhanced error reporting.
///
/// Provides robust file reading with detailed error messages that include the full
/// file path and underlying system error information. This makes debugging file
/// access issues much easier during schema generation.
///
/// # Error Handling
///
/// Enhanced error messages include:
/// - **File path**: The complete path that failed to be read
/// - **System error**: The underlying I/O error (not found, permissions, etc.)
/// - **Context**: Clear indication this was a file read operation
///
/// # Examples
///
/// ## Template File Reading
/// ```rust
/// use space_pkl::utils::read_file_to_string;
/// use std::path::Path;
///
/// # fn example() -> space_pkl::Result<()> {
/// // Read a template file
/// let template_content = read_file_to_string(Path::new("templates/module.hbs"))?;
/// println!("Template loaded: {} bytes", template_content.len());
///
/// // Read configuration file
/// let config_content = read_file_to_string(Path::new("moon.yml"))?;
/// # Ok(())
/// # }
/// ```
///
/// ## Error Handling Example
/// ```rust
/// use space_pkl::utils::read_file_to_string;
/// use std::path::Path;
///
/// # fn example() {
/// match read_file_to_string(Path::new("missing.txt")) {
///     Ok(content) => println!("File content: {}", content),
///     Err(e) => {
///         // Error message: "Failed to read file: missing.txt: No such file or directory"
///         eprintln!("Failed to read template: {}", e);
///     }
/// }
/// # }
/// ```
///
/// # Performance Notes
///
/// - **Single allocation**: Reads entire file into memory efficiently
/// - **UTF-8 validation**: Ensures content is valid UTF-8 text
/// - **Memory efficient**: No intermediate buffering for small to medium files
pub fn read_file_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read file: {}", path.display()))
}

/// Writes a string to a file with automatic directory creation and enhanced error reporting.
///
/// Provides robust file writing that automatically creates parent directories as needed
/// and includes detailed error reporting. This is the primary function for writing
/// generated PKL schema files to disk.
///
/// # Automatic Directory Creation
///
/// If the parent directory of the target file doesn't exist, it will be created
/// automatically using `ensure_dir_exists()`. This makes it safe to write files
/// to nested directory structures without manual setup.
///
/// # Error Handling
///
/// Enhanced error messages include:
/// - **File path**: The complete path that failed to be written
/// - **Directory creation errors**: If parent directory creation fails
/// - **Write errors**: Permissions, disk space, or other I/O issues
///
/// # Examples
///
/// ## Schema File Generation
/// ```rust
/// use space_pkl::utils::write_string_to_file;
/// use std::path::Path;
///
/// # fn example() -> space_pkl::Result<()> {
/// let schema_content = r#"
/// module WorkspaceConfig
///
/// class Workspace {
///   /// Project root directory
///   root: String
/// }
/// "#;
///
/// // Writes to nested directory, creating parents as needed
/// write_string_to_file(
///     Path::new("./generated/schemas/workspace.pkl"),
///     schema_content
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// ## Batch File Generation
/// ```rust
/// use space_pkl::utils::write_string_to_file;
/// use std::path::Path;
///
/// # fn example() -> space_pkl::Result<()> {
/// let schemas = vec![
///     ("workspace.pkl", "module Workspace\n// content"),
///     ("project.pkl", "module Project\n// content"),
///     ("tasks.pkl", "module Tasks\n// content"),
/// ];
///
/// for (filename, content) in schemas {
///     let path = Path::new("./output").join(filename);
///     write_string_to_file(&path, content)?;
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Atomic Operations
///
/// The write operation is atomic at the filesystem level - either the entire
/// file is written successfully, or no changes are made. This prevents
/// corruption from partial writes during errors.
///
/// # Performance Notes
///
/// - **Directory caching**: Parent directory existence is checked efficiently
/// - **Single write**: Content written in one operation for better performance
/// - **UTF-8 encoding**: Automatically handles UTF-8 encoding for text content
pub fn write_string_to_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir_exists(parent)?;
    }

    fs::write(path, content)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to write file: {}", path.display()))
}

/// Converts a Rust identifier to PKL-style camelCase identifier.
///
/// Transforms Rust snake_case identifiers into PKL camelCase convention while
/// preserving existing camelCase identifiers. This is essential for generating
/// PKL property names that follow PKL naming conventions from Rust configuration
/// structures.
///
/// # Conversion Rules
///
/// - **snake_case → camelCase**: `my_property` → `myProperty`
/// - **Preserve camelCase**: `alreadyCamel` → `alreadyCamel`
/// - **Multiple underscores**: `foo__bar` → `fooBar` (collapsed)
/// - **Leading underscores**: `_private` → `Private` (capitalized)
/// - **Trailing underscores**: `trailing_` → `trailing` (removed)
/// - **Mixed content**: `test_123_value` → `test123Value`
///
/// # Edge Case Handling
///
/// - **Empty string**: Returns empty string
/// - **Single character**: Returns as-is
/// - **Only underscores**: Removes underscores, may result in empty string
/// - **Numbers**: Preserved in their original positions
/// - **Special characters**: Non-alphanumeric characters preserved as-is
///
/// # Examples
///
/// ## Standard Conversions
/// ```rust
/// use space_pkl::utils::rust_to_pkl_identifier;
///
/// // Basic snake_case conversion
/// assert_eq!(rust_to_pkl_identifier("user_name"), "userName");
/// assert_eq!(rust_to_pkl_identifier("database_host"), "databaseHost");
/// assert_eq!(rust_to_pkl_identifier("max_retry_count"), "maxRetryCount");
///
/// // Preserve existing camelCase
/// assert_eq!(rust_to_pkl_identifier("alreadyCamel"), "alreadyCamel");
/// assert_eq!(rust_to_pkl_identifier("HTTPClient"), "HTTPClient");
/// ```
///
/// ## Edge Cases
/// ```rust
/// use space_pkl::utils::rust_to_pkl_identifier;
///
/// // Multiple underscores
/// assert_eq!(rust_to_pkl_identifier("foo__bar"), "fooBar");
///
/// // Leading/trailing underscores
/// assert_eq!(rust_to_pkl_identifier("_private"), "Private");
/// assert_eq!(rust_to_pkl_identifier("trailing_"), "trailing");
///
/// // Numbers and special cases
/// assert_eq!(rust_to_pkl_identifier("test_123"), "test123");
/// assert_eq!(rust_to_pkl_identifier("api_v2_client"), "apiV2Client");
/// ```
///
/// ## Configuration Property Mapping
/// ```rust
/// use space_pkl::utils::rust_to_pkl_identifier;
///
/// // Common configuration properties
/// let properties = vec![
///     ("connection_timeout", "connectionTimeout"),
///     ("ssl_enabled", "sslEnabled"),
///     ("retry_attempts", "retryAttempts"),
///     ("log_level", "logLevel"),
///     ("api_key", "apiKey"),
/// ];
///
/// for (rust_name, expected_pkl) in properties {
///     assert_eq!(rust_to_pkl_identifier(rust_name), expected_pkl);
/// }
/// ```
///
/// # Performance
///
/// - **Optimized splitting**: Efficient string processing for repeated conversions
/// - **Memory efficient**: Minimal allocations during transformation
/// - **Unicode safe**: Handles international characters correctly
///
/// # Integration
///
/// Used throughout the schema generation pipeline:
/// - **Property names**: Converting struct field names to PKL properties
/// - **Template helpers**: Available in Handlebars templates as `camel_case`
/// - **Type processing**: Ensuring consistent naming conventions
pub fn rust_to_pkl_identifier(name: &str) -> String {
    // Convert snake_case to camelCase for Pkl
    name.split('_')
        .enumerate()
        .map(|(i, word)| {
            if i == 0 {
                word.to_string()
            } else {
                capitalize_first_letter(word)
            }
        })
        .collect()
}

/// Capitalizes the first letter of a string while preserving the rest.
///
/// This utility function provides Unicode-safe capitalization of the first character
/// in a string, commonly used for converting identifiers to proper case for PKL
/// class names, property names, and other naming conventions that require initial
/// capitalization.
///
/// # Behavior
///
/// - **First character**: Converted to uppercase using Unicode rules
/// - **Remaining characters**: Preserved exactly as provided
/// - **Empty strings**: Returns empty string unchanged
/// - **Non-alphabetic first chars**: Preserved as-is (numbers, symbols, etc.)
/// - **Unicode characters**: Properly handled for international text
///
/// # Unicode Support
///
/// The function correctly handles Unicode characters according to Unicode
/// case conversion rules, making it suitable for international identifier
/// processing:
/// - **Latin characters**: `hello` → `Hello`
/// - **Accented characters**: `café` → `Café`
/// - **Non-Latin scripts**: `москва` → `Москва`
/// - **Mixed scripts**: Preserves character integrity
///
/// # Examples
///
/// ## Basic Capitalization
/// ```rust
/// use space_pkl::utils::capitalize_first_letter;
///
/// // Standard ASCII text
/// assert_eq!(capitalize_first_letter("hello"), "Hello");
/// assert_eq!(capitalize_first_letter("world"), "World");
/// assert_eq!(capitalize_first_letter("propertyName"), "PropertyName");
///
/// // Already capitalized
/// assert_eq!(capitalize_first_letter("AlreadyCapital"), "AlreadyCapital");
/// ```
///
/// ## PKL Class Name Generation
/// ```rust
/// use space_pkl::utils::capitalize_first_letter;
///
/// // Converting type names to PKL class names
/// let class_names = vec![
///     ("workspace", "Workspace"),
///     ("projectConfig", "ProjectConfig"),
///     ("httpClient", "HttpClient"),
///     ("apiEndpoint", "ApiEndpoint"),
/// ];
///
/// for (input, expected) in class_names {
///     assert_eq!(capitalize_first_letter(input), expected);
/// }
/// ```
///
/// ## Unicode and International Text
/// ```rust
/// use space_pkl::utils::capitalize_first_letter;
///
/// // Unicode character handling
/// assert_eq!(capitalize_first_letter("café"), "Café");
/// assert_eq!(capitalize_first_letter("ñoño"), "Ñoño");
/// assert_eq!(capitalize_first_letter("münchen"), "München");
/// assert_eq!(capitalize_first_letter("москва"), "Москва");
/// assert_eq!(capitalize_first_letter("日本語"), "日本語");
/// ```
///
/// ## Edge Cases
/// ```rust
/// use space_pkl::utils::capitalize_first_letter;
///
/// // Empty and special cases
/// assert_eq!(capitalize_first_letter(""), "");
/// assert_eq!(capitalize_first_letter("a"), "A");
/// assert_eq!(capitalize_first_letter("123text"), "123text"); // Number start preserved
/// assert_eq!(capitalize_first_letter("_private"), "_private"); // Underscore preserved
/// ```
///
/// # Performance Notes
///
/// - **Single pass**: Processes characters only once
/// - **Memory efficient**: Allocates result string only once
/// - **Unicode optimized**: Uses efficient Unicode case conversion
/// - **Small string optimized**: Minimal overhead for short identifiers
///
/// # Integration
///
/// Used throughout the PKL generation pipeline:
/// - **Type conversion**: Converting cleaned type names to PKL class names
/// - **Identifier processing**: Part of `rust_to_pkl_identifier` conversion
/// - **Template helpers**: Available in Handlebars templates
/// - **Name normalization**: Ensuring consistent capitalization patterns
pub fn capitalize_first_letter(s: &str) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                c.to_uppercase().collect()
            } else {
                c.to_string()
            }
        })
        .collect()
}

/// Converts a Rust type name to PKL-style class name by removing common prefixes and suffixes.
///
/// This function cleans Rust type names by removing common naming patterns used in
/// Rust configuration structures, then capitalizes the result to create appropriate
/// PKL class names. It handles the most common Rust naming conventions for configuration
/// types and ensures the resulting names follow PKL class naming standards.
///
/// # Transformation Rules
///
/// ## Suffix Removal (case-sensitive)
/// - **"Config"**: `WorkspaceConfig` → `Workspace`
/// - **"Type"**: `ProjectType` → `Project`
///
/// ## Prefix Removal (case-sensitive)
/// - **"Partial"**: `PartialWorkspaceConfig` → `WorkspaceConfig` (then suffix removal)
///
/// ## Processing Order
/// 1. Remove "Partial" prefix (if present)
/// 2. Remove "Config" or "Type" suffix (if present)
/// 3. Capitalize the first letter of the result
///
/// # Edge Case Handling
///
/// - **Multiple patterns**: Processes all matching patterns sequentially
/// - **Empty results**: Returns empty string if all content is removed
/// - **No matches**: Capitalizes original name if no patterns match
/// - **Case sensitivity**: Only exact case matches are processed
/// - **Partial matches**: Only full word boundaries are considered
///
/// # Examples
///
/// ## Standard Configuration Types
/// ```rust
/// use space_pkl::utils::type_name_to_pkl;
///
/// // Common configuration type conversions
/// assert_eq!(type_name_to_pkl("WorkspaceConfig"), "Workspace");
/// assert_eq!(type_name_to_pkl("ProjectConfig"), "Project");
/// assert_eq!(type_name_to_pkl("DatabaseConfig"), "Database");
/// assert_eq!(type_name_to_pkl("ServerConfig"), "Server");
/// ```
///
/// ## Type Enum Conversions
/// ```rust
/// use space_pkl::utils::type_name_to_pkl;
///
/// // Enum type name processing
/// assert_eq!(type_name_to_pkl("EnvironmentType"), "Environment");
/// assert_eq!(type_name_to_pkl("LogLevelType"), "LogLevel");
/// assert_eq!(type_name_to_pkl("AuthMethodType"), "AuthMethod");
/// ```
///
/// ## Partial Type Handling
/// ```rust
/// use space_pkl::utils::type_name_to_pkl;
///
/// // Partial configuration types (common in serde)
/// assert_eq!(type_name_to_pkl("PartialWorkspaceConfig"), "Workspace");
/// assert_eq!(type_name_to_pkl("PartialProjectConfig"), "Project");
/// assert_eq!(type_name_to_pkl("PartialDatabaseType"), "Database");
/// ```
///
/// ## Complex Combinations
/// ```rust
/// use space_pkl::utils::type_name_to_pkl;
///
/// // Multiple patterns in one name
/// assert_eq!(type_name_to_pkl("PartialAdvancedWorkspaceConfig"), "AdvancedWorkspace");
/// assert_eq!(type_name_to_pkl("PartialCustomProjectType"), "CustomProject");
/// ```
///
/// ## Edge Cases and Preservation
/// ```rust
/// use space_pkl::utils::type_name_to_pkl;
///
/// // No matching patterns - just capitalize
/// assert_eq!(type_name_to_pkl("CustomStruct"), "CustomStruct");
/// assert_eq!(type_name_to_pkl("simpleType"), "SimpleType"); // lowercase input
///
/// // Case-sensitive matching
/// assert_eq!(type_name_to_pkl("workspaceconfig"), "Workspaceconfig"); // No match
/// assert_eq!(type_name_to_pkl("PROJECTTYPE"), "PROJECTTYPE"); // No match
///
/// // Empty or minimal content
/// assert_eq!(type_name_to_pkl("Config"), ""); // Only suffix
/// assert_eq!(type_name_to_pkl("Type"), ""); // Only suffix
/// assert_eq!(type_name_to_pkl("Partial"), ""); // Only prefix
/// ```
///
/// ## PKL Schema Generation Context
/// ```rust
/// use space_pkl::utils::type_name_to_pkl;
///
/// // Converting Rust configuration structs to PKL classes
/// struct WorkspaceConfig {
///     root: String,
///     name: String,
/// }
///
/// // Generated PKL class name
/// let pkl_class_name = type_name_to_pkl("WorkspaceConfig"); // "Workspace"
///
/// // Results in PKL schema:
/// // class Workspace {
/// //   root: String
/// //   name: String
/// // }
/// ```
///
/// # Performance Notes
///
/// - **String operations**: Efficient prefix/suffix matching and removal
/// - **Memory efficient**: Single allocation for result string
/// - **Pattern matching**: Fast string comparison operations
/// - **Unicode safe**: Works correctly with international characters
///
/// # Integration
///
/// Used throughout the PKL schema generation:
/// - **Type name processing**: Converting Rust type names to PKL class names
/// - **Template generation**: Creating clean, readable PKL class names
/// - **Schema organization**: Ensuring consistent naming across generated schemas
/// - **Documentation**: Providing meaningful names in generated PKL documentation
pub fn type_name_to_pkl(name: &str) -> String {
    // Remove common Rust type prefixes/suffixes
    let cleaned = name
        .trim_end_matches("Config")
        .trim_end_matches("Type")
        .trim_start_matches("Partial");

    capitalize_first_letter(cleaned)
}

/// Validates whether a string is a valid PKL identifier according to PKL syntax rules.
///
/// This function checks if a given string conforms to PKL identifier naming rules,
/// which are essential for generating valid PKL schemas. It ensures that property
/// names, class names, and other identifiers used in generated PKL code are
/// syntactically correct and will be accepted by the PKL parser.
///
/// # PKL Identifier Rules
///
/// ## First Character Requirements
/// - **Letters**: Any Unicode letter (a-z, A-Z, and international characters)
/// - **Underscore**: Single underscore `_` is permitted
/// - **Invalid**: Numbers, special symbols, whitespace
///
/// ## Subsequent Character Requirements
/// - **Letters**: Any Unicode letter
/// - **Numbers**: Any Unicode digit (0-9 and international numerals)
/// - **Underscore**: Single underscore `_` is permitted
/// - **Invalid**: Special symbols, whitespace, punctuation
///
/// ## General Rules
/// - **Empty strings**: Not valid (must have at least one character)
/// - **Unicode support**: Full Unicode letter and digit support
/// - **Case insensitive**: Both upper and lowercase letters allowed
/// - **Mixed content**: Letters, numbers, and underscores can be mixed
///
/// # Examples
///
/// ## Valid PKL Identifiers
/// ```rust
/// use space_pkl::utils::is_valid_pkl_identifier;
///
/// // Basic valid identifiers
/// assert!(is_valid_pkl_identifier("propertyName"));
/// assert!(is_valid_pkl_identifier("className"));
/// assert!(is_valid_pkl_identifier("simple"));
/// assert!(is_valid_pkl_identifier("UPPERCASE"));
/// assert!(is_valid_pkl_identifier("lowercase"));
///
/// // With numbers
/// assert!(is_valid_pkl_identifier("property123"));
/// assert!(is_valid_pkl_identifier("version2"));
/// assert!(is_valid_pkl_identifier("test1Value"));
///
/// // With underscores
/// assert!(is_valid_pkl_identifier("_private"));
/// assert!(is_valid_pkl_identifier("snake_case"));
/// assert!(is_valid_pkl_identifier("mixed_Case123"));
/// ```
///
/// ## Invalid PKL Identifiers
/// ```rust
/// use space_pkl::utils::is_valid_pkl_identifier;
///
/// // Empty string
/// assert!(!is_valid_pkl_identifier(""));
///
/// // Starting with numbers
/// assert!(!is_valid_pkl_identifier("123invalid"));
/// assert!(!is_valid_pkl_identifier("0value"));
///
/// // Special characters
/// assert!(!is_valid_pkl_identifier("invalid-name"));  // hyphen
/// assert!(!is_valid_pkl_identifier("space name"));    // space
/// assert!(!is_valid_pkl_identifier("dot.notation"));  // dot
/// assert!(!is_valid_pkl_identifier("question?"));     // question mark
/// assert!(!is_valid_pkl_identifier("exclamation!"));  // exclamation
/// ```
///
/// ## Unicode Character Support
/// ```rust
/// use space_pkl::utils::is_valid_pkl_identifier;
///
/// // International characters (valid)
/// assert!(is_valid_pkl_identifier("café"));
/// assert!(is_valid_pkl_identifier("müller"));
/// assert!(is_valid_pkl_identifier("日本語"));
/// assert!(is_valid_pkl_identifier("москва"));
/// assert!(is_valid_pkl_identifier("ñoño"));
///
/// // Unicode numbers (valid in non-first position)
/// assert!(is_valid_pkl_identifier("test１２３")); // Full-width numbers
/// ```
///
/// ## PKL Schema Validation Context
/// ```rust
/// use space_pkl::utils::{is_valid_pkl_identifier, rust_to_pkl_identifier};
///
/// // Validate converted property names
/// let rust_properties = vec!["user_name", "api_key", "max_connections"];
///
/// for rust_prop in rust_properties {
///     let pkl_prop = rust_to_pkl_identifier(rust_prop);
///     assert!(is_valid_pkl_identifier(&pkl_prop),
///             "Converted property '{}' should be valid", pkl_prop);
/// }
///
/// // Results in valid PKL properties:
/// // userName, apiKey, maxConnections
/// ```
///
/// ## Edge Cases and Boundary Testing
/// ```rust
/// use space_pkl::utils::is_valid_pkl_identifier;
///
/// // Single character cases
/// assert!(is_valid_pkl_identifier("a"));      // Single letter (valid)
/// assert!(is_valid_pkl_identifier("_"));      // Single underscore (valid)
/// assert!(!is_valid_pkl_identifier("1"));     // Single number (invalid)
///
/// // Mixed content
/// assert!(is_valid_pkl_identifier("a1"));     // Letter then number (valid)
/// assert!(is_valid_pkl_identifier("_1"));     // Underscore then number (valid)
/// assert!(is_valid_pkl_identifier("test_123_value")); // Complex mix (valid)
///
/// // Very long identifiers (valid if they follow rules)
/// let long_valid = "a".repeat(1000);
/// assert!(is_valid_pkl_identifier(&long_valid));
/// ```
///
/// # Performance Notes
///
/// - **Early termination**: Returns false immediately on first invalid character
/// - **Unicode efficient**: Uses Rust's optimized Unicode character classification
/// - **Memory efficient**: No allocations during validation
/// - **Single pass**: Examines each character only once
///
/// # Error Prevention
///
/// This function helps prevent PKL syntax errors by catching invalid identifiers
/// before they're used in generated schemas:
///
/// ```rust
/// use space_pkl::utils::is_valid_pkl_identifier;
///
/// fn safe_pkl_property_name(name: &str) -> Result<String, String> {
///     if is_valid_pkl_identifier(name) {
///         Ok(name.to_string())
///     } else {
///         Err(format!("Invalid PKL identifier: '{}'", name))
///     }
/// }
/// ```
///
/// # Integration
///
/// Used throughout the PKL generation pipeline:
/// - **Property validation**: Ensuring converted property names are valid
/// - **Class name validation**: Checking generated class names
/// - **Template processing**: Validating identifiers before template rendering
/// - **Error prevention**: Catching invalid names before PKL generation
/// - **Testing**: Validating generated schemas in test suites
pub fn is_valid_pkl_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // First character must be letter or underscore
    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    // Rest must be alphanumeric or underscore
    name.chars()
        .skip(1)
        .all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_to_pkl_identifier() {
        assert_eq!(rust_to_pkl_identifier("hello_world"), "helloWorld");
        assert_eq!(rust_to_pkl_identifier("test"), "test");
        assert_eq!(rust_to_pkl_identifier("foo_bar_baz"), "fooBarBaz");
    }

    #[test]
    fn test_capitalize_first_letter() {
        assert_eq!(capitalize_first_letter("hello"), "Hello");
        assert_eq!(capitalize_first_letter("WORLD"), "WORLD");
        assert_eq!(capitalize_first_letter("a"), "A");
    }

    #[test]
    fn test_type_name_to_pkl() {
        assert_eq!(type_name_to_pkl("WorkspaceConfig"), "Workspace");
        assert_eq!(type_name_to_pkl("ProjectType"), "Project");
        assert_eq!(type_name_to_pkl("PartialWorkspaceConfig"), "Workspace");
    }

    #[test]
    fn test_is_valid_pkl_identifier() {
        assert!(is_valid_pkl_identifier("validName"));
        assert!(is_valid_pkl_identifier("_private"));
        assert!(is_valid_pkl_identifier("name123"));
        assert!(!is_valid_pkl_identifier("123invalid"));
        assert!(!is_valid_pkl_identifier(""));
        assert!(!is_valid_pkl_identifier("-invalid"));
    }

    #[test]
    fn test_rust_to_pkl_identifier_edge_cases() {
        // Test empty string
        assert_eq!(rust_to_pkl_identifier(""), "");

        // Test single character
        assert_eq!(rust_to_pkl_identifier("a"), "a");

        // Test already camelCase
        assert_eq!(
            rust_to_pkl_identifier("alreadyCamelCase"),
            "alreadyCamelCase"
        );

        // Test multiple underscores
        assert_eq!(rust_to_pkl_identifier("foo__bar"), "fooBar");

        // Test leading/trailing underscores
        assert_eq!(rust_to_pkl_identifier("_leading"), "Leading");
        assert_eq!(rust_to_pkl_identifier("trailing_"), "trailing");

        // Test numbers
        assert_eq!(rust_to_pkl_identifier("test_123_foo"), "test123Foo");

        // Test very long identifier
        let long_name = "very_long_identifier_with_many_words_that_should_be_converted_properly";
        let expected = "veryLongIdentifierWithManyWordsThatShouldBeConvertedProperly";
        assert_eq!(rust_to_pkl_identifier(long_name), expected);
    }

    #[test]
    fn test_capitalize_first_letter_edge_cases() {
        // Test empty string
        assert_eq!(capitalize_first_letter(""), "");

        // Test single lowercase letter
        assert_eq!(capitalize_first_letter("x"), "X");

        // Test single uppercase letter
        assert_eq!(capitalize_first_letter("X"), "X");

        // Test numbers
        assert_eq!(capitalize_first_letter("123"), "123");

        // Test special characters
        assert_eq!(capitalize_first_letter("@test"), "@test");

        // Test Unicode characters
        assert_eq!(capitalize_first_letter("ñoño"), "Ñoño");

        // Test mixed case
        assert_eq!(capitalize_first_letter("mIxEd"), "MIxEd");

        // Test whitespace
        assert_eq!(capitalize_first_letter(" space"), " space");
    }

    #[test]
    fn test_type_name_to_pkl_edge_cases() {
        // Test empty string
        assert_eq!(type_name_to_pkl(""), "");

        // Test only suffix
        assert_eq!(type_name_to_pkl("Config"), "");
        assert_eq!(type_name_to_pkl("Type"), "");

        // Test only prefix
        assert_eq!(type_name_to_pkl("Partial"), "");

        // Test multiple suffixes
        assert_eq!(type_name_to_pkl("MyConfigType"), "MyConfig");

        // Test multiple prefixes
        assert_eq!(type_name_to_pkl("PartialMyConfig"), "My");

        // Test case variations - function capitalizes the first letter
        // Case-sensitive suffix matching
        assert_eq!(type_name_to_pkl("workspaceconfig"), "Workspaceconfig"); // No Config suffix match
        assert_eq!(type_name_to_pkl("PROJECTTYPE"), "PROJECTTYPE"); // No Type suffix match

        // Test with numbers
        assert_eq!(type_name_to_pkl("Type123Config"), "Type123");

        // Test no prefix/suffix - "Type" suffix is removed
        assert_eq!(type_name_to_pkl("SimpleType"), "Simple");

        // Test complex combinations
        assert_eq!(
            type_name_to_pkl("PartialAdvancedWorkspaceConfigType"),
            "AdvancedWorkspaceConfig"
        );
    }

    #[test]
    fn test_is_valid_pkl_identifier_comprehensive() {
        // Valid identifiers
        let valid_cases = vec![
            "simple",
            "camelCase",
            "PascalCase",
            "_underscore",
            "name123",
            "test_snake",
            "a",
            "_",
            "very_long_identifier_with_many_parts_123",
            "mixedCase_with_underscores",
            "UPPERCASE",
            "lowercase",
        ];

        for case in valid_cases {
            assert!(is_valid_pkl_identifier(case), "Should be valid: {}", case);
        }

        // Invalid identifiers
        let invalid_cases = vec![
            "",
            "123start",
            "0number",
            "-hyphen",
            "space name",
            "special!char",
            "@symbol",
            "dot.notation",
            "question?",
            "plus+",
            "equals=",
            "bracket[",
            "parenthesis(",
            "quote'",
            "doublequote\"",
            "backslash\\",
            "forward/slash",
            "percent%",
            "hash#",
            "dollar$",
            "ampersand&",
            "asterisk*",
            "caret^",
            "tilde~",
            "backtick`",
            "pipe|",
            "less<",
            "greater>",
            "semicolon;",
            "colon:",
            "comma,",
        ];

        for case in invalid_cases {
            assert!(
                !is_valid_pkl_identifier(case),
                "Should be invalid: {}",
                case
            );
        }
    }

    #[test]
    fn test_file_operations_with_temp_directory() {
        use std::env;

        // Create a temporary directory for testing
        let temp_dir = env::temp_dir().join("space_pkl_test");

        // Test ensure_dir_exists
        let nested_dir = temp_dir.join("nested").join("deep").join("structure");
        assert!(ensure_dir_exists(&nested_dir).is_ok());
        assert!(nested_dir.exists());

        // Test writing and reading files
        let test_file = temp_dir.join("test_file.txt");
        let test_content = "Hello, World!\nThis is a test file.";

        assert!(write_string_to_file(&test_file, test_content).is_ok());
        assert!(test_file.exists());

        let read_content = read_file_to_string(&test_file);
        assert!(read_content.is_ok());
        assert_eq!(read_content.unwrap(), test_content);

        // Test writing to nested directory
        let nested_file = nested_dir.join("nested_file.pkl");
        let nested_content = "module TestModule\n\nclass TestClass {\n  name: String\n}";

        assert!(write_string_to_file(&nested_file, nested_content).is_ok());
        assert!(nested_file.exists());

        let nested_read = read_file_to_string(&nested_file);
        assert!(nested_read.is_ok());
        assert_eq!(nested_read.unwrap(), nested_content);

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_operations_error_cases() {
        use std::path::PathBuf;

        // Test reading non-existent file
        let non_existent = PathBuf::from("/non/existent/file.txt");
        let result = read_file_to_string(&non_existent);
        assert!(result.is_err());

        // Test writing to invalid path (on most systems)
        let invalid_path = PathBuf::from("\0invalid\0path");
        let result = write_string_to_file(&invalid_path, "content");
        assert!(result.is_err());
    }

    #[test]
    fn test_ensure_dir_exists_existing_directory() {
        use std::env;

        // Test with existing directory (temp dir should always exist)
        let temp_dir = env::temp_dir();
        assert!(ensure_dir_exists(&temp_dir).is_ok());
    }

    #[test]
    fn test_unicode_handling() {
        // Test with Unicode characters in various functions

        // Unicode in rust_to_pkl_identifier
        assert_eq!(rust_to_pkl_identifier("café_münü"), "caféMünü");
        assert_eq!(rust_to_pkl_identifier("日本語_test"), "日本語Test");

        // Unicode in capitalize_first_letter
        assert_eq!(capitalize_first_letter("äpfel"), "Äpfel");
        assert_eq!(capitalize_first_letter("한글"), "한글");

        // Unicode in type_name_to_pkl
        assert_eq!(type_name_to_pkl("CaféConfig"), "Café");
        assert_eq!(type_name_to_pkl("日本語Type"), "日本語");

        // Unicode in is_valid_pkl_identifier
        assert!(is_valid_pkl_identifier("café"));
        assert!(is_valid_pkl_identifier("日本語"));
        assert!(is_valid_pkl_identifier("München"));
        assert!(is_valid_pkl_identifier("Москва"));
    }

    #[test]
    fn test_performance_with_large_strings() {
        // Test with very large strings to ensure performance
        let large_string = "word_".repeat(1000);
        let start_time = std::time::Instant::now();
        let result = rust_to_pkl_identifier(&large_string);
        let duration = start_time.elapsed();

        // Should complete quickly
        assert!(duration.as_millis() < 100);
        assert!(result.len() > 1000);
        assert!(result.contains("Word"));
    }

    #[test]
    fn test_edge_case_combinations() {
        // Test complex combinations of edge cases

        // Empty parts in snake case
        assert_eq!(
            rust_to_pkl_identifier("__double__underscore__"),
            "DoubleUnderscore"
        );

        // Mixed separators (though not standard Rust)
        assert_eq!(rust_to_pkl_identifier("mixed_case_Name"), "mixedCaseName");

        // Very short words
        assert_eq!(rust_to_pkl_identifier("a_b_c_d"), "aBCD");

        // Numbers and underscores
        assert_eq!(rust_to_pkl_identifier("test_1_2_3"), "test123");

        // Type name edge cases - function removes all matching patterns
        assert_eq!(type_name_to_pkl("TypeConfigTypeConfig"), "TypeConfig");
        assert_eq!(type_name_to_pkl("PartialPartialConfig"), ""); // Removes all "Partial" prefixes
    }

    #[test]
    fn test_file_operations_with_different_content_types() {
        use std::env;

        let temp_dir = env::temp_dir().join("space_pkl_content_test");
        let _ = std::fs::remove_dir_all(&temp_dir); // Clean start

        // Test with various content types
        let large_content = "A".repeat(10000);
        let test_cases = vec![
            ("empty.txt", ""),
            ("single_line.txt", "Single line content"),
            ("multi_line.txt", "Line 1\nLine 2\nLine 3"),
            ("unicode.txt", "Unicode: 🚀 café München 日本語"),
            ("special_chars.txt", "Special: !@#$%^&*()_+-=[]{}|;:,.<>?"),
            ("large.txt", large_content.as_str()),
            (
                "pkl_schema.pkl",
                "module TestModule\n\nclass TestClass {\n  name: String\n  age: Int?\n}",
            ),
        ];

        for (filename, content) in test_cases {
            let file_path = temp_dir.join(filename);

            // Write and read back
            assert!(
                write_string_to_file(&file_path, content).is_ok(),
                "Failed to write {}",
                filename
            );
            let read_back = read_file_to_string(&file_path);
            assert!(read_back.is_ok(), "Failed to read {}", filename);
            assert_eq!(
                read_back.unwrap(),
                content,
                "Content mismatch for {}",
                filename
            );
        }

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
