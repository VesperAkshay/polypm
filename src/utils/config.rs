// Configuration utilities and TOML parsing

use std::fs;
use std::path::Path;
use crate::models::project::{Project, ProjectToml};
use crate::utils::error::{PpmError, Result};

/// Configuration parsing and validation utilities
pub struct ConfigParser;

impl ConfigParser {
    /// Load and validate a project configuration from project.toml
    pub fn load_project_config<P: AsRef<Path>>(path: P) -> Result<Project> {
        let path = path.as_ref();
        
        // Check if file exists
        if !path.exists() {
            return Err(PpmError::ConfigError(
                format!("Configuration file not found: {}", path.display())
            ));
        }

        // Read file contents
        let content = fs::read_to_string(path)
            .map_err(|e| PpmError::ConfigError(
                format!("Failed to read {}: {}", path.display(), e)
            ))?;

        Self::parse_project_config(&content)
    }

    /// Parse project configuration from TOML string with comprehensive validation
    pub fn parse_project_config(content: &str) -> Result<Project> {
        // Parse TOML
        let project_toml: ProjectToml = toml::from_str(content)
            .map_err(|e| PpmError::ConfigError(
                format!("Invalid TOML syntax: {}", e)
            ))?;

        // Convert to Project
        let project = Project::from(project_toml);

        // Validate the project
        project.validate()
            .map_err(|e| PpmError::ValidationError(e))?;

        // Additional configuration-specific validations
        Self::validate_config_structure(&project)?;

        Ok(project)
    }

    /// Save project configuration to TOML file
    pub fn save_project_config<P: AsRef<Path>>(project: &Project, path: P) -> Result<()> {
        let path = path.as_ref();
        
        // Validate before saving
        project.validate()
            .map_err(|e| PpmError::ValidationError(e))?;

        // Convert to TOML format
        let project_toml = ProjectToml::from(project.clone());
        
        // Serialize to TOML
        let content = toml::to_string_pretty(&project_toml)
            .map_err(|e| PpmError::ConfigError(
                format!("Failed to serialize configuration: {}", e)
            ))?;

        // Write to file
        fs::write(path, content)
            .map_err(|e| PpmError::ConfigError(
                format!("Failed to write {}: {}", path.display(), e)
            ))?;

        Ok(())
    }

    /// Validate configuration-specific rules beyond basic project validation
    fn validate_config_structure(project: &Project) -> Result<()> {
        // Validate virtual environment configuration
        if let Some(venv_config) = &project.venv_config {
            Self::validate_venv_config(venv_config)?;
        }

        // Validate script names don't conflict with built-in commands
        Self::validate_script_names(&project.scripts)?;

        // Validate ecosystem consistency
        Self::validate_ecosystem_consistency(project)?;

        Ok(())
    }

    /// Validate virtual environment configuration
    fn validate_venv_config(venv_config: &crate::models::project::VenvConfig) -> Result<()> {
        // Validate path is not absolute or dangerous
        let path = Path::new(&venv_config.path);
        if path.is_absolute() {
            return Err(PpmError::ValidationError(
                "Virtual environment path must be relative to project root".to_string()
            ));
        }

        // Check for dangerous paths
        if venv_config.path.contains("..") {
            return Err(PpmError::ValidationError(
                "Virtual environment path cannot contain '..' references".to_string()
            ));
        }

        // Validate Python version format if specified
        if let Some(ref python_version) = venv_config.python_version {
            if !Self::is_valid_python_version(python_version) {
                return Err(PpmError::ValidationError(
                    format!("Invalid Python version format: {}", python_version)
                ));
            }
        }

        Ok(())
    }

    /// Validate script names don't conflict with built-in commands
    fn validate_script_names(scripts: &std::collections::HashMap<String, String>) -> Result<()> {
        const RESERVED_NAMES: &[&str] = &["init", "install", "add", "run", "venv", "help"];
        
        for script_name in scripts.keys() {
            if RESERVED_NAMES.contains(&script_name.as_str()) {
                return Err(PpmError::ValidationError(
                    format!("Script name '{}' conflicts with built-in command", script_name)
                ));
            }
        }

        Ok(())
    }

    /// Validate ecosystem consistency
    fn validate_ecosystem_consistency(project: &Project) -> Result<()> {
        use std::collections::HashSet;
        
        // Collect all ecosystems used in dependencies
        let mut used_ecosystems = HashSet::new();
        
        for ecosystem in project.dependencies.keys() {
            used_ecosystems.insert(*ecosystem);
        }
        
        for ecosystem in project.dev_dependencies.keys() {
            used_ecosystems.insert(*ecosystem);
        }

        // If Python dependencies exist, ensure venv config is appropriate
        if used_ecosystems.contains(&crate::models::ecosystem::Ecosystem::Python) {
            if project.venv_config.is_none() {
                // This is a warning, not an error - we can create default venv config
                eprintln!("Warning: Python dependencies found but no venv configuration specified. Using defaults.");
            }
        }

        Ok(())
    }

    /// Validate Python version format (e.g., "3.8", "3.9.5", ">=3.8")
    fn is_valid_python_version(version: &str) -> bool {
        // Remove comparison operators
        let version = version.trim_start_matches(&['>', '<', '=', '~', '^'][..]);
        
        // Split by dots and validate each part
        let parts: Vec<&str> = version.split('.').collect();
        if parts.is_empty() || parts.len() > 3 {
            return false;
        }

        // First part should be a number >= 3
        if let Ok(major) = parts[0].parse::<u32>() {
            if major < 3 {
                return false;
            }
        } else {
            return false;
        }

        // Remaining parts should be numbers
        for part in &parts[1..] {
            if part.parse::<u32>().is_err() {
                return false;
            }
        }

        true
    }
}

pub fn get_ppm_home_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".ppm-store")
}

pub fn get_project_config_path() -> std::path::PathBuf {
    std::path::PathBuf::from("project.toml")
}

pub fn get_lock_file_path() -> std::path::PathBuf {
    std::path::PathBuf::from("ppm.lock")
}
