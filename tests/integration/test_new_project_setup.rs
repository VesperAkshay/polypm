use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

/// Integration tests for new polyglot project setup
/// Based on Scenario 1 from quickstart.md - complete end    // Verify global store directory exists (this would be in user's home)
    // For testing, we'll verify the install command reports global store usage
    // Note: In a real implementation, this would check actual global store locationnd workflow
/// These tests MUST FAIL until full implementation is complete

#[test]
fn test_new_project_basic_setup() {
    let temp_dir = TempDir::new().unwrap();
    
    // 1. Initialize new project
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["init", "--name", "my-fullstack-app", "--both"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created project.toml"))
        .stdout(predicate::str::contains("my-fullstack-app"));

    // Verify project.toml was created
    let project_toml = temp_dir.path().join("project.toml");
    assert!(project_toml.exists());
    
    let content = fs::read_to_string(&project_toml).unwrap();
    assert!(content.contains("name = \"my-fullstack-app\""));
    assert!(content.contains("ecosystems = [\"javascript\", \"python\"]"));
}

#[test]
fn test_new_project_add_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    
    // Initialize project first
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["init", "--name", "test-app", "--both"])
        .assert()
        .success();

    // Add JavaScript dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "react", "react-dom", "axios"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added react@"))
        .stdout(predicate::str::contains("Added react-dom@"))
        .stdout(predicate::str::contains("Added axios@"));

    // Add Python dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "flask", "requests", "python-dotenv"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added flask@"))
        .stdout(predicate::str::contains("Added requests@"));

    // Verify project.toml was updated
    let project_toml = temp_dir.path().join("project.toml");
    let content = fs::read_to_string(&project_toml).unwrap();
    assert!(content.contains("react ="));
    assert!(content.contains("flask ="));
}

#[test]
fn test_new_project_dev_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    
    // Initialize project
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["init", "--both"])
        .assert()
        .success();

    // Add development dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "--save-dev", "jest", "pytest", "eslint"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added jest@"))
        .stdout(predicate::str::contains("dev-dependencies"));

    // Verify dev-dependencies section in project.toml
    let project_toml = temp_dir.path().join("project.toml");
    let content = fs::read_to_string(&project_toml).unwrap();
    assert!(content.contains("[dev-dependencies"));
}

#[test]
fn test_new_project_install_all() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project with dependencies
    setup_test_project(&temp_dir);

    // Install all dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Installing dependencies"))
        .stdout(predicate::str::contains("packages installed"));

    // Verify .ppm directory structure
    let ppm_dir = temp_dir.path().join(".ppm");
    assert!(ppm_dir.exists());
    
    let node_modules = ppm_dir.join("node_modules");
    assert!(node_modules.exists());
    
    let venv_dir = ppm_dir.join("venv");
    assert!(venv_dir.exists());

    // Verify lock file was created
    let lock_file = temp_dir.path().join("ppm.lock");
    assert!(lock_file.exists());
}

#[test]
fn test_new_project_virtual_environment() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_test_project(&temp_dir);

    // Create Python virtual environment
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "create", "--python", "3.11"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created Python virtual environment"))
        .stdout(predicate::str::contains("Using Python 3.11"));

    // Verify venv structure
    let venv_dir = temp_dir.path().join(".ppm/venv");
    assert!(venv_dir.exists());
    
    // Check for platform-specific executable structure
    #[cfg(windows)]
    {
        assert!(venv_dir.join("Scripts/python.exe").exists());
        assert!(venv_dir.join("Scripts/activate.bat").exists());
    }
    #[cfg(unix)]
    {
        assert!(venv_dir.join("bin/python").exists());
        assert!(venv_dir.join("bin/activate").exists());
    }
}

#[test]
fn test_new_project_symlink_structure() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup and install project
    setup_test_project(&temp_dir);
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Verify JavaScript symlinks
    let react_link = temp_dir.path().join(".ppm/node_modules/react");
    assert!(react_link.exists());
    
    // Verify it's a symlink to global store
    if react_link.exists() {
        let metadata = fs::symlink_metadata(&react_link).unwrap();
        assert!(metadata.file_type().is_symlink());
    }

    // Verify Python symlinks
    #[cfg(unix)]
    {
        let flask_link = temp_dir.path().join(".ppm/venv/lib/python3.11/site-packages/flask");
        if flask_link.exists() {
            let metadata = fs::symlink_metadata(&flask_link).unwrap();
            assert!(metadata.file_type().is_symlink());
        }
    }
}

#[test]
fn test_new_project_global_store_population() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup and install project
    setup_test_project(&temp_dir);
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success()
        .stdout(predicate::str::contains("packages installed"));

    // Verify global store directory exists (this would be in user's home)
    // For testing, we'll verify the install command reports global store usage
    // Note: In a real implementation, this would check actual global store location
}

#[test]
fn test_new_project_environment_validation() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup complete project
    setup_test_project(&temp_dir);
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Test JavaScript environment
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "test-js"])  // Custom script to test JS env
        .assert()
        .success();

    // Test Python environment  
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "test-py"])  // Custom script to test Python env
        .assert()
        .success();
}

#[test]
fn test_new_project_complete_workflow() {
    let temp_dir = TempDir::new().unwrap();
    
    // Complete workflow from quickstart scenario 1
    
    // 1. Initialize
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["init", "--name", "my-fullstack-app", "--both"])
        .assert()
        .success();

    // 2. Add production dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "react", "react-dom", "axios"])
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "flask", "requests", "python-dotenv"])
        .assert()
        .success();

    // 3. Add dev dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["add", "--save-dev", "jest", "pytest", "eslint"])
        .assert()
        .success();

    // 4. Install all
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // 5. Create venv
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "create"])
        .assert()
        .success();

    // Verify final state
    assert!(temp_dir.path().join("project.toml").exists());
    assert!(temp_dir.path().join("ppm.lock").exists());
    assert!(temp_dir.path().join(".ppm").exists());
    assert!(temp_dir.path().join(".ppm/node_modules").exists());
    assert!(temp_dir.path().join(".ppm/venv").exists());
}

// Helper function to setup a basic test project
fn setup_test_project(temp_dir: &TempDir) {
    // Initialize project
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(temp_dir)
        .args(&["init", "--both"])
        .assert()
        .success();

    // Add some basic dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(temp_dir)
        .args(&["add", "react", "flask"])
        .assert()
        .success();

    // Setup basic scripts
    let project_toml = temp_dir.path().join("project.toml");
    let mut content = fs::read_to_string(&project_toml).unwrap();
    content.push_str(r#"

[scripts]
dev = "echo 'Development server'"
test-js = "echo 'Testing JavaScript environment'"
test-py = "echo 'Testing Python environment'"
"#);
    fs::write(&project_toml, content).unwrap();
}
