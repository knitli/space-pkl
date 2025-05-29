//! Basic usage example for space-pkl

use space_pkl::prelude::*;
use std::path::PathBuf;

fn main() -> space_pkl::Result<()> {
    println!("🌙 space-pkl Basic Usage Example");
    println!("================================");

    // Example 1: Generate all schemas with default configuration
    println!("\n1. Generating all schemas with defaults...");
    let generator = SchemaGenerator::new(GeneratorConfig::default());
    generator.generate_all()?;
    println!("   ✅ Generated all schemas in ./pkl-schemas/");

    // Example 2: Generate a specific schema
    println!("\n2. Generating workspace schema...");
    let workspace_schema = generator.generate_workspace_schema()?;
    println!("   📝 Workspace schema ({} chars)", workspace_schema.len());

    // Example 3: Custom configuration
    println!("\n3. Generating with custom configuration...");
    let custom_config = GeneratorConfig {
        include_comments: true,
        include_examples: true,
        output_dir: PathBuf::from("./custom-schemas"),
        module_name: "myproject".to_string(),
        header: Some(
            r#"// Custom Moon Configuration Schema
// Generated for MyProject
//

"#.to_string()
        ),
        ..Default::default()
    };

    let custom_generator = SchemaGenerator::new(custom_config);
    custom_generator.generate_all()?;
    println!("   ✅ Generated custom schemas in ./custom-schemas/");

    // Example 4: Generate individual schemas
    println!("\n4. Generating individual schemas...");

    let project_schema = generator.generate_project_schema()?;
    println!("   📋 Project schema: {} chars", project_schema.len());

    let toolchain_schema = generator.generate_toolchain_schema()?;
    println!("   🔧 Toolchain schema: {} chars", toolchain_schema.len());

    // Example 5: Show sample of generated content
    println!("\n5. Sample workspace schema (first 20 lines):");
    println!("   {}", "-".repeat(50));
    for (i, line) in workspace_schema.lines().take(20).enumerate() {
        println!("   {:2} | {}", i + 1, line);
    }
    println!("   {}", "-".repeat(50));

    Ok(())
}
