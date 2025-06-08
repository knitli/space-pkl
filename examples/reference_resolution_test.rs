//! Test demonstrating doc comment reference resolution functionality

use std::collections::HashMap;
use indexmap::IndexMap;
use schematic_types::{Schema, SchemaType, StructType, SchemaField, EnumType, LiteralValue};

use crate::new_renderer::{PklSchemaRenderer, PklSchemaOptions};
use crate::types::TypeMap;

/// Example demonstrating reference resolution
fn test_reference_resolution() {
    let mut schemas = TypeMap::new();

    // Create a Count enum schema
    let count_enum = Schema {
        name: Some("Count".to_string()),
        description: Some("An enum representing count values".to_string()),
        ty: SchemaType::Enum(Box::new(EnumType {
            name: "Count".to_string(),
            values: vec![
                LiteralValue::String("one".to_string()),
                LiteralValue::String("two".to_string()),
            ],
            variants: None,
            default: None,
        })),
        nullable: false,
        deprecated: None,
        optional: false,
    };

    // Create a Config struct that references Count
    let config_struct = Schema {
        name: Some("Config".to_string()),
        description: Some("Configuration with [`Count`] enum and [`Self::count`] field".to_string()),
        ty: SchemaType::Struct(Box::new(StructType {
            fields: {
                let mut fields = IndexMap::new();
                fields.insert("count".to_string(), Box::new(SchemaField {
                    schema: Schema {
                        name: None,
                        description: Some("The count value, see [`Count::Two`] for details".to_string()),
                        ty: SchemaType::Reference("Count".to_string()),
                        nullable: false,
                        deprecated: None,
                        optional: false,
                    },
                    comment: None,
                    default: None,
                    optional: false,
                    hidden: false,
                    deprecated: None,
                }));
                fields
            },
            partial: false,
        })),
        nullable: false,
        deprecated: None,
        optional: false,
    };

    schemas.insert("Count".to_string(), count_enum);
    schemas.insert("Config".to_string(), config_struct);

    // Create renderer with reference resolution
    let mut renderer = PklSchemaRenderer::new(PklSchemaOptions::default());
    renderer.current_schema_name = Some("Config".to_string());

    // Test different reference patterns
    let test_cases = vec![
        // Rustdoc style references
        ("[`Count`]", "[Count](Count)"),
        ("[`Count::Two`]", "[Count::Two](Count)"), // Should fallback to Count
        ("[`Self::count`]", "[Self::count](Config.count)"),
        ("[`self::count`]", "[self::count](Config.count)"),

        // Markdown link style references
        ("[see Count](`Count`)", "[see Count](Count)"),
        ("[see variant](`Count::Two`)", "[see variant](Count)"), // Should fallback
        ("[this field](`Self::count`)", "[this field](Config.count)"),

        // Unresolvable references
        ("[`UnknownType`]", "UnknownType"), // Should become plain text
        ("[`Count::NonExistent`]", "[Count::NonExistent](Count)"), // Should fallback to Count
    ];

    println!("Testing doc comment reference resolution:");
    println!("==========================================");

    for (input, expected) in test_cases {
        let result = renderer.resolve_doc_references(input);
        println!("Input:    {}", input);
        println!("Expected: {}", expected);
        println!("Result:   {}", result);
        println!("Match:    {}", if result == expected { "✓" } else { "✗" });
        println!();
    }
}

/// Test complete documentation rendering with references
fn test_full_documentation_rendering() {
    let mut schemas = TypeMap::new();

    // Add schemas similar to above but with more complex documentation
    // ... (schema setup code would go here)

    let mut renderer = PklSchemaRenderer::new(PklSchemaOptions::default());

    // Test rendering complete documentation
    match renderer.render(schemas) {
        Ok(pkl_output) => {
            println!("Generated Pkl with resolved references:");
            println!("=====================================");
            println!("{}", pkl_output);
        }
        Err(e) => {
            println!("Error rendering: {:?}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_parsing() {
        let renderer = PklSchemaRenderer::new(PklSchemaOptions::default());

        // Test parsing Self references
        let parsed = renderer.parse_reference_path("Self::field");
        assert_eq!(parsed.root, "UnknownType"); // No current schema set
        assert_eq!(parsed.path, vec!["field"]);
        assert!(parsed.is_self_reference);

        // Test parsing normal references
        let parsed = renderer.parse_reference_path("Count::Two");
        assert_eq!(parsed.root, "Count");
        assert_eq!(parsed.path, vec!["Two"]);
        assert!(!parsed.is_self_reference);
    }

    #[test]
    fn test_case_transformations() {
        let renderer = PklSchemaRenderer::new(PklSchemaOptions::default());

        // Test PascalCase transformation
        assert_eq!(renderer.to_pascal_case("my_type"), "MyType");
        assert_eq!(renderer.to_pascal_case("count"), "Count");

        // Test camelCase transformation
        assert_eq!(renderer.to_camel_case("my_field"), "myField");
        assert_eq!(renderer.to_camel_case("count"), "count");
    }
}
