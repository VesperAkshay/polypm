use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::models::ecosystem::Ecosystem;

/// Configuration for Python virtual environment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VenvConfig {
    /// Python version to use for the virtual environment
    pub python_version: Option<String>,
    /// Path to the virtual environment (relative to project root)
    pub path: String,
    /// Whether to create the venv automatically on install
    pub auto_create: bool,
}

impl Default for VenvConfig {
    fn default() -> Self {
        Self {
            python_version: None,
            path: ".ppm/venv".to_string(),
            auto_create: true,
        }
    }
}

/// Version specification for a dependency (e.g., "^1.0.0", ">=2.0.0")
pub type VersionSpec = String;

/// Represents a polyglot project with unified configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// Project name (must be valid identifier)
    pub name: String,
    /// Project version (must follow semver)
    pub version: String,
    /// Production dependencies organized by ecosystem
    pub dependencies: HashMap<Ecosystem, HashMap<String, VersionSpec>>,
    /// Development dependencies organized by ecosystem
    pub dev_dependencies: HashMap<Ecosystem, HashMap<String, VersionSpec>>,
    /// Project scripts (script name → command)
    pub scripts: HashMap<String, String>,
    /// Optional virtual environment configuration for Python
    pub venv_config: Option<VenvConfig>,
}

impl Project {
    /// Create a new Project with the given name and version
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            scripts: HashMap::new(),
            venv_config: None,
        }
    }

    /// Create a new Project with default venv config
    pub fn with_venv_config(name: String, version: String, venv_config: VenvConfig) -> Self {
        Self {
            name,
            version,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            scripts: HashMap::new(),
            venv_config: Some(venv_config),
        }
    }

    /// Create a new Project with both JavaScript and Python ecosystems initialized
    pub fn with_both_ecosystems(name: String, version: String) -> Self {
        let mut project = Self::new(name, version);
        project.ensure_ecosystem(Ecosystem::JavaScript);
        project.ensure_ecosystem(Ecosystem::Python);
        project.venv_config = Some(VenvConfig::default());
        project
    }

    /// Validate the project according to business rules
    pub fn validate(&self) -> Result<(), String> {
        // Validate name is a valid identifier
        self.validate_name()?;

        // Validate version follows semver
        self.validate_version()?;

        // Validate scripts are non-empty
        self.validate_scripts()?;

        // Validate ecosystem dependencies
        self.validate_dependencies()?;

        Ok(())
    }

    /// Validate project name is a valid identifier
    fn validate_name(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Project name cannot be empty".to_string());
        }

        // Check for valid identifier characters (alphanumeric, hyphens, underscores)
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(format!("Invalid project name '{}' (must be valid identifier)", self.name));
        }

        Ok(())
    }

    /// Validate project version follows semver
    fn validate_version(&self) -> Result<(), String> {
        if self.version.is_empty() {
            return Err("Project version cannot be empty".to_string());
        }

        // Basic semver validation (MAJOR.MINOR.PATCH)
        let parts: Vec<&str> = self.version.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version '{}' (must be valid semver)", self.version));
        }

        for part in parts {
            if part.parse::<u32>().is_err() {
                return Err(format!("Invalid version '{}' (must be valid semver)", self.version));
            }
        }

        Ok(())
    }

    /// Validate script commands are non-empty
    fn validate_scripts(&self) -> Result<(), String> {
        for (name, command) in &self.scripts {
            if name.is_empty() {
                return Err("Script name cannot be empty".to_string());
            }
            if command.trim().is_empty() {
                return Err(format!("Script command for '{}' cannot be empty", name));
            }
        }
        Ok(())
    }

    /// Validate dependencies have valid formats
    fn validate_dependencies(&self) -> Result<(), String> {
        // Validate production dependencies
        for (ecosystem, deps) in &self.dependencies {
            for (name, version_spec) in deps {
                if name.is_empty() {
                    return Err(format!("Dependency name cannot be empty for ecosystem {}", ecosystem));
                }
                if version_spec.trim().is_empty() {
                    return Err(format!("Version spec cannot be empty for dependency '{}' in ecosystem {}", name, ecosystem));
                }
            }
        }

        // Validate development dependencies
        for (ecosystem, deps) in &self.dev_dependencies {
            for (name, version_spec) in deps {
                if name.is_empty() {
                    return Err(format!("Dev dependency name cannot be empty for ecosystem {}", ecosystem));
                }
                if version_spec.trim().is_empty() {
                    return Err(format!("Version spec cannot be empty for dev dependency '{}' in ecosystem {}", name, ecosystem));
                }
            }
        }

        Ok(())
    }

    /// Ensure an ecosystem section exists in dependencies
    pub fn ensure_ecosystem(&mut self, ecosystem: Ecosystem) {
        self.dependencies.entry(ecosystem).or_insert_with(HashMap::new);
        self.dev_dependencies.entry(ecosystem).or_insert_with(HashMap::new);
    }

    /// Add a production dependency
    pub fn add_dependency(&mut self, ecosystem: Ecosystem, name: String, version_spec: VersionSpec) {
        self.ensure_ecosystem(ecosystem);
        self.dependencies.get_mut(&ecosystem).unwrap().insert(name, version_spec);
    }

    /// Add a development dependency
    pub fn add_dev_dependency(&mut self, ecosystem: Ecosystem, name: String, version_spec: VersionSpec) {
        self.ensure_ecosystem(ecosystem);
        self.dev_dependencies.get_mut(&ecosystem).unwrap().insert(name, version_spec);
    }

    /// Remove a production dependency
    pub fn remove_dependency(&mut self, ecosystem: &Ecosystem, name: &str) -> bool {
        if let Some(deps) = self.dependencies.get_mut(ecosystem) {
            deps.remove(name).is_some()
        } else {
            false
        }
    }

    /// Remove a development dependency
    pub fn remove_dev_dependency(&mut self, ecosystem: &Ecosystem, name: &str) -> bool {
        if let Some(deps) = self.dev_dependencies.get_mut(ecosystem) {
            deps.remove(name).is_some()
        } else {
            false
        }
    }

    /// Get all dependencies for a specific ecosystem
    pub fn get_dependencies(&self, ecosystem: &Ecosystem) -> Option<&HashMap<String, VersionSpec>> {
        self.dependencies.get(ecosystem)
    }

    /// Get all development dependencies for a specific ecosystem
    pub fn get_dev_dependencies(&self, ecosystem: &Ecosystem) -> Option<&HashMap<String, VersionSpec>> {
        self.dev_dependencies.get(ecosystem)
    }

    /// Add a script
    pub fn add_script(&mut self, name: String, command: String) {
        self.scripts.insert(name, command);
    }

    /// Remove a script
    pub fn remove_script(&mut self, name: &str) -> bool {
        self.scripts.remove(name).is_some()
    }

    /// Get a script command
    pub fn get_script(&self, name: &str) -> Option<&String> {
        self.scripts.get(name)
    }

    /// Check if project has any dependencies for a specific ecosystem
    pub fn has_dependencies_for(&self, ecosystem: &Ecosystem) -> bool {
        self.dependencies.get(ecosystem).map_or(false, |deps| !deps.is_empty()) ||
        self.dev_dependencies.get(ecosystem).map_or(false, |deps| !deps.is_empty())
    }

    /// Check if project has Python dependencies (to determine if venv is needed)
    pub fn needs_venv(&self) -> bool {
        self.has_dependencies_for(&Ecosystem::Python)
    }

    /// Get all supported ecosystems in this project
    pub fn get_ecosystems(&self) -> Vec<Ecosystem> {
        let mut ecosystems = Vec::new();
        for ecosystem in [Ecosystem::JavaScript, Ecosystem::Python] {
            if self.has_dependencies_for(&ecosystem) {
                ecosystems.push(ecosystem);
            }
        }
        ecosystems
    }

    /// Count total number of dependencies (production + dev) for all ecosystems
    pub fn total_dependency_count(&self) -> usize {
        let prod_count: usize = self.dependencies.values().map(|deps| deps.len()).sum();
        let dev_count: usize = self.dev_dependencies.values().map(|deps| deps.len()).sum();
        prod_count + dev_count
    }

    /// Get project identifier (name@version)
    pub fn identifier(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = Project::new("my-app".to_string(), "1.0.0".to_string());
        
        assert_eq!(project.name, "my-app");
        assert_eq!(project.version, "1.0.0");
        assert!(project.dependencies.is_empty());
        assert!(project.dev_dependencies.is_empty());
        assert!(project.scripts.is_empty());
        assert!(project.venv_config.is_none());
    }

    #[test]
    fn test_project_with_venv_config() {
        let venv_config = VenvConfig {
            python_version: Some("3.11".to_string()),
            path: ".venv".to_string(),
            auto_create: false,
        };

        let project = Project::with_venv_config(
            "my-app".to_string(),
            "1.0.0".to_string(),
            venv_config.clone(),
        );

        assert_eq!(project.venv_config, Some(venv_config));
    }

    #[test]
    fn test_project_with_both_ecosystems() {
        let project = Project::with_both_ecosystems("my-app".to_string(), "1.0.0".to_string());
        
        assert!(project.dependencies.contains_key(&Ecosystem::JavaScript));
        assert!(project.dependencies.contains_key(&Ecosystem::Python));
        assert!(project.dev_dependencies.contains_key(&Ecosystem::JavaScript));
        assert!(project.dev_dependencies.contains_key(&Ecosystem::Python));
        assert!(project.venv_config.is_some());
    }

    #[test]
    fn test_project_validation_success() {
        let mut project = Project::new("valid-project".to_string(), "1.2.3".to_string());
        project.add_script("test".to_string(), "npm test && pytest".to_string());
        
        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_project_validation_invalid_name() {
        let project = Project::new("invalid-name!".to_string(), "1.0.0".to_string());
        
        assert!(project.validate().is_err());
        assert!(project.validate().unwrap_err().contains("Invalid project name"));
    }

    #[test]
    fn test_project_validation_empty_name() {
        let project = Project::new("".to_string(), "1.0.0".to_string());
        
        assert!(project.validate().is_err());
        assert!(project.validate().unwrap_err().contains("name cannot be empty"));
    }

    #[test]
    fn test_project_validation_invalid_version() {
        let project = Project::new("my-app".to_string(), "1.x.0".to_string());
        
        assert!(project.validate().is_err());
        assert!(project.validate().unwrap_err().contains("Invalid version"));
    }

    #[test]
    fn test_project_validation_empty_script() {
        let mut project = Project::new("my-app".to_string(), "1.0.0".to_string());
        project.add_script("test".to_string(), "".to_string());
        
        assert!(project.validate().is_err());
        assert!(project.validate().unwrap_err().contains("cannot be empty"));
    }

    #[test]
    fn test_dependency_management() {
        let mut project = Project::new("my-app".to_string(), "1.0.0".to_string());
        
        // Add dependencies
        project.add_dependency(Ecosystem::JavaScript, "react".to_string(), "^18.0.0".to_string());
        project.add_dependency(Ecosystem::Python, "flask".to_string(), ">=2.0.0".to_string());
        project.add_dev_dependency(Ecosystem::JavaScript, "jest".to_string(), "^29.0.0".to_string());
        
        // Check dependencies exist
        assert!(project.has_dependencies_for(&Ecosystem::JavaScript));
        assert!(project.has_dependencies_for(&Ecosystem::Python));
        
        let js_deps = project.get_dependencies(&Ecosystem::JavaScript).unwrap();
        assert_eq!(js_deps.get("react"), Some(&"^18.0.0".to_string()));
        
        let js_dev_deps = project.get_dev_dependencies(&Ecosystem::JavaScript).unwrap();
        assert_eq!(js_dev_deps.get("jest"), Some(&"^29.0.0".to_string()));
        
        // Remove dependency
        assert!(project.remove_dependency(&Ecosystem::JavaScript, "react"));
        assert!(!project.remove_dependency(&Ecosystem::JavaScript, "nonexistent"));
    }

    #[test]
    fn test_script_management() {
        let mut project = Project::new("my-app".to_string(), "1.0.0".to_string());
        
        // Add scripts
        project.add_script("dev".to_string(), "npm run dev".to_string());
        project.add_script("test".to_string(), "npm test && pytest".to_string());
        
        // Check scripts exist
        assert_eq!(project.get_script("dev"), Some(&"npm run dev".to_string()));
        assert_eq!(project.get_script("test"), Some(&"npm test && pytest".to_string()));
        assert_eq!(project.get_script("nonexistent"), None);
        
        // Remove script
        assert!(project.remove_script("dev"));
        assert!(!project.remove_script("nonexistent"));
        assert_eq!(project.get_script("dev"), None);
    }

    #[test]
    fn test_ecosystem_detection() {
        let mut project = Project::new("my-app".to_string(), "1.0.0".to_string());
        
        assert!(!project.needs_venv());
        assert_eq!(project.get_ecosystems(), vec![]);
        
        project.add_dependency(Ecosystem::JavaScript, "react".to_string(), "^18.0.0".to_string());
        assert_eq!(project.get_ecosystems(), vec![Ecosystem::JavaScript]);
        
        project.add_dependency(Ecosystem::Python, "flask".to_string(), ">=2.0.0".to_string());
        assert!(project.needs_venv());
        assert_eq!(project.get_ecosystems().len(), 2);
    }

    #[test]
    fn test_dependency_counting() {
        let mut project = Project::new("my-app".to_string(), "1.0.0".to_string());
        
        assert_eq!(project.total_dependency_count(), 0);
        
        project.add_dependency(Ecosystem::JavaScript, "react".to_string(), "^18.0.0".to_string());
        project.add_dependency(Ecosystem::Python, "flask".to_string(), ">=2.0.0".to_string());
        project.add_dev_dependency(Ecosystem::JavaScript, "jest".to_string(), "^29.0.0".to_string());
        
        assert_eq!(project.total_dependency_count(), 3);
    }

    #[test]
    fn test_project_identifier() {
        let project = Project::new("my-awesome-app".to_string(), "2.1.0".to_string());
        assert_eq!(project.identifier(), "my-awesome-app@2.1.0");
    }

    #[test]
    fn test_venv_config_default() {
        let config = VenvConfig::default();
        assert_eq!(config.path, ".ppm/venv");
        assert!(config.auto_create);
        assert!(config.python_version.is_none());
    }
}
