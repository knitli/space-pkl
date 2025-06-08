use std::str::FromStr;
use std::fmt::Display;
use indexmap::IndexMap;
use schematic::Schema;

use crate::CliError;

// let's define a descriptive type alias for the schemas for clarity.
/// Map of a *named type* to its `Schema`.
pub type TypeMap = IndexMap<String, Schema>;

/// Defines how enum types are translated to Pkl.
///
/// Either as a union typealias (default) or as a literal union. A typealias is the idiomatic way to represent enums in Pkl, while a literal union is less idiomatic but still valid.
#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum EnumTranslation {
    /// typealias - this is the idiomatic "way of [dill] Pkl"
    /// Example: `typealias LanguageType = "rust"|"python"|"typescript"`
    ///          `language: LanguageType`
    #[default]
    Typealias,
    /// This is another way, less idiomatic. But do what you want.
    /// Example: `language: "rust"|"python"|"typescript"`
    LiteralUnion,
}

impl FromStr for EnumTranslation {
    type Err = CliError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "typealias" | "alias" | "type" | "ta" | "0" | "type_alias" | "type-alias" => Ok(EnumTranslation::Typealias),
            "literalunion" | "literal" | "union" | "lu" | "1" | "literal_union" | "literal-union" => Ok(EnumTranslation::LiteralUnion),
            _ => Err(CliError::UnsupportedFormat {
                format: s.to_string(),
                available: vec!["typealias", "literalunion"],
            }),
        }
    }
}

impl Display for EnumTranslation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnumTranslation::Typealias => write!(f, "typealias"),
            EnumTranslation::LiteralUnion => write!(f, "literal_union"),
        }
    }
}

impl EnumTranslation {
   pub fn use_typealias(&self) -> bool {
        matches!(self, EnumTranslation::Typealias)
    }
}

/// Mark structs translated into classes and/or modules with the `open` keyword.
///
/// Since the primary use case is for typed config templates, we default to `Yes`
/// This allows users to `amend` the template, which essentially makes it a type.
/// It also allows them to `extend` it, for example setting their own defaults.
/// Example:
///
/// ```pkl
///
/// // Open (Yes)
/// open module Project
///
/// open class ToolchainConfig {
///   language: LanguageType
///   // ...
/// }
///
/// // Closed (No)
/// module Project
///
/// class ToolchainConfig {
///   language: LanguageType
/// }
/// ```
#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum OpenStructs {
    /// Mark as open
    #[default]
    Open,
    /// Don't mark as `open`
    No,
}

impl FromStr for OpenStructs {
    type Err = CliError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "yes" | "open" | "true" | "1" | "o" | "y" => Ok(OpenStructs::Open),
            "no" | "false" | "0" | "closed" | "c" | "n" | "no_open" | "no-open" => Ok(OpenStructs::No),
            _ => Err(CliError::UnsupportedFormat {
                format: s.to_string(),
                available: vec!["open", "no"],
            }),
        }
    }
}

impl Display for OpenStructs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenStructs::Open => write!(f, "open"),
            OpenStructs::No => write!(f, "no_open"),
        }
    }
}

impl OpenStructs {
    /// Returns true if the struct is marked as open.
    pub fn is_open(&self) -> bool {
        matches!(self, OpenStructs::Open)
    }
}

/// Defines how the `Config` struct itself is translated to Pkl.
///
/// Either a `Module` (default) or `Class`. Any other struct will still be a class. Pkl's `amend` and `extend` features naturally translate to using the `Config` as a module type, but that deviates from typical schema definitions.
#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConfigTranslation {
    /// The top-level `Config` struct will be rendered as a module with its fields as globals.
    #[default]
    Module,
    /// The `Config` struct will be rendered as a class.
    Class,
}

impl FromStr for ConfigTranslation {
    type Err = CliError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "module" | "mod" | "m" | "0" => Ok(ConfigTranslation::Module),
            "class" | "c" | "cls" | "1" => Ok(ConfigTranslation::Class),
            _ => Err(CliError::UnsupportedFormat {
                format: s.to_string(),
                available: vec!["module", "class"],
            }),
        }
    }
}

impl Display for ConfigTranslation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigTranslation::Module => write!(f, "module"),
            ConfigTranslation::Class => write!(f, "class"),
        }
    }
}

impl ConfigTranslation {
    /// Returns true if the `Config` struct is translated as a module.
    pub fn as_module(&self) -> bool {
        matches!(self, ConfigTranslation::Module)
    }
}

/// Clarifies how a type annotation will be rendered when optional in Pkl
///
/// The choices are `Optional` and `OptionalExplicitNothing`. The default is `Optional`, which is the more idiomatic, but you may want to be explicit.
#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum OptionalFormat {
    /// In Pkl, `?` implies default `null`, though `null` can have a [default value](https://pkl-lang.org/main/current/language-reference/index.html#null-coalescing)
    #[default]
    Optional,
    /// Optional with undefined: `prop: type|nothing = nothing`. You can use 'explicit' as shorthand.
    OptionalExplicitNothing,
}

impl FromStr for OptionalFormat {
    type Err = CliError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "optional" | "opt" | "0" | "o" => Ok(OptionalFormat::Optional),

            "optionalexplicitnothing" | "opt-explicit-nothing" | "optional-explicit-nothing" | "opt_explicit_nothing" | "optional_explicit_nothing" | "explicit" | "e" | "1" => Ok(OptionalFormat::OptionalExplicitNothing),
            _ => Err(CliError::UnsupportedFormat {
                format: s.to_string(),
                available: vec!["optional", "explicit"],
            }),
        }
    }
}
impl Display for OptionalFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionalFormat::Optional => write!(f, "optional"),
            OptionalFormat::OptionalExplicitNothing => write!(f, "optional_explicit_nothing"),
        }
    }
}

impl OptionalFormat {
    /// Returns true if the format is `Optional`.
    pub fn is_optional(&self) -> bool {
        matches!(self, OptionalFormat::Optional)
    }

    /// Returns true if the format is `OptionalExplicitNothing`.
    pub fn is_explicit(&self) -> bool {
        matches!(self, OptionalFormat::OptionalExplicitNothing)
    }
}

/// Whether to default to `required` or `optional` when the schema lacks information on optional properties. Defaults to `required`.
#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PropertyDefault {
    /// When unknown, assume properties are required.
    #[default]
    Required,
    /// When unknown, assume properties are optional.
    Optional,
}

impl FromStr for PropertyDefault {
    type Err = CliError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "required" | "req" | "0" | "r" => Ok(PropertyDefault::Required),
            "optional" | "opt" | "1" | "o" => Ok(PropertyDefault::Optional),
            _ => Err(CliError::UnsupportedFormat {
                format: s.to_string(),
                available: vec!["required", "optional"],
            }),
        }
    }
}
impl Display for PropertyDefault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyDefault::Required => write!(f, "required"),
            PropertyDefault::Optional => write!(f, "optional"),
        }
    }
}

impl PropertyDefault {
    /// Returns true if the default is `Required`.
    pub fn is_required(&self) -> bool {
        matches!(self, PropertyDefault::Required)
    }

    /// Returns true if the default is `Optional`.
    pub fn is_optional(&self) -> bool {
        matches!(self, PropertyDefault::Optional)
    }
}
