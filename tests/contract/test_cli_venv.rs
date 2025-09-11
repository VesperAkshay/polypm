use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

/// Contract tests for `ppm venv` command
/// These tests define the expected behavior and MUST FAIL until implementation is complete

#[test]
fn test_venv_create_default() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["python"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created Python virtual environment"))
        .stdout(predicate::str::contains(".ppm/venv"))
        .stdout(predicate::str::contains("Virtual environment ready"));
}

#[test]
fn test_venv_create_explicit() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["python"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "create"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created Python virtual environment"))
        .stdout(predicate::str::contains("Using Python"));
}

#[test]
fn test_venv_create_with_python_version() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["python"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "create", "--python", "3.11"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Using Python 3.11"));
}

#[test]
fn test_venv_create_with_custom_path() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["python"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "create", "--path", "custom-venv"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created Python virtual environment at custom-venv"));
}

#[test]
fn test_venv_create_force() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["python"]
"#).unwrap();

    // Create the venv directory first
    let venv_dir = temp_dir.path().join(".ppm/venv");
    fs::create_dir_all(&venv_dir).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "create", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created Python virtual environment"));
}

#[test]
fn test_venv_info() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["python"]
"#).unwrap();

    // First create a venv
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "create"])
        .assert()
        .success();

    // Then get info
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Python Virtual Environment:"))
        .stdout(predicate::str::contains("Path: .ppm/venv"))
        .stdout(predicate::str::contains("Status: Active"))
        .stdout(predicate::str::contains("Packages:"));
}

#[test]
fn test_venv_remove() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["python"]
"#).unwrap();

    // First create a venv
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "create"])
        .assert()
        .success();

    // Then remove it
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "remove"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed virtual environment"));
}

#[cfg(unix)]
#[test]
fn test_venv_shell_unix() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["python"]
"#).unwrap();

    // First create a venv
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "create"])
        .assert()
        .success();

    // Then activate shell (Unix only)
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "shell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Activating virtual environment"));
}

#[test]
fn test_venv_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["python"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "create", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""status": "success""#))
        .stdout(predicate::str::contains(r#""command": "create""#))
        .stdout(predicate::str::contains(r#""venv_path": ".ppm/venv""#))
        .stdout(predicate::str::contains(r#""python_version""#));
}

#[test]
fn test_venv_no_project_toml() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("No project.toml found"))
        .stderr(predicate::str::contains("run 'ppm init' first"));
}

#[test]
fn test_venv_python_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["python"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "create", "--python", "3.99"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Python 3.99 not found on system"));
}

#[test]
fn test_venv_already_exists() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["python"]
"#).unwrap();

    // Create the venv directory first
    let venv_dir = temp_dir.path().join(".ppm/venv");
    fs::create_dir_all(&venv_dir).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "create"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Virtual environment already exists"))
        .stderr(predicate::str::contains("use --force to recreate"));
}

#[test]
fn test_venv_not_found_for_remove() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["python"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "remove"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("No virtual environment found"));
}

#[test]
fn test_venv_not_found_for_info() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["python"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "info"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("No virtual environment found"));
}
