use clap::Subcommand;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use serde_json::json;

use crate::models::project::{Project, ProjectToml};
use crate::models::ecosystem::Ecosystem;
use crate::models::virtual_environment::{VenvConfig, VenvStatus};
use crate::services::virtual_environment_manager::VirtualEnvironmentManager;
use crate::utils::error::{PpmError, Result};

/// Virtual environment management commands
#[derive(Debug, Subcommand)]
pub enum VenvCommands {
    /// Create new virtual environment (default)
    Create {
        /// Python version to use
        #[arg(long)]
        python: Option<String>,
        /// Custom path for venv
        #[arg(long)]
        path: Option<String>,
        /// Remove existing venv before creating
        #[arg(long)]
        force: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Remove existing virtual environment
    Remove {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show virtual environment information
    Info {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Activate venv in current shell (Unix only)
    Shell,
}

/// Main venv command handler
pub struct VenvHandler {
    pub command: Option<VenvCommands>,
}

impl VenvHandler {
    /// Execute the venv command
    pub async fn execute(&self) -> Result<()> {
        // Load project configuration
        let project = self.load_project()?;

        // Check if Python is supported
        if !project.dependencies.contains_key(&Ecosystem::Python) &&
           !project.dev_dependencies.contains_key(&Ecosystem::Python) &&
           !project.ecosystems.contains(&Ecosystem::Python) {
            return Err(PpmError::ValidationError(
                "This project does not use Python.\n\nTo add Python support:\n  1. Add Python dependencies: ppm add requests --python\n  2. Or manually edit project.toml to include:\n     ecosystems = [\"python\"]\n     [dependencies.python]\n     requests = \"*\"".to_string()
            ));
        }

        // Handle subcommands or default to create
        match &self.command {
            Some(VenvCommands::Create { python, path, force, json }) => {
                self.create_venv(python.as_deref(), path.as_deref(), *force, *json).await
            }
            Some(VenvCommands::Remove { json }) => {
                self.remove_venv(*json).await
            }
            Some(VenvCommands::Info { json }) => {
                self.info_venv(*json).await
            }
            Some(VenvCommands::Shell) => {
                self.shell_venv().await
            }
            None => {
                // Default to create
                self.create_venv(None, None, false, false).await
            }
        }
    }

    /// Normalize path for consistent display (always use forward slashes)
    fn normalize_path_display(&self, path: &Path) -> String {
        path.to_string_lossy().replace('\\', "/")
    }

    /// Load project configuration
    fn load_project(&self) -> Result<Project> {
        let project_path = PathBuf::from("project.toml");
        if !project_path.exists() {
            return Err(PpmError::ConfigError(
                "No project.toml found in current directory.\n\nTo initialize a new PPM project:\n  ppm init\n\nOr navigate to an existing PPM project directory.".to_string()
            ));
        }

        let content = fs::read_to_string("project.toml")
            .map_err(|e| PpmError::IoError(e))?;
        
        let project_toml: ProjectToml = toml::from_str(&content)
            .map_err(|e| PpmError::ConfigError(format!("Invalid project.toml: {}", e)))?;
        
        let project = Project::from(project_toml);
        Ok(project)
    }

    /// Create a new virtual environment
    async fn create_venv(&self, python_version: Option<&str>, custom_path: Option<&str>, force: bool, json: bool) -> Result<()> {
        let venv_path = if let Some(path) = custom_path {
            PathBuf::from(path)
        } else {
            PathBuf::from(".venv")  // Changed from ".ppm/venv" to match VenvConfig default
        };

        // Check if venv already exists
        if venv_path.exists() && !force {
            let error_msg = format!("Virtual environment already exists at {}.\n\nOptions:\n  1. Use existing venv: ppm venv info\n  2. Recreate venv: ppm venv create --force\n  3. Remove venv: ppm venv remove", 
                self.normalize_path_display(&venv_path));
            if json {
                let response = json!({
                    "status": "error",
                    "command": "create",
                    "error": error_msg
                });
                println!("{}", serde_json::to_string_pretty(&response)
                    .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
            } else {
                eprintln!("Error: {}", error_msg);
            }
            return Err(PpmError::ValidationError(error_msg));
        }

        // Remove existing venv if force is specified
        if venv_path.exists() && force {
            fs::remove_dir_all(&venv_path)
                .map_err(|e| PpmError::IoError(e))?;
        }

        // Create parent directory
        if let Some(parent) = venv_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PpmError::IoError(e))?;
        }

        // Create VenvConfig
        let mut config = VenvConfig::default();
        if let Some(python) = python_version {
            config.python = Some(python.to_string());
        }
        if let Some(path) = custom_path {
            config.path = Some(path.to_string());
        }

        // Create VirtualEnvironmentManager
        let manager = VirtualEnvironmentManager::with_config(config.clone());

        // Get project root
        let project_root = std::env::current_dir()
            .map_err(|e| PpmError::IoError(e))?;

        // Create virtual environment using the manager
        match manager.create_python_venv(&project_root, Some("default"), Some(&config)).await {
            Ok(creation_result) => {
                if json {
                    let response = json!({
                        "status": "success",
                        "command": "create",
                        "venv_path": self.normalize_path_display(&creation_result.venv.path),
                        "python_version": creation_result.python_version,
                        "name": creation_result.venv.name,
                        "ecosystem": creation_result.venv.ecosystem
                    });
                    println!("{}", serde_json::to_string_pretty(&response)
                        .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
                } else {
                    println!("Created Python virtual environment at {}", self.normalize_path_display(&creation_result.venv.path));
                    if let Some(version) = &creation_result.python_version {
                        println!("Using Python {}", version);
                    }
                    println!("Virtual environment ready for use");
                }
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to create virtual environment: {}", e);
                if json {
                    let response = json!({
                        "status": "error",
                        "command": "create",
                        "error": error_msg
                    });
                    println!("{}", serde_json::to_string_pretty(&response)
                        .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
                } else {
                    eprintln!("Error: {}", error_msg);
                }
                Err(e)
            }
        }
    }

    /// Remove virtual environment
    async fn remove_venv(&self, json: bool) -> Result<()> {
        let venv_path = PathBuf::from(".venv");  // Changed from ".ppm/venv" to match VenvConfig default

        if !venv_path.exists() {
            let error_msg = "No virtual environment found at .venv";  // Updated error message
            if json {
                let response = json!({
                    "status": "error",
                    "command": "remove",
                    "error": error_msg
                });
                println!("{}", serde_json::to_string_pretty(&response)
                    .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
            } else {
                eprintln!("Error: {}", error_msg);
            }
            return Err(PpmError::ValidationError(error_msg.to_string()));
        }

        // Use VirtualEnvironmentManager to remove the venv
        let manager = VirtualEnvironmentManager::new();
        match manager.remove_venv(&venv_path).await {
            Ok(message) => {
                if json {
                    let response = json!({
                        "status": "success",
                        "command": "remove",
                        "removed_path": self.normalize_path_display(&venv_path),
                        "message": message
                    });
                    println!("{}", serde_json::to_string_pretty(&response)
                        .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
                } else {
                    println!("Removed virtual environment at {}", self.normalize_path_display(&venv_path));
                }
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to remove virtual environment: {}", e);
                if json {
                    let response = json!({
                        "status": "error",
                        "command": "remove",
                        "error": error_msg
                    });
                    println!("{}", serde_json::to_string_pretty(&response)
                        .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
                } else {
                    eprintln!("Error: {}", error_msg);
                }
                Err(e)
            }
        }
    }

    /// Show virtual environment information
    async fn info_venv(&self, json: bool) -> Result<()> {
        let venv_path = PathBuf::from(".venv");  // Changed from ".ppm/venv" to match VenvConfig default

        if !venv_path.exists() {
            let error_msg = "No virtual environment found at .venv";  // Updated error message
            if json {
                let response = json!({
                    "status": "error",
                    "command": "info",
                    "error": error_msg
                });
                println!("{}", serde_json::to_string_pretty(&response)
                    .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
            } else {
                eprintln!("Error: {}", error_msg);
            }
            return Err(PpmError::ValidationError(error_msg.to_string()));
        }

        // Use VirtualEnvironmentManager to get info
        let manager = VirtualEnvironmentManager::new();
        
        // Check venv status
        let status = manager.check_venv_status(&venv_path).await?;
        let status_str = match status {
            VenvStatus::NotCreated => "Not Created",
            VenvStatus::Active => "Active", 
            VenvStatus::Inactive => "Inactive",
            VenvStatus::Corrupted => "Corrupted",
        };

        // Get executables info
        let executables = manager.get_venv_executables(&venv_path).await?;
        
        // Find Python and pip info
        let python_info = executables.iter().find(|e| e.name == "python");
        let pip_info = executables.iter().find(|e| e.name == "pip");

        // Get installed packages (simplified)
        let packages = if status == VenvStatus::Inactive || status == VenvStatus::Active {
            self.get_installed_packages(&venv_path).unwrap_or_default()
        } else {
            HashMap::new()
        };

        if json {
            let response = json!({
                "status": "success",
                "command": "info",
                "venv_path": self.normalize_path_display(&venv_path),
                "python_executable": python_info.map(|p| self.normalize_path_display(&p.path)).unwrap_or_else(|| "Not found".to_string()),
                "python_version": python_info.map(|p| p.version.clone()).unwrap_or_else(|| "Unknown".to_string()),
                "pip_version": pip_info.map(|p| p.version.clone()).unwrap_or_else(|| "Unknown".to_string()),
                "status": status_str,
                "packages": packages
            });
            println!("{}", serde_json::to_string_pretty(&response)
                .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
        } else {
            println!("Python Virtual Environment:");
            println!("  Path: {}", self.normalize_path_display(&venv_path));
            if let Some(python) = python_info {
                println!("  Python: {} ({})", python.version, if python.available { "available" } else { "missing" });
            }
            if let Some(pip) = pip_info {
                println!("  Pip: {} ({})", pip.version, if pip.available { "available" } else { "missing" });
            }
            println!("  Status: {}", status_str);
            println!("  Packages: {} installed", packages.len());
            
            if !packages.is_empty() {
                for (name, version) in packages.iter().take(5) {
                    println!("    {} {}", name, version);
                }
                if packages.len() > 5 {
                    println!("    ... and {} more", packages.len() - 5);
                }
            }
        }

        Ok(())
    }

    /// Activate virtual environment in shell (Unix only)
    async fn shell_venv(&self) -> Result<()> {
        let venv_path = PathBuf::from(".venv");  // Changed from ".ppm/venv" to match VenvConfig default

        if !venv_path.exists() {
            return Err(PpmError::ValidationError(
                "No virtual environment found at .venv".to_string()  // Updated error message
            ));
        }

        if cfg!(target_os = "windows") {
            println!("Shell activation is not supported on Windows.");
            println!("To activate manually, run:");
            println!("  .venv\\Scripts\\activate.bat");  // Updated path
            return Ok(());
        }

        // Unix shell activation
        let activate_script = venv_path.join("bin").join("activate");
        if !activate_script.exists() {
            return Err(PpmError::ValidationError(
                "Virtual environment activation script not found".to_string()
            ));
        }

        println!("Activating virtual environment...");
        println!("To activate in your current shell, run:");
        println!("  source {}", activate_script.display());
        println!("To deactivate, run:");
        println!("  deactivate");

        Ok(())
    }

    /// Get Python executable path within virtual environment
    fn get_venv_python_executable(&self, venv_path: &Path) -> PathBuf {
        if cfg!(target_os = "windows") {
            venv_path.join("Scripts").join("python.exe")
        } else {
            venv_path.join("bin").join("python")
        }
    }

    /// Get installed packages in virtual environment
    fn get_installed_packages(&self, venv_path: &Path) -> Result<HashMap<String, String>> {
        let python_exe = self.get_venv_python_executable(venv_path);
        
        if !python_exe.exists() {
            return Ok(HashMap::new());
        }

        // Use pip list to get installed packages
        let output = Command::new(&python_exe)
            .args(&["-m", "pip", "list", "--format=json"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let json_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(packages) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) {
                    let mut result = HashMap::new();
                    for package in packages {
                        if let (Some(name), Some(version)) = (
                            package.get("name").and_then(|n| n.as_str()),
                            package.get("version").and_then(|v| v.as_str())
                        ) {
                            result.insert(name.to_string(), version.to_string());
                        }
                    }
                    return Ok(result);
                }
            }
            _ => {}
        }

        // Fallback: just return empty map
        Ok(HashMap::new())
    }
}
