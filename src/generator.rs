//! `generator.rs`
//! Core schema generation functionality.
//!
//! (c) 2025 Stash AI Inc (knitli)
//!   - Created by Adam Poulemanos ([@bashandbone](https://github.com/bashandbone))
//! Licensed under the [Plain MIT License](https://plainlicense.org/licenses/permissive/mit/)

use crate::config::{GeneratorConfig, SchemaType as ConfigSchemaType};
use crate::templates::TemplateEngine;
use crate::types::*;
use crate::Result;
use miette::{IntoDiagnostic, WrapErr};
use moon_config::*;
use schematic::schema::SchemaGenerator as SchematicGenerator;
use schematic::Config;
use schematic_types::{Schema, SchemaField, SchemaType};
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// Main schema generator for Moon configurations
pub struct SchemaGenerator {
    config: GeneratorConfig,
    template_engine: TemplateEngine,
}

impl SchemaGenerator {
    /// Create a new schema generator with the given configuration
    pub fn new(config: GeneratorConfig) -> Self {
        let template_engine = TemplateEngine::new(&config);
        Self {
            config,
            template_engine,
        }
    }

    /// Generate all Moon configuration schemas
    pub fn generate_all(&self) -> Result<()> {
        info!("Generating all Moon configuration schemas");

        fs::create_dir_all(&self.config.output_dir)
            .into_diagnostic()
            .wrap_err("Failed to create output directory")?;

        // Generate individual schemas
        self.generate_workspace_schema_file()?;
        self.generate_project_schema_file()?;
        self.generate_template_schema_file()?;
        self.generate_toolchain_schema_file()?;
        self.generate_tasks_schema_file()?;

        // Generate module index
        if self.config.split_types {
            self.generate_module_index()?;
        }

        info!(
            "Successfully generated all schemas in: {}",
            self.config.output_dir.display()
        );
        Ok(())
    }

    /// Generate workspace configuration schema
    pub fn generate_workspace_schema(&self) -> Result<String> {
        debug!("Generating workspace schema");
        self.generate_schema_for_type::<WorkspaceConfig>("Workspace")
    }

    /// Generate project configuration schema
    pub fn generate_project_schema(&self) -> Result<String> {
        debug!("Generating project schema");
        self.generate_schema_for_type::<ProjectConfig>("Project")
    }

    /// Generate template configuration schema
    pub fn generate_template_schema(&self) -> Result<String> {
        debug!("Generating template schema");
        self.generate_schema_for_type::<TemplateConfig>("Template")
    }

    /// Generate toolchain configuration schema
    pub fn generate_toolchain_schema(&self) -> Result<String> {
        debug!("Generating toolchain schema");
        self.generate_schema_for_type::<ToolchainConfig>("Toolchain")
    }

    /// Generate tasks configuration schema
    pub fn generate_tasks_schema(&self) -> Result<String> {
        debug!("Generating tasks schema");
        self.generate_schema_for_type::<InheritedTasksConfig>("Tasks")
    }

    /// Generate schema for a specific type
    fn generate_schema_for_type<T: Config>(&self, type_name: &str) -> Result<String> {
        let mut generator = SchematicGenerator::default();
        generator.add::<T>();

        // Get the schemas from the generator
        let schema_map = generator.schemas;

        // Convert schematic schema to our Pkl representation
        let pkl_module = self.convert_schemas_to_pkl(schema_map, type_name)?;

        // Render using template engine
        self.template_engine
            .render_module(&pkl_module, &self.config)
    }

    /// Write schema to file
    fn generate_workspace_schema_file(&self) -> Result<()> {
        let schema = self.generate_workspace_schema()?;
        let file_path = self
            .config
            .output_dir
            .join(ConfigSchemaType::Workspace.filename());
        self.write_schema_file(&file_path, &schema, "workspace")
    }

    fn generate_project_schema_file(&self) -> Result<()> {
        let schema = self.generate_project_schema()?;
        let file_path = self
            .config
            .output_dir
            .join(ConfigSchemaType::Project.filename());
        self.write_schema_file(&file_path, &schema, "project")
    }

    fn generate_template_schema_file(&self) -> Result<()> {
        let schema = self.generate_template_schema()?;
        let file_path = self
            .config
            .output_dir
            .join(ConfigSchemaType::Template.filename());
        self.write_schema_file(&file_path, &schema, "template")
    }

    fn generate_toolchain_schema_file(&self) -> Result<()> {
        let schema = self.generate_toolchain_schema()?;
        let file_path = self
            .config
            .output_dir
            .join(ConfigSchemaType::Toolchain.filename());
        self.write_schema_file(&file_path, &schema, "toolchain")
    }

    fn generate_tasks_schema_file(&self) -> Result<()> {
        let schema = self.generate_tasks_schema()?;
        let file_path = self
            .config
            .output_dir
            .join(ConfigSchemaType::Tasks.filename());
        self.write_schema_file(&file_path, &schema, "tasks")
    }

    fn write_schema_file(&self, path: &Path, content: &str, schema_name: &str) -> Result<()> {
        fs::write(path, content)
            .into_diagnostic()
            .wrap_err_with(|| {
                format!(
                    "Failed to write {} schema to {}",
                    schema_name,
                    path.display()
                )
            })?;

        info!("Generated {} schema: {}", schema_name, path.display());
        Ok(())
    }

    /// Generate a module index file that imports all schemas
    fn generate_module_index(&self) -> Result<()> {
        let index_content = self.template_engine.render_module_index(&self.config)?;
        let index_path = self.config.output_dir.join("mod.pkl");

        fs::write(&index_path, index_content)
            .into_diagnostic()
            .wrap_err("Failed to write module index")?;

        info!("Generated module index: {}", index_path.display());
        Ok(())
    }

    /// Convert schematic schemas to Pkl module representation
    fn convert_schemas_to_pkl(
        &self,
        schemas: indexmap::IndexMap<String, Schema>,
        type_name: &str,
    ) -> Result<PklModule> {
        let mut module = PklModule {
            name: type_name.to_string(),
            documentation: Some(format!(
                "Moon {} configuration schema",
                type_name.to_lowercase()
            )),
            imports: vec![],
            exports: vec![],
            types: vec![],
            properties: vec![],
        };

        // Convert each schema to a Pkl type
        for (name, schema) in schemas {
            let pkl_type = self.convert_schema_to_pkl_type(&schema, &name)?;
            module.types.push(pkl_type);

            // Add export for the main type
            if name == type_name || name.ends_with("Config") {
                module.exports.push(PklExport {
                    name: name.clone(),
                    type_name: name.clone(),
                });
            }
        }

        Ok(module)
    }

    /// Convert a single schema to a Pkl type
    fn convert_schema_to_pkl_type(&self, schema: &Schema, name: &str) -> Result<PklType> {
        debug!("Converting schema '{}' of type: {:?}", name, schema.ty);
        let mut pkl_type = PklType {
            name: name.to_string(),
            documentation: schema.description.clone(),
            kind: PklTypeKind::Class,
            properties: vec![],
            abstract_type: false,
            extends: vec![],
            enum_values: None,
        };

        match &schema.ty {
            SchemaType::Struct(struct_type) => {
                for (field_name, field) in &struct_type.fields {
                    let property = self.convert_field_to_property(field_name, field)?;
                    pkl_type.properties.push(property);
                }
                debug!(
                    "Created struct class '{}' with {} properties",
                    name,
                    pkl_type.properties.len()
                );
            }
            SchemaType::Enum(enum_type) => {
                // For string enums, create a typealias with union of string literals
                if !enum_type.values.is_empty() {
                    pkl_type.kind = PklTypeKind::TypeAlias;
                    let enum_values: Vec<String> = enum_type
                        .values
                        .iter()
                        .map(|v| match v {
                            schematic_types::LiteralValue::String(s) => format!("\"{}\"", s),
                            schematic_types::LiteralValue::Int(i) => i.to_string(),
                            schematic_types::LiteralValue::Bool(b) => b.to_string(),
                            _ => format!("{:?}", v), // Fallback for other variants
                        })
                        .collect();

                    pkl_type.enum_values = Some(enum_values.join(" | "));
                    debug!(
                        "Created enum typealias '{}' with values: {}",
                        name,
                        pkl_type.enum_values.as_ref().unwrap()
                    );
                } else {
                    // Empty enum, keep as class but mark it
                    pkl_type.documentation = Some(format!(
                        "{}{}This is an empty enum type.",
                        pkl_type.documentation.as_deref().unwrap_or(""),
                        if pkl_type.documentation.is_some() {
                            "\n\n"
                        } else {
                            ""
                        }
                    ));
                    debug!("Created empty enum class '{}'", name);
                }
            }
            SchemaType::Union(union_type) => {
                // Handle union types - create type alias for all unions
                pkl_type.kind = PklTypeKind::TypeAlias;
                let variant_types: Result<Vec<String>> = union_type
                    .variants_types
                    .iter()
                    .map(|v| self.get_pkl_type_name(v))
                    .collect();

                match variant_types {
                    Ok(types) => {
                        let union_str = types.join(" | ");
                        debug!(
                            "Created union typealias '{}' with types: {}",
                            name, union_str
                        );
                        pkl_type.enum_values = Some(union_str);
                        debug!(
                            "Union type for {}: {}",
                            name,
                            pkl_type.enum_values.as_ref().unwrap()
                        );
                    }
                    Err(e) => {
                        warn!("Failed to resolve union types for {}: {}", name, e);
                        pkl_type.enum_values = Some("Any".to_string());
                        debug!("Failed to resolve union '{}', using Any", name);
                    }
                }
            }
            SchemaType::Reference(ref_name) => {
                // For reference types, we should create a proper class if it's not already defined
                // For now, we'll handle this as a struct-like type
                debug!("Reference type for {}: {}", name, ref_name);
                debug!("Created reference class '{}'", name);
                // Keep as class with no properties - the actual properties should come from the referenced schema
            }
            _ => {
                // Handle other schema types as needed
                debug!("Unhandled schema type for {}: {:?}", name, schema.ty);
                debug!("Created fallback class '{}' for unhandled type", name);
                // Keep as empty class for now - this preserves strong typing
            }
        }

        debug!(
            "Final PklType for '{}': kind={:?}, properties={}, enum_values={:?}",
            name,
            pkl_type.kind,
            pkl_type.properties.len(),
            pkl_type.enum_values
        );
        Ok(pkl_type)
    }

    /// Convert a struct field to a Pkl property
    fn convert_field_to_property(&self, name: &str, field: &SchemaField) -> Result<PklProperty> {
        let type_name = self.get_pkl_type_name(&field.schema)?;
        let default = self.extract_default_value(&field.schema)?;
        let constraints = self.extract_constraints(&field.schema)?;
        let examples = self.extract_examples(&field.schema)?;

        Ok(PklProperty {
            name: name.to_string(),
            type_name,
            documentation: field.schema.description.clone(),
            optional: field.optional,
            default,
            constraints,
            examples,
        })
    }

    /// Extract default value from schema if available
    fn extract_default_value(&self, schema: &Schema) -> Result<Option<String>> {
        let default_value = match &schema.ty {
            SchemaType::String(string_type) => {
                if let Some(enum_values) = &string_type.enum_values {
                    if !enum_values.is_empty() {
                        Some(format!("\"{}\"", enum_values[0]))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            SchemaType::Boolean(_) => Some("false".to_string()),
            SchemaType::Integer(int_type) => {
                if let Some(enum_values) = &int_type.enum_values {
                    if !enum_values.is_empty() {
                        Some(enum_values[0].to_string())
                    } else {
                        None
                    }
                } else if let Some(min) = int_type.min {
                    Some(min.to_string())
                } else {
                    None
                }
            }
            SchemaType::Float(float_type) => {
                if let Some(enum_values) = &float_type.enum_values {
                    if !enum_values.is_empty() {
                        Some(enum_values[0].to_string())
                    } else {
                        None
                    }
                } else if let Some(min) = float_type.min {
                    Some(min.to_string())
                } else {
                    None
                }
            }
            SchemaType::Array(_) => Some("new Listing {}".to_string()),
            SchemaType::Object(_) => Some("new Mapping {}".to_string()),
            _ => None,
        };

        Ok(default_value)
    }

    /// Extract validation constraints from schema
    fn extract_constraints(&self, schema: &Schema) -> Result<Vec<PklConstraint>> {
        let mut constraints = Vec::new();

        match &schema.ty {
            SchemaType::String(string_type) => {
                if let Some(min_length) = string_type.min_length {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Length,
                        value: format!("length >= {}", min_length),
                        message: Some(format!("Must be at least {} characters long", min_length)),
                    });
                }

                if let Some(max_length) = string_type.max_length {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Length,
                        value: format!("length <= {}", max_length),
                        message: Some(format!("Must be at most {} characters long", max_length)),
                    });
                }

                if let Some(pattern) = &string_type.pattern {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Pattern,
                        value: format!("matches(Regex(#\"{}\"#))", pattern),
                        message: Some(format!("Must match pattern: {}", pattern)),
                    });
                }

                if let Some(enum_values) = &string_type.enum_values {
                    if enum_values.len() > 1 {
                        let values = enum_values
                            .iter()
                            .map(|v| format!("\"{}\"", v))
                            .collect::<Vec<_>>()
                            .join("|");
                        constraints.push(PklConstraint {
                            kind: PklConstraintKind::Custom,
                            value: format!("oneOf({})", values),
                            message: Some(format!("Must be one of: {}", enum_values.join(", "))),
                        });
                    }
                }
            }

            SchemaType::Integer(int_type) => {
                if let Some(min) = int_type.min {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Min,
                        value: format!("this >= {}", min),
                        message: Some(format!("Must be at least {}", min)),
                    });
                }

                if let Some(max) = int_type.max {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Max,
                        value: format!("this <= {}", max),
                        message: Some(format!("Must be at most {}", max)),
                    });
                }

                if let Some(multiple_of) = int_type.multiple_of {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Custom,
                        value: format!("this % {} == 0", multiple_of),
                        message: Some(format!("Must be a multiple of {}", multiple_of)),
                    });
                }

                if let Some(enum_values) = &int_type.enum_values {
                    if enum_values.len() > 1 {
                        let values = enum_values
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join("|");
                        constraints.push(PklConstraint {
                            kind: PklConstraintKind::Custom,
                            value: format!("oneOf({})", values),
                            message: Some(format!(
                                "Must be one of: {}",
                                enum_values
                                    .iter()
                                    .map(|v| v.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )),
                        });
                    }
                }
            }

            SchemaType::Float(float_type) => {
                if let Some(min) = float_type.min {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Min,
                        value: format!("this >= {}", min),
                        message: Some(format!("Must be at least {}", min)),
                    });
                }

                if let Some(max) = float_type.max {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Max,
                        value: format!("this <= {}", max),
                        message: Some(format!("Must be at most {}", max)),
                    });
                }
            }

            SchemaType::Array(array_type) => {
                if let Some(min_length) = array_type.min_length {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Length,
                        value: format!("length >= {}", min_length),
                        message: Some(format!("Must contain at least {} items", min_length)),
                    });
                }

                if let Some(max_length) = array_type.max_length {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Length,
                        value: format!("length <= {}", max_length),
                        message: Some(format!("Must contain at most {} items", max_length)),
                    });
                }

                if array_type.unique == Some(true) {
                    constraints.push(PklConstraint {
                        kind: PklConstraintKind::Custom,
                        value: "isDistinct".to_string(),
                        message: Some("All items must be unique".to_string()),
                    });
                }
            }

            _ => {}
        }

        Ok(constraints)
    }

    /// Extract example values from schema
    fn extract_examples(&self, schema: &Schema) -> Result<Vec<String>> {
        let mut examples = Vec::new();

        match &schema.ty {
            SchemaType::String(string_type) => {
                if let Some(enum_values) = &string_type.enum_values {
                    examples.extend(enum_values.iter().take(3).map(|v| format!("\"{}\"", v)));
                } else if let Some(format) = &string_type.format {
                    match format.as_str() {
                        "url" => examples.push("\"https://example.com\"".to_string()),
                        "email" => examples.push("\"user@example.com\"".to_string()),
                        "uri" => examples.push("\"https://api.example.com/v1\"".to_string()),
                        "uuid" => {
                            examples.push("\"550e8400-e29b-41d4-a716-446655440000\"".to_string())
                        }
                        "date" => examples.push("\"2023-12-25\"".to_string()),
                        "time" => examples.push("\"14:30:00\"".to_string()),
                        "datetime" => examples.push("\"2023-12-25T14:30:00Z\"".to_string()),
                        _ => examples.push(format!("\"example-{}\"", format)),
                    }
                } else if string_type.pattern.is_some() {
                    examples.push("\"example\"".to_string());
                } else {
                    examples.push("\"example\"".to_string());
                }
            }

            SchemaType::Integer(int_type) => {
                if let Some(enum_values) = &int_type.enum_values {
                    examples.extend(enum_values.iter().take(3).map(|v| v.to_string()));
                } else {
                    let example_value = if let Some(min) = int_type.min {
                        min
                    } else if let Some(max) = int_type.max {
                        std::cmp::max(0, max)
                    } else {
                        42
                    };
                    examples.push(example_value.to_string());
                }
            }

            SchemaType::Float(float_type) => {
                if let Some(enum_values) = &float_type.enum_values {
                    examples.extend(enum_values.iter().take(3).map(|v| v.to_string()));
                } else {
                    let example_value = if let Some(min) = float_type.min {
                        min
                    } else if let Some(max) = float_type.max {
                        f64::max(0.0, max)
                    } else {
                        3.14
                    };
                    examples.push(example_value.to_string());
                }
            }

            SchemaType::Boolean(_) => {
                examples.push("true".to_string());
                examples.push("false".to_string());
            }

            SchemaType::Array(array_type) => {
                let item_type = self.get_pkl_type_name(&array_type.items_type)?;
                examples.push(format!("new Listing<{}> {{}}", item_type));

                match &array_type.items_type.ty {
                    SchemaType::String(_) => {
                        examples.push("new Listing { \"item1\"; \"item2\" }".to_string())
                    }
                    SchemaType::Integer(_) => examples.push("new Listing { 1; 2; 3 }".to_string()),
                    _ => {}
                }
            }

            SchemaType::Object(object_type) => {
                let key_type = self.get_pkl_type_name(&object_type.key_type)?;
                let value_type = self.get_pkl_type_name(&object_type.value_type)?;
                examples.push(format!("new Mapping<{}, {}> {{}}", key_type, value_type));
            }

            SchemaType::Enum(enum_type) => {
                // For enum types, use the debug representation and clean it up for Pkl
                examples.extend(enum_type.values.iter().take(3).map(|v| {
                    let debug_str = format!("{:?}", v);
                    // Clean up the debug representation for Pkl
                    if debug_str.starts_with("String(") && debug_str.ends_with(")") {
                        // Extract string value: String("value") -> "value"
                        debug_str[7..debug_str.len() - 1].to_string()
                    } else if debug_str.starts_with("Int(") && debug_str.ends_with(")") {
                        // Extract int value: Int(42) -> 42
                        debug_str[4..debug_str.len() - 1].to_string()
                    } else if debug_str.starts_with("Bool(") && debug_str.ends_with(")") {
                        // Extract bool value: Bool(true) -> true
                        debug_str[5..debug_str.len() - 1].to_string()
                    } else if debug_str == "Null" {
                        "null".to_string()
                    } else {
                        // Default: wrap in quotes for safety
                        format!("\"{}\"", debug_str.replace("\"", "\\\""))
                    }
                }));
            }

            _ => {}
        }

        Ok(examples)
    }

    /// Get the Pkl type name for a schema
    fn get_pkl_type_name(&self, schema: &Schema) -> Result<String> {
        let type_name = match &schema.ty {
            SchemaType::String(_) => "String".to_string(),
            SchemaType::Boolean(_) => "Boolean".to_string(),
            SchemaType::Integer(_) => "Int".to_string(),
            SchemaType::Float(_) => "Float".to_string(),
            SchemaType::Array(array_type) => {
                let item_type = self.get_pkl_type_name(&array_type.items_type)?;
                format!("Listing<{}>", item_type)
            }
            SchemaType::Object(object_type) => {
                let key_type = self.get_pkl_type_name(&object_type.key_type)?;
                let value_type = self.get_pkl_type_name(&object_type.value_type)?;
                format!("Mapping<{}, {}>", key_type, value_type)
            }
            SchemaType::Reference(ref_name) => ref_name.clone(),
            SchemaType::Struct(_) => {
                // For struct types, use the schema name if available, otherwise "Any"
                schema.name.clone().unwrap_or_else(|| "Any".to_string())
            }
            SchemaType::Enum(_) => {
                // For enum types, use the schema name if available, otherwise "Any"
                schema.name.clone().unwrap_or_else(|| "Any".to_string())
            }
            SchemaType::Union(union_type) => {
                // Handle union types properly, especially nullable patterns
                let variant_types: Result<Vec<String>> = union_type
                    .variants_types
                    .iter()
                    .map(|v| self.get_pkl_type_name(v))
                    .collect();

                match variant_types {
                    Ok(types) => {
                        // Check for nullable pattern (Type | Null)
                        let null_index = types.iter().position(|t| t == "Null");
                        let non_null_types: Vec<&String> =
                            types.iter().filter(|t| *t != "Null").collect();

                        if let Some(_) = null_index {
                            // This is a nullable union
                            if non_null_types.len() == 1 {
                                // Simple nullable: T | Null -> T?
                                format!("{}?", non_null_types[0])
                            } else if non_null_types.len() > 1 {
                                // Complex nullable: (T1 | T2) | Null -> (T1 | T2)?
                                format!(
                                    "({})?",
                                    non_null_types
                                        .iter()
                                        .map(|s| s.as_str())
                                        .collect::<Vec<_>>()
                                        .join(" | ")
                                )
                            } else {
                                // Only Null, shouldn't happen but handle gracefully
                                "Null".to_string()
                            }
                        } else {
                            // Non-nullable union: T1 | T2
                            types.join(" | ")
                        }
                    }
                    Err(_) => {
                        // Fallback to Any if we can't resolve the union types
                        "Any".to_string()
                    }
                }
            }
            SchemaType::Null => "Null".to_string(),
            SchemaType::Unknown => "Any".to_string(),
            _ => "Any".to_string(),
        };

        Ok(self
            .config
            .type_mappings
            .get(&type_name)
            .cloned()
            .unwrap_or(type_name))
    }
}

/// Convenience functions for generating specific schemas
pub fn generate_workspace_schema() -> Result<String> {
    SchemaGenerator::new(GeneratorConfig::default()).generate_workspace_schema()
}

pub fn generate_project_schema() -> Result<String> {
    SchemaGenerator::new(GeneratorConfig::default()).generate_project_schema()
}

pub fn generate_template_schema() -> Result<String> {
    SchemaGenerator::new(GeneratorConfig::default()).generate_template_schema()
}

pub fn generate_toolchain_schema() -> Result<String> {
    SchemaGenerator::new(GeneratorConfig::default()).generate_toolchain_schema()
}

pub fn generate_tasks_schema() -> Result<String> {
    SchemaGenerator::new(GeneratorConfig::default()).generate_tasks_schema()
}
