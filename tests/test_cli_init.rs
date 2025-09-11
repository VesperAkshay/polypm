// Contract test for `ppm init` command
// This test defines the expected behavior and MUST FAIL initially (TDD)

use std::fs;
use tempfile::TempDir;
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_ppm_init_basic_success() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .arg("init");
        
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Created project.toml"));
        
    // Verify project.toml was created
    let config_path = project_path.join("project.toml");
    assert!(config_path.exists(), "project.toml should be created");
    
    // Verify basic TOML structure
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("[project]"));
    assert!(content.contains("[dependencies.javascript]"));
    assert!(content.contains("[dependencies.python]"));
    assert!(content.contains("[scripts]"));
}

#[test]
fn test_ppm_init_with_custom_name_and_version() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["init", "--name", "my-test-app", "--version", "0.2.1"]);
        
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Created project.toml for my-test-app v0.2.1"));
        
    // Verify project.toml content
    let config_path = project_path.join("project.toml");
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("name = \"my-test-app\""));
    assert!(content.contains("version = \"0.2.1\""));
}

#[test]
fn test_ppm_init_javascript_only() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["init", "--javascript"]);
        
    cmd.assert()
        .success();
        
    // Verify only JavaScript dependencies section exists
    let config_path = project_path.join("project.toml");
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("[dependencies.javascript]"));
    assert!(!content.contains("[dependencies.python]"));
}

#[test]
fn test_ppm_init_python_only() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["init", "--python"]);
        
    cmd.assert()
        .success();
        
    // Verify only Python dependencies section exists
    let config_path = project_path.join("project.toml");
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("[dependencies.python]"));
    assert!(!content.contains("[dependencies.javascript]"));
}

#[test]
fn test_ppm_init_file_already_exists_without_force() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    let config_path = project_path.join("project.toml");
    
    // Create existing project.toml
    fs::write(&config_path, "existing content").unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .arg("init");
        
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("project.toml already exists (use --force to overwrite)"));
}

#[test]
fn test_ppm_init_file_already_exists_with_force() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    let config_path = project_path.join("project.toml");
    
    // Create existing project.toml
    fs::write(&config_path, "existing content").unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["init", "--force"]);
        
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Created project.toml"));
        
    // Verify file was overwritten with proper content
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("[project]"));
    assert!(content != "existing content");
}

#[test]
fn test_ppm_init_invalid_project_name() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["init", "--name", "invalid-name!"]);
        
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Invalid project name")
            .and(predicate::str::contains("must be valid identifier")));
}

#[test]
fn test_ppm_init_invalid_version() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["init", "--version", "1.x.y"]);
        
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Invalid version")
            .and(predicate::str::contains("must be valid semver")));
}

#[test]
fn test_ppm_init_with_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .args(&["init", "--json"]);
        
    cmd.assert()
        .success();
        
    // Parse JSON output
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    
    assert_eq!(json["status"], "success");
    assert!(json["project_name"].is_string());
    assert!(json["project_version"].is_string());
    assert_eq!(json["config_path"], "./project.toml");
    assert!(json["ecosystems"].is_array());
}

#[test]
fn test_ppm_init_uses_directory_name_as_default() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test-project-123";
    let project_path = temp_dir.path().join(project_name);
    fs::create_dir_all(&project_path).unwrap();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&project_path)
        .arg("init");
        
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!("Created project.toml for {} v1.0.0", project_name)));
        
    // Verify project name in TOML
    let config_path = project_path.join("project.toml");
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains(&format!("name = \"{}\"", project_name)));
}

#[test]  
fn test_ppm_init_validates_toml_syntax() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(project_path)
        .arg("init");
        
    cmd.assert()
        .success();
        
    // Verify generated TOML can be parsed
    let config_path = project_path.join("project.toml");
    let content = fs::read_to_string(&config_path).unwrap();
    
    // This should not panic if TOML is valid
    let parsed: toml::Value = toml::from_str(&content)
        .expect("Generated project.toml should be valid TOML");
        
    // Verify expected structure
    assert!(parsed["project"].is_table());
    assert!(parsed["dependencies"].is_table());
    assert!(parsed["scripts"].is_table());
}
