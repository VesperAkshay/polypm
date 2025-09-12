use clap::Subcommand;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use serde_json::json;

use crate::models::project::{Project, ProjectToml};
use crate::models::ecosystem::Ecosystem;
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
                "This project does not use Python. Add Python dependencies first.".to_string()
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
                "No project.toml found (run 'ppm init' first)".to_string()
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
            PathBuf::from(".ppm").join("venv")
        };

        // Check if venv already exists
        if venv_path.exists() && !force {
            let error_msg = format!("Virtual environment already exists at {} (use --force to recreate)", 
                venv_path.display());
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

        // Determine Python executable
        let python_cmd = self.find_python_executable(python_version)?;

        // Create virtual environment
        let output = Command::new(&python_cmd)
            .args(&["-m", "venv", venv_path.to_str().unwrap()])
            .output()
            .map_err(|e| PpmError::ExecutionError(format!("Failed to create venv: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(PpmError::ExecutionError(
                format!("Failed to create virtual environment: {}", error)
            ));
        }

        // Get Python version info
        let python_version_info = self.get_python_version(&python_cmd)?;

        if json {
            let response = json!({
                "status": "success",
                "command": "create",
                "venv_path": self.normalize_path_display(&venv_path),
                "python_version": python_version_info,
                "python_executable": python_cmd
            });
            println!("{}", serde_json::to_string_pretty(&response)
                .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
        } else {
            println!("Created Python virtual environment at {}", self.normalize_path_display(&venv_path));
            println!("Using Python {}", python_version_info);
            println!("Virtual environment ready for use");
        }

        Ok(())
    }

    /// Remove virtual environment
    async fn remove_venv(&self, json: bool) -> Result<()> {
        let venv_path = PathBuf::from(".ppm").join("venv");

        if !venv_path.exists() {
            let error_msg = "No virtual environment found at .ppm/venv";
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

        // Remove the directory
        fs::remove_dir_all(&venv_path)
            .map_err(|e| PpmError::IoError(e))?;

        if json {
            let response = json!({
                "status": "success",
                "command": "remove",
                "removed_path": self.normalize_path_display(&venv_path)
            });
            println!("{}", serde_json::to_string_pretty(&response)
                .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
        } else {
            println!("Removed virtual environment at {}", self.normalize_path_display(&venv_path));
        }

        Ok(())
    }

    /// Show virtual environment information
    async fn info_venv(&self, json: bool) -> Result<()> {
        let venv_path = PathBuf::from(".ppm").join("venv");

        if !venv_path.exists() {
            let error_msg = "No virtual environment found at .ppm/venv";
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

        // Get Python executable in venv
        let python_exe = self.get_venv_python_executable(&venv_path);
        
        // Get Python version
        let python_version = if python_exe.exists() {
            self.get_python_version(python_exe.to_str().unwrap()).unwrap_or_else(|_| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        };

        // Get installed packages (simplified)
        let packages = self.get_installed_packages(&venv_path).unwrap_or_default();

        if json {
            let response = json!({
                "status": "success",
                "command": "info",
                "venv_path": self.normalize_path_display(&venv_path),
                "python_executable": self.normalize_path_display(&python_exe),
                "python_version": python_version,
                "status": "Active",
                "packages": packages
            });
            println!("{}", serde_json::to_string_pretty(&response)
                .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
        } else {
            println!("Python Virtual Environment:");
            println!("  Path: {}", self.normalize_path_display(&venv_path));
            println!("  Python: {}", python_version);
            println!("  Status: Active");
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
        let venv_path = PathBuf::from(".ppm").join("venv");

        if !venv_path.exists() {
            return Err(PpmError::ValidationError(
                "No virtual environment found at .ppm/venv".to_string()
            ));
        }

        if cfg!(target_os = "windows") {
            println!("Shell activation is not supported on Windows.");
            println!("To activate manually, run:");
            println!("  .ppm\\venv\\Scripts\\activate.bat");
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

    /// Find Python executable on system
    fn find_python_executable(&self, version: Option<&str>) -> Result<String> {
        let candidates = if let Some(ver) = version {
            vec![
                format!("python{}", ver),
                format!("python{}.exe", ver),
                "python".to_string(),
                "python3".to_string(),
            ]
        } else {
            vec![
                "python".to_string(),
                "python3".to_string(),
                "python.exe".to_string(),
            ]
        };

        for candidate in candidates {
            if let Ok(output) = Command::new(&candidate)
                .arg("--version")
                .output()
            {
                if output.status.success() {
                    let version_output = String::from_utf8_lossy(&output.stdout);
                    if version_output.contains("Python") {
                        // If specific version requested, verify it matches
                        if let Some(requested_ver) = version {
                            if !version_output.contains(requested_ver) {
                                continue;
                            }
                        }
                        return Ok(candidate);
                    }
                }
            }
        }

        if let Some(ver) = version {
            Err(PpmError::ValidationError(
                format!("Python {} not found on system", ver)
            ))
        } else {
            Err(PpmError::ValidationError(
                "Python not found on system. Please install Python first.".to_string()
            ))
        }
    }

    /// Get Python version information
    fn get_python_version(&self, python_cmd: &str) -> Result<String> {
        let output = Command::new(python_cmd)
            .arg("--version")
            .output()
            .map_err(|e| PpmError::ExecutionError(format!("Failed to get Python version: {}", e)))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            Ok(version)
        } else {
            Ok("Unknown".to_string())
        }
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
