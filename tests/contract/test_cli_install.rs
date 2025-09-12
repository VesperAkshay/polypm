// Contract test for `ppm install` command  
// This test defines the expected behavior and MUST FAIL initially (TDD)

use std::fs;
use tempfile::TempDir;
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_ppm_install_basic_success() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create a basic project.toml
    let project_toml = r#"
[project]
name = "test-project"
version = "1.0.0"

[dependencies.javascript]
react = "^18.0.0"

[dependencies.python]
flask = "^2.0.0"
"#;
    fs::write(project_path.join("project.toml"), project_toml).unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .arg("install");
        
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("✓ Resolved"))
        .stdout(predicate::str::contains("JavaScript packages"))
        .stdout(predicate::str::contains("Python packages"))
        .stdout(predicate::str::contains("Installed"));
        
    // Verify lock file was created
    let lock_file_path = project_path.join("ppm.lock");
    assert!(lock_file_path.exists(), "ppm.lock should be created");
    
    // Verify environments were created
    let node_modules_path = project_path.join(".ppm").join("node_modules");
    let venv_path = project_path.join(".ppm").join("venv");
    assert!(node_modules_path.exists(), "node_modules should be created");
    assert!(venv_path.exists(), "venv should be created");
}

#[test]
fn test_ppm_install_specific_packages() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create a basic project.toml
    let project_toml = r#"
[project]
name = "test-project"
version = "1.0.0"

[dependencies.javascript]

[dependencies.python]
"#;
    fs::write(project_path.join("project.toml"), project_toml).unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["install", "lodash", "requests"]);
        
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("✓ Downloaded"))
        .stdout(predicate::str::contains("packages"));
        
    // Verify packages were added to project.toml
    let updated_content = fs::read_to_string(project_path.join("project.toml")).unwrap();
    assert!(updated_content.contains("lodash") || updated_content.contains("requests"));
}

#[test]
fn test_ppm_install_save_dev() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create a basic project.toml
    let project_toml = r#"
[project]
name = "test-project"
version = "1.0.0"

[dependencies.javascript]

[dev-dependencies.javascript]

[dependencies.python]

[dev-dependencies.python]
"#;
    fs::write(project_path.join("project.toml"), project_toml).unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["install", "--save-dev", "jest", "pytest"]);
        
    cmd.assert()
        .success();
        
    // Verify packages were added to dev-dependencies
    let updated_content = fs::read_to_string(project_path.join("project.toml")).unwrap();
    assert!(updated_content.contains("[dev-dependencies"));
}

#[test]
fn test_ppm_install_javascript_only() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let project_toml = r#"
[project]
name = "test-project"
version = "1.0.0"

[dependencies.javascript]
react = "^18.0.0"

[dependencies.python]
flask = "^2.0.0"
"#;
    fs::write(project_path.join("project.toml"), project_toml).unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["install", "--javascript"]);
        
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("JavaScript packages"))
        .stdout(predicate::str::contains("Created symlinks"));
}

#[test]
fn test_ppm_install_python_only() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let project_toml = r#"
[project]
name = "test-project"
version = "1.0.0"

[dependencies.javascript]
react = "^18.0.0"

[dependencies.python]
flask = "^2.0.0"
"#;
    fs::write(project_path.join("project.toml"), project_toml).unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["install", "--python"]);
        
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Python packages"))
        .stdout(predicate::str::contains("Updated Python virtual environment"));
}

#[test]
fn test_ppm_install_no_project_toml() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .arg("install");
        
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("No project.toml found (run 'ppm init' first)"));
}

#[test]
fn test_ppm_install_package_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let project_toml = r#"
[project]
name = "test-project"
version = "1.0.0"

[dependencies.javascript]

[dependencies.python]
"#;
    fs::write(project_path.join("project.toml"), project_toml).unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["install", "nonexistent-package-xyz-123"]);
        
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Package 'nonexistent-package-xyz-123' not found"));
}

#[test]
fn test_ppm_install_frozen_mode() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create project.toml and lock file
    let project_toml = r#"
[project]
name = "test-project"
version = "1.0.0"

[dependencies.javascript]
react = "^18.0.0"
"#;
    fs::write(project_path.join("project.toml"), project_toml).unwrap();
    
    let lock_file = r#"
{
  "version": 1,
  "project_hash": "abc123",
  "ppm_version": "1.0.0",
  "timestamp": "2025-09-11T10:00:00Z",
  "dependencies": {
    "javascript": [
      {"name": "react", "version": "18.2.0", "hash": "def456"}
    ],
    "python": []
  }
}
"#;
    fs::write(project_path.join("ppm.lock"), lock_file).unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["install", "--frozen"]);
        
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Installed"))
        .stdout(predicate::str::contains("packages"));
}

#[test]
fn test_ppm_install_offline_mode() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let project_toml = r#"
[project]
name = "test-project"
version = "1.0.0"

[dependencies.javascript]
some-new-package = "^1.0.0"
"#;
    fs::write(project_path.join("project.toml"), project_toml).unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["install", "--offline"]);
        
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not available offline"));
}

#[test]
fn test_ppm_install_no_symlinks() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let project_toml = r#"
[project]
name = "test-project"
version = "1.0.0"

[dependencies.javascript]
react = "^18.0.0"
"#;
    fs::write(project_path.join("project.toml"), project_toml).unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["install", "--no-symlinks"]);
        
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Installed"))
        .stdout(predicate::str::contains("global store only"));
}

#[test]
fn test_ppm_install_with_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let project_toml = r#"
[project]
name = "test-project"
version = "1.0.0"

[dependencies.javascript]
react = "^18.0.0"

[dependencies.python]
flask = "^2.0.0"
"#;
    fs::write(project_path.join("project.toml"), project_toml).unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["install", "--json"]);
        
    cmd.assert()
        .success();
        
    // Parse JSON output
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    
    assert_eq!(json["status"], "success");
    assert!(json["duration_ms"].is_number());
    assert!(json["packages_installed"].is_number());
    assert!(json["ecosystems"].is_object());
    assert_eq!(json["lock_file"], "ppm.lock");
}

#[test]
fn test_ppm_install_version_conflict() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create a project.toml with conflicting dependencies
    let project_toml = r#"
[project]
name = "test-project"
version = "1.0.0"

[dependencies.javascript]
package-a = "^1.0.0"
package-b = "^2.0.0"
# These would have conflicting sub-dependencies in a real scenario
"#;
    fs::write(project_path.join("project.toml"), project_toml).unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .arg("install");
        
    // This would fail in a real conflict scenario
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Cannot resolve dependencies"));
}

#[test]
fn test_ppm_install_validates_lock_file_format() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let project_toml = r#"
[project]
name = "test-project"
version = "1.0.0"

[dependencies.javascript]
react = "^18.0.0"
"#;
    fs::write(project_path.join("project.toml"), project_toml).unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .arg("install");
        
    cmd.assert()
        .success();
        
    // Verify generated lock file is valid JSON
    let lock_file_path = project_path.join("ppm.lock");
    let lock_content = fs::read_to_string(&lock_file_path).unwrap();
    
    // This should not panic if JSON is valid
    let parsed: serde_json::Value = serde_json::from_str(&lock_content)
        .expect("Generated ppm.lock should be valid JSON");
        
    // Verify expected structure
    assert!(parsed["version"].is_number());
    assert!(parsed["project_hash"].is_string());
    assert!(parsed["resolved_dependencies"].is_object());
}
