use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;
use crate::utils::error::PpmError;
use crate::models::virtual_environment::{VirtualEnvironment, VenvConfig, VenvStatus, ExecutableInfo};
use crate::models::ecosystem::Ecosystem;

/// Cross-platform virtual environment manager for Python packages
#[derive(Debug, Clone)]
pub struct VirtualEnvironmentManager {
    /// Default configuration for virtual environments
    default_config: VenvConfig,
}

/// Result of virtual environment creation
#[derive(Debug, Clone)]
pub struct VenvCreationResult {
    /// The created virtual environment
    pub venv: VirtualEnvironment,
    /// Output from the creation process
    pub output: String,
    /// Whether the creation was successful
    pub success: bool,
    /// Python version detected
    pub python_version: Option<String>,
}

/// Information about a Python installation
#[derive(Debug, Clone)]
pub struct PythonInfo {
    /// Path to the Python executable
    pub executable: PathBuf,
    /// Python version
    pub version: String,
    /// Whether pip is available
    pub has_pip: bool,
    /// Whether venv module is available
    pub has_venv: bool,
}

impl VirtualEnvironmentManager {
    /// Create a new VirtualEnvironmentManager with default configuration
    pub fn new() -> Self {
        Self {
            default_config: VenvConfig::default(),
        }
    }

    /// Create a new VirtualEnvironmentManager with custom configuration
    pub fn with_config(config: VenvConfig) -> Self {
        Self {
            default_config: config,
        }
    }

    /// Create a Python virtual environment
    pub async fn create_python_venv(
        &self,
        project_root: &Path,
        venv_name: Option<&str>,
        config: Option<&VenvConfig>,
    ) -> Result<VenvCreationResult, PpmError> {
        let effective_config = config.unwrap_or(&self.default_config);
        let name = venv_name.unwrap_or("default");

        // Determine venv path
        let venv_path = if let Some(custom_path) = &effective_config.path {
            project_root.join(custom_path)
        } else {
            project_root.join(".venv")
        };

        // Find Python executable
        let python_info = self.find_python_executable(effective_config).await?;
        println!("Found Python: {} (version: {})", python_info.executable.display(), python_info.version);

        // Check if venv already exists
        if venv_path.exists() {
            if fs::read_dir(&venv_path).await.is_ok() {
                return Err(PpmError::ValidationError(format!(
                    "Virtual environment already exists at: {}",
                    venv_path.display()
                )));
            }
        }

        // Create the virtual environment
        let creation_result = self.create_venv_with_python(&python_info, &venv_path, effective_config).await?;

        // Create VirtualEnvironment model
        let mut venv = VirtualEnvironment::new(
            name.to_string(),
            venv_path,
            Ecosystem::Python,
            effective_config.clone(),
        );
        venv.python_version = Some(python_info.version.clone());

        // Set up environment variables
        self.setup_default_env_vars(&mut venv);

        Ok(VenvCreationResult {
            venv,
            output: creation_result,
            success: true,
            python_version: Some(python_info.version),
        })
    }

    /// Remove a virtual environment
    pub async fn remove_venv(&self, venv_path: &Path) -> Result<String, PpmError> {
        if !venv_path.exists() {
            return Err(PpmError::ValidationError(format!(
                "Virtual environment does not exist: {}",
                venv_path.display()
            )));
        }

        // Remove the entire directory
        fs::remove_dir_all(venv_path).await?;

        Ok(format!("Removed virtual environment: {}", venv_path.display()))
    }

    /// Check if a virtual environment exists and is valid
    pub async fn check_venv_status(&self, venv_path: &Path) -> Result<VenvStatus, PpmError> {
        if !venv_path.exists() {
            return Ok(VenvStatus::NotCreated);
        }

        // Check for Python executable
        let python_path = self.get_python_executable_path(venv_path);
        if !python_path.exists() {
            return Ok(VenvStatus::Corrupted);
        }

        // Try to run Python to verify it works
        match Command::new(&python_path)
            .arg("--version")
            .output()
        {
            Ok(output) if output.status.success() => Ok(VenvStatus::Inactive),
            _ => Ok(VenvStatus::Corrupted),
        }
    }

    /// Get information about executables in a virtual environment
    pub async fn get_venv_executables(&self, venv_path: &Path) -> Result<Vec<ExecutableInfo>, PpmError> {
        let mut executables = Vec::new();

        // Python executable
        let python_path = self.get_python_executable_path(venv_path);
        let python_version = if python_path.exists() {
            self.get_python_version(&python_path).await.unwrap_or_else(|_| "unknown".to_string())
        } else {
            "unknown".to_string()
        };

        executables.push(ExecutableInfo {
            name: "python".to_string(),
            path: python_path.clone(),
            version: python_version,
            available: python_path.exists(),
        });

        // Pip executable
        let pip_path = self.get_pip_executable_path(venv_path);
        let pip_version = if pip_path.exists() {
            self.get_pip_version(&pip_path).await.unwrap_or_else(|_| "unknown".to_string())
        } else {
            "unknown".to_string()
        };

        executables.push(ExecutableInfo {
            name: "pip".to_string(),
            path: pip_path.clone(),
            version: pip_version,
            available: pip_path.exists(),
        });

        Ok(executables)
    }

    /// Install packages in a virtual environment
    pub async fn install_packages(
        &self,
        venv_path: &Path,
        packages: &[String],
    ) -> Result<String, PpmError> {
        let pip_path = self.get_pip_executable_path(venv_path);
        
        if !pip_path.exists() {
            return Err(PpmError::ValidationError(format!(
                "Pip not found in virtual environment: {}",
                venv_path.display()
            )));
        }

        let mut cmd = Command::new(&pip_path);
        cmd.arg("install");
        
        for package in packages {
            cmd.arg(package);
        }

        // IMPORTANT: Set virtual environment variables for proper activation
        cmd.env("VIRTUAL_ENV", venv_path.to_string_lossy().to_string());
        cmd.env("PYTHONHOME", ""); // Unset PYTHONHOME to avoid conflicts
        
        // Add Scripts/bin directory to PATH
        let scripts_path = if cfg!(windows) {
            venv_path.join("Scripts")
        } else {
            venv_path.join("bin")
        };
        
        // Prepend venv scripts path to PATH
        if let Ok(current_path) = std::env::var("PATH") {
            let new_path = format!("{}{}{}", 
                scripts_path.to_string_lossy(), 
                if cfg!(windows) { ";" } else { ":" }, 
                current_path);
            cmd.env("PATH", new_path);
        } else {
            cmd.env("PATH", scripts_path.to_string_lossy().to_string());
        }

        println!("Installing packages with activated virtual environment...");
        let output = cmd.output()
            .map_err(|e| PpmError::ExecutionError(format!("Failed to run pip: {}", e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(PpmError::ExecutionError(format!("Pip install failed: {}", stderr)))
        }
    }

    /// Find available Python executable
    async fn find_python_executable(&self, config: &VenvConfig) -> Result<PythonInfo, PpmError> {
        // Try config-specified Python first
        if let Some(python_path) = &config.python {
            if let Ok(info) = self.get_python_info(python_path).await {
                return Ok(info);
            }
        }

        // Try common Python executables
        let python_names = [
            "python3",
            "python",
            "python3.12",
            "python3.11", 
            "python3.10",
            "python3.9",
        ];

        for name in &python_names {
            if let Ok(info) = self.get_python_info(name).await {
                return Ok(info);
            }
        }

        Err(PpmError::ValidationError(
            "No suitable Python executable found. Please install Python 3.8 or later.".to_string()
        ))
    }

    /// Get information about a Python executable
    async fn get_python_info(&self, python_cmd: &str) -> Result<PythonInfo, PpmError> {
        // Get Python version
        let version_output = Command::new(python_cmd)
            .arg("--version")
            .output()
            .map_err(|_| PpmError::ValidationError(format!("Python not found: {}", python_cmd)))?;

        if !version_output.status.success() {
            return Err(PpmError::ValidationError(format!("Invalid Python executable: {}", python_cmd)));
        }

        let version_str = String::from_utf8_lossy(&version_output.stdout);
        let version = version_str
            .trim()
            .strip_prefix("Python ")
            .unwrap_or(version_str.trim())
            .to_string();

        // Check for venv module
        let venv_check = Command::new(python_cmd)
            .args(["-c", "import venv"])
            .output()
            .map_err(|_| PpmError::ValidationError("Failed to check venv module".to_string()))?;

        let has_venv = venv_check.status.success();

        // Check for pip
        let pip_check = Command::new(python_cmd)
            .args(["-m", "pip", "--version"])
            .output()
            .map_err(|_| PpmError::ValidationError("Failed to check pip".to_string()))?;

        let has_pip = pip_check.status.success();

        // Get executable path
        let which_output = Command::new(python_cmd)
            .args(["-c", "import sys; print(sys.executable)"])
            .output()
            .map_err(|_| PpmError::ValidationError("Failed to get Python executable path".to_string()))?;

        let executable_str = String::from_utf8_lossy(&which_output.stdout);
        let executable = PathBuf::from(executable_str.trim());

        Ok(PythonInfo {
            executable,
            version,
            has_pip,
            has_venv,
        })
    }

    /// Create virtual environment using Python
    async fn create_venv_with_python(
        &self,
        python_info: &PythonInfo,
        venv_path: &Path,
        config: &VenvConfig,
    ) -> Result<String, PpmError> {
        if !python_info.has_venv {
            return Err(PpmError::ValidationError(
                "Python installation does not have venv module. Please install python3-venv or update Python.".to_string()
            ));
        }

        // Create parent directories
        if let Some(parent) = venv_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Build venv command
        let mut cmd = Command::new(&python_info.executable);
        cmd.arg("-m").arg("venv");

        // Add options based on config
        if config.system_site_packages {
            cmd.arg("--system-site-packages");
        }

        if config.copies {
            cmd.arg("--copies");
        }

        // Add the target path
        cmd.arg(venv_path);

        // Execute command
        let output = cmd.output()
            .map_err(|e| PpmError::ExecutionError(format!("Failed to create virtual environment: {}", e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(PpmError::ExecutionError(format!("Virtual environment creation failed: {}", stderr)))
        }
    }

    /// Set up default environment variables for a virtual environment
    fn setup_default_env_vars(&self, venv: &mut VirtualEnvironment) {
        // Standard Python virtual environment variables
        venv.set_env_var("VIRTUAL_ENV".to_string(), venv.path.to_string_lossy().to_string());
        venv.set_env_var("PYTHONHOME".to_string(), "".to_string()); // Unset PYTHONHOME

        // Add bin/Scripts to PATH
        let scripts_path = if cfg!(windows) {
            venv.path.join("Scripts")
        } else {
            venv.path.join("bin")
        };
        venv.set_env_var("PPM_VENV_SCRIPTS".to_string(), scripts_path.to_string_lossy().to_string());
    }

    /// Get the path to the Python executable in a virtual environment
    fn get_python_executable_path(&self, venv_path: &Path) -> PathBuf {
        if cfg!(windows) {
            venv_path.join("Scripts").join("python.exe")
        } else {
            venv_path.join("bin").join("python")
        }
    }

    /// Get the path to the pip executable in a virtual environment
    fn get_pip_executable_path(&self, venv_path: &Path) -> PathBuf {
        if cfg!(windows) {
            venv_path.join("Scripts").join("pip.exe")
        } else {
            venv_path.join("bin").join("pip")
        }
    }

    /// Get Python version from executable
    async fn get_python_version(&self, python_path: &Path) -> Result<String, PpmError> {
        let output = Command::new(python_path)
            .arg("--version")
            .output()
            .map_err(|e| PpmError::ExecutionError(format!("Failed to get Python version: {}", e)))?;

        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout);
            let version = version_str
                .trim()
                .strip_prefix("Python ")
                .unwrap_or(version_str.trim())
                .to_string();
            Ok(version)
        } else {
            Err(PpmError::ExecutionError("Failed to get Python version".to_string()))
        }
    }

    /// Get pip version from executable
    async fn get_pip_version(&self, pip_path: &Path) -> Result<String, PpmError> {
        let output = Command::new(pip_path)
            .arg("--version")
            .output()
            .map_err(|e| PpmError::ExecutionError(format!("Failed to get pip version: {}", e)))?;

        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout);
            // Parse "pip 23.0.1 from ..." to extract version
            let version = version_str
                .split_whitespace()
                .nth(1)
                .unwrap_or("unknown")
                .to_string();
            Ok(version)
        } else {
            Err(PpmError::ExecutionError("Failed to get pip version".to_string()))
        }
    }

    /// Activate a virtual environment by setting up environment variables
    pub fn get_activation_env(&self, venv: &VirtualEnvironment) -> HashMap<String, String> {
        let mut env_vars = venv.get_env_vars();

        // Add Scripts/bin directory to PATH
        let scripts_path = if cfg!(windows) {
            venv.path.join("Scripts")
        } else {
            venv.path.join("bin")
        };

        // Prepend to existing PATH
        if let Ok(current_path) = std::env::var("PATH") {
            let new_path = format!("{}{}{}", scripts_path.to_string_lossy(), 
                                  if cfg!(windows) { ";" } else { ":" }, 
                                  current_path);
            env_vars.insert("PATH".to_string(), new_path);
        } else {
            env_vars.insert("PATH".to_string(), scripts_path.to_string_lossy().to_string());
        }

        env_vars
    }
}

impl Default for VirtualEnvironmentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_venv_manager_creation() {
        let manager = VirtualEnvironmentManager::new();
        assert!(manager.default_config.path.is_some());
    }

    #[tokio::test]
    async fn test_venv_manager_with_custom_config() {
        let config = VenvConfig {
            python: Some("python3".to_string()),
            path: Some("custom_venv".to_string()),
            system_site_packages: true,
            ..Default::default()
        };

        let manager = VirtualEnvironmentManager::with_config(config.clone());
        assert_eq!(manager.default_config.python, config.python);
        assert_eq!(manager.default_config.system_site_packages, true);
    }

    #[tokio::test]
    async fn test_find_python_executable() {
        let manager = VirtualEnvironmentManager::new();
        let config = VenvConfig::default();

        // This test will succeed if Python is installed on the system
        match manager.find_python_executable(&config).await {
            Ok(python_info) => {
                assert!(!python_info.version.is_empty());
                assert!(python_info.executable.exists());
                println!("Found Python: {} ({})", python_info.executable.display(), python_info.version);
            }
            Err(e) => {
                println!("Python not found (expected in some test environments): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_venv_paths() {
        let manager = VirtualEnvironmentManager::new();
        let temp_dir = TempDir::new().unwrap();
        let venv_path = temp_dir.path().join("test_venv");

        let python_path = manager.get_python_executable_path(&venv_path);
        let pip_path = manager.get_pip_executable_path(&venv_path);

        #[cfg(windows)]
        {
            assert!(python_path.ends_with("Scripts\\python.exe"));
            assert!(pip_path.ends_with("Scripts\\pip.exe"));
        }

        #[cfg(not(windows))]
        {
            assert!(python_path.ends_with("bin/python"));
            assert!(pip_path.ends_with("bin/pip"));
        }
    }

    #[tokio::test]
    async fn test_activation_env() {
        let temp_dir = TempDir::new().unwrap();
        let venv_path = temp_dir.path().join("test_venv");
        
        let venv = VirtualEnvironment::with_defaults(
            "test".to_string(),
            venv_path.clone(),
            Ecosystem::Python,
        );

        let manager = VirtualEnvironmentManager::new();
        let env_vars = manager.get_activation_env(&venv);

        assert!(env_vars.contains_key("VIRTUAL_ENV"));
        assert!(env_vars.contains_key("PATH"));
        assert_eq!(env_vars.get("VIRTUAL_ENV"), Some(&venv_path.to_string_lossy().to_string()));
    }

    #[tokio::test]
    async fn test_venv_status_not_created() {
        let manager = VirtualEnvironmentManager::new();
        let temp_dir = TempDir::new().unwrap();
        let non_existent_path = temp_dir.path().join("does_not_exist");

        let status = manager.check_venv_status(&non_existent_path).await.unwrap();
        assert_eq!(status, VenvStatus::NotCreated);
    }
}
