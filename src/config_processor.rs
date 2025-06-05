//! Core Logic Module for Space Pklr
//!
//! This module encapsulates the primary business logic for configuration loading, conversion,
//! rendering, and schema/skeleton generation.

use std::path::Path;
use serde_json;
use serde_yaml;
use std::str::FromStr;
use schematic::ConfigLoader;
use moon_config::{ProjectConfig, WorkspaceConfig, TemplateConfig, ToolchainConfig, TaskConfig};

use crate::error::CliError;

/// Simple format enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFormat {
    Yaml,
    Json,
    Pkl,
    // Toml = 3
}

impl std::fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigFormat::Yaml => write!(f, "yaml"),
            ConfigFormat::Json => write!(f, "json"),
            ConfigFormat::Pkl => write!(f, "pkl"),
        }
    }
}

impl FromStr for ConfigFormat {
    type Err = CliError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "yaml" | "yml" => Ok(ConfigFormat::Yaml),
            "json" => Ok(ConfigFormat::Json),
            "pkl" => Ok(ConfigFormat::Pkl),
            _ => Err(CliError::UnsupportedFormat {
                format: s.to_string(),
                available: vec!["yaml", "json", "pkl"],
            }),
        }
    }
}

/// Strongly-typed configuration wrapper
#[derive(Debug, Clone)]
pub enum LoadedConfig {
    Project(ProjectConfig),
    Workspace(WorkspaceConfig),
    Template(TemplateConfig),
    Toolchain(ToolchainConfig),
    Task(TaskConfig),
}

impl LoadedConfig {
    /// Get the config type name for error reporting
    pub fn config_type_name(&self) -> &'static str {
        match self {
            LoadedConfig::Project(_) => "project",
            LoadedConfig::Workspace(_) => "workspace",
            LoadedConfig::Template(_) => "template",
            LoadedConfig::Toolchain(_) => "toolchain",
            LoadedConfig::Task(_) => "task",
        }
    }
}

/// Moon configuration type enum for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoonConfigType {
    Project,
    Workspace,
    Toolchain,
    Template,
    Task,
    All, // Generate for all configuration types
}

impl std::fmt::Display for MoonConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoonConfigType::Project => write!(f, "project"),
            MoonConfigType::Workspace => write!(f, "workspace"),
            MoonConfigType::Toolchain => write!(f, "toolchain"),
            MoonConfigType::Template => write!(f, "template"),
            MoonConfigType::Task => write!(f, "task"),
            MoonConfigType::All => write!(f, "all"),
        }
    }
}

impl FromStr for MoonConfigType {
    type Err = CliError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "project" => Ok(MoonConfigType::Project),
            "workspace" => Ok(MoonConfigType::Workspace),
            "toolchain" => Ok(MoonConfigType::Toolchain),
            "template" => Ok(MoonConfigType::Template),
            "task" => Ok(MoonConfigType::Task),
            "all" => Ok(MoonConfigType::All),
            _ => Err(CliError::UnsupportedFormat {
                format: s.to_string(),
                available: vec!["project", "workspace", "toolchain", "template", "task", "all"],
            }),
        }
    }
}

impl MoonConfigType {
    /// Get all individual configuration types (excluding 'All')
    pub fn all_types() -> Vec<MoonConfigType> {
        vec![
            MoonConfigType::Project,
            MoonConfigType::Workspace,
            MoonConfigType::Toolchain,
            MoonConfigType::Template,
            MoonConfigType::Task,
        ]
    }
}

/// Load and validate a configuration file
pub async fn load_config(
    path: &Path,
    _config_type: MoonConfigType,
    format: Option<ConfigFormat>,
) -> Result<(String, ConfigFormat), CliError> {
    // Read the file content
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| CliError::IoError {
            context: format!("Reading config file: {}", path.display()),
            source: e,
        })?;

    // Determine format
    let detected_format = if let Some(fmt) = format {
        fmt
    } else {
        detect_format_from_path(path)?
    };

    // Validate that the content can be parsed
    validate_content_format(&content, &detected_format)?;

    Ok((content, detected_format))
}

/// Load configuration using schematic's ConfigLoader with proper type safety
pub async fn load_config_with_schematic(
    path: &Path,
    config_type: MoonConfigType,
    _format: Option<ConfigFormat>,
) -> Result<LoadedConfig, CliError> {
    match config_type {
        MoonConfigType::Project => {
            let mut loader = ConfigLoader::<ProjectConfig>::new();
            loader.file(path).map_err(|e| CliError::ValidationError {
                source: Box::new(e)
            })?;

            let result = loader.load().map_err(|e| CliError::ValidationError {
                source: Box::new(e)
            })?;

            Ok(LoadedConfig::Project(result.config))
        }
        MoonConfigType::Workspace => {
            let mut loader = ConfigLoader::<WorkspaceConfig>::new();
            loader.file(path).map_err(|e| CliError::ValidationError {
                source: Box::new(e)
            })?;

            let result = loader.load().map_err(|e| CliError::ValidationError {
                source: Box::new(e)
            })?;

            Ok(LoadedConfig::Workspace(result.config))
        }
        MoonConfigType::Toolchain => {
            let mut loader = ConfigLoader::<ToolchainConfig>::new();
            loader.file(path).map_err(|e| CliError::ValidationError {
                source: Box::new(e)
            })?;

            let result = loader.load().map_err(|e| CliError::ValidationError {
                source: Box::new(e)
            })?;

            Ok(LoadedConfig::Toolchain(result.config))
        }
        MoonConfigType::Template => {
            let mut loader = ConfigLoader::<TemplateConfig>::new();
            loader.file(path).map_err(|e| CliError::ValidationError {
                source: Box::new(e)
            })?;

            let result = loader.load().map_err(|e| CliError::ValidationError {
                source: Box::new(e)
            })?;

            Ok(LoadedConfig::Template(result.config))
        }
        MoonConfigType::Task => {
            let mut loader = ConfigLoader::<TaskConfig>::new();
            loader.file(path).map_err(|e| CliError::ValidationError {
                source: Box::new(e)
            })?;

            let result = loader.load().map_err(|e| CliError::ValidationError {
                source: Box::new(e)
            })?;

            Ok(LoadedConfig::Task(result.config))
        }
        MoonConfigType::All => {
            Err(CliError::Generic("Cannot load config with type 'All' - specify a specific config type".to_string()))
        }
    }
}

/// Render configuration using schematic's built-in renderers
pub fn render_config_with_schematic(
    config: &LoadedConfig,
    format: ConfigFormat,
) -> Result<String, CliError> {
    match format {
        ConfigFormat::Yaml => {
            let result = match config {
                LoadedConfig::Project(c) => serde_yaml::to_string(c),
                LoadedConfig::Workspace(c) => serde_yaml::to_string(c),
                LoadedConfig::Template(c) => serde_yaml::to_string(c),
                LoadedConfig::Toolchain(c) => serde_yaml::to_string(c),
                LoadedConfig::Task(c) => serde_yaml::to_string(c),
            };
            result.map_err(|e| CliError::RenderError {
                config_type: config.config_type_name().to_string(),
                format,
                source: Box::new(e),
            })
        }
        ConfigFormat::Json => {
            let result = match config {
                LoadedConfig::Project(c) => serde_json::to_string_pretty(c),
                LoadedConfig::Workspace(c) => serde_json::to_string_pretty(c),
                LoadedConfig::Template(c) => serde_json::to_string_pretty(c),
                LoadedConfig::Toolchain(c) => serde_json::to_string_pretty(c),
                LoadedConfig::Task(c) => serde_json::to_string_pretty(c),
            };
            result.map_err(|e| CliError::RenderError {
                config_type: config.config_type_name().to_string(),
                format,
                source: Box::new(e),
            })
        }
        ConfigFormat::Pkl => {
            // For Pkl format, we need to generate proper module syntax
            generate_pkl_module(config).map_err(|e| CliError::RenderError {
                config_type: config.config_type_name().to_string(),
                format,
                source: Box::new(e),
            })
        }
    }
}

/// Generate Pkl module syntax for configuration
fn generate_pkl_module(config: &LoadedConfig) -> Result<String, serde_yaml::Error> {
    // First serialize to YAML, then convert to Pkl module syntax
    let yaml_content = match config {
        LoadedConfig::Project(c) => serde_yaml::to_string(c)?,
        LoadedConfig::Workspace(c) => serde_yaml::to_string(c)?,
        LoadedConfig::Template(c) => serde_yaml::to_string(c)?,
        LoadedConfig::Toolchain(c) => serde_yaml::to_string(c)?,
        LoadedConfig::Task(c) => serde_yaml::to_string(c)?,
    };

    // Convert YAML to Pkl module format
    let pkl_content = yaml_to_pkl_module(&yaml_content, config.config_type_name());
    Ok(pkl_content)
}

/// Convert YAML content to Pkl module format
fn yaml_to_pkl_module(yaml_content: &str, config_type: &str) -> String {
    let header = format!(
        "// Generated {} configuration in Pkl format\n// Generated by Space Pklr\n\n",
        config_type
    );

    // Parse YAML and convert to Pkl syntax
    if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(yaml_content) {
        let pkl_body = yaml_to_pkl(&yaml_value);
        format!("{}{}", header, pkl_body)
    } else {
        format!("{}// Error: Could not parse YAML content", header)
    }
}

/// Convert configuration content between formats
pub fn convert_config(
    content: &str,
    from_format: ConfigFormat,
    to_format: ConfigFormat,
) -> Result<String, CliError> {
    match (from_format.clone(), to_format.clone()) {
        // Same format - no conversion needed
        (ConfigFormat::Yaml, ConfigFormat::Yaml) |
        (ConfigFormat::Json, ConfigFormat::Json) |
        (ConfigFormat::Pkl, ConfigFormat::Pkl) => {
            Ok(content.to_string())
        }
        // YAML to JSON
        (ConfigFormat::Yaml, ConfigFormat::Json) => {
            let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(e),
                })?;

            let json_value: serde_json::Value = serde_yaml::from_value(yaml_value)
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(e),
                })?;

            serde_json::to_string_pretty(&json_value)
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(e),
                })
        }
        // JSON to YAML
        (ConfigFormat::Json, ConfigFormat::Yaml) => {
            let json_value: serde_json::Value = serde_json::from_str(content)
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(e),
                })?;

            let yaml_value: serde_yaml::Value = serde_json::from_value(json_value)
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(e),
                })?;

            serde_yaml::to_string(&yaml_value)
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(e),
                })
        }
        // Pkl conversions using schematic
        (ConfigFormat::Pkl, ConfigFormat::Yaml) | (ConfigFormat::Pkl, ConfigFormat::Json) => {
            // For Pkl to other formats, we need to use schematic to parse Pkl and render to target format
            convert_from_pkl(content, to_format)
        }
        (ConfigFormat::Yaml, ConfigFormat::Pkl) | (ConfigFormat::Json, ConfigFormat::Pkl) => {
            // For other formats to Pkl, parse the content and render to Pkl
            convert_to_pkl(content, from_format)
        }
    }
}

/// Detect format from file path extension
pub fn detect_format_from_path(path: &Path) -> Result<ConfigFormat, CliError> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| CliError::UnsupportedFormat {
            format: "unknown".to_string(),
            available: vec!["yaml", "yml", "json", "pkl"],
        })?;

    ConfigFormat::from_str(extension)
}

/// Validate that content can be parsed as the specified format
fn validate_content_format(content: &str, format: &ConfigFormat) -> Result<(), CliError> {
    match format {
        ConfigFormat::Yaml => {
            serde_yaml::from_str::<serde_yaml::Value>(content)
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(e),
                })?;
        }
        ConfigFormat::Json => {
            serde_json::from_str::<serde_json::Value>(content)
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(e),
                })?;
        }
        ConfigFormat::Pkl => {
            // For Pkl validation, we'll rely on schematic's Pkl parsing
            // This is a basic validation - full validation happens during conversion
            if content.trim().is_empty() {
                return Err(CliError::ValidationError {
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Empty Pkl content",
                    )),
                });
            }
        }
    }

    Ok(())
}

/// Apply intelligent format defaults based on input
pub fn apply_format_defaults(
    input_format: Option<ConfigFormat>,
    output_format: Option<ConfigFormat>,
) -> ConfigFormat {
    if let Some(format) = output_format {
        return format;
    }

    // Default logic: if input is YAML, output JSON; if input is JSON, output YAML
    match input_format {
        Some(ConfigFormat::Yaml) => ConfigFormat::Json,
        Some(ConfigFormat::Json) => ConfigFormat::Yaml,
        Some(ConfigFormat::Pkl) => ConfigFormat::Yaml, // For when Pkl is supported
        None => ConfigFormat::Json, // Default to JSON
    }
}

/// Convert from Pkl to other formats using schematic
fn convert_from_pkl(pkl_content: &str, to_format: ConfigFormat) -> Result<String, CliError> {
    // This is a placeholder implementation
    // In a full implementation, we would use schematic to parse the Pkl
    // and then render to the target format
    match to_format {
        ConfigFormat::Yaml => {
            // For now, return a basic conversion message
            // In the full implementation, this would use schematic's Pkl parsing
            Ok(format!("# Converted from Pkl\n# TODO: Implement Pkl->YAML conversion via schematic\n{}", pkl_content))
        }
        ConfigFormat::Json => {
            // For now, return a basic conversion message
            Ok(format!("{{ \"_comment\": \"Converted from Pkl - TODO: Implement via schematic\", \"content\": {} }}",
                serde_json::to_string(pkl_content).unwrap_or_else(|_| "\"invalid\"".to_string())))
        }
        ConfigFormat::Pkl => Ok(pkl_content.to_string()),
    }
}

/// Convert to Pkl from other formats using schematic
fn convert_to_pkl(content: &str, from_format: ConfigFormat) -> Result<String, CliError> {
    match from_format {
        ConfigFormat::Yaml => {
            // Parse YAML and convert to Pkl syntax
            let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(e),
                })?;

            // Convert to Pkl syntax
            let pkl_content = yaml_to_pkl(&yaml_value);
            Ok(format!("// Converted from YAML to Pkl\n// Generated by Space Pklr\n\n{}", pkl_content))
        }
        ConfigFormat::Json => {
            // Parse JSON and convert to Pkl syntax
            let json_value: serde_json::Value = serde_json::from_str(content)
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(e),
                })?;

            // Convert to Pkl syntax
            let pkl_content = json_to_pkl(&json_value);
            Ok(format!("// Converted from JSON to Pkl\n// Generated by Space Pklr\n\n{}", pkl_content))
        }
        ConfigFormat::Pkl => Ok(content.to_string()),
    }
}

/// Convert YAML value to Pkl syntax
fn yaml_to_pkl(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::Null => "null".to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::String(s) => format!("\"{}\"", escape_string(s)),
        serde_yaml::Value::Sequence(seq) => {
            let items: Vec<String> = seq.iter()
                .map(yaml_to_pkl)
                .collect();
            if items.is_empty() {
                "new Listing {}".to_string()
            } else {
                format!("new Listing {{\n{}\n}}",
                    items.iter()
                        .map(|item| format!("  {}", item))
                        .collect::<Vec<_>>()
                        .join("\n"))
            }
        }
        serde_yaml::Value::Mapping(map) => {
            let items: Vec<String> = map.iter()
                .map(|(k, v)| {
                    let key = match k {
                        serde_yaml::Value::String(s) => {
                            if is_valid_pkl_identifier(s) {
                                s.clone()
                            } else {
                                format!("\"{}\"", escape_string(s))
                            }
                        }
                        _ => format!("\"{}\"", k.as_str().unwrap_or("unknown")),
                    };
                    format!("{} = {}", key, yaml_to_pkl(v))
                })
                .collect();

            if items.is_empty() {
                "new Dynamic {}".to_string()
            } else {
                format!("new Dynamic {{\n{}\n}}",
                    items.iter()
                        .map(|item| format!("  {}", item))
                        .collect::<Vec<_>>()
                        .join("\n"))
            }
        }
        _ => "null".to_string(),
    }
}

/// Convert JSON value to Pkl syntax
fn json_to_pkl(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", escape_string(s)),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter()
                .map(json_to_pkl)
                .collect();
            if items.is_empty() {
                "new Listing {}".to_string()
            } else {
                format!("new Listing {{\n{}\n}}",
                    items.iter()
                        .map(|item| format!("  {}", item))
                        .collect::<Vec<_>>()
                        .join("\n"))
            }
        }
        serde_json::Value::Object(obj) => {
            let items: Vec<String> = obj.iter()
                .map(|(k, v)| {
                    let key = if is_valid_pkl_identifier(k) {
                        k.clone()
                    } else {
                        format!("\"{}\"", escape_string(k))
                    };
                    format!("{} = {}", key, json_to_pkl(v))
                })
                .collect();

            if items.is_empty() {
                "new Dynamic {}".to_string()
            } else {
                format!("new Dynamic {{\n{}\n}}",
                    items.iter()
                        .map(|item| format!("  {}", item))
                        .collect::<Vec<_>>()
                        .join("\n"))
            }
        }
    }
}

/// Check if a string is a valid Pkl identifier
fn is_valid_pkl_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first_char = s.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Escape string for Pkl
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}

/// Enhanced format detection that includes Pkl support
pub fn detect_format_from_path_enhanced(path: &std::path::Path) -> Result<ConfigFormat, CliError> {
    detect_format_from_path(path)
}

/// Check if Pkl CLI is available for Pkl operations
pub async fn ensure_pkl_available() -> Result<crate::pkl_tooling::PklCli, CliError> {
    // Try to find existing Pkl installation
    if let Ok(Some(pkl_cli)) = crate::pkl_tooling::find_pkl_executable().await {
        return Ok(pkl_cli);
    }

    // If not found, suggest installation
    Err(CliError::PklInstallFailed {
        reason: "Pkl CLI not found".to_string(),
        help: Some("Install Pkl CLI with: spklr install pkl".to_string()),
    })
}

/// Generate JSON schema for a Moon configuration type using schematic's existing capabilities
pub fn generate_schema(
    config_type: MoonConfigType,
    format: &str,
) -> Result<String, CliError> {
    use schematic::schema::{SchemaGenerator, JsonSchemaRenderer, TypeScriptRenderer};

    let mut generator = SchemaGenerator::default();

    // Add the appropriate config type to the generator using schematic's existing capabilities
    match config_type {
        MoonConfigType::Project => {
            generator.add::<moon_config::ProjectConfig>();
        }
        MoonConfigType::Workspace => {
            generator.add::<moon_config::WorkspaceConfig>();
        }
        MoonConfigType::Toolchain => {
            generator.add::<moon_config::ToolchainConfig>();
        }
        MoonConfigType::Template => {
            generator.add::<moon_config::TemplateConfig>();
        }
        MoonConfigType::Task => {
            generator.add::<moon_config::TaskConfig>();
        }
        MoonConfigType::All => {
            return Err(CliError::Generic("Cannot generate schema for 'All' - use generate_all_schemas functions".to_string()));
        }
    }

    // Generate schema using schematic's existing renderers
    match format {
        "json-schema" => {
            let temp_file = std::env::temp_dir().join("schema.json");
            generator.generate(&temp_file, JsonSchemaRenderer::default())
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                })?;

            std::fs::read_to_string(&temp_file)
                .map_err(|e| CliError::IoError {
                    context: "Reading generated schema".to_string(),
                    source: e,
                })
        }
        "typescript" => {
            let temp_file = std::env::temp_dir().join("types.ts");
            generator.generate(&temp_file, TypeScriptRenderer::default())
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                })?;

            std::fs::read_to_string(&temp_file)
                .map_err(|e| CliError::IoError {
                    context: "Reading generated TypeScript types".to_string(),
                    source: e,
                })
        }
        _ => Err(CliError::UnsupportedFormat {
            format: format.to_string(),
            available: vec!["json-schema", "typescript"],
        })
    }
}

/// Generate schema for all configuration types and formats
pub fn generate_all_schemas(format: &str) -> Result<Vec<(String, String)>, CliError> {
    let mut results = Vec::new();

    for config_type in MoonConfigType::all_types() {
        let schema_content = generate_schema(config_type, format)?;
        let filename = format!("{}_schema.{}", config_type,
            match format {
                "json-schema" => "json",
                "typescript" => "ts",
                _ => format,
            }
        );
        results.push((filename, schema_content));
    }

    Ok(results)
}

/// Generate schemas for all formats for a specific config type
pub fn generate_all_formats_schema(config_type: MoonConfigType) -> Result<Vec<(String, String)>, CliError> {
    let formats = vec!["json-schema", "typescript"];
    let mut results = Vec::new();

    for format in formats {
        let schema_content = generate_schema(config_type, format)?;
        let filename = format!("{}_schema.{}", config_type,
            match format {
                "json-schema" => "json",
                "typescript" => "ts",
                _ => format,
            }
        );
        results.push((filename, schema_content));
    }

    Ok(results)
}

/// Generate all schemas for all types and all formats
pub fn generate_all_schemas_all_formats() -> Result<Vec<(String, String)>, CliError> {
    let formats = vec!["json-schema", "typescript"];
    let mut results = Vec::new();

    for config_type in MoonConfigType::all_types() {
        for format in formats.iter() {
            let schema_content = generate_schema(config_type, format)?;
            let filename = format!("{}_schema.{}", config_type,
                match *format {
                    "json-schema" => "json",
                    "typescript" => "ts",
                    _ => format,
                }
            );
            results.push((filename, schema_content));
        }
    }

    Ok(results)
}

/// Generate schema using schematic's built-in renderers
pub fn generate_schema_with_schematic(
    config_type: MoonConfigType,
    format: &str,
) -> Result<String, CliError> {
    // For now, delegate to the existing working implementation
    // This will be enhanced once we have the proper schematic API integration
    generate_schema(config_type, format)
}

/// Generate default/skeleton configuration using existing moon_config templates and defaults
pub fn generate_skeleton(
    config_type: MoonConfigType,
    format: ConfigFormat,
) -> Result<String, CliError> {
    // Use existing moon_config templates when available, or generate defaults using schematic
    let template_content = match config_type {
        MoonConfigType::Project => {
            // Generate minimal project config using defaults
            let config = moon_config::ProjectConfig::default();
            serialize_config_in_format(&config, &format)?
        }
        MoonConfigType::Workspace => {
            // Generate minimal workspace config using defaults
            let mut config = moon_config::WorkspaceConfig::default();
            // Set some sensible defaults for workspace
            config.projects = moon_config::WorkspaceProjects::Globs(vec!["projects/*".to_string()]);
            serialize_config_in_format(&config, &format)?
        }
        MoonConfigType::Toolchain => {
            // Generate minimal toolchain config using defaults
            let config = moon_config::ToolchainConfig::default();
            serialize_config_in_format(&config, &format)?
        }
        MoonConfigType::Template => {
            // Generate minimal template config using defaults
            let config = moon_config::TemplateConfig::default();
            serialize_config_in_format(&config, &format)?
        }
        MoonConfigType::Task => {
            // Generate minimal task config using defaults
            let config = moon_config::TaskConfig::default();
            serialize_config_in_format(&config, &format)?
        }
        MoonConfigType::All => {
            return Err(CliError::Generic("Cannot generate skeleton for 'All' - use generate_all_skeletons functions".to_string()));
        }
    };

    // Convert to requested format if needed
    match format {
        ConfigFormat::Yaml => {
            // If template is already YAML, return as is, otherwise convert
            if template_content.starts_with('#') || template_content.contains(':') {
                Ok(template_content)
            } else {
                convert_to_format(&template_content, ConfigFormat::Json, ConfigFormat::Yaml)
            }
        }
        ConfigFormat::Json => {
            convert_to_format(&template_content, ConfigFormat::Yaml, ConfigFormat::Json)
        }
        ConfigFormat::Pkl => {
            convert_to_format(&template_content, ConfigFormat::Yaml, ConfigFormat::Pkl)
        }
    }
}

/// Generate skeleton for all configuration types
pub fn generate_all_skeletons(format: ConfigFormat) -> Result<Vec<(String, String)>, CliError> {
    let mut results = Vec::new();

    for config_type in MoonConfigType::all_types() {
        let skeleton_content = generate_skeleton(config_type, format.clone())?;
        let filename = format!("{}.{}", config_type, format);
        results.push((filename, skeleton_content));
    }

    Ok(results)
}

/// Generate skeletons for all formats for a specific config type
pub fn generate_all_formats_skeleton(config_type: MoonConfigType) -> Result<Vec<(String, String)>, CliError> {
    let formats = vec![ConfigFormat::Yaml, ConfigFormat::Json, ConfigFormat::Pkl];
    let mut results = Vec::new();

    for format in formats {
        let skeleton_content = generate_skeleton(config_type, format.clone())?;
        let filename = format!("{}.{}", config_type, format);
        results.push((filename, skeleton_content));
    }

    Ok(results)
}

/// Generate all skeletons for all types and all formats
pub fn generate_all_skeletons_all_formats() -> Result<Vec<(String, String)>, CliError> {
    let formats = vec![ConfigFormat::Yaml, ConfigFormat::Json, ConfigFormat::Pkl];
    let mut results = Vec::new();

    for config_type in MoonConfigType::all_types() {
        for format in formats.iter() {
            let skeleton_content = generate_skeleton(config_type, format.clone())?;
            let filename = format!("{}.{}", config_type, format);
            results.push((filename, skeleton_content));
        }
    }

    Ok(results)
}

/// Generate skeleton configurations using schematic's default mechanisms
pub fn generate_skeleton_with_schematic(
    config_type: MoonConfigType,
    format: ConfigFormat,
) -> Result<String, CliError> {
    // Create default configuration using schematic's default mechanisms
    let loaded_config = match config_type {
        MoonConfigType::Project => {
            let config = ProjectConfig::default();
            LoadedConfig::Project(config)
        }
        MoonConfigType::Workspace => {
            let mut config = WorkspaceConfig::default();
            // Set some sensible defaults for workspace
            config.projects = moon_config::WorkspaceProjects::Globs(vec!["projects/*".to_string()]);
            LoadedConfig::Workspace(config)
        }
        MoonConfigType::Toolchain => {
            let config = ToolchainConfig::default();
            LoadedConfig::Toolchain(config)
        }
        MoonConfigType::Template => {
            let config = TemplateConfig::default();
            LoadedConfig::Template(config)
        }
        MoonConfigType::Task => {
            let config = TaskConfig::default();
            LoadedConfig::Task(config)
        }
        MoonConfigType::All => {
            return Err(CliError::Generic("Cannot generate skeleton for 'all' - use specific functions".to_string()));
        }
    };

    // Use the new schematic-based renderer
    render_config_with_schematic(&loaded_config, format)
}

/// Helper to serialize a config struct in the requested format
fn serialize_config_in_format<T: serde::Serialize>(
    config: &T,
    format: &ConfigFormat,
) -> Result<String, CliError> {
    match format {
        ConfigFormat::Yaml => {
            serde_yaml::to_string(config)
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(e)
                })
        }
        ConfigFormat::Json => {
            serde_json::to_string_pretty(config)
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(e)
                })
        }
        ConfigFormat::Pkl => {
            // Convert to YAML first, then to Pkl
            let yaml = serde_yaml::to_string(config)
                .map_err(|e| CliError::ValidationError {
                    source: Box::new(e)
                })?;
            convert_to_pkl(&yaml, ConfigFormat::Yaml)
        }
    }
}

/// Helper to convert between formats
fn convert_to_format(
    content: &str,
    from_format: ConfigFormat,
    to_format: ConfigFormat,
) -> Result<String, CliError> {
    if from_format == to_format {
        return Ok(content.to_string());
    }

    convert_config(content, from_format, to_format)
}
