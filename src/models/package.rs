use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::models::ecosystem::Ecosystem;
use crate::models::dependency::Dependency;

/// Metadata for a package, containing additional information beyond basic fields
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// Optional description of the package
    pub description: Option<String>,
    /// Author information
    pub author: Option<String>,
    /// License identifier
    pub license: Option<String>,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// Keywords/tags
    pub keywords: Vec<String>,
    /// Additional metadata fields that may be ecosystem-specific
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for PackageMetadata {
    fn default() -> Self {
        Self {
            description: None,
            author: None,
            license: None,
            homepage: None,
            repository: None,
            keywords: Vec::new(),
            extra: HashMap::new(),
        }
    }
}

/// Represents a versioned package from either ecosystem stored in global store
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Package {
    /// Package name (must be valid identifier)
    pub name: String,
    /// Package version (must follow ecosystem rules)
    pub version: String,
    /// Which ecosystem this package belongs to
    pub ecosystem: Ecosystem,
    /// SHA-256 content hash for integrity verification
    pub hash: String,
    /// Additional package metadata
    pub metadata: PackageMetadata,
    /// Transitive dependencies of this package
    pub dependencies: Vec<Dependency>,
    /// Path where package is stored in global store
    pub store_path: PathBuf,
}

impl Package {
    /// Create a new Package with the given parameters
    pub fn new(
        name: String,
        version: String,
        ecosystem: Ecosystem,
        hash: String,
        store_path: PathBuf,
    ) -> Self {
        Self {
            name,
            version,
            ecosystem,
            hash,
            metadata: PackageMetadata::default(),
            dependencies: Vec::new(),
            store_path,
        }
    }

    /// Create a Package with metadata
    pub fn with_metadata(
        name: String,
        version: String,
        ecosystem: Ecosystem,
        hash: String,
        metadata: PackageMetadata,
        store_path: PathBuf,
    ) -> Self {
        Self {
            name,
            version,
            ecosystem,
            hash,
            metadata,
            dependencies: Vec::new(),
            store_path,
        }
    }

    /// Validate the package according to the business rules
    pub fn validate(&self) -> Result<(), String> {
        // Validate name is non-empty
        if self.name.is_empty() {
            return Err("Package name cannot be empty".to_string());
        }

        // Validate version format according to ecosystem
        let parser = self.ecosystem.version_parser();
        parser.parse_version(&self.version)
            .map_err(|_| format!("Invalid version '{}' for ecosystem {}", self.version, self.ecosystem))?;

        // Validate hash is valid SHA-256 (64 hex characters)
        if !self.is_valid_sha256(&self.hash) {
            return Err("Hash must be a valid SHA-256 (64 hex characters)".to_string());
        }

        // Note: store_path existence check would require filesystem access
        // and is better suited for integration tests

        Ok(())
    }

    /// Check if a string is a valid SHA-256 hash
    fn is_valid_sha256(&self, hash: &str) -> bool {
        hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Get package identifier (name@version)
    pub fn identifier(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    /// Get full package identifier with ecosystem
    pub fn full_identifier(&self) -> String {
        format!("{}:{}@{}", self.ecosystem, self.name, self.version)
    }

    /// Add a dependency to this package
    pub fn add_dependency(&mut self, dependency: Dependency) {
        self.dependencies.push(dependency);
    }

    /// Remove all dependencies
    pub fn clear_dependencies(&mut self) {
        self.dependencies.clear();
    }

    /// Get dependencies for a specific ecosystem
    pub fn dependencies_for_ecosystem(&self, ecosystem: &Ecosystem) -> Vec<&Dependency> {
        self.dependencies
            .iter()
            .filter(|dep| &dep.ecosystem == ecosystem)
            .collect()
    }

    /// Check if package has any dependencies
    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_package_creation() {
        let package = Package::new(
            "express".to_string(),
            "4.18.2".to_string(),
            Ecosystem::JavaScript,
            "a".repeat(64),
            PathBuf::from("/store/packages/aaa..."),
        );

        assert_eq!(package.name, "express");
        assert_eq!(package.version, "4.18.2");
        assert_eq!(package.ecosystem, Ecosystem::JavaScript);
        assert_eq!(package.hash.len(), 64);
        assert!(package.dependencies.is_empty());
    }

    #[test]
    fn test_package_with_metadata() {
        let mut metadata = PackageMetadata::default();
        metadata.description = Some("Fast, unopinionated web framework".to_string());
        metadata.author = Some("TJ Holowaychuk".to_string());
        metadata.license = Some("MIT".to_string());
        metadata.keywords = vec!["web".to_string(), "framework".to_string()];

        let package = Package::with_metadata(
            "express".to_string(),
            "4.18.2".to_string(),
            Ecosystem::JavaScript,
            "b".repeat(64),
            metadata,
            PathBuf::from("/store/packages/bbb..."),
        );

        assert_eq!(package.metadata.description, Some("Fast, unopinionated web framework".to_string()));
        assert_eq!(package.metadata.author, Some("TJ Holowaychuk".to_string()));
        assert_eq!(package.metadata.license, Some("MIT".to_string()));
        assert_eq!(package.metadata.keywords.len(), 2);
    }

    #[test]
    fn test_package_validation_success() {
        let package = Package::new(
            "lodash".to_string(),
            "4.17.21".to_string(),
            Ecosystem::JavaScript,
            "c".repeat(64),
            PathBuf::from("/store/packages/ccc..."),
        );

        assert!(package.validate().is_ok());
    }

    #[test]
    fn test_package_validation_empty_name() {
        let package = Package::new(
            "".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            "d".repeat(64),
            PathBuf::from("/store/packages/ddd..."),
        );

        assert!(package.validate().is_err());
        assert!(package.validate().unwrap_err().contains("name cannot be empty"));
    }

    #[test]
    fn test_package_validation_invalid_version() {
        let package = Package::new(
            "test-package".to_string(),
            "not-a-version".to_string(),
            Ecosystem::JavaScript,
            "e".repeat(64),
            PathBuf::from("/store/packages/eee..."),
        );

        assert!(package.validate().is_err());
    }

    #[test]
    fn test_package_validation_invalid_hash() {
        let package = Package::new(
            "test-package".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            "not-a-hash".to_string(),
            PathBuf::from("/store/packages/invalid..."),
        );

        assert!(package.validate().is_err());
        assert!(package.validate().unwrap_err().contains("valid SHA-256"));
    }

    #[test]
    fn test_package_identifier() {
        let package = Package::new(
            "react".to_string(),
            "18.2.0".to_string(),
            Ecosystem::JavaScript,
            "f".repeat(64),
            PathBuf::from("/store/packages/fff..."),
        );

        assert_eq!(package.identifier(), "react@18.2.0");
        assert_eq!(package.full_identifier(), "javascript:react@18.2.0");
    }

    #[test]
    fn test_package_dependencies() {
        let mut package = Package::new(
            "test-package".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            "g".repeat(64),
            PathBuf::from("/store/packages/ggg..."),
        );

        assert!(!package.has_dependencies());

        // Note: This would require the Dependency model to be implemented
        // For now, we'll test the basic dependency management methods
        assert_eq!(package.dependencies.len(), 0);
        
        package.clear_dependencies();
        assert_eq!(package.dependencies.len(), 0);
    }

    #[test]
    fn test_sha256_validation() {
        let package = Package::new(
            "test".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            "h".repeat(64),
            PathBuf::from("/store"),
        );

        assert!(package.is_valid_sha256(&"a".repeat(64)));
        assert!(package.is_valid_sha256("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"));
        assert!(!package.is_valid_sha256(&"a".repeat(63))); // Too short
        assert!(!package.is_valid_sha256(&"a".repeat(65))); // Too long
        assert!(!package.is_valid_sha256("gggg")); // Invalid hex characters
    }

    #[test]
    fn test_metadata_default() {
        let metadata = PackageMetadata::default();
        assert!(metadata.description.is_none());
        assert!(metadata.author.is_none());
        assert!(metadata.license.is_none());
        assert!(metadata.homepage.is_none());
        assert!(metadata.repository.is_none());
        assert!(metadata.keywords.is_empty());
        assert!(metadata.extra.is_empty());
    }
}
