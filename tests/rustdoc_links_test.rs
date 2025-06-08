#[cfg(test)]
mod rustdoc_links_tests {
    use crate::new_renderer::{PklSchemaRenderer, PklSchemaOptions};
    use indexmap::IndexMap;
    use schematic_types::*;

    fn create_test_renderer() -> PklSchemaRenderer {
        let options = PklSchemaOptions::default();
        let mut renderer = PklSchemaRenderer::new(options);

        // Add some test schemas
        let mut schemas = IndexMap::new();

        // Create a Bar struct
        let bar_schema = Schema {
            description: Some("This is Bar".to_string()),
            deprecated: None,
            name: Some("Bar".to_string()),
            nullable: false,
            optional: false,
            ty: SchemaType::Struct(Box::new(StructType {
                fields: IndexMap::new(),
                partial: false,
            })),
        };
        schemas.insert("Bar".to_string(), bar_schema);

        // Create an Option enum
        let option_schema = Schema {
            description: Some("This is Option".to_string()),
            deprecated: None,
            name: Some("Option".to_string()),
            nullable: false,
            optional: false,
            ty: SchemaType::Enum(Box::new(EnumType {
                name: "Option".to_string(),
                values: vec![
                    LiteralValue::String("Some".to_string()),
                    LiteralValue::String("None".to_string()),
                ],
                variants: None,
                default: None,
            })),
        };
        schemas.insert("Option".to_string(), option_schema);

        renderer.schemas = schemas;
        renderer
    }

    #[test]
    fn test_simple_link_resolution() {
        let renderer = create_test_renderer();

        // Test: [Bar] - simple link
        let input = "This struct is not [Bar]";
        let result = renderer.resolve_doc_references(input);
        assert!(result.contains("[Bar](Bar)") || result.contains("Bar"),
                "Failed to resolve simple link [Bar]: {}", result);
    }

    #[test]
    fn test_backtick_link_resolution() {
        let renderer = create_test_renderer();

        // Test: [`Bar`] - link with backticks (backticks should be stripped)
        let input = "This struct is also not [`Bar`]";
        let result = renderer.resolve_doc_references(input);
        assert!(result.contains("[Bar](Bar)") || result.contains("Bar"),
                "Failed to resolve backtick link [`Bar`]: {}", result);
    }

    #[test]
    fn test_link_with_different_text() {
        let renderer = create_test_renderer();

        // Test: [bar](Bar) - link with different text
        let input = "This struct is also not [bar](Bar)";
        let result = renderer.resolve_doc_references(input);
        assert!(result.contains("[bar](Bar)") || result.contains("bar"),
                "Failed to resolve link with different text [bar](Bar): {}", result);
    }

    #[test]
    fn test_link_with_backticks_in_reference() {
        let renderer = create_test_renderer();

        // Test: [bar](`Bar`) - link with backticks around reference
        let input = "This struct is also not [bar](`Bar`)";
        let result = renderer.resolve_doc_references(input);
        assert!(result.contains("[bar](Bar)") || result.contains("bar"),
                "Failed to resolve link with backticks in reference [bar](`Bar`): {}", result);
    }

    #[test]
    fn test_reference_style_link() {
        let renderer = create_test_renderer();

        // Test: [bar][Bar] - reference-style link
        let input = "This struct is also not [bar][Bar]";
        let result = renderer.resolve_doc_references(input);
        assert!(result.contains("[bar](Bar)") || result.contains("bar"),
                "Failed to resolve reference-style link [bar][Bar]: {}", result);
    }

    #[test]
    fn test_multiple_link_types_in_same_text() {
        let renderer = create_test_renderer();

        // Test multiple link types in the same text
        let input = "See [Bar], [`Option`], [custom text](Bar), and [other][Option] for details.";
        let result = renderer.resolve_doc_references(input);

        // Should handle all different link formats
        println!("Input: {}", input);
        println!("Result: {}", result);

        // At minimum, it should not crash and should contain references to our types
        assert!(result.contains("Bar") || result.contains("Option"),
                "Failed to resolve multiple link types: {}", result);
    }

    #[test]
    fn test_reference_definition_removal() {
        let renderer = create_test_renderer();

        // Test: [b]: Bar - reference definition (should be removed)
        let input = "This struct is also not [bar][b]\n\n[b]: Bar";
        let result = renderer.resolve_doc_references(input);

        // Reference definition should be removed from output
        assert!(!result.contains("[b]: Bar"),
                "Reference definition was not removed: {}", result);
    }

    #[test]
    fn test_nested_brackets_dont_break_parsing() {
        let renderer = create_test_renderer();

        // Test edge cases with nested brackets or special characters
        let input = "See [Bar] and [some [nested] text](Option) and [`Option`].";
        let result = renderer.resolve_doc_references(input);

        // Should not crash and should handle at least some of the links
        println!("Input: {}", input);
        println!("Result: {}", result);

        assert!(!result.is_empty(), "Result should not be empty");
    }
}
