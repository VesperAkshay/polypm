use serde::{Deserialize, Serialize};
use crate::models::ecosystem::Ecosystem;

/// Represents a dependency relationship with version constraints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    /// Name of the dependency package
    pub name: String,
    /// Version specification/constraint (e.g., "^1.0.0", ">=2.0.0", "~1.2.0")
    pub version_spec: String,
    /// Resolved version after dependency resolution (None if not resolved yet)
    pub resolved_version: Option<String>,
    /// Which ecosystem this dependency belongs to
    pub ecosystem: Ecosystem,
    /// Whether this is a development-only dependency
    pub dev_only: bool,
}

impl Dependency {
    /// Create a new Dependency with the given parameters
    pub fn new(
        name: String,
        version_spec: String,
        ecosystem: Ecosystem,
        dev_only: bool,
    ) -> Self {
        Self {
            name,
            version_spec,
            resolved_version: None,
            ecosystem,
            dev_only,
        }
    }

    /// Create a production dependency (dev_only = false)
    pub fn production(name: String, version_spec: String, ecosystem: Ecosystem) -> Self {
        Self::new(name, version_spec, ecosystem, false)
    }

    /// Create a development dependency (dev_only = true)
    pub fn development(name: String, version_spec: String, ecosystem: Ecosystem) -> Self {
        Self::new(name, version_spec, ecosystem, true)
    }

    /// Create a dependency with a resolved version
    pub fn with_resolved_version(
        name: String,
        version_spec: String,
        resolved_version: String,
        ecosystem: Ecosystem,
        dev_only: bool,
    ) -> Self {
        Self {
            name,
            version_spec,
            resolved_version: Some(resolved_version),
            ecosystem,
            dev_only,
        }
    }

    /// Validate the dependency according to business rules
    pub fn validate(&self) -> Result<(), String> {
        // Validate name is non-empty
        if self.name.is_empty() {
            return Err("Dependency name cannot be empty".to_string());
        }

        // Validate version_spec is non-empty
        if self.version_spec.is_empty() {
            return Err("Version specification cannot be empty".to_string());
        }

        // Validate version_spec format according to ecosystem
        self.validate_version_spec()?;

        // If resolved_version exists, validate it satisfies version_spec
        if let Some(ref resolved_version) = self.resolved_version {
            self.validate_resolved_version(resolved_version)?;
        }

        Ok(())
    }

    /// Validate the version specification format
    fn validate_version_spec(&self) -> Result<(), String> {
        // Basic validation - check for common version spec patterns
        if self.version_spec.trim().is_empty() {
            return Err("Version specification cannot be empty or whitespace".to_string());
        }

        // For now, we'll do basic validation. More sophisticated validation
        // would require parsing the version spec according to ecosystem rules
        match self.ecosystem {
            Ecosystem::JavaScript => self.validate_npm_version_spec(),
            Ecosystem::Python => self.validate_python_version_spec(),
        }
    }

    /// Validate NPM/semver version specification
    fn validate_npm_version_spec(&self) -> Result<(), String> {
        let spec = &self.version_spec;
        
        // Allow common semver patterns
        if spec.starts_with('^') || spec.starts_with('~') || spec.starts_with(">=") 
            || spec.starts_with("<=") || spec.starts_with('>') || spec.starts_with('<')
            || spec.starts_with('=') || spec.chars().next().unwrap_or('0').is_ascii_digit() {
            Ok(())
        } else {
            Err(format!("Invalid NPM version specification: '{}'", spec))
        }
    }

    /// Validate Python/PEP 440 version specification
    fn validate_python_version_spec(&self) -> Result<(), String> {
        let spec = &self.version_spec;
        
        // Allow common PEP 440 patterns
        if spec.starts_with(">=") || spec.starts_with("<=") || spec.starts_with("!=")
            || spec.starts_with("==") || spec.starts_with('>') || spec.starts_with('<')
            || spec.starts_with('~') || spec.chars().next().unwrap_or('0').is_ascii_digit() {
            Ok(())
        } else {
            Err(format!("Invalid Python version specification: '{}'", spec))
        }
    }

    /// Validate that resolved version satisfies the version specification
    fn validate_resolved_version(&self, resolved_version: &str) -> Result<(), String> {
        // Basic validation - ensure resolved version is not empty
        if resolved_version.trim().is_empty() {
            return Err("Resolved version cannot be empty".to_string());
        }

        // For now, we'll do basic validation. Full validation would require
        // implementing version constraint satisfaction logic
        let parser = self.ecosystem.version_parser();
        parser.parse_version(resolved_version)
            .map_err(|_| format!("Invalid resolved version '{}' for ecosystem {}", 
                               resolved_version, self.ecosystem))?;

        Ok(())
    }

    /// Check if this dependency has been resolved
    pub fn is_resolved(&self) -> bool {
        self.resolved_version.is_some()
    }

    /// Set the resolved version
    pub fn resolve(&mut self, version: String) -> Result<(), String> {
        // Validate the resolved version first
        self.validate_resolved_version(&version)?;
        self.resolved_version = Some(version);
        Ok(())
    }

    /// Clear the resolved version
    pub fn clear_resolution(&mut self) {
        self.resolved_version = None;
    }

    /// Get dependency identifier (name@version_spec)
    pub fn identifier(&self) -> String {
        format!("{}@{}", self.name, self.version_spec)
    }

    /// Get full dependency identifier with ecosystem
    pub fn full_identifier(&self) -> String {
        format!("{}:{}@{}", self.ecosystem, self.name, self.version_spec)
    }

    /// Get resolved identifier if resolved (name@resolved_version)
    pub fn resolved_identifier(&self) -> Option<String> {
        self.resolved_version.as_ref().map(|v| format!("{}@{}", self.name, v))
    }

    /// Check if this dependency is compatible with another ecosystem
    pub fn is_compatible_with(&self, other_ecosystem: &Ecosystem) -> bool {
        &self.ecosystem == other_ecosystem
    }

    /// Get dependency type as string
    pub fn dependency_type(&self) -> &'static str {
        if self.dev_only {
            "development"
        } else {
            "production"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_creation() {
        let dep = Dependency::new(
            "express".to_string(),
            "^4.18.0".to_string(),
            Ecosystem::JavaScript,
            false,
        );

        assert_eq!(dep.name, "express");
        assert_eq!(dep.version_spec, "^4.18.0");
        assert_eq!(dep.ecosystem, Ecosystem::JavaScript);
        assert!(!dep.dev_only);
        assert!(dep.resolved_version.is_none());
    }

    #[test]
    fn test_production_dependency() {
        let dep = Dependency::production(
            "lodash".to_string(),
            "~4.17.0".to_string(),
            Ecosystem::JavaScript,
        );

        assert!(!dep.dev_only);
        assert_eq!(dep.dependency_type(), "production");
    }

    #[test]
    fn test_development_dependency() {
        let dep = Dependency::development(
            "jest".to_string(),
            "^29.0.0".to_string(),
            Ecosystem::JavaScript,
        );

        assert!(dep.dev_only);
        assert_eq!(dep.dependency_type(), "development");
    }

    #[test]
    fn test_dependency_with_resolved_version() {
        let dep = Dependency::with_resolved_version(
            "react".to_string(),
            "^18.0.0".to_string(),
            "18.2.0".to_string(),
            Ecosystem::JavaScript,
            false,
        );

        assert!(dep.is_resolved());
        assert_eq!(dep.resolved_version, Some("18.2.0".to_string()));
        assert_eq!(dep.resolved_identifier(), Some("react@18.2.0".to_string()));
    }

    #[test]
    fn test_dependency_validation_success() {
        let dep = Dependency::new(
            "numpy".to_string(),
            ">=1.20.0".to_string(),
            Ecosystem::Python,
            false,
        );

        assert!(dep.validate().is_ok());
    }

    #[test]
    fn test_dependency_validation_empty_name() {
        let dep = Dependency::new(
            "".to_string(),
            "^1.0.0".to_string(),
            Ecosystem::JavaScript,
            false,
        );

        assert!(dep.validate().is_err());
        assert!(dep.validate().unwrap_err().contains("name cannot be empty"));
    }

    #[test]
    fn test_dependency_validation_empty_version_spec() {
        let dep = Dependency::new(
            "test-package".to_string(),
            "".to_string(),
            Ecosystem::JavaScript,
            false,
        );

        assert!(dep.validate().is_err());
        assert!(dep.validate().unwrap_err().contains("Version specification cannot be empty"));
    }

    #[test]
    fn test_npm_version_spec_validation() {
        let valid_specs = vec!["^1.0.0", "~2.1.0", ">=3.0.0", "<=4.0.0", ">1.0.0", "<2.0.0", "=1.0.0", "1.0.0"];
        
        for spec in valid_specs {
            let dep = Dependency::new(
                "test-package".to_string(),
                spec.to_string(),
                Ecosystem::JavaScript,
                false,
            );
            assert!(dep.validate().is_ok(), "Should be valid: {}", spec);
        }
    }

    #[test]
    fn test_python_version_spec_validation() {
        let valid_specs = vec![">=1.0.0", "<=2.0.0", "!=1.5.0", "==1.0.0", ">1.0.0", "<2.0.0", "~=1.4.2", "1.0.0"];
        
        for spec in valid_specs {
            let dep = Dependency::new(
                "test-package".to_string(),
                spec.to_string(),
                Ecosystem::Python,
                false,
            );
            assert!(dep.validate().is_ok(), "Should be valid: {}", spec);
        }
    }

    #[test]
    fn test_dependency_resolution() {
        let mut dep = Dependency::new(
            "express".to_string(),
            "^4.18.0".to_string(),
            Ecosystem::JavaScript,
            false,
        );

        assert!(!dep.is_resolved());

        let resolve_result = dep.resolve("4.18.2".to_string());
        assert!(resolve_result.is_ok());
        assert!(dep.is_resolved());
        assert_eq!(dep.resolved_version, Some("4.18.2".to_string()));

        dep.clear_resolution();
        assert!(!dep.is_resolved());
        assert!(dep.resolved_version.is_none());
    }

    #[test]
    fn test_dependency_identifiers() {
        let dep = Dependency::with_resolved_version(
            "vue".to_string(),
            "^3.0.0".to_string(),
            "3.3.4".to_string(),
            Ecosystem::JavaScript,
            false,
        );

        assert_eq!(dep.identifier(), "vue@^3.0.0");
        assert_eq!(dep.full_identifier(), "javascript:vue@^3.0.0");
        assert_eq!(dep.resolved_identifier(), Some("vue@3.3.4".to_string()));
    }

    #[test]
    fn test_ecosystem_compatibility() {
        let js_dep = Dependency::new(
            "lodash".to_string(),
            "^4.17.0".to_string(),
            Ecosystem::JavaScript,
            false,
        );

        assert!(js_dep.is_compatible_with(&Ecosystem::JavaScript));
        assert!(!js_dep.is_compatible_with(&Ecosystem::Python));
    }

    #[test]
    fn test_invalid_resolved_version() {
        let mut dep = Dependency::new(
            "test-package".to_string(),
            "^1.0.0".to_string(),
            Ecosystem::JavaScript,
            false,
        );

        let resolve_result = dep.resolve("not-a-version".to_string());
        assert!(resolve_result.is_err());
        assert!(!dep.is_resolved());
    }
}
