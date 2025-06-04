//! Template Engine Module for Pkl Schema Generation
//!
//! This module provides a comprehensive template system for generating Pkl configuration
//! schemas from structured type definitions. It uses a unified, DRY approach with
//! Handlebars templating to convert type definitions into properly formatted Pkl files.

use crate::config::GeneratorConfig;
use crate::types::*;
use crate::Result;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use handlebars_misc_helpers;
use miette::{IntoDiagnostic, WrapErr};
use serde_json::Value;
use std::collections::HashMap;
use tracing::debug;

/// Template engine for rendering Pkl schemas from type definitions.
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

/// Represents the type of item being rendered
#[derive(Debug, PartialEq, Clone)]
pub enum ItemType {
    /// Represents a Pkl module
    Module,
    /// Represents a Pkl class (similar to a struct)
    Class,
    /// Represents a Pkl property
    Property,
}

/// Configuration for rendering behavior based on item type
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// The type of item being rendered (Module, Class, Property)
    pub item_type: ItemType,
    /// Whether to indent the output (typically true for properties)
    pub indent: bool,
    /// Whether to include deprecated items in the output
    pub include_deprecated: bool,
    /// Whether to mark classes as `open`
    pub make_open: bool,
}

impl TemplateEngine {
    /// Creates a new template engine with clean, modern configuration
    pub fn new(config: &GeneratorConfig) -> Self {
        let mut handlebars = Handlebars::new();

        // Disable HTML escaping for Pkl output
        handlebars.register_escape_fn(handlebars::no_escape);

        // Register templates and helpers
        Self::register_templates(&mut handlebars);
        Self::register_helpers(&mut handlebars);

        // Load custom templates if configured
        Self::load_custom_templates(&mut handlebars, config);

        Self { handlebars }
    }

    /// Renders a complete Pkl module
    pub fn render_module(&self, module: &PklModule, config: &GeneratorConfig) -> Result<String> {
        debug!("Rendering module '{}' with {} types, {} properties",
               module.name, module.types.len(), module.properties.len());

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

    fn register_templates(handlebars: &mut Handlebars) {
        // Single module template that handles everything
        handlebars
            .register_template_string("module", MODULE_TEMPLATE)
            .expect("Failed to register module template");
    }

    fn register_helpers(handlebars: &mut Handlebars) {
        // String manipulation helpers from handlebars_misc_helpers
        handlebars_misc_helpers::register(handlebars);

        // Core rendering helpers
        handlebars.register_helper("render_item", Box::new(render_item));
        handlebars.register_helper("should_render", Box::new(should_render));
        handlebars.register_helper("imports_section", Box::new(imports_section));
        handlebars.register_helper("module_examples", Box::new(module_examples));
        handlebars.register_helper("example_path", Box::new(example_path));

        // Type checking helpers
        handlebars.register_helper("is_typealias", Box::new(is_typealias));
        handlebars.register_helper("is_pkl_keyword", Box::new(is_pkl_keyword));
    }

    fn load_custom_templates(handlebars: &mut Handlebars, config: &GeneratorConfig) {
        if let Some(template_dir) = &config.template.template_dir {
            if template_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(template_dir) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                            let extension = &config.template.template_extension.trim_start_matches('.');
                            if entry.path().extension().and_then(|s| s.to_str()) == Some(extension) {
                                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                    let _ = handlebars.register_template_string(name, content);
                                    debug!("Loaded custom template: {}", name);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// =============================================================================
// UNIFIED TEMPLATE - Single source of truth
// =============================================================================

const MODULE_TEMPLATE: &str = r#"
{{~#if config.header}}{{config.header}}
{{/if~}}
{{~render_item module "module"}}
{{~#if config.include_examples~}}
/// # Using this Template
///
/// ## Amending a Template
///
/// You `amend` your config file with this template. This tells
/// `Pkl` that your config is a `{{to_pascal_case module.name}}` module type,
/// and will give you modern IDE type checking, tooltips, and completions with
/// the `Pkl` extension installed ([available for VSCode-family, nvim, IntelliJ]((https://pkl-lang.org/intellij/current/index.html)).
///
/// You do that like this:
///
/// ```pkl
///
/// // you would save your config as
/// // {{#if (eq module.name 'Project')}}`path/to/your/project/moon.pkl`{{else}}`/your-repo-root/.moon/{{to_lower_case module.name}}`{{/if}}
/// //
/// amends "{{example_path module.name}}.pkl"
///
/// {{module_examples module}}
///
/// ```
///
/// ### Local Import
///
/// You can use `space-pkl` to generate pkl schema and use them locally.
/// Above we assume they're in a directory called `pkl-schema` in the root of your repo.
/// You could generate this by running `space-pkl generate` from the repo root
/// (pkl-schema is the default directory name).
///
/// ### Web Import
///
/// You could also keep your templates on the web, but this will
/// require an https request every time Pkl evaluates the config.
/// It's as simple as `amends "https://yourdomain/some-path/{{to_pascal_case module.name}}.pkl"`
/// (this could be a github repo or any public or, with [additional setup](https://pkl-lang.org/main/current/pkl-cli/index.html#http-proxy), private URL).
///
/// ### Package Import
///
/// The `space-pkl` project provides packaged and versioned schema for every
/// [`moon-config`](https://crates.io/crates/moon_config) version, beginning with `0.1.4`.
/// Our package versions mirror the `moon-config` version numbers for clarity. You can
/// easily import and amend our packages from your config like this:
///
/// ```pkl
///
/// amends "package://github.com/knitli/space-pkl/blob/main/pkgs/moon-config@version#/{{to_pascal_case module.name}}"
///
/// ```
///
/// ## Extending a Template
///
/// You may also use `Pkl`'s `extend` features to *extend* a template.
/// (We haven't tested this yet, so we'd like to hear about your experience,
/// particularly how Moon handles new properties). This allows you to:
/// - Set different/custom default properties for all child configs that `amend` it.
/// - Add new properties, classes, or helpers.
/// - Enforce stricter type constraints, or extend the `moon` types.
///
/// Practically, you extend a template in the exact same way, replacing `amends`
///  with `extends`. For more information, see the [Pkl documentation](https://pkl-lang.org/main/current/language-reference/index.html#module-extend)
///
{{/if~}}

// module open for extension
open module {{to_pascal_case module.name}}

{{imports_section module.imports~}}

{{~#each module.types~}}
{{render_item this "class"}}

{{~/each~}}

{{~#each module.properties~}}
{{render_item this.name "property"}}

{{~/each~}}
{{~#if config.footer}}{{config.footer}}{{/if~}}
"#;

// =============================================================================
// CORE RENDERING LOGIC - Clean and unified
// =============================================================================

/// Main rendering function - handles all item types with unified logic
fn render_item(
    h: &Helper,
    _: &Handlebars,
    ctx: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let item = h.param(0).ok_or_else(|| handlebars::RenderErrorReason::ParamNotFoundForName("render_item", "item".to_string()))?;
    let item_type_str = h.param(1)
        .and_then(|p| p.value().as_str())
        .unwrap_or("auto");

    let item_value = item.value();
    let item_type = parse_item_type(item_type_str, item_value);
    let config = get_render_config(&item_type, ctx);

    // Check if we should render this item (deprecation filtering)
    if !should_render_item_with_config(item_value, &config) {
        return Ok(());
    }

    // Render header (documentation, examples, deprecation)
    render_header(item_value, &config, out)?;

    // Render body (type-specific content)
    render_body(item_value, &config, ctx, out)?;

    Ok(())
}

fn parse_item_type(type_str: &str, item: &Value) -> ItemType {
    match type_str {
        "module" => ItemType::Module,
        "class" => ItemType::Class,
        "property" => ItemType::Property,
        "auto" => detect_item_type(item),
        _ => ItemType::Property, // Safe default
    }
}

fn detect_item_type(item: &Value) -> ItemType {
    if item.get("types").is_some() && item.get("properties").is_some() {
        ItemType::Module
    } else if item.get("kind").is_some() {
        ItemType::Class
    } else {
        ItemType::Property
    }
}

fn get_render_config(item_type: &ItemType, ctx: &Context) -> RenderConfig {
    let include_deprecated = ctx.data()
        .get("config")
        .and_then(|c| c.get("include_deprecated"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let no_extends = ctx.data()
        .get("config")
        .and_then(|c| c.get("no_extends"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let make_open = !no_extends;

    RenderConfig {
        item_type: item_type.clone(),
        indent: matches!(item_type, ItemType::Property),
        include_deprecated,
        make_open,
    }
}

// =============================================================================
// HEADER RENDERING - Documentation, examples, deprecation
// =============================================================================

fn render_header(item: &Value, config: &RenderConfig, out: &mut dyn Output) -> HelperResult {
  // Documentation
  if let Some(doc) = item.get("documentation").and_then(|v| v.as_str()) {
      if !doc.trim().is_empty() {
          render_documentation(doc, config.indent, out)?;
      }
  }

  // Examples
  if let Some(examples) = item.get("examples").and_then(|v| v.as_array()) {
      if !examples.is_empty() {
          render_examples(examples, config.indent, out)?;
      }
  }

  // Deprecation (always show if item is deprecated and we're including deprecated items)
  if let Some(deprecated) = item.get("deprecated").and_then(|v| v.as_str()) {
      render_deprecation(deprecated, config.indent, out)?;
  }

  Ok(())
}

fn render_documentation(doc: &str, indent: bool, out: &mut dyn Output) -> HelperResult {
    let prefix = if indent { "  /// " } else { "/// " };
    let empty_prefix = if indent { "  ///" } else { "///" };

    for line in doc.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            out.write(empty_prefix)?;
        } else {
            out.write(&format!("{}{}", prefix, trimmed))?;
        }
        out.write("\n")?;
    }
    Ok(())
}

fn render_examples(examples: &[Value], indent: bool, out: &mut dyn Output) -> HelperResult {
    let prefix = if indent { "  /// " } else { "/// " };

    out.write(&format!("{}\n", prefix))?;
    out.write(&format!("{}Examples:\n", prefix))?;

    for example in examples {
        if let Some(example_str) = example.as_str() {
            out.write(&format!("{}- `{}`\n", prefix, example_str))?;
        }
    }
    Ok(())
}

fn render_deprecation(message: &str, indent: bool, out: &mut dyn Output) -> HelperResult {
    let prefix = if indent { "  " } else { "" };

    if message.trim().is_empty() {
        out.write(&format!("{}@Deprecated\n", prefix))?;
    } else {
        out.write(&format!("{}@Deprecated {{ message = \"{}\" }}\n", prefix, message.trim()))?;
    }
    Ok(())
}

// =============================================================================
// BODY RENDERING - Type-specific content
// =============================================================================

fn render_body(item: &Value, config: &RenderConfig, ctx: &Context, out: &mut dyn Output) -> HelperResult {
    match config.item_type {
        ItemType::Module => {
            // Module body is handled by template structure
            Ok(())
        },
        ItemType::Class => {
            render_class_body(item, ctx, out)?;
            out.write("\n\n")?;
            Ok(())
        },
        ItemType::Property => {
            render_property_body(item, out)?;
            out.write("\n")?;
            Ok(())
        },
    }
}

fn render_class_body(item: &Value, ctx: &Context, out: &mut dyn Output) -> HelperResult {
    let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
    let kind = item.get("kind").and_then(|v| v.as_str()).unwrap_or("Class");
    let is_open = if kind != "Class" {
      false
    } else {
      item.get("open").and_then(|v| v.as_bool()).unwrap_or(true)
    };

    if kind == "TypeAlias" {
        let enum_values = item.get("enum_values")
            .and_then(|v| v.as_str())
            .unwrap_or("Any");
        out.write(&format!("typealias {} = {}", escape_keyword(name), enum_values))?;
        return Ok(());
    }

    // Regular class
    let mut header = String::new();

    if item.get("abstract_type").and_then(|v| v.as_bool()).unwrap_or(false) {
        header.push_str("abstract ");
    }

    let keywords = if is_open {
      "open class"
    } else {
      "class"
    };

    header.push_str(&format!("{} {}", keywords, escape_keyword(name)));

    if let Some(extends) = item.get("extends").and_then(|v| v.as_array()) {
        if !extends.is_empty() {
            let extends_list: Vec<String> = extends
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
            if !extends_list.is_empty() {
                header.push_str(&format!(" extends {}", extends_list.join(", ")));
            }
        }
    }

    out.write(&format!("{} {{\n", header))?;

    // Render properties
    if let Some(properties) = item.get("properties").and_then(|v| v.as_array()) {
        for property in properties {
            let include_deprecated = ctx.data()
                .get("config")
                .and_then(|c| c.get("include_deprecated"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            let make_open = false;

            let property_config = RenderConfig {
                item_type: ItemType::Property,
                indent: true,
                include_deprecated,
                make_open,
            };

            if should_render_item_with_config(property, &property_config) {
                render_header(property, &property_config, out)?;
                render_property_body(property, out)?;
                out.write("\n")?;
            }
        }
    }

    out.write("}")?;
    Ok(())
}

fn render_property_body(item: &Value, out: &mut dyn Output) -> HelperResult {
    let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
    let type_name = item.get("type_name").and_then(|v| v.as_str()).unwrap_or("Any");
    let optional = item.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);
    let default = item.get("default").and_then(|v| v.as_str());

    let escaped_name = escape_keyword(name);
    let mut declaration = format!("  {}: {}", escaped_name, type_name);

    if optional {
        declaration.push('?');
    }

    if let Some(default_val) = default {
        declaration.push_str(&format!(" = {}", default_val));
    }

    out.write(&declaration)?;
    Ok(())
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn should_render(
    h: &Helper,
    _: &Handlebars,
    ctx: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let item = h.param(0);
    let should_render_flag = item
        .map(|p| should_render_item(p.value(), ctx))
        .unwrap_or(true);

    if should_render_flag {
        out.write("true")?;
    }
    Ok(())
}

fn should_render_item_with_config(item: &Value, config: &RenderConfig) -> bool {
    let is_deprecated = item.get("deprecated").is_some();

    // If the item is deprecated and we're not including deprecated items, skip it
    if is_deprecated && !config.include_deprecated {
        return false;
    }

    true
}

fn should_render_item(item: &Value, ctx: &Context) -> bool {
    let is_deprecated = item.get("deprecated").is_some();
    let include_deprecated = ctx.data()
        .get("config")
        .and_then(|c| c.get("include_deprecated"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // If the item is deprecated and we're not including deprecated items, skip it
    if is_deprecated && !include_deprecated {
        return false;
    }

    true
}

fn imports_section(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(imports_param) = h.param(0) {
        if let Some(imports) = imports_param.value().as_array() {
            for import in imports {
                if let Some(import_obj) = import.as_object() {
                    let path = import_obj.get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let alias = import_obj.get("alias")
                        .and_then(|v| v.as_str());

                    match alias {
                        Some(alias_str) => out.write(&format!("import \"{}\" as {}\n", path, alias_str))?,
                        None => out.write(&format!("import \"{}\"\n", path))?,
                    }
                }
            }
        }
    }
    Ok(())
}

fn module_examples(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(module_param) = h.param(0) {
        let module = module_param.value();
        if let Some(properties) = module.get("properties").and_then(|p| p.as_array()) {
            let examples: Vec<String> = properties
                .iter()
                .filter(|prop| !prop.get("deprecated").is_some())
                .take(3)
                .filter_map(|prop| {
                    let name = prop.get("name")?.as_str()?;
                    let type_name = prop.get("type_name")?.as_str()?;
                    Some(format!("/// {}: {}", escape_keyword(name), type_name))
                })
                .collect();

            if !examples.is_empty() {
                out.write(&examples.join("\n"))?;
            }
        }
    }
    Ok(())
}

fn example_path(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(name_param) = h.param(0) {
        if let Some(name) = name_param.value().as_str() {
            let path = if name.contains("Project") {
                ".../pkl-schemas/Project.pkl"
            } else {
                &format!("../pkl-schemas/{}.pkl", name)
            };
            out.write(path)?;
        }
    }
    Ok(())
}

fn is_typealias(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let is_alias = h.param(0)
        .and_then(|p| p.value().as_str())
        .map(|kind| kind == "TypeAlias")
        .unwrap_or(false);

    if is_alias {
        out.write("true")?;
    }
    Ok(())
}

fn is_pkl_keyword(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let is_keyword = h.param(0)
        .and_then(|p| p.value().as_str())
        .map(|name| pkl_keyword(name))
        .unwrap_or(false);

    if is_keyword {
        out.write("true")?;
    }
    Ok(())
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

fn escape_keyword(name: &str) -> String {
    if pkl_keyword(name) && !name.contains('`') {
        format!("`{}`", name)
    } else {
        name.to_string()
    }
}

fn pkl_keyword(name: &str) -> bool {
    matches!(name,
        "abstract" | "amends" | "as" | "case" | "class" | "const" |
        "default" | "delete" | "else" | "extends" | "external" | "false" |
        "fixed" | "for" | "function" | "hidden" | "if" | "import" |
        "import*" | "in" | "is" | "let" | "local" | "module" | "new" |
        "nothing" | "null" | "open" | "out" | "outer" | "override" |
        "overrides" | "protected" | "read" | "read*" | "record" |
        "super" | "switch" | "this" | "throw" | "trace" | "true" |
        "typealias" | "unknown" | "vararg" | "when"
    )
}
