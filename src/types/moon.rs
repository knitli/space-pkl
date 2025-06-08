use crate::types::{CliError, InternalError, SchemaFormat, TypeMap};
use moon_config::{ProjectConfig, TaskConfig, TemplateConfig, ToolchainConfig, WorkspaceConfig};
use schematic_types::SchemaType;
use serde_json::Value;
use std::collections::HashSet;
use std::str::FromStr;

/// Represents supported Moon config formats.
///
/// We use this enum to warn users that other supported types are not
/// currently implemented. The use case here is to provide a means to translate moon configurations for use in CI/CD processes that may not support these formats, or to generate or use them programmatically (i.e. with Typescript).

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoonConfigFormat {
    Pkl,
    Yaml,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum MoonType {
    ProjectConfig(ProjectConfig),
    WorkspaceConfig(WorkspaceConfig),
    TemplateConfig(TemplateConfig),
    ToolchainConfig(ToolchainConfig),
    TaskConfig(TaskConfig),
}
//todo  TODO add a function to infer a type from a loaded config

/// Unknown configuration that preserves structure and format information
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct UnknownConfig {
    /// Optional name of the configuration, if available
    pub name: Option<String>,
    /// The actual configuration content
    #[serde(flatten)]
    pub content: Value,

    /// Optional metadata about the original format (stored separately, not serialized)
    #[serde(skip)]
    pub original_format: Option<SchemaFormat>,

    /// Optional type hint if we can guess what kind of config this might be
    #[serde(skip)]
    pub type_hint: Option<String>,
}

impl Default for UnknownConfig {
    fn default() -> Self {
        Self {
            content: Value::Object(serde_json::Map::new()),
            original_format: None,
            type_hint: None,
            name: None,
        }
    }
}

impl UnknownConfig {
    /// Create a new unknown config from a Value
    pub fn new(content: Value) -> Self {
        Self {
            content,
            original_format: None,
            type_hint: None,
            name: None,
        }
    }

    /// Create with format information
    pub fn with_format(content: Value, format: SchemaFormat) -> Self {
        Self {
            content,
            original_format: Some(format),
            type_hint: None,
            name: None,
        }
    }
}

/// Strongly-typed configuration wrapper
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum LoadedConfig {
    Project(ProjectConfig),
    Workspace(WorkspaceConfig),
    Template(TemplateConfig),
    Toolchain(ToolchainConfig),
    Task(TaskConfig),
    Unknown(UnknownConfig),
}

/// Enum to hold any of the config types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigValue {
    Project(ProjectConfig),
    Workspace(WorkspaceConfig),
    Template(TemplateConfig),
    Toolchain(ToolchainConfig),
    Task(TaskConfig),
}

impl LoadedConfig {
    /// Get the config type name for error reporting
    pub fn config_type_name(&self, schemas: Option<TypeMap>) -> String {
        match self {
            LoadedConfig::Project(_) => "project".to_string(),
            LoadedConfig::Workspace(_) => "workspace".to_string(),
            LoadedConfig::Template(_) => "template".to_string(),
            LoadedConfig::Toolchain(_) => "toolchain".to_string(),
            LoadedConfig::Task(_) => "task".to_string(),
            LoadedConfig::Unknown(config) if config.name.is_some() => {
                config.name.clone().unwrap_or_else(|| "unknown".to_string())
            }
            _ => self.attempt_to_resolve_name(schemas),
        }
    }

    pub fn attempt_to_resolve_name(&self, schemas: Option<TypeMap>) -> String {
        match self {
            LoadedConfig::Unknown(config) => {
                // First check if config has a name
                if let Some(name) = &config.name {
                    return name.clone();
                }

                // Then try to resolve from schemas
                let Some(schemas) = schemas else {
                    return "unknown".to_string();
                };

                self.find_root_schema_name(&schemas)
            }
            _ => self.struct_name().to_string(),
        }
    }

    /// Helper method to find the root schema name from a collection of schemas
    fn find_root_schema_name(&self, schemas: &TypeMap) -> String {
        let keys: Vec<&str> = schemas.keys().map(|k| k.as_str()).collect();

        // Single schema case
        if keys.len() == 1 {
            return keys[0].to_string();
        }

        // Find referenced schema names
        let referenced_names: HashSet<&str> = schemas
            .values()
            .filter(|schema| schema.deprecated.is_none() && schema.ty.is_reference())
            .filter_map(|schema| {
                if let SchemaType::Reference(name) = &schema.ty {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect();

        // Try to find a non-deprecated struct that isn't referenced by others
        if let Some(root_name) = keys.iter().find(|&name| {
            !referenced_names.contains(name)
                && schemas[*name].deprecated.is_none()
                && matches!(schemas[*name].ty, SchemaType::Struct(_))
        }) {
            return root_name.to_string();
        }

        // Fallback: find any non-deprecated, non-hidden struct
        if let Some(struct_name) = keys.iter().find(|&name| {
            if let SchemaType::Struct(struct_type) = &schemas[*name].ty {
                schemas[*name].deprecated.is_none() && !struct_type.is_hidden()
            } else {
                false
            }
        }) {
            return struct_name.to_string();
        }

        // Last resort: return first available name or "unknown"
        keys.first().unwrap_or(&"unknown").to_string()
    }

    /// Convert the loaded config to a MoonConfig type
    pub fn to_moon_config(&self) -> Result<MoonConfig, InternalError> {
        match self {
            LoadedConfig::Project(_) => Ok(MoonConfig::Project),
            LoadedConfig::Workspace(_) => Ok(MoonConfig::Workspace),
            LoadedConfig::Toolchain(_) => Ok(MoonConfig::Toolchain),
            LoadedConfig::Template(_) => Ok(MoonConfig::Template),
            LoadedConfig::Task(_) => Ok(MoonConfig::Task),
            LoadedConfig::Unknown(_) => Err(InternalError::ValueError {
                message: "Cannot convert UnknownConfig to MoonConfig".to_string(),
                context: "LoadedConfig::to_moon_config".to_string(),
            }),
        }
    }

    /// Get the struct name for this config type
    pub fn struct_name(&self) -> &'static str {
        match self {
            LoadedConfig::Project(_) => "ProjectConfig",
            LoadedConfig::Workspace(_) => "WorkspaceConfig",
            LoadedConfig::Template(_) => "TemplateConfig",
            LoadedConfig::Toolchain(_) => "ToolchainConfig",
            LoadedConfig::Task(_) => "TaskConfig",
            LoadedConfig::Unknown(_) => "UnknownConfig",
        }
    }

    /// Convert the loaded config to a MoonType
    pub fn moon_type(&self) -> Result<MoonType, InternalError> {
        match self {
            LoadedConfig::Project(config) => Ok(MoonType::ProjectConfig(config.clone())),
            LoadedConfig::Workspace(config) => Ok(MoonType::WorkspaceConfig(config.clone())),
            LoadedConfig::Template(config) => Ok(MoonType::TemplateConfig(config.clone())),
            LoadedConfig::Toolchain(config) => Ok(MoonType::ToolchainConfig(config.clone())),
            LoadedConfig::Task(config) => Ok(MoonType::TaskConfig(config.clone())),
            LoadedConfig::Unknown(_config) => Err(InternalError::ValueError {
                message: "Cannot convert UnknownConfig to MoonType".to_string(),
                context: "LoadedConfig::moon_type".to_string(),
            }),
        }
    }

    /// Get the underlying config value
    pub fn get_config(&self) -> Result<ConfigValue, InternalError> {
        match self {
            LoadedConfig::Project(config) => Ok(ConfigValue::Project(config.clone())),
            LoadedConfig::Workspace(config) => Ok(ConfigValue::Workspace(config.clone())),
            LoadedConfig::Template(config) => Ok(ConfigValue::Template(config.clone())),
            LoadedConfig::Toolchain(config) => Ok(ConfigValue::Toolchain(config.clone())),
            LoadedConfig::Task(config) => Ok(ConfigValue::Task(config.clone())),
            LoadedConfig::Unknown(_) => Err(InternalError::ValueError {
                message: "Cannot extract config value from UnknownConfig".to_string(),
                context: "LoadedConfig::get_config".to_string(),
            }),
        }
    }
}

impl MoonConfigFormat {
    /// Get supported moon config formats for variants
    fn supported_extensions(&self) -> Vec<&'static str> {
        match self {
            // `pcf` is a static subset of Pkl.
            MoonConfigFormat::Pkl => vec!["pkl", "pcf"],
            MoonConfigFormat::Yaml => vec!["yaml", "yml"],
        }
    }

    fn is_supported_extension(&self, ext: &str) -> bool {
        self.supported_extensions().contains(&ext)
    }

    fn all_supported_extensions() -> Vec<&'static str> {
        vec!["pkl", "pcf", "yaml", "yml"]
    }
}

impl std::fmt::Display for MoonConfigFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoonConfigFormat::Pkl => write!(f, "pkl"),
            MoonConfigFormat::Yaml => write!(f, "yaml"),
        }
    }
}

impl FromStr for MoonConfigFormat {
    type Err = CliError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pkl" | "pcf" => Ok(MoonConfigFormat::Pkl),
            "yaml" | "yml" => Ok(MoonConfigFormat::Yaml),
            _ => Err(CliError::UnsupportedFormat {
                format: s.to_string(),
                available: MoonConfigFormat::all_supported_extensions(),
            }),
        }
    }
}

/// Moon configuration type enum for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoonConfig {
    Project,
    Workspace,
    Toolchain,
    Template,
    Task,
    All, // Generate for all configuration types
}

impl std::fmt::Display for MoonConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoonConfig::Project => write!(f, "project"),
            MoonConfig::Workspace => write!(f, "workspace"),
            MoonConfig::Toolchain => write!(f, "toolchain"),
            MoonConfig::Template => write!(f, "template"),
            MoonConfig::Task => write!(f, "task"),
            MoonConfig::All => write!(f, "all"),
        }
    }
}

impl FromStr for MoonConfig {
    type Err = CliError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "project" => Ok(MoonConfig::Project),
            "workspace" => Ok(MoonConfig::Workspace),
            "toolchain" => Ok(MoonConfig::Toolchain),
            "template" => Ok(MoonConfig::Template),
            "task" => Ok(MoonConfig::Task),
            "all" => Ok(MoonConfig::All),
            _ => Err(CliError::UnsupportedFormat {
                format: s.to_string(),
                available: MoonConfig::all_types()
                    .iter()
                    .map(|cfg| cfg.to_string().leak() as &'static str)
                    .collect::<Vec<&'static str>>(),
            }),
        }
    }
}

impl MoonConfig {
    /// Get all individual configuration types (excluding 'All')
    pub fn all_types() -> Vec<MoonConfig> {
        vec![
            MoonConfig::Project,
            MoonConfig::Workspace,
            MoonConfig::Toolchain,
            MoonConfig::Template,
            MoonConfig::Task,
        ]
    }

    pub fn basename(&self) -> Result<&'static str, InternalError> {
        match self {
            MoonConfig::Project => Ok("moon"),
            MoonConfig::Workspace => Ok("workspace"),
            MoonConfig::Toolchain => Ok("toolchain"),
            MoonConfig::Template => Ok("template"),
            MoonConfig::Task => Ok("tasks"),
            _ => Err(InternalError::ValueError {
              message: (r#"To get basenames for `all` configurations, iterate `MoonConfig.basename()` using `MoonConfig.all_types()`:

            ```rust
            for config in MoonConfig::all_types() {
                println!(\"{}\", config.basename());
            }
            ```
            "#).to_string(),
            context: "MoonConfig::basename".to_string(),
            }),
      }
    }
}
