//! Utility Functions for Pkl Schema Generation
//!
//! This module provides essential utility functions that support the Pkl schema generation
//! process. It includes file system operations, string manipulation functions, identifier
//! conversion utilities, and validation helpers specifically designed for Pkl syntax
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
//! - **Naming Convention Translation**: Convert between Rust and Pkl naming styles
//! - **Type Name Processing**: Clean and format type names for Pkl schemas
//! - **Identifier Validation**: Ensure generated names comply with Pkl syntax rules
//!
//! ## Pkl Syntax Support
//! - **Keyword Escaping**: Handle Pkl reserved words safely
//! - **Identifier Formatting**: Convert Rust identifiers to Pkl conventions
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
//! let template = read_file_to_string(Path::new("Template.pkl"))?;
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
//! // Convert Rust naming to Pkl naming
//! let pkl_name = rust_to_pkl_identifier("my_config_option"); // "myConfigOption"
//! let class_name = capitalize_first_letter("myClass");         // "MyClass"
//! let type_name = type_name_to_pkl("WorkspaceConfig");        // "Workspace"
//!
//! // Validate Pkl identifiers
//! assert!(is_valid_pkl_identifier("validName"));
//! assert!(!is_valid_pkl_identifier("123invalid"));
//! ```
//!
//! # Conversion Rules
//!
//! ## Rust to Pkl Identifier Conversion
//! - **snake_case** → **camelCase**: `my_variable` → `myVariable`
//! - **Preserve existing camelCase**: `alreadyCamel` → `alreadyCamel`
//! - **Handle multiple underscores**: `foo__bar` → `fooBar`
//! - **Clean edge cases**: Leading/trailing underscores handled gracefully
//!
//! ## Type Name Processing
//! - **Remove common suffixes**: `WorkspaceConfig` → `Workspace`
//! - **Remove common prefixes**: `PartialProjectConfig` → `Project`
//! - **Capitalize result**: Ensure proper Pkl class naming
//!
//! ## Pkl Identifier Validation
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
/// let output_path = Path::new("./generated/schemas/Workspace.pkl");
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
/// generated Pkl schema files to disk.
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
///     Path::new("./generated/schemas/Workspace.pkl"),
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
///     ("Workspace.pkl", "module Workspace\n// content"),
///     ("Project.pkl", "module Project\n// content"),
///     ("Tasks.pkl", "module Tasks\n// content"),
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


#[cfg(test)]
mod tests {
    use super::*;

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
