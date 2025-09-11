use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::models::ecosystem::Ecosystem;

/// Virtual environment for isolated package management and execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualEnvironment {
    /// Name of the virtual environment
    pub name: String,
    /// Path to the virtual environment directory
    pub path: PathBuf,
    /// Which ecosystem this virtual environment supports
    pub ecosystem: Ecosystem,
    /// Python version for Python environments
    pub python_version: Option<String>,
    /// Node.js version for JavaScript environments
    pub node_version: Option<String>,
    /// Environment variables to set when activating
    pub env_vars: HashMap<String, String>,
    /// Whether this environment is currently active
    pub is_active: bool,
    /// When this environment was created
    pub created_at: String, // RFC 3339 timestamp
    /// When this environment was last used
    pub last_used: String, // RFC 3339 timestamp
    /// Configuration for this virtual environment
    pub config: VenvConfig,
}

/// Configuration for virtual environment creation and management
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VenvConfig {
    /// Python executable path or version for Python environments
    pub python: Option<String>,
    /// Node.js executable path or version for JavaScript environments
    pub node: Option<String>,
    /// Relative path from project root where venv should be created
    pub path: Option<String>,
    /// Whether to include system site packages (Python only)
    pub system_site_packages: bool,
    /// Whether to copy the Python executable (vs symlink)
    pub copies: bool,
    /// Additional packages to install in the environment
    pub additional_packages: Vec<String>,
    /// Environment variables to set by default
    pub default_env_vars: HashMap<String, String>,
}

impl Default for VenvConfig {
    fn default() -> Self {
        Self {
            python: None,
            node: None,
            path: Some(".venv".to_string()),
            system_site_packages: false,
            copies: false,
            additional_packages: Vec::new(),
            default_env_vars: HashMap::new(),
        }
    }
}

/// Status of a virtual environment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VenvStatus {
    /// Environment doesn't exist yet
    NotCreated,
    /// Environment exists but is not active
    Inactive,
    /// Environment is currently active
    Active,
    /// Environment exists but has errors
    Corrupted,
}

/// Information about an executable in the virtual environment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutableInfo {
    /// Name of the executable (python, node, pip, npm, etc.)
    pub name: String,
    /// Full path to the executable
    pub path: PathBuf,
    /// Version of the executable
    pub version: String,
    /// Whether this executable is available
    pub available: bool,
}

impl VirtualEnvironment {
    /// Create a new VirtualEnvironment
    pub fn new(
        name: String,
        path: PathBuf,
        ecosystem: Ecosystem,
        config: VenvConfig,
    ) -> Self {
        Self {
            name,
            path,
            ecosystem,
            python_version: None,
            node_version: None,
            env_vars: HashMap::new(),
            is_active: false,
            created_at: current_timestamp(),
            last_used: current_timestamp(),
            config,
        }
    }

    /// Create a VirtualEnvironment with default config
    pub fn with_defaults(name: String, path: PathBuf, ecosystem: Ecosystem) -> Self {
        Self::new(name, path, ecosystem, VenvConfig::default())
    }

    /// Validate the virtual environment configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate name is non-empty
        if self.name.is_empty() {
            return Err("Virtual environment name cannot be empty".to_string());
        }

        // Validate path is not empty
        if self.path.as_os_str().is_empty() {
            return Err("Virtual environment path cannot be empty".to_string());
        }

        // Validate ecosystem-specific requirements
        match self.ecosystem {
            Ecosystem::Python => {
                if self.config.python.is_none() && self.python_version.is_none() {
                    return Err("Python virtual environment must specify Python version".to_string());
                }
            }
            Ecosystem::JavaScript => {
                if self.config.node.is_none() && self.node_version.is_none() {
                    return Err("JavaScript virtual environment must specify Node.js version".to_string());
                }
            }
        }

        // Validate timestamps are not empty
        if self.created_at.is_empty() {
            return Err("Created timestamp cannot be empty".to_string());
        }

        if self.last_used.is_empty() {
            return Err("Last used timestamp cannot be empty".to_string());
        }

        // Validate config
        self.config.validate()?;

        Ok(())
    }

    /// Get the status of this virtual environment
    pub fn get_status(&self) -> VenvStatus {
        // Simplified implementation - in real code would check filesystem
        if self.is_active {
            VenvStatus::Active
        } else {
            VenvStatus::Inactive
        }
    }

    /// Check if the virtual environment exists on disk
    pub fn exists(&self) -> bool {
        // Simplified implementation - in real code would check filesystem
        true
    }

    /// Activate the virtual environment
    pub fn activate(&mut self) -> Result<(), String> {
        if !self.exists() {
            return Err("Cannot activate non-existent virtual environment".to_string());
        }

        self.is_active = true;
        self.last_used = current_timestamp();
        Ok(())
    }

    /// Deactivate the virtual environment
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// Get environment variables that should be set when activated
    pub fn get_env_vars(&self) -> HashMap<String, String> {
        let mut env_vars = self.config.default_env_vars.clone();
        env_vars.extend(self.env_vars.clone());

        // Add ecosystem-specific environment variables
        match self.ecosystem {
            Ecosystem::Python => {
                env_vars.insert("VIRTUAL_ENV".to_string(), self.path.to_string_lossy().to_string());
                env_vars.insert("PYTHONHOME".to_string(), "".to_string()); // Unset
            }
            Ecosystem::JavaScript => {
                env_vars.insert("NODE_PATH".to_string(), 
                    format!("{}/node_modules", self.path.to_string_lossy()));
            }
        }

        env_vars
    }

    /// Get the executable path for the primary interpreter
    pub fn get_interpreter_path(&self) -> PathBuf {
        match self.ecosystem {
            Ecosystem::Python => {
                let mut path = self.path.clone();
                #[cfg(windows)]
                path.push("Scripts/python.exe");
                #[cfg(not(windows))]
                path.push("bin/python");
                path
            }
            Ecosystem::JavaScript => {
                let mut path = self.path.clone();
                #[cfg(windows)]
                path.push("node.exe");
                #[cfg(not(windows))]
                path.push("bin/node");
                path
            }
        }
    }

    /// Get the package manager path
    pub fn get_package_manager_path(&self) -> PathBuf {
        match self.ecosystem {
            Ecosystem::Python => {
                let mut path = self.path.clone();
                #[cfg(windows)]
                path.push("Scripts/pip.exe");
                #[cfg(not(windows))]
                path.push("bin/pip");
                path
            }
            Ecosystem::JavaScript => {
                let mut path = self.path.clone();
                #[cfg(windows)]
                path.push("npm.cmd");
                #[cfg(not(windows))]
                path.push("bin/npm");
                path
            }
        }
    }

    /// Get information about available executables
    pub fn get_executables(&self) -> Vec<ExecutableInfo> {
        let mut executables = Vec::new();

        match self.ecosystem {
            Ecosystem::Python => {
                executables.push(ExecutableInfo {
                    name: "python".to_string(),
                    path: self.get_interpreter_path(),
                    version: self.python_version.clone().unwrap_or_default(),
                    available: true, // Simplified
                });
                executables.push(ExecutableInfo {
                    name: "pip".to_string(),
                    path: self.get_package_manager_path(),
                    version: "latest".to_string(), // Simplified
                    available: true, // Simplified
                });
            }
            Ecosystem::JavaScript => {
                executables.push(ExecutableInfo {
                    name: "node".to_string(),
                    path: self.get_interpreter_path(),
                    version: self.node_version.clone().unwrap_or_default(),
                    available: true, // Simplified
                });
                executables.push(ExecutableInfo {
                    name: "npm".to_string(),
                    path: self.get_package_manager_path(),
                    version: "latest".to_string(), // Simplified
                    available: true, // Simplified
                });
            }
        }

        executables
    }

    /// Set an environment variable
    pub fn set_env_var(&mut self, key: String, value: String) {
        self.env_vars.insert(key, value);
    }

    /// Remove an environment variable
    pub fn remove_env_var(&mut self, key: &str) {
        self.env_vars.remove(key);
    }

    /// Update the last used timestamp
    pub fn mark_used(&mut self) {
        self.last_used = current_timestamp();
    }

    /// Check if this environment supports a given ecosystem
    pub fn supports_ecosystem(&self, ecosystem: &Ecosystem) -> bool {
        &self.ecosystem == ecosystem
    }

    /// Get a user-friendly identifier for this environment
    pub fn identifier(&self) -> String {
        format!("{}:{}", self.ecosystem, self.name)
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: VenvConfig) -> Result<(), String> {
        config.validate()?;
        self.config = config;
        Ok(())
    }

    /// Check if environment needs to be recreated due to config changes
    pub fn needs_recreation(&self, new_config: &VenvConfig) -> bool {
        // Major config changes that require recreation
        self.config.python != new_config.python ||
        self.config.node != new_config.node ||
        self.config.system_site_packages != new_config.system_site_packages ||
        self.config.copies != new_config.copies
    }
}

impl VenvConfig {
    /// Create a Python-specific virtual environment config
    pub fn python(python_version: String) -> Self {
        let mut config = Self::default();
        config.python = Some(python_version);
        config
    }

    /// Create a JavaScript-specific virtual environment config
    pub fn javascript(node_version: String) -> Self {
        let mut config = Self::default();
        config.node = Some(node_version);
        config
    }

    /// Validate the virtual environment configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate path if specified
        if let Some(path) = &self.path {
            if path.is_empty() {
                return Err("Virtual environment path cannot be empty".to_string());
            }
        }

        // Validate additional packages
        for package in &self.additional_packages {
            if package.is_empty() {
                return Err("Additional package name cannot be empty".to_string());
            }
        }

        // Validate environment variable names
        for (key, _) in &self.default_env_vars {
            if key.is_empty() {
                return Err("Environment variable name cannot be empty".to_string());
            }
        }

        Ok(())
    }

    /// Get the target path for the virtual environment
    pub fn get_target_path(&self, base_path: &PathBuf) -> PathBuf {
        if let Some(path) = &self.path {
            if PathBuf::from(path).is_absolute() {
                PathBuf::from(path)
            } else {
                base_path.join(path)
            }
        } else {
            base_path.join(".venv")
        }
    }

    /// Add an additional package to install
    pub fn add_package(&mut self, package: String) {
        if !self.additional_packages.contains(&package) {
            self.additional_packages.push(package);
        }
    }

    /// Remove an additional package
    pub fn remove_package(&mut self, package: &str) {
        self.additional_packages.retain(|p| p != package);
    }

    /// Set a default environment variable
    pub fn set_env_var(&mut self, key: String, value: String) {
        self.default_env_vars.insert(key, value);
    }

    /// Check if this config is for Python
    pub fn is_python(&self) -> bool {
        self.python.is_some()
    }

    /// Check if this config is for JavaScript
    pub fn is_javascript(&self) -> bool {
        self.node.is_some()
    }
}

/// Get current timestamp in RFC 3339 format
fn current_timestamp() -> String {
    // Simplified implementation - in real code would use proper datetime library
    "2025-09-11T10:00:00Z".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_environment_creation() {
        let venv = VirtualEnvironment::new(
            "test-env".to_string(),
            PathBuf::from("/tmp/test-env"),
            Ecosystem::Python,
            VenvConfig::python("3.9".to_string()),
        );

        assert_eq!(venv.name, "test-env");
        assert_eq!(venv.path, PathBuf::from("/tmp/test-env"));
        assert_eq!(venv.ecosystem, Ecosystem::Python);
        assert!(!venv.is_active);
        assert_eq!(venv.config.python, Some("3.9".to_string()));
    }

    #[test]
    fn test_virtual_environment_with_defaults() {
        let venv = VirtualEnvironment::with_defaults(
            "default-env".to_string(),
            PathBuf::from("/tmp/default"),
            Ecosystem::JavaScript,
        );

        assert_eq!(venv.name, "default-env");
        assert_eq!(venv.ecosystem, Ecosystem::JavaScript);
        assert_eq!(venv.config.path, Some(".venv".to_string()));
        assert!(!venv.config.system_site_packages);
    }

    #[test]
    fn test_virtual_environment_validation_success() {
        let mut venv = VirtualEnvironment::with_defaults(
            "valid-env".to_string(),
            PathBuf::from("/tmp/valid"),
            Ecosystem::Python,
        );
        venv.config.python = Some("3.9".to_string());

        assert!(venv.validate().is_ok());
    }

    #[test]
    fn test_virtual_environment_validation_empty_name() {
        let venv = VirtualEnvironment::with_defaults(
            "".to_string(),
            PathBuf::from("/tmp/test"),
            Ecosystem::Python,
        );

        assert!(venv.validate().is_err());
        assert!(venv.validate().unwrap_err().contains("name cannot be empty"));
    }

    #[test]
    fn test_virtual_environment_validation_missing_python() {
        let venv = VirtualEnvironment::with_defaults(
            "python-env".to_string(),
            PathBuf::from("/tmp/python"),
            Ecosystem::Python,
        );

        assert!(venv.validate().is_err());
        assert!(venv.validate().unwrap_err().contains("must specify Python version"));
    }

    #[test]
    fn test_virtual_environment_validation_missing_node() {
        let venv = VirtualEnvironment::with_defaults(
            "node-env".to_string(),
            PathBuf::from("/tmp/node"),
            Ecosystem::JavaScript,
        );

        assert!(venv.validate().is_err());
        assert!(venv.validate().unwrap_err().contains("must specify Node.js version"));
    }

    #[test]
    fn test_virtual_environment_activation() {
        let mut venv = VirtualEnvironment::new(
            "test-env".to_string(),
            PathBuf::from("/tmp/test"),
            Ecosystem::Python,
            VenvConfig::python("3.9".to_string()),
        );

        assert!(!venv.is_active);
        
        let result = venv.activate();
        assert!(result.is_ok());
        assert!(venv.is_active);
        assert_eq!(venv.get_status(), VenvStatus::Active);
        
        venv.deactivate();
        assert!(!venv.is_active);
    }

    #[test]
    fn test_virtual_environment_env_vars() {
        let mut venv = VirtualEnvironment::new(
            "test-env".to_string(),
            PathBuf::from("/tmp/test"),
            Ecosystem::Python,
            VenvConfig::python("3.9".to_string()),
        );

        venv.set_env_var("TEST_VAR".to_string(), "test_value".to_string());
        
        let env_vars = venv.get_env_vars();
        assert!(env_vars.contains_key("TEST_VAR"));
        assert_eq!(env_vars.get("TEST_VAR"), Some(&"test_value".to_string()));
        assert!(env_vars.contains_key("VIRTUAL_ENV"));
        
        venv.remove_env_var("TEST_VAR");
        let env_vars_after = venv.get_env_vars();
        assert!(!env_vars_after.contains_key("TEST_VAR"));
    }

    #[test]
    fn test_virtual_environment_executables() {
        let venv = VirtualEnvironment::new(
            "test-env".to_string(),
            PathBuf::from("/tmp/test"),
            Ecosystem::Python,
            VenvConfig::python("3.9".to_string()),
        );

        let executables = venv.get_executables();
        assert_eq!(executables.len(), 2);
        
        let python_exec = executables.iter().find(|e| e.name == "python").unwrap();
        assert!(python_exec.available);
        
        let pip_exec = executables.iter().find(|e| e.name == "pip").unwrap();
        assert!(pip_exec.available);
    }

    #[test]
    fn test_virtual_environment_paths() {
        let venv = VirtualEnvironment::new(
            "test-env".to_string(),
            PathBuf::from("/tmp/test"),
            Ecosystem::Python,
            VenvConfig::python("3.9".to_string()),
        );

        let interpreter_path = venv.get_interpreter_path();
        let package_manager_path = venv.get_package_manager_path();

        #[cfg(windows)]
        {
            assert!(interpreter_path.to_string_lossy().ends_with("Scripts/python.exe"));
            assert!(package_manager_path.to_string_lossy().ends_with("Scripts/pip.exe"));
        }
        #[cfg(not(windows))]
        {
            assert!(interpreter_path.to_string_lossy().ends_with("bin/python"));
            assert!(package_manager_path.to_string_lossy().ends_with("bin/pip"));
        }
    }

    #[test]
    fn test_venv_config_creation() {
        let python_config = VenvConfig::python("3.9".to_string());
        assert_eq!(python_config.python, Some("3.9".to_string()));
        assert!(python_config.is_python());
        assert!(!python_config.is_javascript());

        let js_config = VenvConfig::javascript("18.0.0".to_string());
        assert_eq!(js_config.node, Some("18.0.0".to_string()));
        assert!(js_config.is_javascript());
        assert!(!js_config.is_python());
    }

    #[test]
    fn test_venv_config_validation() {
        let config = VenvConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = VenvConfig::default();
        invalid_config.path = Some("".to_string());
        assert!(invalid_config.validate().is_err());
        assert!(invalid_config.validate().unwrap_err().contains("path cannot be empty"));
    }

    #[test]
    fn test_venv_config_packages() {
        let mut config = VenvConfig::default();
        
        config.add_package("pytest".to_string());
        config.add_package("flask".to_string());
        assert_eq!(config.additional_packages.len(), 2);
        
        // Adding duplicate should not increase count
        config.add_package("pytest".to_string());
        assert_eq!(config.additional_packages.len(), 2);
        
        config.remove_package("pytest");
        assert_eq!(config.additional_packages.len(), 1);
        assert!(!config.additional_packages.contains(&"pytest".to_string()));
    }

    #[test]
    fn test_venv_config_target_path() {
        let config = VenvConfig::default();
        let base_path = PathBuf::from("/project");
        
        let target = config.get_target_path(&base_path);
        assert_eq!(target, PathBuf::from("/project/.venv"));

        let mut custom_config = VenvConfig::default();
        custom_config.path = Some("custom-venv".to_string());
        let custom_target = custom_config.get_target_path(&base_path);
        assert_eq!(custom_target, PathBuf::from("/project/custom-venv"));
    }

    #[test]
    fn test_virtual_environment_identifier() {
        let venv = VirtualEnvironment::new(
            "my-env".to_string(),
            PathBuf::from("/tmp/my-env"),
            Ecosystem::Python,
            VenvConfig::python("3.9".to_string()),
        );

        assert_eq!(venv.identifier(), "python:my-env");
    }

    #[test]
    fn test_virtual_environment_ecosystem_support() {
        let python_venv = VirtualEnvironment::new(
            "python-env".to_string(),
            PathBuf::from("/tmp/python"),
            Ecosystem::Python,
            VenvConfig::python("3.9".to_string()),
        );

        assert!(python_venv.supports_ecosystem(&Ecosystem::Python));
        assert!(!python_venv.supports_ecosystem(&Ecosystem::JavaScript));
    }

    #[test]
    fn test_virtual_environment_recreation() {
        let venv = VirtualEnvironment::new(
            "test-env".to_string(),
            PathBuf::from("/tmp/test"),
            Ecosystem::Python,
            VenvConfig::python("3.9".to_string()),
        );

        let same_config = VenvConfig::python("3.9".to_string());
        assert!(!venv.needs_recreation(&same_config));

        let different_config = VenvConfig::python("3.10".to_string());
        assert!(venv.needs_recreation(&different_config));
    }
}
