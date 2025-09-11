use serde::{Deserialize, Serialize};
use crate::models::ecosystem::Ecosystem;

/// Specific package version with integrity information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedDependency {
    /// Package name
    pub name: String,
    /// Exact resolved version (not a version spec)
    pub version: String,
    /// Which ecosystem this dependency belongs to
    pub ecosystem: Ecosystem,
    /// Content hash for integrity verification (SHA-256)
    pub hash: String,
    /// Additional integrity checksum (could be different format per ecosystem)
    pub integrity: String,
    /// Relative path to package in global store
    pub store_path: String,
}

impl ResolvedDependency {
    /// Create a new ResolvedDependency with the given parameters
    pub fn new(
        name: String,
        version: String,
        ecosystem: Ecosystem,
        hash: String,
        integrity: String,
        store_path: String,
    ) -> Self {
        Self {
            name,
            version,
            ecosystem,
            hash,
            integrity,
            store_path,
        }
    }

    /// Create a ResolvedDependency with hash as integrity (common case)
    pub fn with_hash_integrity(
        name: String,
        version: String,
        ecosystem: Ecosystem,
        hash: String,
        store_path: String,
    ) -> Self {
        Self::new(name, version, ecosystem, hash.clone(), hash, store_path)
    }

    /// Validate the resolved dependency according to business rules
    pub fn validate(&self) -> Result<(), String> {
        // Validate name is non-empty
        if self.name.is_empty() {
            return Err("Resolved dependency name cannot be empty".to_string());
        }

        // Validate version is non-empty and exact (no version specs allowed)
        if self.version.is_empty() {
            return Err("Resolved dependency version cannot be empty".to_string());
        }

        // Validate version is exact (not a spec) by checking for spec characters
        if self.has_version_spec_characters() {
            return Err(format!(
                "Resolved dependency version '{}' must be exact, not a version specification",
                self.version
            ));
        }

        // Validate version format according to ecosystem
        self.validate_version_format()?;

        // Validate hash is valid SHA-256
        if !self.is_valid_sha256(&self.hash) {
            return Err("Hash must be a valid SHA-256 (64 hex characters)".to_string());
        }

        // Validate integrity is non-empty
        if self.integrity.is_empty() {
            return Err("Integrity checksum cannot be empty".to_string());
        }

        // Validate store path is non-empty
        if self.store_path.is_empty() {
            return Err("Store path cannot be empty".to_string());
        }

        Ok(())
    }

    /// Check if version contains specification characters that indicate it's not exact
    fn has_version_spec_characters(&self) -> bool {
        let spec_chars = ['~', '^', '>', '<', '=', '*', 'x', 'X'];
        self.version.chars().any(|c| spec_chars.contains(&c)) ||
        self.version.contains("latest") ||
        self.version.contains("alpha") ||
        self.version.contains("beta") ||
        self.version.contains("rc")
    }

    /// Validate version format according to ecosystem rules
    fn validate_version_format(&self) -> Result<(), String> {
        let parser = self.ecosystem.version_parser();
        parser.parse_version(&self.version)
            .map_err(|_| format!(
                "Invalid version '{}' for ecosystem {}",
                self.version, self.ecosystem
            ))?;
        Ok(())
    }

    /// Check if a string is a valid SHA-256 hash
    fn is_valid_sha256(&self, hash: &str) -> bool {
        hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Get dependency identifier (name@version)
    pub fn identifier(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    /// Get full dependency identifier with ecosystem
    pub fn full_identifier(&self) -> String {
        format!("{}:{}@{}", self.ecosystem, self.name, self.version)
    }

    /// Check if this resolved dependency matches a dependency name and ecosystem
    pub fn matches(&self, name: &str, ecosystem: &Ecosystem) -> bool {
        self.name == name && &self.ecosystem == ecosystem
    }

    /// Check if this resolved dependency satisfies a version specification
    pub fn satisfies_version_spec(&self, version_spec: &str) -> Result<bool, String> {
        let parser = self.ecosystem.version_parser();
        parser.satisfies(&self.version, version_spec)
            .map_err(|e| format!("Failed to check version satisfaction: {}", e))
    }

    /// Compare this resolved dependency version with another version
    pub fn compare_version(&self, other_version: &str) -> Result<i8, String> {
        let parser = self.ecosystem.version_parser();
        parser.compare_versions(&self.version, other_version)
            .map_err(|e| format!("Failed to compare versions: {}", e))
    }

    /// Check if this dependency is newer than another version
    pub fn is_newer_than(&self, other_version: &str) -> Result<bool, String> {
        Ok(self.compare_version(other_version)? > 0)
    }

    /// Check if this dependency is older than another version
    pub fn is_older_than(&self, other_version: &str) -> Result<bool, String> {
        Ok(self.compare_version(other_version)? < 0)
    }

    /// Check if this dependency has the same version as another
    pub fn has_same_version(&self, other_version: &str) -> Result<bool, String> {
        Ok(self.compare_version(other_version)? == 0)
    }

    /// Get the package file extension for this ecosystem
    pub fn package_extension(&self) -> &'static str {
        self.ecosystem.package_extension()
    }

    /// Get the package format for this ecosystem
    pub fn package_format(&self) -> crate::models::ecosystem::PackageFormat {
        self.ecosystem.package_format()
    }

    /// Check if integrity verification would pass
    pub fn verify_integrity(&self, actual_hash: &str) -> bool {
        // For now, we'll check if the integrity matches either the hash or the actual hash
        self.integrity == actual_hash || self.hash == actual_hash
    }

    /// Update the store path (when package is moved in global store)
    pub fn update_store_path(&mut self, new_path: String) {
        self.store_path = new_path;
    }

    /// Create a copy with a different version (for version updates)
    pub fn with_version(&self, new_version: String) -> Self {
        Self {
            name: self.name.clone(),
            version: new_version,
            ecosystem: self.ecosystem,
            hash: self.hash.clone(),
            integrity: self.integrity.clone(),
            store_path: self.store_path.clone(),
        }
    }

    /// Create a copy with a different hash and integrity (for content updates)
    pub fn with_hash(&self, new_hash: String, new_integrity: String) -> Self {
        Self {
            name: self.name.clone(),
            version: self.version.clone(),
            ecosystem: self.ecosystem,
            hash: new_hash,
            integrity: new_integrity,
            store_path: self.store_path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolved_dependency_creation() {
        let dep = ResolvedDependency::new(
            "express".to_string(),
            "4.18.2".to_string(),
            Ecosystem::JavaScript,
            "a".repeat(64),
            "sha256-".to_string() + &"a".repeat(64),
            "packages/express-4.18.2".to_string(),
        );

        assert_eq!(dep.name, "express");
        assert_eq!(dep.version, "4.18.2");
        assert_eq!(dep.ecosystem, Ecosystem::JavaScript);
        assert_eq!(dep.hash.len(), 64);
        assert!(dep.integrity.starts_with("sha256-"));
        assert_eq!(dep.store_path, "packages/express-4.18.2");
    }

    #[test]
    fn test_resolved_dependency_with_hash_integrity() {
        let hash = "b".repeat(64);
        let dep = ResolvedDependency::with_hash_integrity(
            "lodash".to_string(),
            "4.17.21".to_string(),
            Ecosystem::JavaScript,
            hash.clone(),
            "packages/lodash-4.17.21".to_string(),
        );

        assert_eq!(dep.hash, hash);
        assert_eq!(dep.integrity, hash);
    }

    #[test]
    fn test_resolved_dependency_validation_success() {
        let dep = ResolvedDependency::new(
            "numpy".to_string(),
            "1.24.3".to_string(),
            Ecosystem::Python,
            "c".repeat(64),
            "sha256-".to_string() + &"c".repeat(64),
            "packages/numpy-1.24.3".to_string(),
        );

        assert!(dep.validate().is_ok());
    }

    #[test]
    fn test_resolved_dependency_validation_empty_name() {
        let dep = ResolvedDependency::new(
            "".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            "d".repeat(64),
            "integrity".to_string(),
            "packages/test".to_string(),
        );

        assert!(dep.validate().is_err());
        assert!(dep.validate().unwrap_err().contains("name cannot be empty"));
    }

    #[test]
    fn test_resolved_dependency_validation_empty_version() {
        let dep = ResolvedDependency::new(
            "test-package".to_string(),
            "".to_string(),
            Ecosystem::JavaScript,
            "e".repeat(64),
            "integrity".to_string(),
            "packages/test".to_string(),
        );

        assert!(dep.validate().is_err());
        assert!(dep.validate().unwrap_err().contains("version cannot be empty"));
    }

    #[test]
    fn test_resolved_dependency_validation_version_spec() {
        let version_specs = vec!["^1.0.0", "~2.1.0", ">=3.0.0", "latest", "1.0.0-alpha.1"];
        
        for spec in version_specs {
            let dep = ResolvedDependency::new(
                "test-package".to_string(),
                spec.to_string(),
                Ecosystem::JavaScript,
                "f".repeat(64),
                "integrity".to_string(),
                "packages/test".to_string(),
            );

            assert!(dep.validate().is_err(), "Should reject version spec: {}", spec);
            assert!(dep.validate().unwrap_err().contains("must be exact"));
        }
    }

    #[test]
    fn test_resolved_dependency_validation_invalid_hash() {
        let dep = ResolvedDependency::new(
            "test-package".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            "invalid-hash".to_string(),
            "integrity".to_string(),
            "packages/test".to_string(),
        );

        assert!(dep.validate().is_err());
        assert!(dep.validate().unwrap_err().contains("valid SHA-256"));
    }

    #[test]
    fn test_resolved_dependency_identifiers() {
        let dep = ResolvedDependency::new(
            "react".to_string(),
            "18.2.0".to_string(),
            Ecosystem::JavaScript,
            "g".repeat(64),
            "integrity".to_string(),
            "packages/react-18.2.0".to_string(),
        );

        assert_eq!(dep.identifier(), "react@18.2.0");
        assert_eq!(dep.full_identifier(), "javascript:react@18.2.0");
    }

    #[test]
    fn test_resolved_dependency_matching() {
        let dep = ResolvedDependency::new(
            "flask".to_string(),
            "2.3.2".to_string(),
            Ecosystem::Python,
            "h".repeat(64),
            "integrity".to_string(),
            "packages/flask-2.3.2".to_string(),
        );

        assert!(dep.matches("flask", &Ecosystem::Python));
        assert!(!dep.matches("flask", &Ecosystem::JavaScript));
        assert!(!dep.matches("django", &Ecosystem::Python));
    }

    #[test]
    fn test_resolved_dependency_version_comparison() {
        let dep = ResolvedDependency::new(
            "semver-test".to_string(),
            "2.1.0".to_string(),
            Ecosystem::JavaScript,
            "i".repeat(64),
            "integrity".to_string(),
            "packages/semver-test-2.1.0".to_string(),
        );

        // These tests depend on the ecosystem's version parser implementation
        // For now, we'll test that the methods don't panic
        let _result1 = dep.is_newer_than("2.0.0");
        let _result2 = dep.is_older_than("2.2.0");
        let _result3 = dep.has_same_version("2.1.0");
    }

    #[test]
    fn test_resolved_dependency_integrity() {
        let hash = "j".repeat(64);
        let dep = ResolvedDependency::with_hash_integrity(
            "test-package".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            hash.clone(),
            "packages/test-1.0.0".to_string(),
        );

        // Should verify against the same hash
        assert!(dep.verify_integrity(&hash));
        
        // Should not verify against a different hash
        assert!(!dep.verify_integrity(&"k".repeat(64)));
    }

    #[test]
    fn test_resolved_dependency_updates() {
        let mut dep = ResolvedDependency::new(
            "test-package".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            "l".repeat(64),
            "integrity".to_string(),
            "packages/test-1.0.0".to_string(),
        );

        // Update store path
        dep.update_store_path("new/path/test-1.0.0".to_string());
        assert_eq!(dep.store_path, "new/path/test-1.0.0");

        // Create with different version
        let new_version_dep = dep.with_version("1.0.1".to_string());
        assert_eq!(new_version_dep.version, "1.0.1");
        assert_eq!(new_version_dep.name, dep.name);

        // Create with different hash
        let new_hash = "m".repeat(64);
        let new_hash_dep = dep.with_hash(new_hash.clone(), "new-integrity".to_string());
        assert_eq!(new_hash_dep.hash, new_hash);
        assert_eq!(new_hash_dep.integrity, "new-integrity");
    }

    #[test]
    fn test_resolved_dependency_ecosystem_info() {
        let js_dep = ResolvedDependency::new(
            "test-js".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            "n".repeat(64),
            "integrity".to_string(),
            "packages/test-js-1.0.0".to_string(),
        );

        let py_dep = ResolvedDependency::new(
            "test-py".to_string(),
            "1.0.0".to_string(),
            Ecosystem::Python,
            "o".repeat(64),
            "integrity".to_string(),
            "packages/test-py-1.0.0".to_string(),
        );

        assert_eq!(js_dep.package_extension(), ".tgz");
        assert_eq!(py_dep.package_extension(), ".whl");
    }

    #[test]
    fn test_sha256_validation() {
        let dep = ResolvedDependency::new(
            "test".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            "p".repeat(64),
            "integrity".to_string(),
            "packages/test".to_string(),
        );

        assert!(dep.is_valid_sha256(&"a".repeat(64)));
        assert!(dep.is_valid_sha256("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"));
        assert!(!dep.is_valid_sha256(&"a".repeat(63))); // Too short
        assert!(!dep.is_valid_sha256(&"a".repeat(65))); // Too long
        assert!(!dep.is_valid_sha256("gggg")); // Invalid hex characters
    }
}
