//! `templates` module
//! Template engine for generating Pkl schemas.
//!
//! (c) 2025 Stash AI Inc (knitli)
//!   - Created by Adam Poulemanos ([@bashandbone](https://github.com/bashandbone))
//! Licensed under the [Plain MIT License](https://plainlicense.org/licenses/permissive/mit/)

use crate::config::GeneratorConfig;
use crate::types::*;
use crate::Result;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use miette::{IntoDiagnostic, WrapErr};
use serde_json::json;
use std::collections::HashMap;
use tracing::debug;

/// Template engine for rendering Pkl schemas
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new(config: &GeneratorConfig) -> Self {
        let mut handlebars = Handlebars::new();

        // Disable HTML escaping to prevent &lt; &gt; &quot; in output
        handlebars.register_escape_fn(handlebars::no_escape);

        // Register built-in templates
        Self::register_builtin_templates(&mut handlebars);

        // Register helper functions
        Self::register_helpers(&mut handlebars);

        // Load custom templates if specified
        if let Some(template_dir) = &config.template.template_dir {
            if template_dir.exists() {
                // Register templates from directory manually since register_templates_directory
                // may not be available in this version
                if let Ok(entries) = std::fs::read_dir(template_dir) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                            if entry.path().extension().and_then(|s| s.to_str())
                                == Some(&config.template.template_extension.trim_start_matches('.'))
                            {
                                if let Ok(template_content) = std::fs::read_to_string(entry.path())
                                {
                                    let _ =
                                        handlebars.register_template_string(name, template_content);
                                }
                            }
                        }
                    }
                }
            }
        }

        Self { handlebars }
    }

    /// Render a Pkl module
    pub fn render_module(&self, module: &PklModule, config: &GeneratorConfig) -> Result<String> {
        // Debug logging to see what types we're rendering
        for pkl_type in &module.types {
            debug!(
                "Rendering type '{}': kind={:?}, properties={}, enum_values={:?}",
                pkl_type.name,
                pkl_type.kind,
                pkl_type.properties.len(),
                pkl_type.enum_values
            );

            // Serialize the type to see how it looks in JSON
            if let Ok(serialized) = serde_json::to_string(pkl_type) {
                debug!("Serialized PklType '{}': {}", pkl_type.name, serialized);
            }

            // Test individual template rendering
            if let Ok(rendered) = self.handlebars.render("class", pkl_type) {
                debug!("Rendered class '{}': {}", pkl_type.name, rendered);
            }
        }

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

    /// Render a module index file
    pub fn render_module_index(&self, config: &GeneratorConfig) -> Result<String> {
        let context = json!({
            "module_name": config.module_name,
            "config": config,
            "schemas": [
                {"name": "Workspace", "file": "workspace.pkl"},
                {"name": "Project", "file": "project.pkl"},
                {"name": "Template", "file": "template.pkl"},
                {"name": "Toolchain", "file": "toolchain.pkl"},
                {"name": "Tasks", "file": "tasks.pkl"},
            ]
        });

        self.handlebars
            .render("index", &context)
            .into_diagnostic()
            .wrap_err("Failed to render index template")
    }

    /// Register built-in templates
    fn register_builtin_templates(handlebars: &mut Handlebars) {
        // Main module template
        handlebars
            .register_template_string("module", MODULE_TEMPLATE)
            .expect("Failed to register module template");

        // Index template
        handlebars
            .register_template_string("index", INDEX_TEMPLATE)
            .expect("Failed to register index template");

        // Type templates
        handlebars
            .register_template_string("class", CLASS_TEMPLATE)
            .expect("Failed to register class template");

        handlebars
            .register_template_string("property", PROPERTY_TEMPLATE)
            .expect("Failed to register property template");
    }

    /// Register helper functions
    fn register_helpers(handlebars: &mut Handlebars) {
        // Helper for capitalizing strings
        handlebars.register_helper("capitalize", Box::new(capitalize_helper));

        // Helper for converting to snake_case
        handlebars.register_helper("snake_case", Box::new(snake_case_helper));

        // Helper for converting to camelCase
        handlebars.register_helper("camel_case", Box::new(camel_case_helper));

        // Helper for rendering documentation
        handlebars.register_helper("doc", Box::new(doc_helper));

        // Helper for rendering optional types
        handlebars.register_helper("optional", Box::new(optional_helper));

        // Helper for type alias check
        handlebars.register_helper("is_typealias", Box::new(is_typealias_helper));
    }
}

// Template constants
const MODULE_TEMPLATE: &str = r#"{{#if config.header}}{{config.header}}{{/if}}

{{~#if module.documentation}}
/// {{module.documentation}}
{{/if}}
{{~#if config.include_examples}}
///
/// ## Example
///
/// ```pkl
/// import "{{module.name}}.pkl"
///
/// config: {{module.name}} = new {
/// // Add your configuration here
/// }
/// ```
{{/if}}
module {{module.name}}

{{#each module.imports}}
import "{{path}}"{{#if alias}} as {{alias}}{{/if}}
{{/each}}

{{#each module.types}}
{{> class this}}

{{/each}}

{{#if config.footer}}{{config.footer}}{{/if}}"#;

const INDEX_TEMPLATE: &str = r#"/// Moon Configuration Schemas Module Index
///
/// This module provides access to all Moon configuration schemas.

module {{module_name}}

{{#each schemas}}
import "{{file}}"
{{/each}}

{{#each schemas}}
/// {{name}} configuration schema
typealias {{name}} = {{file}}.{{name}}
{{/each}}"#;

const CLASS_TEMPLATE: &str = r#"{{#if documentation}}
{{doc documentation}}{{/if}}
{{~#if (is_typealias kind)}}typealias {{name}} = {{#if enum_values}}{{enum_values}}
{{else}}Any{{/if}}{{else}}{{#if abstract_type}}abstract {{/if}}class {{name}}{{#if extends}} extends {{#each extends}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}{{/if}} {
{{#each properties}}
{{> property this}}
{{/each}}
}
{{/if}}"#;

const PROPERTY_TEMPLATE: &str = r#"{{#if documentation}}
{{doc documentation}}
{{/if}}
{{#if examples}}
  ///
  /// Examples:
{{#each examples}}
  /// - `{{this}}`
{{/each}}
{{/if}}
  {{#if optional}}{{name}}: ({{{type_name}}})?{{else}}{{name}}: {{{type_name}}}{{/if}}{{#if default}} = {{default}}
  {{/if}}
"#;

// Helper functions
fn capitalize_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(value) = param.value().as_str() {
            let capitalized = value
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if i == 0 {
                        c.to_uppercase().collect::<String>()
                    } else {
                        c.to_string()
                    }
                })
                .collect::<String>();
            out.write(&capitalized)?;
        }
    }
    Ok(())
}

fn snake_case_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(value) = param.value().as_str() {
            let snake_case = value
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if i > 0 && c.is_uppercase() {
                        format!("_{}", c.to_lowercase())
                    } else {
                        c.to_lowercase().collect()
                    }
                })
                .collect::<String>();
            out.write(&snake_case)?;
        }
    }
    Ok(())
}

fn camel_case_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(value) = param.value().as_str() {
            let camel_case = value
                .split('_')
                .enumerate()
                .map(|(i, word)| {
                    if i == 0 {
                        word.to_lowercase()
                    } else {
                        word.chars()
                            .enumerate()
                            .map(|(j, c)| {
                                if j == 0 {
                                    c.to_uppercase().collect()
                                } else {
                                    c.to_string()
                                }
                            })
                            .collect()
                    }
                })
                .collect::<String>();
            out.write(&camel_case)?;
        }
    }
    Ok(())
}

fn doc_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(value) = param.value().as_str() {
            for line in value.lines() {
                out.write(&format!("/// {}\n", line.trim()))?;
            }
        }
    }
    Ok(())
}

fn optional_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(type_name) = param.value().as_str() {
            out.write(&format!("({})?", type_name))?;
        }
    }
    Ok(())
}

fn is_typealias_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    // Get the 'kind' value from the current context
    let is_typealias = if let Some(kind_param) = h.param(0) {
        let kind_value = match kind_param.value() {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string().trim_matches('"').to_string(),
        };

        tracing::debug!("is_typealias_helper: checking kind = '{}'", kind_value);

        kind_value == "TypeAlias"
    } else {
        false
    };

    tracing::debug!("is_typealias_helper: result = {}", is_typealias);

    // Return empty string for false (falsy), non-empty for true (truthy)
    if is_typealias {
        out.write("true")?;
    } else {
        out.write("")?;
    }

    Ok(())
}
