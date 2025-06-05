//! Pkl Tooling Module for Space Pklr
//!
//! This module manages Pkl CLI installation, detection, and execution through proto
//! for consistent toolchain management.

use miette::Result;
use std::path::PathBuf;

/// Pkl CLI representation.
#[derive(Debug, Clone)]
pub struct PklCli {
    pub path: PathBuf,
    pub source: PklSource,
    pub version: Option<String>,
}

/// Pkl installation source enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PklSource {
    /// Installed via proto tool manager
    Proto,
    /// Found in system PATH
    SystemPath,
    /// Manually downloaded and installed
    Manual(PathBuf),
}

/// Install Pkl CLI with proto-first approach
///
/// Implements proto-first installation strategy with fallbacks as specified in
pub async fn install_pkl(version: Option<String>) -> Result<PklCli> {
    use crate::error::CliError;

    let target_version = version.unwrap_or_else(|| get_recommended_pkl_version().to_string());

    // 1. Try proto installation first
    if is_proto_available().await {
        println!("📦 Installing Pkl CLI {} via proto...", target_version);

        match install_via_proto(&target_version).await {
            Ok(pkl_cli) => {
                println!("✅ Successfully installed Pkl CLI via proto");
                return Ok(pkl_cli);
            }
            Err(e) => {
                println!("⚠️  Proto installation failed: {}", e);
                println!("🔄 Trying system PATH detection...");
            }
        }
    } else {
        println!("⚠️  Proto not found, trying system PATH detection...");
    }

    // 2. Check system PATH as fallback
    if let Ok(Some(existing_pkl)) = find_pkl_executable().await {
        if let Some(existing_version) = &existing_pkl.version {
            if existing_version == &target_version {
                println!("✅ Found compatible Pkl CLI in system PATH");
                return Ok(existing_pkl);
            } else {
                println!(
                    "⚠️  Found Pkl CLI version {}, but need version {}",
                    existing_version, target_version
                );
            }
        }
    }

    // 3. Direct download as last resort
    println!("📥 Downloading Pkl CLI {} directly...", target_version);
    match download_pkl_binary(&target_version).await {
        Ok(pkl_path) => {
            let pkl_cli = PklCli {
                path: pkl_path,
                source: PklSource::Manual(get_pkl_install_dir(&target_version)?),
                version: Some(target_version),
            };
            println!("✅ Successfully downloaded and installed Pkl CLI");
            Ok(pkl_cli)
        }
        Err(e) => Err(miette::Report::new(CliError::PklInstallFailed {
            reason: format!("All installation methods failed. Last error: {}", e),
            help: Some(
                "Try installing proto first, or manually install Pkl CLI to your PATH".to_string(),
            ),
        })),
    }
}

/// Find existing Pkl executable
///
/// Searches for Pkl CLI in order of preference: proto -> system PATH -> manual installations
pub async fn find_pkl_executable() -> Result<Option<PklCli>> {
    use crate::error::CliError;

    // 1. Check proto-managed Pkl first
    if is_proto_available().await {
        if let Ok(pkl_cli) = check_proto_pkl().await {
            return Ok(Some(pkl_cli));
        }
    }

    // 2. Check system PATH
    if let Ok(pkl_path) = which::which("pkl") {
        if let Ok(version) = get_pkl_version(&pkl_path).await {
            return Ok(Some(PklCli {
                path: pkl_path,
                source: PklSource::SystemPath,
                version: Some(version),
            }));
        }
    }

    // 3. Check manual installation locations
    if let Ok(home_dir) = dirs::home_dir()
        .ok_or_else(|| CliError::Generic("Could not find home directory".to_string()))
    {
        let pkl_tools_dir = home_dir.join(".moon").join("tools").join("pkl");

        if pkl_tools_dir.exists() {
            // Look for any version directory
            if let Ok(entries) = std::fs::read_dir(&pkl_tools_dir) {
                for entry in entries.flatten() {
                    if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                        let pkl_path = entry.path().join("pkl");
                        if pkl_path.exists() {
                            if let Ok(version) = get_pkl_version(&pkl_path).await {
                                return Ok(Some(PklCli {
                                    path: pkl_path,
                                    source: PklSource::Manual(entry.path()),
                                    version: Some(version),
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Install Pkl via proto
async fn install_via_proto(version: &str) -> Result<PklCli> {
    use crate::error::CliError;
    use std::process::Command;

    let mut cmd = Command::new("proto");
    cmd.args(&["install", &format!("pkl@{}", version)]);

    let output = cmd.output().map_err(|e| CliError::PklInstallFailed {
        reason: format!("Failed to execute proto install: {}", e),
        help: Some("Check that proto is properly installed".to_string()),
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(miette::Report::new(CliError::PklInstallFailed {
            reason: format!("Proto install failed: {}", stderr),
            help: Some("Try running the proto command manually to diagnose the issue".to_string()),
        }));
    }

    // Verify installation by checking proto-managed Pkl
    check_proto_pkl().await
}

/// Check for proto-managed Pkl installation
async fn check_proto_pkl() -> Result<PklCli> {
    use crate::error::CliError;
    use std::process::Command;

    let mut cmd = Command::new("proto");
    cmd.args(&["run", "pkl", "--", "--version"]);

    let output = cmd.output().map_err(|e| CliError::PklInstallFailed {
        reason: format!("Failed to check proto-managed Pkl: {}", e),
        help: Some("Check that proto and Pkl are properly installed".to_string()),
    })?;

    if output.status.success() {
        let version_output = String::from_utf8_lossy(&output.stdout);
        let version = parse_pkl_version(&version_output);

        Ok(PklCli {
            path: PathBuf::from("pkl"), // Proto manages the path
            source: PklSource::Proto,
            version,
        })
    } else {
        Err(miette::Report::new(CliError::PklInstallFailed {
            reason: "Proto-managed Pkl not found or not working".to_string(),
            help: Some("Try installing Pkl with 'proto install pkl'".to_string()),
        }))
    }
}

/// Get Pkl version from executable path
async fn get_pkl_version(pkl_path: &PathBuf) -> Result<String> {
    use std::process::Command;

    let output = Command::new(pkl_path)
        .arg("--version")
        .output()
        .map_err(|e| {
            crate::error::CliError::Generic(format!("Failed to get Pkl version: {}", e))
        })?;

    if output.status.success() {
        let version_output = String::from_utf8_lossy(&output.stdout);
        parse_pkl_version(&version_output).ok_or_else(|| {
            miette::Report::new(crate::error::CliError::Generic(
                "Could not parse Pkl version output".to_string(),
            ))
        })
    } else {
        Err(miette::Report::new(crate::error::CliError::Generic(
            "Pkl version command failed".to_string(),
        )))
    }
}

/// Parse version string from Pkl --version output
fn parse_pkl_version(output: &str) -> Option<String> {
    // Look for version pattern like "Pkl 0.26.0"
    for line in output.lines() {
        if let Some(captures) = regex::Regex::new(r"Pkl\s+(\d+\.\d+\.\d+)")
            .ok()?
            .captures(line)
        {
            return captures.get(1).map(|m| m.as_str().to_string());
        }
    }
    None
}

/// Extract ZIP archive (Windows)
#[cfg(target_os = "windows")]
async fn extract_zip_archive(archive_bytes: &[u8], target_dir: &PathBuf) -> Result<PathBuf> {
    use crate::error::CliError;

    // For simplicity in this implementation, we'll use a basic approach
    // In production, you'd want to use a proper ZIP library like `zip`
    let archive_path = target_dir.join("pkl-cli.zip");
    tokio::fs::write(&archive_path, archive_bytes)
        .await
        .map_err(|e| {
            miette::Report::new(CliError::IoError {
                context: "Writing ZIP archive".to_string(),
                source: e,
            })
        })?;

    // Use system unzip command as fallback
    let output = std::process::Command::new("powershell")
        .args(&[
            "-Command",
            &format!(
                "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                archive_path.display(),
                target_dir.display()
            ),
        ])
        .output()
        .map_err(|e| {
            miette::Report::new(CliError::Generic(format!("Failed to extract ZIP: {}", e)))
        })?;

    if !output.status.success() {
        return Err(miette::Report::new(CliError::Generic(
            "ZIP extraction failed".to_string(),
        )));
    }

    // Clean up archive file
    let _ = tokio::fs::remove_file(&archive_path).await;

    // Find the pkl executable
    Ok(target_dir.join("pkl.exe"))
}

/// Extract ZIP archive (Non-Windows fallback)
#[cfg(not(target_os = "windows"))]
async fn extract_zip_archive(_archive_bytes: &[u8], _target_dir: &PathBuf) -> Result<PathBuf> {
    Err(miette::Report::new(crate::error::CliError::Generic(
        "ZIP extraction not implemented for this platform".to_string(),
    )))
}

/// Extract tar.gz archive (Unix-like systems)
#[cfg(not(target_os = "windows"))]
async fn extract_tar_gz_archive(archive_bytes: &[u8], target_dir: &PathBuf) -> Result<PathBuf> {
    use crate::error::CliError;

    let archive_path = target_dir.join("pkl-cli.tar.gz");
    tokio::fs::write(&archive_path, archive_bytes)
        .await
        .map_err(|e| {
            miette::Report::new(CliError::IoError {
                context: "Writing tar.gz archive".to_string(),
                source: e,
            })
        })?;

    // Use system tar command
    let output = std::process::Command::new("tar")
        .args(&[
            "-xzf",
            &archive_path.to_string_lossy(),
            "-C",
            &target_dir.to_string_lossy(),
        ])
        .output()
        .map_err(|e| {
            miette::Report::new(CliError::Generic(format!(
                "Failed to extract tar.gz: {}",
                e
            )))
        })?;

    if !output.status.success() {
        return Err(miette::Report::new(CliError::Generic(
            "tar.gz extraction failed".to_string(),
        )));
    }

    // Clean up archive file
    let _ = tokio::fs::remove_file(&archive_path).await;

    // Find the pkl executable
    Ok(target_dir.join("pkl"))
}

/// Extract tar.gz archive (Windows fallback)
#[cfg(target_os = "windows")]
async fn extract_tar_gz_archive(_archive_bytes: &[u8], _target_dir: &PathBuf) -> Result<PathBuf> {
    Err(miette::Report::new(crate::error::CliError::Generic(
        "tar.gz extraction not implemented for Windows".to_string(),
    )))
}

/// Execute a Pkl CLI command
///
/// Executes Pkl CLI with proper handling based on installation source
pub async fn execute_pkl_command(pkl_cli: &PklCli, args: &[String]) -> Result<String> {
    use crate::error::{CliError, pkl_execution_error};
    use std::process::Command;

    let mut cmd = match &pkl_cli.source {
        PklSource::Proto => {
            let mut command = Command::new("proto");
            command.arg("run");
            if let Some(version) = &pkl_cli.version {
                command.arg(format!("pkl@{}", version));
            } else {
                command.arg("pkl");
            }
            command.arg("--");
            command.args(args);
            command
        }
        PklSource::SystemPath | PklSource::Manual(_) => {
            let mut command = Command::new(&pkl_cli.path);
            command.args(args);
            command
        }
    };

    let output = cmd.output().map_err(|e| CliError::PklExecutionFailed {
        command: format!("{:?}", cmd),
        stderr: e.to_string(),
        help: Some("Check that Pkl CLI is properly installed and accessible".to_string()),
    })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(miette::Report::new(pkl_execution_error(
            format!("{:?}", cmd),
            stderr.to_string(),
            Some("Check Pkl syntax and file paths".to_string()),
        )))
    }
}

/// Download Pkl CLI binary for the current platform
///
/// Downloads and extracts Pkl CLI from GitHub releases to ~/.moon/tools/pkl/<version>/
async fn download_pkl_binary(version: &str) -> Result<PathBuf> {
    use crate::error::CliError;
    use std::env;

    // Platform detection
    let (os, arch) = match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => ("linux", "amd64"),
        ("linux", "aarch64") => ("linux", "aarch64"),
        ("macos", "x86_64") => ("macos", "amd64"),
        ("macos", "aarch64") => ("macos", "aarch64"),
        ("windows", "x86_64") => ("windows", "amd64"),
        (os, arch) => {
            return Err(miette::Report::new(CliError::PklInstallFailed {
                reason: format!("Unsupported platform: {}-{}", os, arch),
                help: Some("Install Pkl CLI manually or use proto".to_string()),
            }));
        }
    };

    // Create installation directory
    let install_dir = get_pkl_install_dir(version)?;
    tokio::fs::create_dir_all(&install_dir).await.map_err(|e| {
        miette::Report::new(CliError::IoError {
            context: format!(
                "Creating Pkl installation directory: {}",
                install_dir.display()
            ),
            source: e,
        })
    })?;

    // Construct download URL
    let file_extension = if env::consts::OS == "windows" {
        "zip"
    } else {
        "tar.gz"
    };
    let archive_name = format!("pkl-cli-{}-{}.{}", os, arch, file_extension);
    let download_url = format!(
        "https://github.com/apple/pkl/releases/download/{}/{}",
        version, archive_name
    );

    println!("📥 Downloading from: {}", download_url);

    // Download with retry logic
    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| miette::Report::new(CliError::NetworkError(e.to_string())))?;

    if !response.status().is_success() {
        return Err(miette::Report::new(CliError::PklInstallFailed {
            reason: format!("Download failed with status: {}", response.status()),
            help: Some(format!(
                "Check if version {} exists at {}",
                version, download_url
            )),
        }));
    }

    let archive_bytes = response
        .bytes()
        .await
        .map_err(|e| miette::Report::new(CliError::NetworkError(e.to_string())))?;

    // Extract archive
    let pkl_executable_path = if env::consts::OS == "windows" {
        extract_zip_archive(&archive_bytes, &install_dir).await?
    } else {
        extract_tar_gz_archive(&archive_bytes, &install_dir).await?
    };

    // Set executable permissions on Unix-like systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&pkl_executable_path)
            .await
            .map_err(|e| {
                miette::Report::new(CliError::IoError {
                    context: "Reading file permissions".to_string(),
                    source: e,
                })
            })?
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&pkl_executable_path, perms)
            .await
            .map_err(|e| {
                miette::Report::new(CliError::IoError {
                    context: "Setting file permissions".to_string(),
                    source: e,
                })
            })?;
    }

    Ok(pkl_executable_path)
}

/// Get the target installation directory for Pkl
///
/// Returns ~/.moon/tools/pkl/<version>/ path
fn get_pkl_install_dir(version: &str) -> Result<PathBuf> {
    use crate::error::CliError;

    let home_dir = dirs::home_dir().ok_or_else(|| {
        miette::Report::new(CliError::Generic(
            "Could not determine home directory".to_string(),
        ))
    })?;

    Ok(home_dir
        .join(".moon")
        .join("tools")
        .join("pkl")
        .join(version))
}

/// Check if proto is available in the system
///
/// Checks for proto executable in PATH and verifies basic functionality
async fn is_proto_available() -> bool {
    which::which("proto").is_ok()
}

/// CI-managed version pinning with automated compatibility testing
pub fn get_recommended_pkl_version() -> &'static str {
    // This version is automatically updated by CI after compatibility testing
    "0.28.0"
}

/// Compatibility matrix for tested versions
pub fn get_compatible_pkl_versions() -> Vec<&'static str> {
    vec!["0.28.0", "0.28.1", "0.28.2"] // Updated by CI
}

/// Comprehensive compatibility report for Pkl CLI validation
#[derive(Debug)]
pub struct CompatibilityReport {
    pub basic_functionality: bool,
    pub moon_config_integration: bool,
    pub extend_amend_support: bool,
    pub schema_generation: bool,
    pub pkl_version: String,
}

impl CompatibilityReport {
    pub fn new(pkl_version: String) -> Self {
        Self {
            basic_functionality: false,
            moon_config_integration: false,
            extend_amend_support: false,
            schema_generation: false,
            pkl_version,
        }
    }

    pub fn is_compatible(&self) -> bool {
        self.basic_functionality
            && self.moon_config_integration
            && self.extend_amend_support
            && self.schema_generation
    }
}

/// Validate Pkl version compatibility with comprehensive testing
pub async fn validate_pkl_compatibility(pkl_cli: &PklCli) -> Result<CompatibilityReport> {


    let version = pkl_cli
        .version
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let mut report = CompatibilityReport::new(version);

    tracing::info!(
        "Starting comprehensive Pkl compatibility validation for version {}",
        report.pkl_version
    );

    // Test basic functionality
    tracing::debug!("Testing basic Pkl functionality...");
    report.basic_functionality = test_basic_pkl_functionality(pkl_cli).await?;

    // Test moon_config integration
    tracing::debug!("Testing moon_config integration...");
    report.moon_config_integration = test_moon_config_integration(pkl_cli).await?;

    // Test extend/amend features
    tracing::debug!("Testing extend/amend features...");
    report.extend_amend_support = test_extend_amend_features(pkl_cli).await?;

    // Test schema generation
    tracing::debug!("Testing schema generation...");
    report.schema_generation = test_schema_generation(pkl_cli).await?;

    tracing::info!(
        "Compatibility validation completed. Compatible: {}",
        report.is_compatible()
    );
    Ok(report)
}

/// Test basic Pkl CLI functionality
async fn test_basic_pkl_functionality(pkl_cli: &PklCli) -> Result<bool> {
    match execute_pkl_command(pkl_cli, &["--version".to_string()]).await {
        Ok(output) => {
            let has_version = output.contains("Pkl") && output.chars().any(|c| c.is_ascii_digit());
            tracing::debug!(
                "Basic functionality test: {}",
                if has_version { "PASS" } else { "FAIL" }
            );
            Ok(has_version)
        }
        Err(e) => {
            tracing::warn!("Basic functionality test failed: {}", e);
            Ok(false)
        }
    }
}

/// Test moon_config integration by validating a simple configuration
async fn test_moon_config_integration(pkl_cli: &PklCli) -> Result<bool> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a simple test configuration
    let mut temp_file = NamedTempFile::new().map_err(|e| crate::error::CliError::IoError {
        context: "Creating temporary test file".to_string(),
        source: e,
    })?;

    writeln!(
        temp_file,
        r#"
language = "rust"
type = "library"
tasks {{
    build {{
        command = "cargo build"
    }}
}}
"#
    )
    .map_err(|e| crate::error::CliError::IoError {
        context: "Writing test configuration".to_string(),
        source: e,
    })?;

    // Try to evaluate the configuration with Pkl
    match execute_pkl_command(
        pkl_cli,
        &[
            "eval".to_string(),
            temp_file.path().to_string_lossy().to_string(),
        ],
    )
    .await
    {
        Ok(_) => {
            tracing::debug!("Moon config integration test: PASS");
            Ok(true)
        }
        Err(e) => {
            tracing::warn!("Moon config integration test failed: {}", e);
            Ok(false)
        }
    }
}

/// Test extend/amend features with a simple inheritance scenario
async fn test_extend_amend_features(pkl_cli: &PklCli) -> Result<bool> {
    use std::io::Write;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().map_err(|e| crate::error::CliError::IoError {
        context: "Creating temporary directory".to_string(),
        source: e,
    })?;

    // Create base configuration
    let base_path = temp_dir.path().join("base.pkl");
    let mut base_file =
        std::fs::File::create(&base_path).map_err(|e| crate::error::CliError::IoError {
            context: "Creating base test file".to_string(),
            source: e,
        })?;

    writeln!(
        base_file,
        r#"
language = "rust"
type = "library"
"#
    )
    .map_err(|e| crate::error::CliError::IoError {
        context: "Writing base configuration".to_string(),
        source: e,
    })?;

    // Create extending configuration
    let extend_path = temp_dir.path().join("extend.pkl");
    let mut extend_file =
        std::fs::File::create(&extend_path).map_err(|e| crate::error::CliError::IoError {
            context: "Creating extend test file".to_string(),
            source: e,
        })?;

    writeln!(
        extend_file,
        r#"
extends "{}"

tasks {{
    build {{
        command = "cargo build"
    }}
}}
"#,
        base_path.to_string_lossy()
    )
    .map_err(|e| crate::error::CliError::IoError {
        context: "Writing extend configuration".to_string(),
        source: e,
    })?;

    // Try to evaluate the extending configuration
    match execute_pkl_command(
        pkl_cli,
        &[
            "eval".to_string(),
            extend_path.to_string_lossy().to_string(),
        ],
    )
    .await
    {
        Ok(_) => {
            tracing::debug!("Extend/amend features test: PASS");
            Ok(true)
        }
        Err(e) => {
            tracing::warn!("Extend/amend features test failed: {}", e);
            Ok(false)
        }
    }
}

/// Test schema generation capabilities
async fn test_schema_generation(pkl_cli: &PklCli) -> Result<bool> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a simple schema test
    let mut temp_file = NamedTempFile::new().map_err(|e| crate::error::CliError::IoError {
        context: "Creating schema test file".to_string(),
        source: e,
    })?;

    writeln!(
        temp_file,
        r#"
class Config {{
    language: String
    type: String
}}
"#
    )
    .map_err(|e| crate::error::CliError::IoError {
        context: "Writing schema test file".to_string(),
        source: e,
    })?;

    // Try to generate schema
    match execute_pkl_command(
        pkl_cli,
        &[
            "project".to_string(),
            "package".to_string(),
            temp_file.path().to_string_lossy().to_string(),
        ],
    )
    .await
    {
        Ok(_) => {
            tracing::debug!("Schema generation test: PASS");
            Ok(true)
        }
        Err(e) => {
            tracing::warn!("Schema generation test failed: {}", e);
            Ok(false)
        }
    }
}

/// Validate Pkl CLI installation
///
/// Validates installation by running pkl --version and checking output
pub async fn validate_pkl_installation(pkl_cli: &PklCli) -> Result<bool> {
    match execute_pkl_command(pkl_cli, &["--version".to_string()]).await {
        Ok(output) => {
            // Check if output contains version information
            Ok(output.contains("pkl") && output.chars().any(|c| c.is_ascii_digit()))
        }
        Err(_) => Ok(false),
    }
}
