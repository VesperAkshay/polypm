use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

/// Contract tests for `ppm add` command
/// These tests define the expected behavior and MUST FAIL until implementation is complete

#[test]
fn test_add_basic_packages() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript", "python"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "react", "flask"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added react@"))
        .stdout(predicate::str::contains("Added flask@"))
        .stdout(predicate::str::contains("Updated project.toml"))
        .stdout(predicate::str::contains("Installed 2 packages"));
}

#[test]
fn test_add_save_dev() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript", "python"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "--save-dev", "jest", "pytest"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added jest@"))
        .stdout(predicate::str::contains("Added pytest@"))
        .stdout(predicate::str::contains("dev-dependencies"));
}

#[test]
fn test_add_explicit_javascript() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "--javascript", "react@18.2.0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added react@18.2.0"))
        .stdout(predicate::str::contains("JavaScript dependencies"));
}

#[test]
fn test_add_explicit_python() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["python"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "--python", "flask>=2.0.0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added flask@"))
        .stdout(predicate::str::contains("Python dependencies"));
}

#[test]
fn test_add_version_constraint() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "--version", "^18.0.0", "react"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added react@^18.0.0"));
}

#[test]
fn test_add_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "--json", "react"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""status": "success""#))
        .stdout(predicate::str::contains(r#""packages_added""#))
        .stdout(predicate::str::contains(r#""name": "react""#));
}

#[test]
fn test_add_no_project_toml() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "react"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("No project.toml found"))
        .stderr(predicate::str::contains("run 'ppm init' first"));
}

#[test]
fn test_add_no_packages_specified() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("No packages specified"))
        .stderr(predicate::str::contains("usage: ppm add <packages...>"));
}

#[test]
fn test_add_package_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "nonexistent-package-xyz"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Package 'nonexistent-package-xyz' not found"));
}

#[test]
fn test_add_ecosystem_detection_failed() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript", "python"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "ambiguous-name"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Could not detect ecosystem"))
        .stderr(predicate::str::contains("use --javascript or --python"));
}

#[test]
fn test_add_invalid_version_constraint() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "--version", "invalid-version", "react"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Invalid version constraint 'invalid-version'"));
}

#[test]
fn test_add_package_already_exists() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript"]

[dependencies.javascript]
react = "^18.0.0"
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "react"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Package 'react' already exists"))
        .stderr(predicate::str::contains("use ppm update to change version"));
}
