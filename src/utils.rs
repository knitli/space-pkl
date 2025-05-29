//! Utility functions for the schema generator.

use crate::Result;
use miette::{IntoDiagnostic, WrapErr};
use std::fs;
use std::path::Path;

/// Ensure a directory exists, creating it if necessary
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

/// Read a file to string with better error messages
pub fn read_file_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read file: {}", path.display()))
}

/// Write string to file with better error messages
pub fn write_string_to_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir_exists(parent)?;
    }
    
    fs::write(path, content)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to write file: {}", path.display()))
}

/// Convert a Rust identifier to Pkl-style identifier
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

/// Capitalize the first letter of a string
pub fn capitalize_first_letter(s: &str) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_uppercase().collect() } else { c.to_string() })
        .collect()
}

/// Convert a type name to Pkl format
pub fn type_name_to_pkl(name: &str) -> String {
    // Remove common Rust type prefixes/suffixes
    let cleaned = name
        .trim_end_matches("Config")
        .trim_end_matches("Type")
        .trim_start_matches("Partial");
    
    capitalize_first_letter(cleaned)
}

/// Check if a string is a valid Pkl identifier
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
    name.chars().skip(1).all(|c| c.is_alphanumeric() || c == '_')
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
}
