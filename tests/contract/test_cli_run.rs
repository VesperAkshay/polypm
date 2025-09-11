use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

/// Contract tests for `ppm run` command
/// These tests define the expected behavior and MUST FAIL until implementation is complete

#[test]
fn test_run_basic_script() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript"]

[scripts]
dev = "echo 'Development server starting'"
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "dev"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Development server starting"));
}

#[test]
fn test_run_script_with_args() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript"]

[scripts]
test = "echo 'Running tests'"
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "test", "--", "--verbose"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Running tests"));
}

#[test]
fn test_run_polyglot_script() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript", "python"]

[scripts]
full = "echo 'JS build' && echo 'Python deploy'"
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "full"])
        .assert()
        .success()
        .stdout(predicate::str::contains("JS build"))
        .stdout(predicate::str::contains("Python deploy"));
}

#[test]
fn test_run_list_scripts() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript", "python"]

[scripts]
dev = "npm run dev && python app.py"
test = "npm test && pytest"
build = "npm run build"
start = "python app.py --prod"
"#).unwrap();

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
fn test_run_show_env() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript", "python"]

[scripts]
dev = "echo 'Starting'"
"#).unwrap();

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
fn test_run_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript"]

[scripts]
quick = "echo 'Quick task'"
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "--json", "quick"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""status": "success""#))
        .stdout(predicate::str::contains(r#""script": "quick""#))
        .stdout(predicate::str::contains(r#""exit_code": 0"#));
}

#[test]
fn test_run_no_project_toml() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "dev"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("No project.toml found"))
        .stderr(predicate::str::contains("run 'ppm init' first"));
}

#[test]
fn test_run_script_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript"]

[scripts]
dev = "echo 'Development'"
test = "echo 'Testing'"
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "nonexistent"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Script 'nonexistent' not found"))
        .stderr(predicate::str::contains("Available scripts: dev, test"));
}

#[test]
fn test_run_no_scripts_defined() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "dev"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("No scripts defined in project.toml"));
}

#[test]
fn test_run_script_execution_failed() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript"]

[scripts]
failing = "exit 1"
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "failing"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_run_complex_environment() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript", "python"]

[scripts]
env_test = "echo $NODE_PATH && echo $PYTHONPATH"
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "env_test"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".ppm"));
}

#[test]
fn test_run_script_with_special_chars() {
    let temp_dir = TempDir::new().unwrap();
    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, r#"
[project]
name = "test-project"
ecosystems = ["javascript"]

[scripts]
complex = "echo 'Hello World' | grep Hello"
"#).unwrap();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "complex"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}
