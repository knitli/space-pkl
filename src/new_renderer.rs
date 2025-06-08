/**========================================================================
 * *                              About
 *
 *   (c) 2025 Stash AI Inc. (aka Knitli)
 *   Written by Adam Poulemanos [@bashandbone](https://github.com/bashandbone)
 *   Licensed under the [Plain MIT License](https://plainlicense.org/licenses/permissive/mit/) (Tl;dr: do what you want, give credit, assume nothing)
 *   moonrepo, Inc. created and maintains moon and schematic, under the
 *   (traditional) MIT license. I don't know them, they seem nice.
 *
 *========================================================================**/
//! =========================================================================
//!                           # PklSchemaRenderer
//! =========================================================================
//! A full-service [schematic](https://moonrepo.github.io/schematic/) `SchemaRenderer` for Pkl.
//!
//! Schematic already has a [Pkl *template* renderer](https://github.com/moonrepo/schematic/blob/master/crates/schematic/src/schema/renderers/pkl_template.rs), but it only provides basic rendering for template/template generation. This implementation produces robustly typed schemas with type annotations, constraints, defaults, nuanced type handling, and idiomatic Pkl constructs.
//!
//! ## Why?
//!
//! Pkl offers a powerful schema system, with an exceptionally robust type system. This makes it ideal for configuration management in large repos and organizations. By opening the door to direct schema generation, you can now write configurations based on those schema that:
//! - Have first-class IDE support, providing as-you-type type information, usage tips, documentation, syntax linting/highlighting, and more. See [the Pkl tools docs](https://pkl-lang.org/main/current/tools.html).
//! - Can force schema, default, and config alignment across large repos. Pkl's powerful `extend`/`amend` capabilities allow you to *treat the root config as an enforced type*, and, if you allow it, allow people to make reasonable changes to defaults. A single source of truth with built-in flexibility.
//! - Pkl itself is a powerful dynamic language. It is purpose-built for configurations. You can use its dynamic configs directly, or to generate conditional configs for any common format (yaml, toml, json, messagepack...). Or, use your Pkl to generate native Pkl static configs, `pcf`. (Syntax-wise, it feels closest to `Swift`)
//! - The pkl language has sophisticated capabiilties you won't find in config formats -- for and when generators, complex conditionals, built-in converters, a lazy-evaluation-by-default framework.
//!
//! Bottom line: it's pretty cool.
//!
//! ## Key Features and Design Notes
//!
//! The renderer aims to:
//! - Render idiomatic Pkl aligned to the [Pkl Style Guide](https://pkl-lang.org/main/current/style-guide/index.html) by default. There are some options that allow you to customize away from that default benchmark, but I wanted to deliver uncompromising pkl.
//! - Provide robust type annotations and constraints, including:
//!   - Full type coverage for deeply nested, complex, and optional types.
//!   - Full use of Pkl's type system -- even including [`DataSize`](https://pkl-lang.org/main/current/language-reference/index.html#data-sizes) and [`Duration`](https://pkl-lang.org/main/current/language-reference/index.html#durations) if correctly marked by schematic.
//!   - Complete implementation of schematic's available type constraints. Pkl's type system allows arbitrary constrained types. This is a valid type in Pkl:
//!     ```pkl
//!
//!    /// self-validating email type -- valid pkl
//!    typealias Email = String(
//!       matches(
//!         Regex(
//!           #"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"#
//!         )
//!        )
//!       )
//!
//!     // and so is:
//!
//!     /// You could also define this long anonymous function in a
//!     /// separate named function and just call it in the annotation.
//!     /// You could also define it inline without defining an alias.
//!     typealias UserData: Mapping<String, Listing<String>>(
//!       List("email", "address", "id")
//!         .every(
//!           (k) -> this.keys.containsKey(k)) && // required keys present
//!         this.every(
//!           (k,v) -> !k.isEmpty && //no empty keys
//!             !v.isEmpty &&                   // no empty values
//!             if (k == "email"))
//!               v.every(
//!                 (email) -> email is Email   // all valid emails
//!               ) &&
//!               v.isDistinct                  // all emails are unique
//!       )
//!
//!     class Customers {
//!       users: UserData
//!       product: AcmeType
//!     }
//!     ```
//!   (The example is intentionally over-the-top, but hopefully you see why this helps make Pkl a powerful configuration language.)
//!
//!   - Handle complex types like `Struct`, `Array`, `Object`, `Tuple`, and `Union` with full type annotations and constraints.
//!   - Support enum translations as type aliases or literal unions, with full type annotations.
//!   - Allow for including or excluding (default) deprecated types. Included deprecations use Pkl's `@Deprecated` decorator with reason and `since` version if available from schematic.
//!   - Correct marking of default values, such as with the `*` operator.
//!   - Support for `open` classes/modules, enabling Pkl's `extend` and `amend` features.
//!   - Renders the top-level `Config` struct as a module by default, but can be switched to a class. This allows you to directly use the generated module as a type using `amends`.
//!   - Customizable options for module/class naming, indentation, and more.

/**========================================================================
 **                       ## A Crash Course in schematic
 **========================================================================
 **       (You can skip this if you're not going to work on the Renderer)
 *========================================================================**/
//
//! I'm going to explain this simply because the type structure was hard to understand.
//! This is my `schematic 101`. The [docs](https://moonrepo.github.io/schematic/) are good, they just didn't click for me.
//!
//! # Schematic’s Core Types & Traits
//!
//! - **`Config` (trait)**: Marks types you want to use as configuration roots. The runtime config (e.g., `ConfigSettingMap`) isn’t directly relevant for schema rendering—what matters is the structure of your Rust config types and how they’re described in the schema system.
//!
//! - **`SchemaGenerator`**: The bridge between your Rust types and the schema representation. It recursively walks your config’s type tree and produces an `IndexMap<String, Schema>`, where keys are **names of named types** (structs, enums, unions, and sometimes type aliases) and values are their `Schema` definitions. **Let's call this the `TypeMap`**.
//!
//!   **Key points about the `TypeMap`:**
//!   - It’s flat at the top level: **all named types** from your config (root and nested, at any depth) are present as siblings.
//!   - “Named type” = any type that schematic gives a name to (struct, enum, union, sometimes type alias). Primitives (`String`, `i32`, etc.) and standard generics (`Vec<_>`, `Option<_>`, `HashMap<_, _>`, etc.) are NOT included as top-level entries, unless you’ve defined a named alias and schematic captures that name.
//!   - It **maps type names** (not field/property names) **to their schemas**. Fields are described within the `fields` property of struct or enum schemas.
//!   - The type graph is potentially deep/complex: schemas reference each other, creating nested structures and supporting recursion.
//!   - The `SchemaGenerator` creates this map and hands it to a `SchemaRenderer`.
//!
//! - **`SchemaRenderer` (trait)**: Takes the `TypeMap` and translates it into your target format (e.g., Pkl, TypeScript, JSON Schema, etc.). **The renderer is the translator.**
//!
//! ## `Schema` and `SchemaType`
//!
//! `Schema` represents a single named type in the `TypeMap`. Its most important field is `ty: SchemaType`, which describes the actual type.
//!
//! ```text
//! Schema
//!  ├── deprecated: Option<String>       // marked deprecated?
//!  ├── description: Option<String>      // doc comment/description
//!  ├── name: Option<String>             // type name (if any)
//!  ├── nullable: bool                   // nullable?
//!  └── ty: SchemaType                   // ← **This is the important part**
//!      └── Struct(Box<StructType>)      // (could be any SchemaType variant)
//!          ├── fields: IndexMap<String, Box<SchemaField>>
//!          │   ├── "field1" -> SchemaField
//!          │   │   └── schema: Schema
//!          │   │       └── ty: SchemaType::String | Enum | Struct | ...
//!          │   └── "field2" -> SchemaField
//!          │       └── schema: Schema
//!          │           └── ty: SchemaType::...
//!          └── partial: bool
//! ```
//!
//! **`SchemaType`** is an enum describing what kind of type the schema represents:
//!
//! ```rust,ignore
//! pub enum SchemaType {
//!   Null, Unknown,
//!   Array(Box<ArrayType>), Boolean(Box<BooleanType>), Enum(Box<EnumType>),
//!   Float(Box<FloatType>), Integer(Box<IntegerType>), Literal(Box<LiteralType>),
//!   Object(Box<ObjectType>), Struct(Box<StructType>), String(Box<StringType>),
//!   Tuple(Box<TupleType>), Union(Box<UnionType>),
//!   Reference(String),  // ← This one's special
//! }
//! ```
//!
//! **`Reference`** is special. It’s a pointer to another named type **by name**, rather than by inlining the whole definition. In Rust, this happens when a struct (or enum/union) field uses a named type:
//!
//! ```rust,ignore
//! pub struct Heroes {
//!   marvel: Marvel, // ← `Marvel` is a `Reference` here
//!   dc: DC,         // ← `DC` is also a `Reference`
//! }
//! ```
//! References enable recursion (e.g., linked lists), sharing types, and keeping the type graph acyclic for rendering (that's arcane programmer-speak for 'no infinite loops').
//!
//! # How to introspect `SchemaType` variants
//!
//! - **`Struct`**: Access its fields via the `fields` property (`IndexMap<String, Box<SchemaField>>`).
//! - **`Array`**: Look at the `items_type` property, which is a `Schema` for the array’s element type.
//! - **`Object`**: Use `key_type` and `value_type` (both `Schema`) for key/value types.
//! - **`Tuple`**: Use `items_types` (a vector of `Schema`) for each tuple slot. (Trick: `Tuple` = multiple `items_types`, `Array` = one `items_type`)
//! - **`Enum`**: Use `values` for C-like literals (`Vec<LiteralValue>`), and `variants` (if present) for struct/tuple variants (`Option<IndexMap<String, Box<SchemaField>>>`).
//! - **`Reference`**: The `String` is the name; look up that named type in the `TypeMap`.
//!

use std::collections::HashSet;
use indexmap::{IndexMap, IndexSet};
use schematic::format::Format;
use schematic::schema::{RenderResult, SchemaRenderer, RenderError};
use schematic_types::*;
use regex::Regex;
use std::sync::OnceLock;

use crate::constants::{DATA_SIZE_UNITS, DURATION_UNITS};
use crate::types::{TypeMap, EnumTranslation, OpenStructs, ConfigTranslation, OptionalFormat, PropertyDefault, LoadedConfig};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderType {
    Template,
    #[default]
    Schema,
}

impl std::str::FromStr for RenderType {
  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "template" | "tmpl" | "t" => Ok(RenderType::Template),
      "schema" | "sch" | "s" => Ok(RenderType::Schema),
      _ => Err(RenderError::UnsupportedFormat {
        format: s.to_string(),
        available: vec!["template", "schema"],
      }),
    }
  }
}
#[derive(Debug, Clone)]
struct ParsedReference {
    /// The root type name (e.g., "Count" in "Count::Two")
    root: String,
    /// The path components after the root (e.g., ["Two"] in "Count::Two")
    path: Vec<String>,
    /// Whether this was originally a Self/self reference
    is_self_reference: bool,
}

#[derive(Debug, Clone)]
enum ResolvedReference {
    /// Successfully resolved to a type
    Type {
        name: String,      // Transformed type name (PascalCase)
        schema: Schema,    // The resolved schema
    },
    /// Successfully resolved to a property
    Property {
        type_name: String,      // Parent type (PascalCase)
        property_name: String,  // Property name (camelCase)
        field: SchemaField,     // The resolved field
    },
    /// Resolved to parent type when specific member couldn't be found
    FallbackToParent {
        parent_name: String,    // Parent type we fell back to
        original_path: Vec<String>, // Original path that couldn't be resolved
    },
    /// Could not be resolved at all
    Unresolved {
        original_text: String,  // Original reference text
    },
}

struct LinkMatch {
  name: Option<String>,
  url: Option<String>,
  reference: Option<String>,
  full_match: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentType {
  #[default]
  Doc,
  Line,
  Block,
  None,
}

impl CommentType {
    fn normalize(&self, text: &str) -> String {
      // Normalize line endings to LF
      text.replace("\r\n", "\n")
          .replace("\r", "\n")
          .trim()
          .to_string()
    }

    pub fn to_comment(&self, text: &str, indent: &str) -> String {
      let content = self.normalize(text);
      if content.is_empty() {
        return String::new();
      }
      match self {
        CommentType::None => {
          return String::new();
        }
        CommentType::Doc => {
          return content
            .lines()
            .map(|line| format!("{}/// {}", indent, line.trim()))
            .collect::<Vec<_>>()
            .join("\n");
        }
        CommentType::Line => {
          return content
            .lines()
            .map(|line| format!("{}// {}", indent, line.trim()))
            .collect::<Vec<_>>()
            .join("\n");
        }
        CommentType::Block => {
          let lines = if content.lines().count() > 1 {
            content
              .lines()
              .map(|line| format!("{} * {}", indent, line.trim()))
              .collect::<Vec<_>>()
              .join("\n");
              return format!("{}/*\n{}\n{}\n*/", indent, lines, indent);
          } else {
            return format!("{}/* {} */", indent, content);
          };
        }
      }
    }

    /// Convert comment with reference resolution
    pub fn to_comment_with_resolver(&self, text: &str, indent: &str, resolver: &PklSchemaRenderer) -> String {
      let resolved_content = resolver.resolve_doc_references(text);
      self.to_comment(&resolved_content, indent)
    }
}

#[derive(Debug, Clone)]
pub struct PklSchemaOptions {
  /// The name of the config to use for the root schema, LoadedConfig (moon config type or one you give); no default
  ///
  pub config: LoadedConfig,
  /// Include documentation comments from schema descriptions
  pub include_docs: bool,

  /// Include type constraints where available
  /// Pkl allows for arbitrary type constraints within its types, so constraints will be enforced by Pkl's evaluator. Constraints are limited to those supported by schematic, which vary by type (they include regex pattern, min/max length or number, and required keys).
  pub include_constraints: bool,

  /// are you using this for a template or a schema? Primarily affects case decisions.
  pub render_as: RenderType,

  /// Disable references and render all types inline recursively.
  pub disable_references: bool,

  /// Indentation string (default: 2 spaces)
  pub indent: String,

  /// Provide an optional import path to a pkl module to `extend` from
  pub extend_from: Option<String>,

  /// Headers to include in the generated schema, such as license or copyright information. **Don't include comment syntax** -- we'll add it for you.
  pub headers: Vec<String>,

  /// Footers to include in the generated schema. **Don't include comment syntax** -- we'll add it for you.
  pub footers: Vec<String>,

  /// Pkl is a dynamic language, so you can optionally include function definitions for the output. These could be helpers, converters, or other utility functions that are useful for the schema. Each needs to be a correctly formatted Pkl function, and any comments need to be correctly marked (i.e. `///` for doc comments -- immediately before a function, `//` for line comments, or `/* */` for block comments).
  pub functions: Vec<String>,

  /// Pkl has the concept of 'output statements', which allow you to control exactly what information from the module is available to the outside world. As this can be anything, you need to define it yourself in valid Pkl syntax and we'll insert it for you.
  pub output_statement: String,

  /// Include default values in the schema
  pub include_defaults: bool,

  /// Include deprecated fields in the schema
  pub include_deprecated: bool,

  /// Whether to comment out optional fields in the schema, useful for template-style generation
  pub comment_out_optional: bool,

  /// A list of properties to exclude from created schema
  pub exclude_properties: Vec<&str>,

  /// A list of valid pkl import uris
  pub added_imports: Vec<&str>,

  /// How to translate enum types (typealias/literal_union; default: typealias)
  pub enum_translation: EnumTranslation,

  /// Whether to mark public structs as `open` when translated to classes (open/no; default: open)
  pub open_structs: OpenStructs,

  /// Whether to render the module as `open module ModuleName` (open/no; default: open)
  pub open_module: OpenStructs,

  /// How to translate the top-level `Config` struct (module/class; default: module)
  pub config_translation: ConfigTranslation,

  /// How to render optional type annotations (optional/optional_explicit_nothing; default: optional)
  pub optional_format: OptionalFormat,

  /// Whether to default to requiring properties or marking them optional when the schema lacks information on optionality.
  pub property_default: PropertyDefault,
}

impl Default for PklSchemaOptions {
  fn default() -> Self {
      Self {
        config: LoadedConfig::default(),
        include_docs: true,
        include_constraints: true,
        render_as: RenderType::Schema,
        disable_references: false,
        indent: "  ".to_string(),
        extend_from: None,
        headers: vec![
          "Schema generated by Pklr's PklSchemaRenderer.".to_string(),
        ],
        footers: Vec::new(),
        functions: Vec::new(),
        output_statement: String::new(),
        include_defaults: true,
        include_deprecated: false,
        comment_out_optional: false,
        exclude_properties: Vec::new(),
        added_imports: Vec::new(),
        enum_translation: EnumTranslation::TypeAlias,
        open_structs: OpenStructs::Open,
        open_module: OpenStructs::Open,
        config_translation: ConfigTranslation::Module,
        optional_format: OptionalFormat::Optional,
        property_default: PropertyDefault::RequireProperties,
      }
  }
}

/// Renders idiomatic Pkl schema definitions with type annotations and constraints.
pub struct PklSchemaRenderer {
  schemas: TypeMap,
  options: PklSchemaOptions,
  depth: usize,
  /// Track `Reference`s to prevent the universe from imploding
  references: IndexSet<String>,
  included_schemas: TypeMap,
  // Construction queues:
  module_properties: IndexMap<String, String>,
  typealiases: IndexMap<String, String>,
  classes: IndexMap<String, String>,
  module: Option<Schema>,
  /// Track current schema name for Self/self resolution
  current_schema_name: Option<String>,
}
impl PklSchemaRenderer {
  /// Creates a new [`PklSchemaRenderer`] with the given schemas and options.
  pub fn new(options: PklSchemaOptions) -> Self {
    Self {
      options,
      depth: 0,
      schemas: TypeMap::new(),
      references: IndexSet::new(),
      included_schemas: TypeMap::new(),
      module_properties: IndexMap::new(),
      typealiases: IndexMap::new(),
      classes: IndexMap::new(),
      module: None,
      current_schema_name: None,
    }
  }
  }

  /// If enabled, comments out a non-required section.
  pub fn comment_out(&self, text: &str) -> String {
    if !self.options.comment_out_optional {
      return text.to_string();
    }
    return CommentType::Block.to_comment(text, &self.indent());
  }

  /// Returns the indent string based on the current depth and options.
  fn indent(&self) -> String {
    let chars = if self.options.indent.is_empty() {
        "  " // default to 2 spaces if indent is empty
    } else {
        &self.options.indent
    };

    if self.depth == 0 {
        String::new()
    } else {
        chars.repeat(self.depth)
    }
  }

  /// Returns the imports as a formatted string.
  fn imports(&self) -> String {
      let mut imports = self.options.added_imports;
      if !imports.is_empty() {
          imports = imports.map(|import| {
              if import.starts_with("import ") &&
                (import.contains('"') || import.contains("'")) {
                  import.trim().to_string() // already formatted
              } else {
                  format!("import \"{}\"", import.trim().replace('"', "").replace("'", ""))
              }
          })
          .collect::<Vec<_>>();
          imports.push("".to_string()); // add a blank line after imports
      }
      imports.join("\n")
  }

  /// Returns the name of the struct being rendered
  /// Guaranteed if it's a moon config, otherwise it
  /// will attempt to find the root struct name using
  /// the [`LoadConfig.attempt_to_resolve_name`] method.
  fn get_struct_name(&self) -> String {
    match self.options.config {
        LoadedConfig::Unknown => self.options.config.attempt_to_resolve_name(&self.included_schemas),
        _ => self.options.config.struct_name(),
    }
  }

  /// Returns the name of the module being rendered
  /// Formatted according to Pkl conventions and the options provided.
  fn get_module_name(&self, struct_name: &str) -> String {
    let module_prefix = match self.options.open_module {
        OpenStructs::Open => "open module",
        OpenStructs::No => "module",
    };

    let base_name = if struct_name.is_empty() {
        self.options.config.config_type_name(&self.included_schemas)
    } else {
        match self.options.config_translation {
            ConfigTranslation::Module => struct_name.to_string(),
            ConfigTranslation::Class => self.options.config.config_type_name(&self.included_schemas),
        };
        if let Some(schema_struct) = self.schemas.get(struct_name) {
          // it's our top-level schema, so whether it's a module or a class,
          // we set it and can reference it
            self.module = Some(schema_struct.clone());
        }
    };

    let formatted_name = match self.options.render_as {
        RenderType::Template => self.to_camel_case(&base_name),
        RenderType::Schema => self.to_pascal_case(&base_name),
    };

    format!("{} {}", module_prefix, formatted_name)
  }

  /// Assembles the complete header, including [added headers](`PklSchemaOptions::headers`), module documentation,
  /// docstring, module name, extension, and imports based on [settings](`PklSchemaOptions`).
  fn render_header(&self) -> String {
    let mut header = String::new();

    // Add block comment headers
    if !self.options.headers.is_empty() {
        header.push_str(&CommentType::Block.to_comment(
            &self.options.headers.join("\n"),
            &self.indent()
        ));
        header.push('\n');
    }

    let struct_name = self.get_struct_name();

    // Add module documentation
    if self.options.include_docs {
        if let Some(description) = self.schemas.get(&struct_name)
            .and_then(|s| s.description.as_ref()) {
            header.push_str(&CommentType::Doc.to_comment_with_resolver(description, &self.indent(), self));
            header.push_str("\n\n");
        } else {
            header.push('\n');
        }
    }

    // Build import section
    let mut sections = vec![self.get_module_name(&struct_name)];

    if let Some(extend_from) = &self.options.extend_from {
        sections.push(format!("extends \"{}\"", extend_from));
    }

    if !self.options.added_imports.is_empty() {
        sections.push(self.imports());
    }

    header.push_str(&sections.join("\n\n"));
    header
  }
  /// Checks if a reference should be excluded based on the options.
  fn is_excluded(&self, name: &str) -> bool {
    self.options.exclude_properties.iter().any(|r| *r == name)
  }

  fn to_render(&self, schema: &Schema) -> String {
    // Render docstring if enabled
    let mut output = if self.options.include_docs && let Some(description) = &schema.description {
          CommentType::Doc.to_comment(description, &self.indent())
        } else {
          String::new()
        };
    // Render the type
    let type_str = match &schema.ty {
        SchemaType::Struct(struct_type) => self.render_struct(struct_type),
        SchemaType::Enum(enum_type) => self.render_enum(enum_type),
        SchemaType::Array(array_type) => self.render_array(array_type),
        SchemaType::Float(float_type) => self.render_float(float_type),
        SchemaType::Integer(integer_type) => self.render_integer(integer_type),
        SchemaType::String(string_type) => self.render_string(string_type),
        SchemaType::Boolean(boolean_type) => self.render_boolean(boolean_type),
        SchemaType::Object(object_type) => self.render_object(object_type),
        SchemaType::Tuple(tuple_type) => self.render_tuple(tuple_type),
        SchemaType::Union(union_type) => self.render_union(union_type),
        SchemaType::Reference(reference) => self.render_reference(reference),
        SchemaType::Unknown(unknown_type) => self.render_unknown(unknown_type),
        SchemaType::Null(null_type) => self.render_null(null_type),
    };
    output.push_str(type_str.as_str());
    output
  }

  fn render_properties(&self) -> String {
    if self.module.is_some() {
      // TODO: Implement property rendering
      String::new()
    } else {
      String::new()
    }
  }

  /// Convert to PascalCase for classes and modules
  fn to_pascal_case(&self, name: &str) -> String {
    if name.is_empty() {
      return name.to_string();
    }

    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in name.chars() {
      if ch == '_' || ch == '-' {
        capitalize_next = true;
      } else if capitalize_next {
        result.push(ch.to_uppercase().next().unwrap_or(ch));
        capitalize_next = false;
      } else {
        result.push(ch);
      }
    }

    result
  }

  /// Convert to camelCase for properties
  fn to_camel_case(&self, name: &str) -> String {
    if name.is_empty() {
      return name.to_string();
    }

    let mut result = String::new();
    let mut capitalize_next = false;
    let mut first_char = true;

    for ch in name.chars() {
      if ch == '_' || ch == '-' {
        capitalize_next = true;
      } else if capitalize_next {
        result.push(ch.to_uppercase().next().unwrap_or(ch));
        capitalize_next = false;
      } else if first_char {
        result.push(ch.to_lowercase().next().unwrap_or(ch));
        first_char = false;
      } else {
        result.push(ch);
      }
    }

    result
  }
  /// Main entry point for resolving doc comment references
  fn resolve_doc_references(&self, text: &str) -> String {
    static BACKTICK_REF: OnceLock<Regex> = OnceLock::new();
    static SIMPLE_REF: OnceLock<Regex> = OnceLock::new();
    static LINK_WITH_BACKTICKS: OnceLock<Regex> = OnceLock::new();
    static LINK_WITHOUT_BACKTICKS: OnceLock<Regex> = OnceLock::new();
    static REFERENCE_STYLE: OnceLock<Regex> = OnceLock::new();
    static REFERENCE_DEFINITION: OnceLock<Regex> = OnceLock::new();

    // [`reference`] style - backticks around the link will be stripped
    let backtick_regex = BACKTICK_REF.get_or_init(|| {
      Regex::new(r"\[`(?P<ref>[^`\]]+)`\]").unwrap()
    });

    // [reference] style - simple link without backticks
    let simple_regex = SIMPLE_REF.get_or_init(|| {
      Regex::new(r"\[(?P<ref>[^\]`\(\)]+)\](?!\(|\[)").unwrap()
    });

    // [text](`reference`) style - link with backticks around reference
    let link_backticks_regex = LINK_WITH_BACKTICKS.get_or_init(|| {
      Regex::new(r"\[(?P<text>[^\]]+)\]\(`(?P<ref>[^`\)]+)`\)").unwrap()
    });

    // [text](reference) style - link without backticks around reference
    let link_no_backticks_regex = LINK_WITHOUT_BACKTICKS.get_or_init(|| {
      Regex::new(r"\[(?P<text>[^\]]+)\]\((?P<ref>[^\)`]+)\)").unwrap()
    });

    // [text][reference] style - reference-style link
    let reference_style_regex = REFERENCE_STYLE.get_or_init(|| {
      Regex::new(r"\[(?P<text>[^\]]+)\]\[(?P<ref>[^\]]+)\]").unwrap()
    });

    // [reference]: target - reference definition (we'll ignore these for now)
    let reference_def_regex = REFERENCE_DEFINITION.get_or_init(|| {
      Regex::new(r"^\s*\[(?P<ref>[^\]]+)\]:\s*(?P<target>.+)$").unwrap()
    });

    let mut result = text.to_string();

    // Handle [`reference`] style - backticks around the link will be stripped
    result = backtick_regex.replace_all(&result, |caps: &regex::Captures| {
      let reference = &caps["ref"];
      let parsed = self.parse_reference_path(reference);
      let resolved = self.resolve_reference_target(&parsed);
      self.generate_pkl_link(resolved, None)
    }).to_string();

    // Handle [reference] style - simple link
    result = simple_regex.replace_all(&result, |caps: &regex::Captures| {
      let reference = &caps["ref"];
      let parsed = self.parse_reference_path(reference);
      let resolved = self.resolve_reference_target(&parsed);
      self.generate_pkl_link(resolved, None)
    }).to_string();

    // Handle [text](`reference`) style - link with backticks
    result = link_backticks_regex.replace_all(&result, |caps: &regex::Captures| {
      let text = &caps["text"];
      let reference = &caps["ref"];
      let parsed = self.parse_reference_path(reference);
      let resolved = self.resolve_reference_target(&parsed);
      self.generate_pkl_link(resolved, Some(text))
    }).to_string();

    // Handle [text](reference) style - link without backticks
    result = link_no_backticks_regex.replace_all(&result, |caps: &regex::Captures| {
      let text = &caps["text"];
      let reference = &caps["ref"];
      let parsed = self.parse_reference_path(reference);
      let resolved = self.resolve_reference_target(&parsed);
      self.generate_pkl_link(resolved, Some(text))
    }).to_string();

    // Handle [text][reference] style - reference-style link
    result = reference_style_regex.replace_all(&result, |caps: &regex::Captures| {
      let text = &caps["text"];
      let reference = &caps["ref"];
      let parsed = self.parse_reference_path(reference);
      let resolved = self.resolve_reference_target(&parsed);
      self.generate_pkl_link(resolved, Some(text))
    }).to_string();

    // Remove reference definitions (they shouldn't appear in output)
    result = reference_def_regex.replace_all(&result, "").to_string();

    result
  }

  /// Parse a reference path like "Count::Two" into components
  fn parse_reference_path(&self, reference: &str) -> ParsedReference {
    let parts: Vec<&str> = reference.split("::").collect();

    if parts.is_empty() {
      return ParsedReference {
        root: String::new(),
        path: Vec::new(),
        is_self_reference: false,
      };
    }

    let (root, is_self_reference) = if parts[0] == "Self" || parts[0] == "self" {
      // Resolve Self/self to actual type name
      let actual_root = self.current_schema_name.as_deref().unwrap_or("UnknownType");
      (actual_root.to_string(), true)
    } else {
      (parts[0].to_string(), false)
    };

    ParsedReference {
      root,
      path: parts[1..].iter().map(|s| s.to_string()).collect(),
      is_self_reference,
    }
  }

  /// Resolve a parsed reference to actual schema elements with fallback
  fn resolve_reference_target(&self, parsed: &ParsedReference) -> ResolvedReference {
    // Try to resolve the full path first
    if let Some(resolved) = self.try_full_path_resolution(parsed) {
      return resolved;
    }

    // Fall back to progressively shorter paths
    for i in (1..=parsed.path.len()).rev() {
      let partial_path = &parsed.path[..i-1];
      if let Some(parent) = self.try_partial_resolution(&parsed.root, partial_path) {
        return ResolvedReference::FallbackToParent {
          parent_name: parent,
          original_path: std::iter::once(parsed.root.clone())
            .chain(parsed.path.clone())
            .collect(),
        };
      }
    }

    // No resolution possible
    ResolvedReference::Unresolved {
      original_text: std::iter::once(parsed.root.clone())
        .chain(parsed.path.clone())
        .collect::<Vec<_>>()
        .join("::"),
    }
  }

  /// Try to resolve the full reference path
  fn try_full_path_resolution(&self, parsed: &ParsedReference) -> Option<ResolvedReference> {
    // First try to resolve as a type reference
    if parsed.path.is_empty() {
      return self.resolve_type_reference(&parsed.root);
    }

    // Try to resolve as a property reference
    self.resolve_property_reference(&parsed.root, &parsed.path)
  }

  /// Try to resolve a partial path for fallback
  fn try_partial_resolution(&self, root: &str, partial_path: &[String]) -> Option<String> {
    if partial_path.is_empty() {
      // Try just the root type
      if self.schemas.contains_key(root) {
        return Some(self.to_pascal_case(root));
      }
    }

    // TODO: Implement more sophisticated partial resolution
    // For now, just try the root type
    if self.schemas.contains_key(root) {
      Some(self.to_pascal_case(root))
    } else {
      None
    }
  }

  /// Resolve a type reference
  fn resolve_type_reference(&self, type_name: &str) -> Option<ResolvedReference> {
    let resolved_name = if type_name == "Self" || type_name == "self" {
      self.current_schema_name.as_ref()?
    } else {
      type_name
    };

    // Look up in TypeMap
    let schema = self.schemas.get(resolved_name)?;
    Some(ResolvedReference::Type {
      name: self.to_pascal_case(resolved_name),
      schema: schema.clone(),
    })
  }

  /// Resolve a property reference with enum awareness
  fn resolve_property_reference(&self, type_name: &str, property_path: &[String]) -> Option<ResolvedReference> {
    let schema = self.schemas.get(type_name)?;

    match &schema.ty {
      SchemaType::Struct(struct_type) => {
        // Navigate through struct fields
        self.resolve_struct_property(struct_type, property_path, type_name)
      },
      SchemaType::Enum(_) => {
        // For enums, we can't resolve to specific variants
        // This will trigger fallback resolution
        None
      },
      _ => None,
    }
  }

  /// Resolve a property within a struct
  fn resolve_struct_property(
    &self,
    struct_type: &StructType,
    property_path: &[String],
    type_name: &str
  ) -> Option<ResolvedReference> {
    if property_path.is_empty() {
      return None;
    }

    let field_name = &property_path[0];
    let field = struct_type.fields.get(field_name)?;

    if property_path.len() == 1 {
      // Found the final property
      Some(ResolvedReference::Property {
        type_name: self.to_pascal_case(type_name),
        property_name: self.to_camel_case(field_name),
        field: *field.clone(),
      })
    } else {
      // TODO: Handle nested property resolution
      None
    }
  }

  /// Generate the final Pkl link format
  fn generate_pkl_link(&self, resolved: ResolvedReference, display_text: Option<&str>) -> String {
    match resolved {
      ResolvedReference::Type { name, .. } => {
        let display = display_text.unwrap_or(&name);
        format!("[{}]({})", display, name)
      },
      ResolvedReference::Property { type_name, property_name, .. } => {
        let target = format!("{}.{}", type_name, property_name);
        let display = display_text.unwrap_or(&target);
        format!("[{}]({})", display, target)
      },
      ResolvedReference::FallbackToParent { parent_name, original_path, .. } => {
        // Keep original display text but link to parent
        let display = display_text.unwrap_or(&original_path.join("::"));
        format!("[{}]({})", display, parent_name)
      },
      ResolvedReference::Unresolved { original_text } => {
        // Remove link formatting but keep text content
        display_text.unwrap_or(&original_text).to_string()
      },
    }
  }

impl SchemaRenderer for PklSchemaRenderer {

    fn render_struct(&self, struct_type: &StructType, _schema: &Schema) -> RenderResult<String> {
        Ok("struct".to_string()) // TODO: Implement
    }

    fn render_enum(&self, enum_type: &EnumType, _schema: &Schema) -> RenderResult<String> {
        Ok("enum".to_string()) // TODO: Implement
    }

    fn render_array(&self, array_type: &ArrayType, _schema: &Schema) -> RenderResult<String> {
        Ok("array".to_string()) // TODO: Implement
    }

    fn render_float(&self, float_type: &FloatType, _schema: &Schema) -> RenderResult<String> {
        Ok("Float".to_string()) // TODO: Implement
    }

    fn render_integer(&self, integer_type: &IntegerType, _schema: &Schema) -> RenderResult<String> {
        Ok("Int".to_string()) // TODO: Implement
    }

    fn render_string(&self, string_type: &StringType, _schema: &Schema) -> RenderResult<String> {
        Ok("String".to_string()) // TODO: Implement
    }

    fn render_boolean(&self, boolean_type: &BooleanType, _schema: &Schema) -> RenderResult<String> {
        Ok("Boolean".to_string()) // TODO: Implement
    }

    fn render_object(&self, object_type: &ObjectType, _schema: &Schema) -> RenderResult<String> {
        Ok("object".to_string()) // TODO: Implement
    }

    fn render_tuple(&self, tuple_type: &TupleType, _schema: &Schema) -> RenderResult<String> {
        Ok("tuple".to_string()) // TODO: Implement
    }

    fn render_union(&self, union_type: &UnionType, _schema: &Schema) -> RenderResult<String> {
        Ok("union".to_string()) // TODO: Implement
    }

    fn render_reference(&self, reference: &str, _schema: &Schema) -> RenderResult<String> {
        Ok(self.to_pascal_case(reference)) // TODO: Implement
    }

    fn render_unknown(&self, _schema: &Schema) -> RenderResult<String> {
        Ok("unknown".to_string()) // TODO: Implement
    }

    fn render_null(&self, _schema: &Schema) -> RenderResult<String> {
        Ok("nothing".to_string()) // TODO: Implement
    }


    /// Renders the schema as a Pkl string.
    ///
    /// The constructed order of elements will follow the [Pkl Style Guide](https://pkl-lang.org/main/current/style-guide/index.html#module-body):
    /// 1. [any provided header](`PklSchemaOptions::headers`)
    /// 2. [module docstring](`PklSchemaOptions::module_docstring`)
    /// 3. [imports](`PklSchemaOptions::added_imports`)
    /// 4. [properties](`PklSchemaOptions::module_properties`)
    /// 5. [methods/functions](`PklSchemaOptions::methods`)
    /// 6. [classes](`PklSchemaOptions::classes`)
    /// 7. [type aliases](`PklSchemaOptions::type_aliases`)
    /// 8. [amended output](`PklSchemaOptions::amended_output`).
    fn render(&mut self, schemas: TypeMap) -> RenderResult<String> {
        self.schemas = schemas.clone();
        self.references = self.schemas
          .keys()
          .filter(|name| !self.is_excluded(name))
          .cloned()
          .collect();
        // filter schemas -- used for finding module names
        if self.options.exclude_properties.is_empty() {
          // we're not mutating the schemas, so we can use a reference
            self.included_schemas = schemas;
        } else {
            self.included_schemas = schemas.into_iter()
                .filter(|(name, _schema)| {
                    !self.is_excluded(name) && self.references.contains(name)
                })
                .collect();
        }
        // render the header
        let mut output = self.render_header();

        Ok(output)
    }
}
