use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::models::ecosystem::Ecosystem;
use crate::models::resolved_dependency::ResolvedDependency;

/// Represents a timestamp for lock file generation
/// Using RFC 3339 format string for simplicity and JSON/TOML compatibility
pub type Timestamp = String;

/// Snapshot of resolved dependencies for reproducible installations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockFile {
    /// Lock file format version for future compatibility
    pub version: u32,
    /// Hash of the project.toml content when lock was generated
    pub project_hash: String,
    /// Resolved dependencies organized by ecosystem
    pub resolved_dependencies: HashMap<Ecosystem, Vec<ResolvedDependency>>,
    /// When this lock file was generated (RFC 3339 format)
    pub generation_timestamp: Timestamp,
    /// Version of PPM that generated this lock file
    pub ppm_version: String,
}

impl LockFile {
    /// Current lock file format version
    pub const CURRENT_VERSION: u32 = 1;

    /// Create a new LockFile with the given project hash
    pub fn new(project_hash: String, ppm_version: String) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            project_hash,
            resolved_dependencies: HashMap::new(),
            generation_timestamp: Self::current_timestamp(),
            ppm_version,
        }
    }

    /// Create a LockFile with resolved dependencies
    pub fn with_dependencies(
        project_hash: String,
        ppm_version: String,
        resolved_dependencies: HashMap<Ecosystem, Vec<ResolvedDependency>>,
    ) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            project_hash,
            resolved_dependencies,
            generation_timestamp: Self::current_timestamp(),
            ppm_version,
        }
    }

    /// Validate the lock file according to business rules
    pub fn validate(&self) -> Result<(), String> {
        // Validate version is supported
        if self.version == 0 {
            return Err("Lock file version cannot be 0".to_string());
        }

        if self.version > Self::CURRENT_VERSION {
            return Err(format!(
                "Lock file version {} is newer than supported version {}",
                self.version, Self::CURRENT_VERSION
            ));
        }

        // Validate project hash
        if self.project_hash.is_empty() {
            return Err("Project hash cannot be empty".to_string());
        }

        if !self.is_valid_sha256(&self.project_hash) {
            return Err("Project hash must be a valid SHA-256".to_string());
        }

        // Validate PPM version
        if self.ppm_version.is_empty() {
            return Err("PPM version cannot be empty".to_string());
        }

        // Validate timestamp format (basic check)
        if self.generation_timestamp.is_empty() {
            return Err("Generation timestamp cannot be empty".to_string());
        }

        // Validate resolved dependencies
        self.validate_dependencies()?;

        Ok(())
    }

    /// Validate resolved dependencies structure
    fn validate_dependencies(&self) -> Result<(), String> {
        for (ecosystem, dependencies) in &self.resolved_dependencies {
            for dep in dependencies {
                if dep.name.is_empty() {
                    return Err(format!("Dependency name cannot be empty for ecosystem {}", ecosystem));
                }
                if dep.version.is_empty() {
                    return Err(format!("Dependency version cannot be empty for '{}' in ecosystem {}", dep.name, ecosystem));
                }
                if dep.hash.is_empty() {
                    return Err(format!("Dependency hash cannot be empty for '{}' in ecosystem {}", dep.name, ecosystem));
                }
                if !self.is_valid_sha256(&dep.hash) {
                    return Err(format!("Invalid hash for dependency '{}' in ecosystem {}", dep.name, ecosystem));
                }
                if dep.ecosystem != *ecosystem {
                    return Err(format!("Dependency '{}' ecosystem mismatch: expected {}, got {}", dep.name, ecosystem, dep.ecosystem));
                }
            }
        }
        Ok(())
    }

    /// Check if a string is a valid SHA-256 hash
    fn is_valid_sha256(&self, hash: &str) -> bool {
        hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Get current timestamp in RFC 3339 format
    fn current_timestamp() -> String {
        // For now, return a placeholder. In a real implementation, this would use chrono
        "2025-09-11T12:00:00Z".to_string()
    }

    /// Add resolved dependencies for an ecosystem
    pub fn add_ecosystem_dependencies(&mut self, ecosystem: Ecosystem, dependencies: Vec<ResolvedDependency>) {
        self.resolved_dependencies.insert(ecosystem, dependencies);
        self.update_timestamp();
    }

    /// Get resolved dependencies for a specific ecosystem
    pub fn get_dependencies(&self, ecosystem: &Ecosystem) -> Option<&Vec<ResolvedDependency>> {
        self.resolved_dependencies.get(ecosystem)
    }

    /// Get all resolved dependencies across all ecosystems
    pub fn get_all_dependencies(&self) -> Vec<&ResolvedDependency> {
        self.resolved_dependencies
            .values()
            .flat_map(|deps| deps.iter())
            .collect()
    }

    /// Check if lock file has dependencies for a specific ecosystem
    pub fn has_dependencies_for(&self, ecosystem: &Ecosystem) -> bool {
        self.resolved_dependencies.get(ecosystem).map_or(false, |deps| !deps.is_empty())
    }

    /// Clear all resolved dependencies
    pub fn clear_dependencies(&mut self) {
        self.resolved_dependencies.clear();
        self.update_timestamp();
    }

    /// Remove dependencies for a specific ecosystem
    pub fn remove_ecosystem_dependencies(&mut self, ecosystem: &Ecosystem) -> bool {
        let removed = self.resolved_dependencies.remove(ecosystem).is_some();
        if removed {
            self.update_timestamp();
        }
        removed
    }

    /// Update the generation timestamp to current time
    pub fn update_timestamp(&mut self) {
        self.generation_timestamp = Self::current_timestamp();
    }

    /// Check if lock file needs to be regenerated based on project hash
    pub fn needs_regeneration(&self, current_project_hash: &str) -> bool {
        self.project_hash != current_project_hash
    }

    /// Update project hash (when project.toml changes)
    pub fn update_project_hash(&mut self, new_hash: String) {
        self.project_hash = new_hash;
        self.update_timestamp();
    }

    /// Count total number of resolved dependencies across all ecosystems
    pub fn total_dependency_count(&self) -> usize {
        self.resolved_dependencies.values().map(|deps| deps.len()).sum()
    }

    /// Get ecosystems that have resolved dependencies
    pub fn get_ecosystems(&self) -> Vec<Ecosystem> {
        self.resolved_dependencies.keys().copied().collect()
    }

    /// Check if lock file is empty (no resolved dependencies)
    pub fn is_empty(&self) -> bool {
        self.resolved_dependencies.is_empty() || 
        self.resolved_dependencies.values().all(|deps| deps.is_empty())
    }

    /// Get lock file state description
    pub fn state(&self) -> LockFileState {
        if self.is_empty() {
            LockFileState::Empty
        } else if self.version < Self::CURRENT_VERSION {
            LockFileState::Outdated
        } else {
            LockFileState::Valid
        }
    }

    /// Find a specific dependency by name across all ecosystems
    pub fn find_dependency(&self, name: &str) -> Option<&ResolvedDependency> {
        self.get_all_dependencies()
            .into_iter()
            .find(|dep| dep.name == name)
    }

    /// Find dependencies by ecosystem and name
    pub fn find_dependency_in_ecosystem(&self, ecosystem: &Ecosystem, name: &str) -> Option<&ResolvedDependency> {
        self.get_dependencies(ecosystem)?
            .iter()
            .find(|dep| dep.name == name)
    }
}

/// Lock file state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockFileState {
    /// No resolved dependencies
    Empty,
    /// Has dependencies and is up to date
    Valid,
    /// Older format version
    Outdated,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_resolved_dependency() -> ResolvedDependency {
        ResolvedDependency::new(
            "express".to_string(),
            "4.18.2".to_string(),
            Ecosystem::JavaScript,
            "a".repeat(64),
            "sha256-".to_string() + &"a".repeat(64),
            "packages/express-4.18.2".to_string(),
        )
    }

    #[test]
    fn test_lock_file_creation() {
        let lock_file = LockFile::new(
            "b".repeat(64),
            "1.0.0".to_string(),
        );

        assert_eq!(lock_file.version, LockFile::CURRENT_VERSION);
        assert_eq!(lock_file.project_hash.len(), 64);
        assert_eq!(lock_file.ppm_version, "1.0.0");
        assert!(lock_file.resolved_dependencies.is_empty());
        assert!(!lock_file.generation_timestamp.is_empty());
    }

    #[test]
    fn test_lock_file_with_dependencies() {
        let mut deps = HashMap::new();
        deps.insert(Ecosystem::JavaScript, vec![sample_resolved_dependency()]);

        let lock_file = LockFile::with_dependencies(
            "c".repeat(64),
            "1.0.0".to_string(),
            deps,
        );

        assert!(lock_file.has_dependencies_for(&Ecosystem::JavaScript));
        assert!(!lock_file.has_dependencies_for(&Ecosystem::Python));
        assert_eq!(lock_file.total_dependency_count(), 1);
    }

    #[test]
    fn test_lock_file_validation_success() {
        let lock_file = LockFile::new(
            "d".repeat(64),
            "1.0.0".to_string(),
        );

        assert!(lock_file.validate().is_ok());
    }

    #[test]
    fn test_lock_file_validation_empty_project_hash() {
        let mut lock_file = LockFile::new(
            "e".repeat(64),
            "1.0.0".to_string(),
        );
        lock_file.project_hash = "".to_string();

        assert!(lock_file.validate().is_err());
        assert!(lock_file.validate().unwrap_err().contains("Project hash cannot be empty"));
    }

    #[test]
    fn test_lock_file_validation_invalid_project_hash() {
        let lock_file = LockFile::new(
            "invalid-hash".to_string(),
            "1.0.0".to_string(),
        );

        assert!(lock_file.validate().is_err());
        assert!(lock_file.validate().unwrap_err().contains("valid SHA-256"));
    }

    #[test]
    fn test_lock_file_validation_future_version() {
        let mut lock_file = LockFile::new(
            "f".repeat(64),
            "1.0.0".to_string(),
        );
        lock_file.version = 999;

        assert!(lock_file.validate().is_err());
        assert!(lock_file.validate().unwrap_err().contains("newer than supported"));
    }

    #[test]
    fn test_dependency_management() {
        let mut lock_file = LockFile::new(
            "g".repeat(64),
            "1.0.0".to_string(),
        );

        assert!(lock_file.is_empty());
        assert_eq!(lock_file.state(), LockFileState::Empty);

        // Add dependencies
        let js_deps = vec![sample_resolved_dependency()];
        lock_file.add_ecosystem_dependencies(Ecosystem::JavaScript, js_deps);

        assert!(!lock_file.is_empty());
        assert_eq!(lock_file.state(), LockFileState::Valid);
        assert_eq!(lock_file.total_dependency_count(), 1);
        assert!(lock_file.has_dependencies_for(&Ecosystem::JavaScript));

        // Find dependencies
        let found = lock_file.find_dependency("express");
        assert!(found.is_some());
        assert_eq!(found.unwrap().version, "4.18.2");

        let found_in_ecosystem = lock_file.find_dependency_in_ecosystem(&Ecosystem::JavaScript, "express");
        assert!(found_in_ecosystem.is_some());

        // Clear dependencies
        lock_file.clear_dependencies();
        assert!(lock_file.is_empty());
        assert_eq!(lock_file.total_dependency_count(), 0);
    }

    #[test]
    fn test_project_hash_management() {
        let initial_hash = "h".repeat(64);
        let new_hash = "i".repeat(64);
        
        let mut lock_file = LockFile::new(initial_hash.clone(), "1.0.0".to_string());

        assert!(!lock_file.needs_regeneration(&initial_hash));
        assert!(lock_file.needs_regeneration(&new_hash));

        lock_file.update_project_hash(new_hash.clone());
        assert_eq!(lock_file.project_hash, new_hash);
        assert!(!lock_file.needs_regeneration(&new_hash));
    }

    #[test]
    fn test_ecosystem_operations() {
        let mut lock_file = LockFile::new(
            "j".repeat(64),
            "1.0.0".to_string(),
        );

        // Add dependencies for JavaScript
        let js_deps = vec![sample_resolved_dependency()];
        lock_file.add_ecosystem_dependencies(Ecosystem::JavaScript, js_deps);

        // Add dependencies for Python
        let py_dep = ResolvedDependency::new(
            "flask".to_string(),
            "2.3.0".to_string(),
            Ecosystem::Python,
            "k".repeat(64),
            "sha256-".to_string() + &"k".repeat(64),
            "packages/flask-2.3.0".to_string(),
        );
        lock_file.add_ecosystem_dependencies(Ecosystem::Python, vec![py_dep]);

        assert_eq!(lock_file.get_ecosystems().len(), 2);
        assert_eq!(lock_file.total_dependency_count(), 2);

        // Remove one ecosystem
        assert!(lock_file.remove_ecosystem_dependencies(&Ecosystem::Python));
        assert!(!lock_file.remove_ecosystem_dependencies(&Ecosystem::Python)); // Already removed

        assert_eq!(lock_file.total_dependency_count(), 1);
        assert!(lock_file.has_dependencies_for(&Ecosystem::JavaScript));
        assert!(!lock_file.has_dependencies_for(&Ecosystem::Python));
    }

    #[test]
    fn test_dependency_validation() {
        let mut deps = HashMap::new();
        let invalid_dep = ResolvedDependency::new(
            "".to_string(), // Invalid: empty name
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            "l".repeat(64),
            "sha256-".to_string() + &"l".repeat(64),
            "packages/invalid".to_string(),
        );
        deps.insert(Ecosystem::JavaScript, vec![invalid_dep]);

        let lock_file = LockFile::with_dependencies(
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".to_string(), // Valid hex hash
            "1.0.0".to_string(),
            deps,
        );

        let validation_result = lock_file.validate();
        assert!(validation_result.is_err());
        let error_message = validation_result.unwrap_err();
        assert!(error_message.contains("Dependency name cannot be empty"));
    }

    #[test]
    fn test_sha256_validation() {
        let lock_file = LockFile::new("n".repeat(64), "1.0.0".to_string());

        assert!(lock_file.is_valid_sha256(&"a".repeat(64)));
        assert!(lock_file.is_valid_sha256("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"));
        assert!(!lock_file.is_valid_sha256(&"a".repeat(63))); // Too short
        assert!(!lock_file.is_valid_sha256(&"a".repeat(65))); // Too long
        assert!(!lock_file.is_valid_sha256("gggg")); // Invalid hex characters
    }
}
