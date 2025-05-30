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

    let test_content = format!(
        r#"
amends "pkl:test"

import "{}/pkl-schemas/workspace.pkl" as workspace

local validConfig = new workspace.WorkspaceConfig {{
  versionConstraint = ">=1.0.0"
  projects = new workspace.WorkspaceProjectsConfig {{
    globs = new Listing {{ "apps/*"; "packages/*" }}
  }}
  hasher = new workspace.HasherConfig {{
    optimization = "accuracy"
    walkStrategy = "vcs"
  }}
  vcs = new workspace.VcsConfig {{
    manager = "git"
    provider = "github"
    hookFormat = "bash"
    defaultBranch = "main"
  }}
}}

facts {{
  ["Valid workspace configuration should work"] {{
    validConfig.versionConstraint == ">=1.0.0"
    validConfig.vcs.manager == "git"
    validConfig.hasher.optimization == "accuracy"
  }}
}}
"#,
        std::env::current_dir().unwrap().display()
    );

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

    let test_content = format!(
        r#"
amends "pkl:test"

import "{}/pkl-schemas/project.pkl" as project

local validConfig = new project.ProjectConfig {{
  language = "typescript" as project.LanguageType
  platform = "node" as project.PlatformType
  type = "application" as project.ProjectType
  stack = "frontend" as project.StackType
}}

facts {{
  ["Valid project configuration should work"] {{
    validConfig.language == "typescript"
    validConfig.platform == "node"
    validConfig.type == "application"
    validConfig.stack == "frontend"
  }}
}}
"#,
        std::env::current_dir().unwrap().display()
    );

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
    let invalid_content = format!(
        r#"
import "{}/pkl-schemas/workspace.pkl" as workspace

// This should fail - invalid enum values
local invalidConfig = new workspace.WorkspaceConfig {{
  hasher = new workspace.HasherConfig {{
    optimization = "invalid_optimization"  // Should be "accuracy" or "performance"
    walkStrategy = "invalid_strategy"      // Should be "glob" or "vcs"
  }}
  vcs = new workspace.VcsConfig {{
    manager = "invalid_vcs"  // Should be "git"
    provider = "invalid_provider"  // Should be bitbucket|github|gitlab|other
    hookFormat = "invalid_format"  // Should be "bash" or "native"
  }}
}}

output {{ invalid: invalidConfig }}
"#,
        std::env::current_dir().unwrap().display()
    );

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

    let test_content = format!(
        r#"
amends "pkl:test"

import "{}/pkl-schemas/mod.pkl" as moon

// Test that we can access exported schemas by creating instances
local workspaceExample = new moon.Workspace {{
  versionConstraint = ">=1.0.0"
  projects = new {{ globs = new Listing {{ "apps/*" }} }}
  hasher = new {{ optimization = "accuracy"; walkStrategy = "vcs" }}
  vcs = new {{ manager = "git"; provider = "github"; hookFormat = "bash" }}
}}

local projectExample = new moon.Project {{
  language = "typescript"
  platform = "node"
  type = "application"
  stack = "frontend"
}}

facts {{
  ["Can create workspace instance"] {{ workspaceExample != null }}
  ["Can create project instance"] {{ projectExample != null }}
  ["Workspace has correct VCS"] {{ workspaceExample.vcs.manager == "git" }}
  ["Project has correct language"] {{ projectExample.language == "typescript" }}
}}
"#,
        std::env::current_dir().unwrap().display()
    );

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

    // If we reach here, the test passed (no assertion errors from pkl)
    println!("Schema imports test passed successfully");
}

/// Test PKL type constraints and enums
#[test]
fn test_type_constraints() {
    let project_schema = Path::new("pkl-schemas/project.pkl");
    assert!(project_schema.exists(), "Project schema should exist");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test valid enum values
    let valid_test_file = temp_dir.path().join("valid_enums.pkl");
    let valid_content = format!(
        r#"
amends "pkl:test"

import "{}/pkl-schemas/project.pkl" as project

local validConfig = new project.ProjectConfig {{
  language = "typescript" as project.LanguageType
  platform = "node" as project.PlatformType
  type = "application" as project.ProjectType
  stack = "frontend" as project.StackType
}}

facts {{
  ["Valid enum values should work"] {{
    validConfig.language == "typescript"
    validConfig.platform == "node"
    validConfig.type == "application"
    validConfig.stack == "frontend"
  }}
}}
"#,
        std::env::current_dir().unwrap().display()
    );

    fs::write(&valid_test_file, valid_content).expect("Failed to write valid test file");

    let output = Command::new("pkl")
        .arg("eval")
        .arg("--format")
        .arg("json")
        .arg(&valid_test_file)
        .output()
        .expect("Failed to execute pkl command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "Valid enum values should be accepted. PKL error: {}",
            stderr
        );
    }

    // Test invalid enum values
    let invalid_test_file = temp_dir.path().join("invalid_enums.pkl");
    let invalid_content = format!(
        r#"
import "{}/pkl-schemas/project.pkl" as project

// This should fail with invalid enum values
local invalidConfig = new project.ProjectConfig {{
  language = "invalid_language" as project.LanguageType
  platform = "invalid_platform" as project.PlatformType
}}

output {{ invalid: invalidConfig }}
"#,
        std::env::current_dir().unwrap().display()
    );

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
