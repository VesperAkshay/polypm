use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

/// Integration tests for script execution
/// Based on Scenario 3 from quickstart.md - cross-ecosystem script execution
/// These tests MUST FAIL until full implementation is complete

#[test]
fn test_script_list_available() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project with multiple scripts
    setup_script_project(&temp_dir);

    // List available scripts
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Available scripts:"))
        .stdout(predicate::str::contains("dev"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("start"));
}

#[test]
fn test_script_basic_execution() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_script_project(&temp_dir);

    // Install dependencies first
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Run basic script
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}

#[test]
fn test_script_cross_ecosystem_execution() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_script_project(&temp_dir);

    // Install dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Run script that uses both ecosystems
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "full-stack"])
        .assert()
        .success()
        .stdout(predicate::str::contains("JavaScript"))
        .stdout(predicate::str::contains("Python"));
}

#[test]
fn test_script_environment_variables() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_script_project(&temp_dir);

    // Install dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Show environment variables
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "--env", "dev"])
        .assert()
        .success()
        .stdout(predicate::str::contains("NODE_PATH"))
        .stdout(predicate::str::contains("PYTHONPATH"))
        .stdout(predicate::str::contains("VIRTUAL_ENV"));
}

#[test]
fn test_script_with_arguments() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_script_project(&temp_dir);

    // Install dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Run script with arguments
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "echo-args", "--", "--verbose", "--output", "test.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--verbose"))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("test.txt"));
}

#[test]
fn test_script_development_workflow() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_script_project(&temp_dir);

    // Install dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Run development script (simulates starting both React and Flask)
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "dev"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Development server")
            .or(predicate::str::contains("dev")));
}

#[test]
fn test_script_test_workflow() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_script_project(&temp_dir);

    // Install dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Run test script (runs both Jest and pytest)
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Testing")
            .or(predicate::str::contains("test")));
}

#[test]
fn test_script_build_workflow() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_script_project(&temp_dir);

    // Install dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Run build script with production flag
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "build", "--", "--prod"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Building")
            .or(predicate::str::contains("build"))
            .or(predicate::str::contains("--prod")));
}

#[test]
fn test_script_environment_isolation() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_script_project(&temp_dir);

    // Install dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Test that environment variables are properly set
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "check-env"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".ppm"));
}

#[test]
fn test_script_shell_operators() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_script_project(&temp_dir);

    // Install dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Test script with shell operators (&&, ||, |)
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "complex"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello")
            .and(predicate::str::contains("World")));
}

#[test]
fn test_script_exit_code_handling() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_script_project(&temp_dir);

    // Test successful script
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "success"])
        .assert()
        .success();

    // Test failing script
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "failure"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_script_json_output() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_script_project(&temp_dir);

    // Install dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Run script with JSON output
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "--json", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""status": "success""#))
        .stdout(predicate::str::contains(r#""script": "hello""#))
        .stdout(predicate::str::contains(r#""exit_code": 0"#));
}

#[test]
fn test_script_real_time_output() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_script_project(&temp_dir);

    // Install dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Run script that should stream output
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "streaming"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stream")
            .or(predicate::str::contains("output")));
}

#[test]
fn test_script_working_directory() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_script_project(&temp_dir);

    // Install dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Test that scripts run from project root
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "check-pwd"])
        .assert()
        .success()
        .stdout(predicate::str::contains(temp_dir.path().file_name().unwrap().to_str().unwrap())
            .or(predicate::str::contains("project root")));
}

#[test]
fn test_script_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_script_project(&temp_dir);

    // Test script not found
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Script 'nonexistent' not found"));

    // Test no scripts defined
    let project_toml = temp_dir.path().join("project.toml");
    let content = r#"
[project]
name = "no-scripts"
ecosystems = ["javascript"]
"#;
    fs::write(&project_toml, content).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "anything"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No scripts defined"));
}

// Helper function to setup a project with various scripts
fn setup_script_project(temp_dir: &TempDir) {
    // Create project.toml with comprehensive script examples
    let project_toml_content = r#"
[project]
name = "script-test-app"
version = "1.0.0"
ecosystems = ["javascript", "python"]

[dependencies.javascript]
react = "^18.2.0"

[dependencies.python]
flask = "^2.3.0"

[scripts]
hello = "echo 'Hello World'"
dev = "echo 'Development server starting'"
test = "echo 'Testing both ecosystems'"
build = "echo 'Building application'"
start = "echo 'Starting production server'"
full-stack = "echo 'JavaScript build' && echo 'Python deploy'"
echo-args = "echo"
check-env = "echo $NODE_PATH && echo $PYTHONPATH"
complex = "echo 'Hello' | grep Hello"
success = "exit 0"
failure = "exit 1"
streaming = "echo 'Stream output line 1' && echo 'Stream output line 2'"
check-pwd = "pwd"
"#;

    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, project_toml_content).unwrap();

    // Initialize the project
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(temp_dir)
        .args(&["init", "--force"])  // Force to overwrite the project.toml we just created
        .assert()
        .success();

    // Rewrite with our script content (since init might overwrite)
    fs::write(&project_toml, project_toml_content).unwrap();
}
