use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Test that all PKL schemas can be parsed without errors
#[test]
fn test_pkl_schemas_parse_successfully() {
    let schema_dir = Path::new("pkl-schemas");
    assert!(schema_dir.exists(), "PKL schemas directory should exist");

    let schema_files = [
        "workspace.pkl",
        "project.pkl",
        "tasks.pkl",
        "template.pkl",
        "toolchain.pkl",
        "mod.pkl",
    ];

    for schema_file in &schema_files {
        let schema_path = schema_dir.join(schema_file);
        assert!(
            schema_path.exists(),
            "Schema file {} should exist",
            schema_file
        );

        let output = Command::new("pkl")
            .arg("eval")
            .arg("--format")
            .arg("json")
            .arg(&schema_path)
            .output()
            .expect("Failed to execute pkl command");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("PKL parsing failed for {}: {}", schema_file, stderr);
        }
    }
}

/// Test that PKL test files execute successfully
#[test]
fn test_pkl_test_suite() {
    let test_script = Path::new("scripts/run-pkl-tests.sh");
    assert!(test_script.exists(), "PKL test runner script should exist");

    let output = Command::new("bash")
        .arg(test_script)
        .output()
        .expect("Failed to execute PKL test script");

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "PKL tests failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
            stdout, stderr
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("All PKL tests passed"),
        "PKL test suite should pass"
    );
}

/// Test specific PKL schema validation
#[test]
fn test_workspace_schema_validation() {
    let workspace_schema = Path::new("pkl-schemas/workspace.pkl");
    assert!(workspace_schema.exists(), "Workspace schema should exist");

    // Test valid workspace configuration
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_file = temp_dir.path().join("test_workspace.pkl");

    let test_content = r#"
amends "../../../pkl-schemas/workspace.pkl"

// Basic workspace configuration
$schema = "https://moonrepo.dev/schemas/workspace.json"
versionConstraint = ">=1.0.0"

projects = new Listing {
    "apps/*"
    "packages/*"
}

docker = new {
    scaffold = new {
        include = new Listing { "Dockerfile" }
    }
}

vcs = new {
    manager = "git"
    defaultBranch = "main"
}
"#;

    fs::write(&test_file, test_content).expect("Failed to write test file");

    let output = Command::new("pkl")
        .arg("eval")
        .arg("--format")
        .arg("json")
        .arg(&test_file)
        .output()
        .expect("Failed to execute pkl command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Workspace schema validation failed: {}", stderr);
    }
}

/// Test project schema validation
#[test]
fn test_project_schema_validation() {
    let project_schema = Path::new("pkl-schemas/project.pkl");
    assert!(project_schema.exists(), "Project schema should exist");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_file = temp_dir.path().join("test_project.pkl");

    let test_content = r#"
amends "../../../pkl-schemas/project.pkl"

// Basic project configuration
$schema = "https://moonrepo.dev/schemas/project.json"
type = "application"
language = "typescript"
platform = "node"

project = new {
    name = "my-app"
    description = "A test application"
}

tasks = new Mapping {
    ["build"] = new {
        command = "npm run build"
        inputs = new Listing { "src/**/*" }
        outputs = new Listing { "dist/**/*" }
    }
    ["test"] = new {
        command = "npm test"
        inputs = new Listing { "src/**/*", "tests/**/*" }
    }
}
"#;

    fs::write(&test_file, test_content).expect("Failed to write test file");

    let output = Command::new("pkl")
        .arg("eval")
        .arg("--format")
        .arg("json")
        .arg(&test_file)
        .output()
        .expect("Failed to execute pkl command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Project schema validation failed: {}", stderr);
    }
}

/// Test that invalid configurations are properly rejected
#[test]
fn test_invalid_configuration_rejection() {
    let workspace_schema = Path::new("pkl-schemas/workspace.pkl");
    assert!(workspace_schema.exists(), "Workspace schema should exist");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_file = temp_dir.path().join("invalid_workspace.pkl");

    // Create an invalid configuration with wrong types
    let invalid_content = r#"
amends "../../../pkl-schemas/workspace.pkl"

// Invalid configuration - versionConstraint should be a string, not number
versionConstraint = 123

// Invalid project path type
projects = "invalid_type"
"#;

    fs::write(&test_file, invalid_content).expect("Failed to write test file");

    let output = Command::new("pkl")
        .arg("eval")
        .arg("--format")
        .arg("json")
        .arg(&test_file)
        .output()
        .expect("Failed to execute pkl command");

    // This should fail - invalid configurations should be rejected
    assert!(
        !output.status.success(),
        "Invalid configuration should be rejected"
    );
}

/// Test PKL schema imports and module relationships
#[test]
fn test_schema_imports() {
    let mod_schema = Path::new("pkl-schemas/mod.pkl");
    assert!(mod_schema.exists(), "Module schema should exist");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_file = temp_dir.path().join("test_imports.pkl");

    let test_content = r#"
import "../../../pkl-schemas/mod.pkl"

// Test that we can access exported schemas
output {
    hasWorkspaceSchema = mod.Workspace != null
    hasProjectSchema = mod.Project != null
    hasTasksSchema = mod.Tasks != null
    hasTemplateSchema = mod.Template != null
    hasToolchainSchema = mod.Toolchain != null
}
"#;

    fs::write(&test_file, test_content).expect("Failed to write test file");

    let output = Command::new("pkl")
        .arg("eval")
        .arg("--format")
        .arg("json")
        .arg(&test_file)
        .output()
        .expect("Failed to execute pkl command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Schema imports test failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("PKL output should be valid JSON");

    assert_eq!(json["hasWorkspaceSchema"], true);
    assert_eq!(json["hasProjectSchema"], true);
    assert_eq!(json["hasTasksSchema"], true);
    assert_eq!(json["hasTemplateSchema"], true);
    assert_eq!(json["hasToolchainSchema"], true);
}

/// Test PKL type constraints and enums
#[test]
fn test_type_constraints() {
    let project_schema = Path::new("pkl-schemas/project.pkl");
    assert!(project_schema.exists(), "Project schema should exist");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test valid enum values
    let valid_test_file = temp_dir.path().join("valid_enums.pkl");
    let valid_content = r#"
amends "../../../pkl-schemas/project.pkl"

language = "typescript"  // Valid LanguageType
platform = "node"       // Valid PlatformType
type = "application"     // Valid ProjectType
stack = "frontend"       // Valid StackType
"#;

    fs::write(&valid_test_file, valid_content).expect("Failed to write valid test file");

    let output = Command::new("pkl")
        .arg("eval")
        .arg("--format")
        .arg("json")
        .arg(&valid_test_file)
        .output()
        .expect("Failed to execute pkl command");

    assert!(
        output.status.success(),
        "Valid enum values should be accepted"
    );

    // Test invalid enum values
    let invalid_test_file = temp_dir.path().join("invalid_enums.pkl");
    let invalid_content = r#"
amends "../../../pkl-schemas/project.pkl"

language = "invalid_language"  // Invalid LanguageType
platform = "invalid_platform"  // Invalid PlatformType
"#;

    fs::write(&invalid_test_file, invalid_content).expect("Failed to write invalid test file");

    let output = Command::new("pkl")
        .arg("eval")
        .arg("--format")
        .arg("json")
        .arg(&invalid_test_file)
        .output()
        .expect("Failed to execute pkl command");

    assert!(
        !output.status.success(),
        "Invalid enum values should be rejected"
    );
}

/// Benchmark PKL schema parsing performance
#[test]
fn test_pkl_performance() {
    let workspace_schema = Path::new("pkl-schemas/workspace.pkl");
    assert!(workspace_schema.exists(), "Workspace schema should exist");

    let start = std::time::Instant::now();

    // Parse the schema multiple times to test performance
    for _ in 0..10 {
        let output = Command::new("pkl")
            .arg("eval")
            .arg("--format")
            .arg("json")
            .arg(workspace_schema)
            .output()
            .expect("Failed to execute pkl command");

        assert!(output.status.success(), "PKL parsing should succeed");
    }

    let duration = start.elapsed();
    println!(
        "PKL schema parsing performance: {:?} for 10 iterations",
        duration
    );

    // Ensure reasonable performance (less than 10 seconds for 10 iterations)
    assert!(
        duration.as_secs() < 10,
        "PKL parsing should be reasonably fast"
    );
}
