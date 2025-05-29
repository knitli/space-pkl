//! Core schema generation functionality.

use crate::config::{GeneratorConfig, SchemaType};
use crate::templates::TemplateEngine;
use crate::types::*;
use crate::Result;
use miette::{IntoDiagnostic, WrapErr};
use moon_config::*;
use schematic::{Config, SchemaGenerator as SchematicGenerator};
use std::fs;
use std::path::Path;
use tracing::{info, debug};

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

        info!("Successfully generated all schemas in: {}", self.config.output_dir.display());
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
        let schemas = generator.generate()
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to generate schema for {}", type_name))?;

        // Convert schematic schema to our Pkl representation
        let pkl_module = self.convert_schemas_to_pkl(schemas, type_name)?;
        
        // Render using template engine
        self.template_engine.render_module(&pkl_module, &self.config)
    }

    /// Write schema to file
    fn generate_workspace_schema_file(&self) -> Result<()> {
        let schema = self.generate_workspace_schema()?;
        let file_path = self.config.output_dir.join(SchemaType::Workspace.filename());
        self.write_schema_file(&file_path, &schema, "workspace")
    }

    fn generate_project_schema_file(&self) -> Result<()> {
        let schema = self.generate_project_schema()?;
        let file_path = self.config.output_dir.join(SchemaType::Project.filename());
        self.write_schema_file(&file_path, &schema, "project")
    }

    fn generate_template_schema_file(&self) -> Result<()> {
        let schema = self.generate_template_schema()?;
        let file_path = self.config.output_dir.join(SchemaType::Template.filename());
        self.write_schema_file(&file_path, &schema, "template")
    }

    fn generate_toolchain_schema_file(&self) -> Result<()> {
        let schema = self.generate_toolchain_schema()?;
        let file_path = self.config.output_dir.join(SchemaType::Toolchain.filename());
        self.write_schema_file(&file_path, &schema, "toolchain")
    }

    fn generate_tasks_schema_file(&self) -> Result<()> {
        let schema = self.generate_tasks_schema()?;
        let file_path = self.config.output_dir.join(SchemaType::Tasks.filename());
        self.write_schema_file(&file_path, &schema, "tasks")
    }

    fn write_schema_file(&self, path: &Path, content: &str, schema_name: &str) -> Result<()> {
        fs::write(path, content)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to write {} schema to {}", schema_name, path.display()))?;
        
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
        schemas: indexmap::IndexMap<String, schematic_types::Schema>,
        type_name: &str,
    ) -> Result<PklModule> {
        let mut module = PklModule {
            name: type_name.to_string(),
            documentation: Some(format!("Moon {} configuration schema", type_name.to_lowercase())),
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
    fn convert_schema_to_pkl_type(
        &self,
        schema: &schematic_types::Schema,
        name: &str,
    ) -> Result<PklType> {
        use schematic_types::SchemaType;

        let mut pkl_type = PklType {
            name: name.to_string(),
            documentation: schema.description.clone(),
            kind: PklTypeKind::Class,
            properties: vec![],
            abstract_type: false,
            extends: vec![],
        };

        match &schema.ty {
            SchemaType::Struct(struct_type) => {
                for (field_name, field) in &struct_type.fields {
                    let property = self.convert_field_to_property(field_name, field)?;
                    pkl_type.properties.push(property);
                }
            }
            SchemaType::Enum(enum_type) => {
                pkl_type.kind = PklTypeKind::Union;
                // Handle enum variants
                for value in &enum_type.values {
                    // Convert enum values to properties or constraints
                    // This is simplified - you'd want more sophisticated enum handling
                }
            }
            _ => {
                // Handle other schema types as needed
                debug!("Unhandled schema type for {}: {:?}", name, schema.ty);
            }
        }

        Ok(pkl_type)
    }

    /// Convert a struct field to a Pkl property
    fn convert_field_to_property(
        &self,
        name: &str,
        field: &schematic_types::SchemaField,
    ) -> Result<PklProperty> {
        let type_name = self.get_pkl_type_name(&field.schema)?;
        
        Ok(PklProperty {
            name: name.to_string(),
            type_name,
            documentation: field.schema.description.clone(),
            optional: field.optional,
            default: field.default.as_ref().map(|v| format!("{:?}", v)),
            constraints: vec![], // TODO: Convert validation constraints
            examples: vec![], // TODO: Extract examples if available
        })
    }

    /// Get the Pkl type name for a schema
    fn get_pkl_type_name(&self, schema: &schematic_types::Schema) -> Result<String> {
        use schematic_types::SchemaType;
        
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
            SchemaType::Union(_) => "Any".to_string(), // Simplified
            SchemaType::Null => "Null".to_string(),
            SchemaType::Unknown => "Any".to_string(),
            _ => "Any".to_string(),
        };

        // Apply custom type mappings
        Ok(self.config.type_mappings.get(&type_name).cloned().unwrap_or(type_name))
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
