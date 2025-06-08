
use std::str::FromStr;
use std::fmt::Display;
use schematic::Format;

use crate::types::CliError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    Template,
    Schema,
}

/// Simple format enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateFormat {
    Pkl,
    Yaml,
    Json,
    JsonC,
    Toml,
    Typescript,
}
impl TemplateFormat {
    pub fn all_supported_extensions() -> Vec<&'static str> {
        vec!["pkl", "yml", "json", "jsonc", "toml", "ts"]
    }

    pub fn is_supported_extension(&self, ext: &str) -> bool {
        Self::all_supported_extensions().contains(&ext)
    }

    pub fn to_schematic(&self) -> Format {
        match self {
            TemplateFormat::Pkl => Format::Pkl,
            TemplateFormat::Yaml => Format::Yaml,
            TemplateFormat::Json => Format::Json,
            TemplateFormat::Toml => Format::Toml,
            _ => Format::None,
        }
    }
}
impl Display for TemplateFormat {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
          TemplateFormat::Pkl => write!(f, "pkl"),
          TemplateFormat::Yaml => write!(f, "yaml"),
          TemplateFormat::Json => write!(f, "json"),
          TemplateFormat::JsonC => write!(f, "jsonc"),
          TemplateFormat::Toml => write!(f, "toml"),
          TemplateFormat::Typescript => write!(f, "typescript"),
      }
  }
}

impl FromStr for TemplateFormat {
  type Err = CliError;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
      match s.to_lowercase().as_str() {
          "pkl" | "pcf" | "p" => Ok(TemplateFormat::Pkl),
          "yaml" | "yml" | "y" => Ok(TemplateFormat::Yaml),
          "json" | "jsonschema" | "json-schema" | "json_schema" | "j" => Ok(TemplateFormat::Json),
          "jsonc" | "json-commented" | "json-with-comments" | "json_commented" | "json_with_comments" | "jsoncomment" | "jsc" | "jc" => Ok(TemplateFormat::JsonC),
          "toml" | "t" => Ok(TemplateFormat::Toml),
          "typescript" | "ts" | "type-script" | "type_script" => Ok(TemplateFormat::Typescript),
          _ => Err(CliError::UnsupportedFormat {
              format: s.to_string(),
              available: vec!["pkl", "yaml", "json", "jsonc", "toml", "typescript"],
          }),
      }
  }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaFormat {
    Pkl,
    Json,
    Typescript,
}

impl SchemaFormat {
    pub fn all_supported_extensions() -> Vec<&'static str> {
        vec!["pkl", "json", "ts"]
    }

    pub fn is_supported_extension(&self, ext: &str) -> bool {
        Self::all_supported_extensions().contains(&ext)
    }

    pub fn to_schematic(&self) -> Format {
        match self {
            SchemaFormat::Pkl => Format::Pkl,
            SchemaFormat::Json => Format::Json,
            SchemaFormat::Typescript => Format::None,
        }
    }
}

impl Display for SchemaFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaFormat::Json => write!(f, "json"),
            SchemaFormat::Pkl => write!(f, "pkl"),
            SchemaFormat::Typescript => write!(f, "typescript"),
        }
    }
}

impl FromStr for SchemaFormat {
    type Err = CliError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" | "jsonschema" | "json-schema" | "json_schema" => Ok(SchemaFormat::Json),
            "pkl" | "pklr" | "pcf" => Ok(SchemaFormat::Pkl),
            "typescript" | "ts" => Ok(SchemaFormat::Typescript),
            _ => Err(CliError::UnsupportedFormat {
                format: s.to_string(),
                available: vec!["json", "pkl", "typescript"],
            }),
        }
    }
}
