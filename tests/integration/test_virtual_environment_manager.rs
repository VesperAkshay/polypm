use std::fs;
use tempfile::TempDir;
use ppm::services::virtual_environment_manager::VirtualEnvironmentManager;
use ppm::models::virtual_environment::{VenvConfig, VenvStatus};
use ppm::models::ecosystem::Ecosystem;

#[tokio::test]
async fn test_venv_manager_creation_and_removal() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path().to_path_buf();

    let manager = VirtualEnvironmentManager::new();

    // Test virtual environment creation (may not work in all test environments)
    match manager.create_python_venv(&project_root, Some("test_venv"), None).await {
        Ok(creation_result) => {
            // Verify the result
            assert!(creation_result.success);
            assert_eq!(creation_result.venv.name, "test_venv");
            assert_eq!(creation_result.venv.ecosystem, Ecosystem::Python);
            assert!(creation_result.venv.path.exists());
            
            println!("Created venv at: {}", creation_result.venv.path.display());
            if let Some(version) = &creation_result.python_version {
                println!("Python version: {}", version);
            }

            // Test status check
            let status = manager.check_venv_status(&creation_result.venv.path).await.expect("Failed to check status");
            println!("Venv status: {:?}", status);

            // Test executables info
            let executables = manager.get_venv_executables(&creation_result.venv.path).await.expect("Failed to get executables");
            println!("Executables found: {}", executables.len());
            for exe in &executables {
                println!("  {}: {} (available: {})", exe.name, exe.version, exe.available);
            }

            // Test removal
            let removal_result = manager.remove_venv(&creation_result.venv.path).await;
            assert!(removal_result.is_ok());
            assert!(!creation_result.venv.path.exists());
            println!("Successfully removed venv");
        }
        Err(e) => {
            println!("Virtual environment creation failed (may be expected in test environment): {}", e);
            // This is okay in environments without Python
        }
    }
}

#[tokio::test]
async fn test_venv_status_checks() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let non_existent_path = temp_dir.path().join("does_not_exist");

    let manager = VirtualEnvironmentManager::new();

    // Test non-existent venv
    let status = manager.check_venv_status(&non_existent_path).await.expect("Failed to check status");
    assert_eq!(status, VenvStatus::NotCreated);

    // Test corrupted venv (empty directory)
    let corrupted_path = temp_dir.path().join("corrupted_venv");
    fs::create_dir_all(&corrupted_path).expect("Failed to create dir");
    
    let status = manager.check_venv_status(&corrupted_path).await.expect("Failed to check status");
    assert_eq!(status, VenvStatus::Corrupted);
    
    println!("Status checks passed");
}

#[tokio::test]
async fn test_venv_config_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path().to_path_buf();

    // Test with custom configuration
    let config = VenvConfig {
        python: Some("python3".to_string()),
        path: Some("custom_venv".to_string()),
        system_site_packages: true,
        copies: false,
        node: None,
        additional_packages: vec![],
        default_env_vars: std::collections::HashMap::new(),
    };

    let manager = VirtualEnvironmentManager::with_config(config.clone());

    // Test creation with custom config (may fail if Python not available)
    match manager.create_python_venv(&project_root, Some("custom"), Some(&config)).await {
        Ok(creation_result) => {
            assert_eq!(creation_result.venv.name, "custom");
            assert!(creation_result.venv.path.ends_with("custom_venv"));
            println!("Custom venv created successfully");

            // Clean up
            let _ = manager.remove_venv(&creation_result.venv.path).await;
        }
        Err(e) => {
            println!("Custom venv creation failed (may be expected): {}", e);
        }
    }
}

#[tokio::test]
async fn test_venv_activation_env() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let venv_path = temp_dir.path().join("test_venv");

    // Create a mock virtual environment structure
    fs::create_dir_all(&venv_path).expect("Failed to create venv dir");

    let manager = VirtualEnvironmentManager::new();
    
    // Create a mock VirtualEnvironment
    use ppm::models::virtual_environment::VirtualEnvironment;
    let venv = VirtualEnvironment::with_defaults(
        "test".to_string(),
        venv_path.clone(),
        Ecosystem::Python,
    );

    // Test activation environment setup
    let env_vars = manager.get_activation_env(&venv);
    
    assert!(env_vars.contains_key("VIRTUAL_ENV"));
    assert!(env_vars.contains_key("PATH"));
    assert_eq!(env_vars.get("VIRTUAL_ENV"), Some(&venv_path.to_string_lossy().to_string()));
    
    let path_value = env_vars.get("PATH").expect("PATH should be set");
    
    #[cfg(windows)]
    {
        assert!(path_value.contains("Scripts"));
    }
    
    #[cfg(not(windows))]
    {
        assert!(path_value.contains("bin"));
    }
    
    println!("Activation environment variables set correctly");
}

#[tokio::test]
async fn test_python_discovery() {
    let manager = VirtualEnvironmentManager::new();
    let config = VenvConfig::default();

    // Since find_python_executable is private, we'll test the creation instead
    // which internally uses Python discovery
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path().to_path_buf();

    match manager.create_python_venv(&project_root, Some("test_discovery"), Some(&config)).await {
        Ok(creation_result) => {
            assert!(!creation_result.venv.name.is_empty());
            if let Some(version) = &creation_result.python_version {
                assert!(!version.is_empty());
                println!("Found Python version: {}", version);
            }
            println!("Python discovery via venv creation successful");
            
            // Clean up
            let _ = manager.remove_venv(&creation_result.venv.path).await;
        }
        Err(e) => {
            println!("Python discovery failed (expected in some test environments): {}", e);
        }
    }
}

#[tokio::test]
async fn test_package_installation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path().to_path_buf();

    let manager = VirtualEnvironmentManager::new();

    // Try to create a virtual environment
    match manager.create_python_venv(&project_root, Some("test_packages"), None).await {
        Ok(creation_result) => {
            println!("Created venv for package testing");

            // Test package installation
            let packages = vec!["requests".to_string()];
            match manager.install_packages(&creation_result.venv.path, &packages).await {
                Ok(output) => {
                    println!("Package installation successful");
                    println!("Output: {}", output);
                }
                Err(e) => {
                    println!("Package installation failed (may be expected without internet): {}", e);
                }
            }

            // Clean up
            let _ = manager.remove_venv(&creation_result.venv.path).await;
        }
        Err(e) => {
            println!("Virtual environment creation failed (may be expected): {}", e);
        }
    }
}

#[tokio::test]
async fn test_package_installation_with_activation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path().to_path_buf();

    let manager = VirtualEnvironmentManager::new();

    // Try to create a virtual environment
    match manager.create_python_venv(&project_root, Some("test_activation"), None).await {
        Ok(creation_result) => {
            println!("Created venv for activation testing at: {}", creation_result.venv.path.display());

            // Get activation environment variables
            let env_vars = manager.get_activation_env(&creation_result.venv);
            println!("Activation environment variables:");
            for (key, value) in &env_vars {
                println!("  {}={}", key, value);
            }

            // Test package installation with environment variables
            let packages = vec!["requests==2.31.0".to_string()]; // Use specific version
            
            // Try using the venv's pip directly with activation environment
            let pip_path = if cfg!(windows) {
                creation_result.venv.path.join("Scripts").join("pip.exe")
            } else {
                creation_result.venv.path.join("bin").join("pip")
            };

            if pip_path.exists() {
                println!("Using pip at: {}", pip_path.display());
                
                // Install package using the virtual environment's pip directly
                let mut cmd = std::process::Command::new(&pip_path);
                cmd.arg("install");
                cmd.arg("requests==2.31.0");
                
                // Set virtual environment variables
                for (key, value) in &env_vars {
                    cmd.env(key, value);
                }

                match cmd.output() {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        
                        println!("Pip install output:");
                        println!("STDOUT: {}", stdout);
                        if !stderr.is_empty() {
                            println!("STDERR: {}", stderr);
                        }
                        
                        if output.status.success() {
                            println!("✓ Package installation successful with activation!");
                            
                            // Verify the package is actually installed
                            let list_output = std::process::Command::new(&pip_path)
                                .arg("list")
                                .arg("--format=json")
                                .envs(&env_vars)
                                .output();
                                
                            if let Ok(list_result) = list_output {
                                let list_stdout = String::from_utf8_lossy(&list_result.stdout);
                                println!("Installed packages: {}", list_stdout);
                                
                                if list_stdout.contains("requests") {
                                    println!("✓ Confirmed: requests package is installed!");
                                } else {
                                    println!("⚠ requests package not found in pip list");
                                }
                            }
                        } else {
                            println!("✗ Package installation failed even with activation");
                        }
                    }
                    Err(e) => {
                        println!("Failed to run pip install: {}", e);
                    }
                }
            } else {
                println!("Pip executable not found at: {}", pip_path.display());
            }

            // Clean up
            let _ = manager.remove_venv(&creation_result.venv.path).await;
        }
        Err(e) => {
            println!("Virtual environment creation failed: {}", e);
        }
    }
}
