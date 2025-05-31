//! Test suite for deprecation handling

use space_pkl::{
    config::GeneratorConfig,
    templates::TemplateEngine,
    types::{PklModule, PklProperty, PklType, PklTypeKind},
    Result,
};

#[tokio::test]
async fn test_template_renders_deprecated_class() -> Result<()> {
    // Create a deprecated class manually
    let deprecated_class = PklType {
        name: "DeprecatedConfig".to_string(),
        kind: PklTypeKind::Class,
        deprecated: Some("This class is deprecated. Use NewConfig instead.".to_string()),
        documentation: Some("A deprecated configuration class".to_string()),
        properties: vec![PklProperty {
            name: "normal_field".to_string(),
            type_name: "String".to_string(),
            optional: false,
            deprecated: None,
            documentation: Some("A normal field".to_string()),
            default: None,
            constraints: vec![],
            examples: vec![],
        }],
        enum_values: None,
        extends: vec![],
        abstract_type: false,
    };

    let module = PklModule {
        name: "test".to_string(),
        documentation: Some("Test module".to_string()),
        types: vec![deprecated_class],
        imports: vec![],
        exports: vec![],
        properties: vec![],
    };

    let config = GeneratorConfig {
        include_deprecated: true,
        ..Default::default()
    };

    let template_engine = TemplateEngine::new(&config);
    let rendered = template_engine.render_module(&module, &config)?;

    // Check that @Deprecated decorator is present for the class
    assert!(
        rendered.contains("@Deprecated"),
        "Should contain @Deprecated decorator for class"
    );
    assert!(
        rendered.contains("DeprecatedConfig"),
        "Should contain deprecated class name"
    );

    println!("Rendered PKL with deprecated class:");
    println!("{}", rendered);

    Ok(())
}

#[tokio::test]
async fn test_template_renders_deprecated_property() -> Result<()> {
    // Create a class with deprecated property
    let deprecated_property = PklProperty {
        name: "deprecated_field".to_string(),
        type_name: "String".to_string(),
        optional: false,
        deprecated: Some("Use new_field instead".to_string()),
        documentation: Some("A deprecated field".to_string()),
        default: None,
        constraints: vec![],
        examples: vec![],
    };

    let normal_property = PklProperty {
        name: "normal_field".to_string(),
        type_name: "String".to_string(),
        optional: false,
        deprecated: None,
        documentation: Some("A normal field".to_string()),
        default: None,
        constraints: vec![],
        examples: vec![],
    };

    let test_class = PklType {
        name: "TestConfig".to_string(),
        kind: PklTypeKind::Class,
        deprecated: None,
        documentation: Some("A test configuration class".to_string()),
        properties: vec![deprecated_property, normal_property],
        enum_values: None,
        extends: vec![],
        abstract_type: false,
    };

    let module = PklModule {
        name: "test".to_string(),
        documentation: Some("Test module".to_string()),
        types: vec![test_class],
        imports: vec![],
        exports: vec![],
        properties: vec![],
    };

    let config = GeneratorConfig {
        include_deprecated: true,
        ..Default::default()
    };

    let template_engine = TemplateEngine::new(&config);
    let rendered = template_engine.render_module(&module, &config)?;

    // Check that @Deprecated decorator is present for the deprecated property
    assert!(
        rendered.contains("@Deprecated"),
        "Should contain @Deprecated decorator for property"
    );
    assert!(
        rendered.contains("deprecated_field"),
        "Should contain deprecated field name"
    );
    assert!(
        rendered.contains("normal_field"),
        "Should contain normal field name"
    );

    println!("Rendered PKL with deprecated property:");
    println!("{}", rendered);

    Ok(())
}

#[tokio::test]
async fn test_deprecated_property_filtering() -> Result<()> {
    // Test template behavior with deprecated properties when include_deprecated is false
    let deprecated_property = PklProperty {
        name: "deprecated_field".to_string(),
        type_name: "String".to_string(),
        optional: false,
        deprecated: Some("Use new_field instead".to_string()),
        documentation: Some("A deprecated field".to_string()),
        default: None,
        constraints: vec![],
        examples: vec![],
    };

    let normal_property = PklProperty {
        name: "normal_field".to_string(),
        type_name: "String".to_string(),
        optional: false,
        deprecated: None,
        documentation: Some("A normal field".to_string()),
        default: None,
        constraints: vec![],
        examples: vec![],
    };

    let test_class = PklType {
        name: "TestConfig".to_string(),
        kind: PklTypeKind::Class,
        deprecated: None,
        documentation: Some("A test configuration class".to_string()),
        properties: vec![deprecated_property, normal_property],
        enum_values: None,
        extends: vec![],
        abstract_type: false,
    };

    let module = PklModule {
        name: "test".to_string(),
        documentation: Some("Test module".to_string()),
        types: vec![test_class],
        imports: vec![],
        exports: vec![],
        properties: vec![],
    };

    // Test with include_deprecated = false (default)
    let config = GeneratorConfig {
        include_deprecated: false,
        ..Default::default()
    };

    let template_engine = TemplateEngine::new(&config);
    let rendered = template_engine.render_module(&module, &config)?;

    // In the template rendering, deprecated properties should still be present
    // because the filtering happens at the generator level before creating PklType
    // The template just renders what it's given
    assert!(
        rendered.contains("deprecated_field"),
        "Template should render deprecated field if it's in the type"
    );
    assert!(
        rendered.contains("normal_field"),
        "Should contain normal field name"
    );

    println!("Rendered PKL with include_deprecated=false:");
    println!("{}", rendered);

    Ok(())
}
