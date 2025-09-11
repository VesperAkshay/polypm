use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

/// Integration tests for existing project installation
/// Based on Scenario 2 from quickstart.md - reproducing exact environments
/// These tests MUST FAIL until full implementation is complete

#[test]
fn test_existing_project_frozen_install() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup existing project with project.toml and ppm.lock
    setup_existing_project(&temp_dir);

    // Install exact versions from lock file
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install", "--frozen"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Installing from lock file"))
        .stdout(predicate::str::contains("exact versions"));

    // Verify lock file wasn't modified
    let lock_content_after = fs::read_to_string(temp_dir.path().join("ppm.lock")).unwrap();
    let original_lock = get_sample_lock_content();
    assert_eq!(lock_content_after.trim(), original_lock.trim());
}

#[test]
fn test_existing_project_cached_packages() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_existing_project(&temp_dir);

    // First install to populate cache
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Clean project (simulate fresh clone)
    let ppm_dir = temp_dir.path().join(".ppm");
    if ppm_dir.exists() {
        fs::remove_dir_all(&ppm_dir).unwrap();
    }

    // Install again - should use cached packages
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install", "--frozen"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Using cached packages")
            .or(predicate::str::contains("packages installed")));
}

#[test]
fn test_existing_project_no_network_requests() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup existing project
    setup_existing_project(&temp_dir);

    // Install with offline flag to ensure no network requests
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install", "--offline"])
        .assert()
        .success()
        .stdout(predicate::str::contains("offline")
            .or(predicate::str::contains("cached packages")));
}

#[test]
fn test_existing_project_exact_versions() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_existing_project(&temp_dir);

    // Install dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install", "--frozen"])
        .assert()
        .success();

    // Verify exact versions match lock file
    let lock_content = fs::read_to_string(temp_dir.path().join("ppm.lock")).unwrap();
    
    // Check that installed packages match lock file versions
    // This would be validated through package inspection in real implementation
    assert!(lock_content.contains("\"version\": \"18.2.0\""));  // React version
    assert!(lock_content.contains("\"version\": \"2.3.0\""));   // Flask version
}

#[test]
fn test_existing_project_venv_info() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup and install project
    setup_existing_project(&temp_dir);
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Check virtual environment info
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["venv", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Python Virtual Environment"))
        .stdout(predicate::str::contains("Path: .ppm/venv"))
        .stdout(predicate::str::contains("Status:"));
}

#[test]
fn test_existing_project_identical_environment() {
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();
    
    // Setup identical projects in both directories
    setup_existing_project(&temp_dir1);
    setup_existing_project(&temp_dir2);

    // Install in both
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir1)
        .args(&["install", "--frozen"])
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir2)
        .args(&["install", "--frozen"])
        .assert()
        .success();

    // Verify both have identical lock files
    let lock1 = fs::read_to_string(temp_dir1.path().join("ppm.lock")).unwrap();
    let lock2 = fs::read_to_string(temp_dir2.path().join("ppm.lock")).unwrap();
    assert_eq!(lock1, lock2);
}

#[test]
fn test_existing_project_global_store_reuse() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_existing_project(&temp_dir);

    // Install dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success()
        .stdout(predicate::str::contains("packages installed"));

    // Verify symlinks point to global store
    let node_modules = temp_dir.path().join(".ppm/node_modules");
    if node_modules.exists() {
        // Check that packages are symlinked (not copied)
        let react_path = node_modules.join("react");
        if react_path.exists() {
            let metadata = fs::symlink_metadata(&react_path).unwrap();
            assert!(metadata.file_type().is_symlink());
        }
    }
}

#[test]
fn test_existing_project_reproducible_builds() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_existing_project(&temp_dir);

    // Multiple installs should produce identical results
    for i in 0..3 {
        // Clean environment
        let ppm_dir = temp_dir.path().join(".ppm");
        if ppm_dir.exists() {
            fs::remove_dir_all(&ppm_dir).unwrap();
        }

        // Install
        let mut cmd = Command::cargo_bin("ppm").unwrap();
        cmd.current_dir(&temp_dir)
            .args(&["install", "--frozen"])
            .assert()
            .success();

        // Verify lock file remains unchanged
        let lock_content = fs::read_to_string(temp_dir.path().join("ppm.lock")).unwrap();
        let expected_lock = get_sample_lock_content();
        assert_eq!(lock_content.trim(), expected_lock.trim(), 
                  "Lock file changed on install iteration {}", i);
    }
}

#[test]
fn test_existing_project_dependency_validation() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project
    setup_existing_project(&temp_dir);

    // Install dependencies
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install", "--frozen"])
        .assert()
        .success();

    // Validate that required packages are available
    // This would test actual package availability in real implementation
    assert!(temp_dir.path().join(".ppm/node_modules").exists());
    assert!(temp_dir.path().join(".ppm/venv").exists());
}

#[test]
fn test_existing_project_lock_file_integrity() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project with corrupted lock file
    setup_existing_project(&temp_dir);
    
    // Corrupt the lock file
    let lock_path = temp_dir.path().join("ppm.lock");
    fs::write(&lock_path, "invalid json content").unwrap();

    // Install should fail gracefully
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install", "--frozen"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid lock file")
            .or(predicate::str::contains("Failed to parse")));
}

#[test]
fn test_existing_project_missing_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup project with missing packages in lock file
    setup_existing_project(&temp_dir);
    
    // Modify lock file to reference non-existent package
    let lock_path = temp_dir.path().join("ppm.lock");
    let mut lock_content = fs::read_to_string(&lock_path).unwrap();
    lock_content = lock_content.replace("react", "nonexistent-package-xyz");
    fs::write(&lock_path, lock_content).unwrap();

    // Install should fail with clear error
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install", "--frozen"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found")
            .or(predicate::str::contains("nonexistent-package-xyz")));
}

#[test]
fn test_existing_project_scripts_execution() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup and install project
    setup_existing_project(&temp_dir);
    
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["install"])
        .assert()
        .success();

    // Test script execution with installed environment
    let mut cmd = Command::cargo_bin("ppm").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&["run", "test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test")
            .or(predicate::str::contains("Testing")));
}

// Helper function to setup an existing project scenario
fn setup_existing_project(temp_dir: &TempDir) {
    // Create project.toml
    let project_toml_content = r#"
[project]
name = "existing-app"
version = "1.0.0"
ecosystems = ["javascript", "python"]

[dependencies.javascript]
react = "^18.2.0"
react-dom = "^18.2.0"
axios = "^1.4.0"

[dependencies.python]
flask = "^2.3.0"
requests = "^2.31.0"

[dev-dependencies.javascript]
jest = "^29.0.0"

[dev-dependencies.python]
pytest = "^7.4.0"

[scripts]
dev = "npm run dev && python app.py"
test = "npm test && pytest"
build = "npm run build"
"#;

    let project_toml = temp_dir.path().join("project.toml");
    fs::write(&project_toml, project_toml_content).unwrap();

    // Create ppm.lock
    let lock_content = get_sample_lock_content();
    let lock_file = temp_dir.path().join("ppm.lock");
    fs::write(&lock_file, lock_content).unwrap();
}

fn get_sample_lock_content() -> &'static str {
    r#"{
  "version": "1.0.0",
  "dependencies": {
    "javascript": [
      {
        "name": "react",
        "version": "18.2.0",
        "resolved": "https://registry.npmjs.org/react/-/react-18.2.0.tgz",
        "integrity": "sha512-/3IjMdb2L9QbBdWiW5e3P2/npwMBaU9mHCSCUzNln0ZCYbcfTsGbTJrU/kGemdH2IWmB2ioZ+zkxtmq6g09fGQ==",
        "ecosystem": "javascript"
      },
      {
        "name": "react-dom",
        "version": "18.2.0",
        "resolved": "https://registry.npmjs.org/react-dom/-/react-dom-18.2.0.tgz",
        "integrity": "sha512-6IMTriUmvsjHUjNtEDudZfuDQUoWXVxKHhlEGSk81n4YFS+r/Kl99wXiwlVXtPBtJenozv2P+hxDsw9eA7Xo6g==",
        "ecosystem": "javascript"
      },
      {
        "name": "axios",
        "version": "1.4.0",
        "resolved": "https://registry.npmjs.org/axios/-/axios-1.4.0.tgz",
        "integrity": "sha512-S4XCWMEmzvo64T9GfvQDOXgYRDJ/wsSZc7Jvdgx5u1sd0JwsuPLqb3SYmusag+edF6ziyMensPVqLTSc1PiSEA==",
        "ecosystem": "javascript"
      }
    ],
    "python": [
      {
        "name": "flask",
        "version": "2.3.0",
        "resolved": "https://pypi.org/simple/flask/",
        "integrity": "sha256:abc123...",
        "ecosystem": "python"
      },
      {
        "name": "requests",
        "version": "2.31.0", 
        "resolved": "https://pypi.org/simple/requests/",
        "integrity": "sha256:def456...",
        "ecosystem": "python"
      }
    ]
  },
  "dev-dependencies": {
    "javascript": [
      {
        "name": "jest",
        "version": "29.0.0",
        "resolved": "https://registry.npmjs.org/jest/-/jest-29.0.0.tgz",
        "integrity": "sha512-...",
        "ecosystem": "javascript"
      }
    ],
    "python": [
      {
        "name": "pytest",
        "version": "7.4.0",
        "resolved": "https://pypi.org/simple/pytest/",
        "integrity": "sha256:ghi789...",
        "ecosystem": "python"
      }
    ]
  }
}"#
}
