use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::models::ecosystem::Ecosystem;
use crate::models::resolved_dependency::ResolvedDependency;

/// Cross-platform symlink structure for package management
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymlinkStructure {
    /// Root directory where symlinks are created (e.g., node_modules)
    pub root_path: PathBuf,
    /// Ecosystem this structure serves
    pub ecosystem: Ecosystem,
    /// Map of package names to their symlink entries
    pub links: HashMap<String, SymlinkEntry>,
    /// When this structure was created
    pub created_at: String, // RFC 3339 timestamp
    /// When this structure was last modified
    pub last_modified: String, // RFC 3339 timestamp
    /// Version of the symlink structure format
    pub version: u32,
}

/// Individual symlink entry for a package
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymlinkEntry {
    /// Package name
    pub name: String,
    /// Version of the linked package
    pub version: String,
    /// Target path in the global store
    pub target_path: PathBuf,
    /// Symlink path relative to root
    pub link_path: PathBuf,
    /// Type of symlink created
    pub link_type: SymlinkType,
    /// Whether the symlink currently exists
    pub exists: bool,
    /// When this symlink was created
    pub created_at: String, // RFC 3339 timestamp
    /// Hash of the target package for integrity
    pub target_hash: String,
    /// Whether this is a development dependency symlink
    pub is_dev_dependency: bool,
}

/// Type of symlink being created
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SymlinkType {
    /// Directory symlink (most common for packages)
    Directory,
    /// File symlink (for individual files)
    File,
    /// Junction on Windows (for compatibility)
    Junction,
    /// Hard link (when symlinks aren't supported)
    HardLink,
}

/// Configuration for symlink creation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymlinkConfig {
    /// Whether to use junctions on Windows instead of symlinks
    pub use_junctions_on_windows: bool,
    /// Whether to fall back to hard links if symlinks fail
    pub fallback_to_hardlinks: bool,
    /// Whether to create parent directories automatically
    pub create_parent_dirs: bool,
    /// Whether to overwrite existing symlinks
    pub overwrite_existing: bool,
    /// Maximum depth for nested package structures
    pub max_depth: u32,
    /// Whether to validate symlink targets exist
    pub validate_targets: bool,
}

impl Default for SymlinkConfig {
    fn default() -> Self {
        Self {
            use_junctions_on_windows: true,
            fallback_to_hardlinks: false,
            create_parent_dirs: true,
            overwrite_existing: false,
            max_depth: 10,
            validate_targets: true,
        }
    }
}

/// Status of a symlink operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SymlinkStatus {
    /// Symlink was created successfully
    Created,
    /// Symlink already exists
    AlreadyExists,
    /// Symlink creation failed
    Failed(String),
    /// Target doesn't exist
    TargetNotFound,
    /// Permission denied
    PermissionDenied,
    /// Platform doesn't support symlinks
    NotSupported,
}

impl SymlinkStructure {
    /// Create a new SymlinkStructure
    pub fn new(root_path: PathBuf, ecosystem: Ecosystem) -> Self {
        Self {
            root_path,
            ecosystem,
            links: HashMap::new(),
            created_at: current_timestamp(),
            last_modified: current_timestamp(),
            version: 1,
        }
    }

    /// Create a SymlinkStructure for JavaScript node_modules
    pub fn node_modules(project_root: PathBuf) -> Self {
        let mut root = project_root;
        root.push("node_modules");
        Self::new(root, Ecosystem::JavaScript)
    }

    /// Create a SymlinkStructure for Python site-packages
    pub fn site_packages(venv_path: PathBuf) -> Self {
        let mut root = venv_path;
        root.push("lib");
        root.push("python3.x"); // Simplified - real implementation would detect version
        root.push("site-packages");
        Self::new(root, Ecosystem::Python)
    }

    /// Validate the symlink structure
    pub fn validate(&self) -> Result<(), String> {
        // Validate root path is not empty
        if self.root_path.as_os_str().is_empty() {
            return Err("Symlink structure root path cannot be empty".to_string());
        }

        // Validate version is supported
        if self.version == 0 {
            return Err("Symlink structure version must be greater than 0".to_string());
        }

        // Validate timestamps
        if self.created_at.is_empty() {
            return Err("Created timestamp cannot be empty".to_string());
        }

        if self.last_modified.is_empty() {
            return Err("Last modified timestamp cannot be empty".to_string());
        }

        // Validate all symlink entries
        for (name, entry) in &self.links {
            if name != &entry.name {
                return Err(format!(
                    "Symlink entry name mismatch: key '{}' vs entry '{}'",
                    name, entry.name
                ));
            }
            entry.validate()?;
        }

        Ok(())
    }

    /// Add a symlink entry for a resolved dependency
    pub fn add_dependency_link(
        &mut self,
        dependency: &ResolvedDependency,
        global_store_path: &PathBuf,
        config: &SymlinkConfig,
    ) -> Result<SymlinkStatus, String> {
        // Validate dependency
        dependency.validate()?;

        // Check if link already exists
        if self.links.contains_key(&dependency.name) {
            if !config.overwrite_existing {
                return Ok(SymlinkStatus::AlreadyExists);
            }
        }

        // Create target path in global store
        let mut target_path = global_store_path.clone();
        target_path.push(&dependency.store_path);

        // Create link path
        let link_path = self.create_link_path(&dependency.name)?;

        // Determine symlink type based on ecosystem and platform
        let link_type = self.determine_link_type(config);

        // Create symlink entry
        let entry = SymlinkEntry {
            name: dependency.name.clone(),
            version: dependency.version.clone(),
            target_path,
            link_path,
            link_type,
            exists: false, // Will be set to true after actual creation
            created_at: current_timestamp(),
            target_hash: dependency.hash.clone(),
            is_dev_dependency: false, // TODO: Get from dependency context
        };

        // Validate entry
        entry.validate()?;

        // Add to structure
        self.links.insert(dependency.name.clone(), entry);
        self.last_modified = current_timestamp();

        Ok(SymlinkStatus::Created)
    }

    /// Remove a symlink entry
    pub fn remove_link(&mut self, package_name: &str) -> Result<bool, String> {
        if let Some(_entry) = self.links.remove(package_name) {
            self.last_modified = current_timestamp();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get a symlink entry by package name
    pub fn get_link(&self, package_name: &str) -> Option<&SymlinkEntry> {
        self.links.get(package_name)
    }

    /// Get all symlink entries
    pub fn get_all_links(&self) -> Vec<&SymlinkEntry> {
        self.links.values().collect()
    }

    /// Get symlink entries by type
    pub fn get_links_by_type(&self, link_type: &SymlinkType) -> Vec<&SymlinkEntry> {
        self.links
            .values()
            .filter(|entry| &entry.link_type == link_type)
            .collect()
    }

    /// Check if a package has a symlink
    pub fn has_link(&self, package_name: &str) -> bool {
        self.links.contains_key(package_name)
    }

    /// Get count of symlinks
    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    /// Get count of existing vs missing symlinks
    pub fn get_link_stats(&self) -> (usize, usize) {
        let existing = self.links.values().filter(|e| e.exists).count();
        let missing = self.links.len() - existing;
        (existing, missing)
    }

    /// Update symlink existence status
    pub fn update_link_status(&mut self, package_name: &str, exists: bool) {
        if let Some(entry) = self.links.get_mut(package_name) {
            entry.exists = exists;
            self.last_modified = current_timestamp();
        }
    }

    /// Create the appropriate link path for a package based on ecosystem
    fn create_link_path(&self, package_name: &str) -> Result<PathBuf, String> {
        match self.ecosystem {
            Ecosystem::JavaScript => {
                // Simple flat structure for node_modules
                let mut path = PathBuf::new();
                path.push(package_name);
                Ok(path)
            }
            Ecosystem::Python => {
                // Python packages can have more complex structures
                let mut path = PathBuf::new();
                path.push(package_name);
                Ok(path)
            }
        }
    }

    /// Determine the appropriate symlink type based on config and platform
    fn determine_link_type(&self, config: &SymlinkConfig) -> SymlinkType {
        #[cfg(windows)]
        {
            if config.use_junctions_on_windows {
                SymlinkType::Junction
            } else {
                SymlinkType::Directory
            }
        }
        #[cfg(not(windows))]
        {
            SymlinkType::Directory
        }
    }

    /// Clean up broken or invalid symlinks
    pub fn cleanup_broken_links(&mut self) -> Vec<String> {
        let mut removed = Vec::new();
        
        self.links.retain(|name, entry| {
            if !entry.exists || !entry.is_valid() {
                removed.push(name.clone());
                false
            } else {
                true
            }
        });

        if !removed.is_empty() {
            self.last_modified = current_timestamp();
        }

        removed
    }

    /// Get full symlink path for a package
    pub fn get_full_link_path(&self, package_name: &str) -> Option<PathBuf> {
        self.links.get(package_name).map(|entry| {
            let mut full_path = self.root_path.clone();
            full_path.push(&entry.link_path);
            full_path
        })
    }

    /// Mark structure as modified
    pub fn mark_modified(&mut self) {
        self.last_modified = current_timestamp();
    }

    /// Check if structure is compatible with ecosystem
    pub fn supports_ecosystem(&self, ecosystem: &Ecosystem) -> bool {
        &self.ecosystem == ecosystem
    }

    /// Get identifier for this symlink structure
    pub fn identifier(&self) -> String {
        format!("{}:{}", self.ecosystem, self.root_path.to_string_lossy())
    }
}

impl SymlinkEntry {
    /// Validate the symlink entry
    pub fn validate(&self) -> Result<(), String> {
        // Validate name is not empty
        if self.name.is_empty() {
            return Err("Symlink entry name cannot be empty".to_string());
        }

        // Validate version is not empty
        if self.version.is_empty() {
            return Err("Symlink entry version cannot be empty".to_string());
        }

        // Validate target path is not empty
        if self.target_path.as_os_str().is_empty() {
            return Err("Symlink target path cannot be empty".to_string());
        }

        // Validate link path is not empty
        if self.link_path.as_os_str().is_empty() {
            return Err("Symlink path cannot be empty".to_string());
        }

        // Validate target hash
        if !self.is_valid_sha256(&self.target_hash) {
            return Err("Target hash must be a valid SHA-256 (64 hex characters)".to_string());
        }

        // Validate timestamp
        if self.created_at.is_empty() {
            return Err("Created timestamp cannot be empty".to_string());
        }

        Ok(())
    }

    /// Check if target hash is valid SHA-256
    fn is_valid_sha256(&self, hash: &str) -> bool {
        hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Check if symlink entry is valid (basic checks)
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Get identifier for this symlink entry
    pub fn identifier(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    /// Check if this entry points to a specific target hash
    pub fn targets_hash(&self, hash: &str) -> bool {
        self.target_hash == hash
    }

    /// Update the existence status
    pub fn set_exists(&mut self, exists: bool) {
        self.exists = exists;
    }
}

impl SymlinkConfig {
    /// Create config optimized for Windows
    pub fn windows_optimized() -> Self {
        Self {
            use_junctions_on_windows: true,
            fallback_to_hardlinks: true,
            create_parent_dirs: true,
            overwrite_existing: false,
            max_depth: 10,
            validate_targets: true,
        }
    }

    /// Create config optimized for Unix systems
    pub fn unix_optimized() -> Self {
        Self {
            use_junctions_on_windows: false,
            fallback_to_hardlinks: false,
            create_parent_dirs: true,
            overwrite_existing: false,
            max_depth: 10,
            validate_targets: true,
        }
    }

    /// Create config for development (more permissive)
    pub fn development() -> Self {
        Self {
            use_junctions_on_windows: true,
            fallback_to_hardlinks: true,
            create_parent_dirs: true,
            overwrite_existing: true,
            max_depth: 20,
            validate_targets: false,
        }
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

    fn sample_dependency() -> ResolvedDependency {
        ResolvedDependency::new(
            "react".to_string(),
            "18.2.0".to_string(),
            Ecosystem::JavaScript,
            "a".repeat(64),
            "sha256-".to_string() + &"a".repeat(64),
            "packages/react-18.2.0".to_string(),
        )
    }

    #[test]
    fn test_symlink_structure_creation() {
        let structure = SymlinkStructure::new(
            PathBuf::from("/project/node_modules"),
            Ecosystem::JavaScript,
        );

        assert_eq!(structure.root_path, PathBuf::from("/project/node_modules"));
        assert_eq!(structure.ecosystem, Ecosystem::JavaScript);
        assert!(structure.links.is_empty());
        assert_eq!(structure.version, 1);
    }

    #[test]
    fn test_node_modules_creation() {
        let structure = SymlinkStructure::node_modules(PathBuf::from("/project"));
        
        assert_eq!(structure.root_path, PathBuf::from("/project/node_modules"));
        assert_eq!(structure.ecosystem, Ecosystem::JavaScript);
    }

    #[test]
    fn test_site_packages_creation() {
        let structure = SymlinkStructure::site_packages(PathBuf::from("/venv"));
        
        assert!(structure.root_path.to_string_lossy().contains("site-packages"));
        assert_eq!(structure.ecosystem, Ecosystem::Python);
    }

    #[test]
    fn test_symlink_structure_validation_success() {
        let structure = SymlinkStructure::new(
            PathBuf::from("/project/node_modules"),
            Ecosystem::JavaScript,
        );

        assert!(structure.validate().is_ok());
    }

    #[test]
    fn test_symlink_structure_validation_empty_path() {
        let structure = SymlinkStructure::new(
            PathBuf::new(),
            Ecosystem::JavaScript,
        );

        assert!(structure.validate().is_err());
        assert!(structure.validate().unwrap_err().contains("root path cannot be empty"));
    }

    #[test]
    fn test_add_dependency_link() {
        let mut structure = SymlinkStructure::new(
            PathBuf::from("/project/node_modules"),
            Ecosystem::JavaScript,
        );
        let dependency = sample_dependency();
        let global_store = PathBuf::from("/global/store");
        let config = SymlinkConfig::default();

        let result = structure.add_dependency_link(&dependency, &global_store, &config);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), SymlinkStatus::Created));
        
        assert!(structure.has_link("react"));
        let entry = structure.get_link("react").unwrap();
        assert_eq!(entry.name, "react");
        assert_eq!(entry.version, "18.2.0");
        assert_eq!(entry.target_hash, "a".repeat(64));
    }

    #[test]
    fn test_add_duplicate_link() {
        let mut structure = SymlinkStructure::new(
            PathBuf::from("/project/node_modules"),
            Ecosystem::JavaScript,
        );
        let dependency = sample_dependency();
        let global_store = PathBuf::from("/global/store");
        let config = SymlinkConfig::default();

        // Add first time
        structure.add_dependency_link(&dependency, &global_store, &config).unwrap();
        
        // Add second time - should return AlreadyExists
        let result = structure.add_dependency_link(&dependency, &global_store, &config);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), SymlinkStatus::AlreadyExists));
    }

    #[test]
    fn test_remove_link() {
        let mut structure = SymlinkStructure::new(
            PathBuf::from("/project/node_modules"),
            Ecosystem::JavaScript,
        );
        let dependency = sample_dependency();
        let global_store = PathBuf::from("/global/store");
        let config = SymlinkConfig::default();

        // Add link
        structure.add_dependency_link(&dependency, &global_store, &config).unwrap();
        assert!(structure.has_link("react"));

        // Remove link
        let removed = structure.remove_link("react").unwrap();
        assert!(removed);
        assert!(!structure.has_link("react"));

        // Try to remove non-existent link
        let not_removed = structure.remove_link("vue").unwrap();
        assert!(!not_removed);
    }

    #[test]
    fn test_link_stats() {
        let mut structure = SymlinkStructure::new(
            PathBuf::from("/project/node_modules"),
            Ecosystem::JavaScript,
        );
        let dependency = sample_dependency();
        let global_store = PathBuf::from("/global/store");
        let config = SymlinkConfig::default();

        // Initially no links
        assert_eq!(structure.link_count(), 0);
        let (existing, missing) = structure.get_link_stats();
        assert_eq!(existing, 0);
        assert_eq!(missing, 0);

        // Add link
        structure.add_dependency_link(&dependency, &global_store, &config).unwrap();
        assert_eq!(structure.link_count(), 1);
        let (existing, missing) = structure.get_link_stats();
        assert_eq!(existing, 0); // Created but not marked as existing
        assert_eq!(missing, 1);

        // Mark as existing
        structure.update_link_status("react", true);
        let (existing, missing) = structure.get_link_stats();
        assert_eq!(existing, 1);
        assert_eq!(missing, 0);
    }

    #[test]
    fn test_symlink_entry_validation() {
        let entry = SymlinkEntry {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            target_path: PathBuf::from("/global/store/packages/test"),
            link_path: PathBuf::from("test-package"),
            link_type: SymlinkType::Directory,
            exists: true,
            created_at: "2025-09-11T10:00:00Z".to_string(),
            target_hash: "b".repeat(64),
            is_dev_dependency: false,
        };

        assert!(entry.validate().is_ok());
        assert!(entry.is_valid());
        assert_eq!(entry.identifier(), "test-package@1.0.0");
        assert!(entry.targets_hash(&"b".repeat(64)));
    }

    #[test]
    fn test_symlink_entry_validation_invalid_hash() {
        let entry = SymlinkEntry {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            target_path: PathBuf::from("/global/store/packages/test"),
            link_path: PathBuf::from("test-package"),
            link_type: SymlinkType::Directory,
            exists: true,
            created_at: "2025-09-11T10:00:00Z".to_string(),
            target_hash: "invalid-hash".to_string(),
            is_dev_dependency: false,
        };

        assert!(entry.validate().is_err());
        assert!(entry.validate().unwrap_err().contains("valid SHA-256"));
    }

    #[test]
    fn test_symlink_config_variants() {
        let windows_config = SymlinkConfig::windows_optimized();
        assert!(windows_config.use_junctions_on_windows);
        assert!(windows_config.fallback_to_hardlinks);

        let unix_config = SymlinkConfig::unix_optimized();
        assert!(!unix_config.use_junctions_on_windows);
        assert!(!unix_config.fallback_to_hardlinks);

        let dev_config = SymlinkConfig::development();
        assert!(dev_config.overwrite_existing);
        assert!(!dev_config.validate_targets);
    }

    #[test]
    fn test_cleanup_broken_links() {
        let mut structure = SymlinkStructure::new(
            PathBuf::from("/project/node_modules"),
            Ecosystem::JavaScript,
        );
        
        // Add a valid entry and mark it as existing
        let dependency = sample_dependency();
        let global_store = PathBuf::from("/global/store");
        let config = SymlinkConfig::default();
        structure.add_dependency_link(&dependency, &global_store, &config).unwrap();
        structure.update_link_status("react", true); // Mark as existing
        
        // Add an invalid entry directly (simulating corruption)
        let invalid_entry = SymlinkEntry {
            name: "broken".to_string(),
            version: "1.0.0".to_string(),
            target_path: PathBuf::from("/nonexistent"),
            link_path: PathBuf::from("broken"),
            link_type: SymlinkType::Directory,
            exists: false,
            created_at: "".to_string(), // Invalid timestamp
            target_hash: "c".repeat(64),
            is_dev_dependency: false,
        };
        structure.links.insert("broken".to_string(), invalid_entry);

        assert_eq!(structure.link_count(), 2);
        
        let removed = structure.cleanup_broken_links();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], "broken");
        assert_eq!(structure.link_count(), 1);
        assert!(structure.has_link("react"));
    }

    #[test]
    fn test_full_link_path() {
        let structure = SymlinkStructure::new(
            PathBuf::from("/project/node_modules"),
            Ecosystem::JavaScript,
        );
        
        let mut structure_with_link = structure;
        let dependency = sample_dependency();
        let global_store = PathBuf::from("/global/store");
        let config = SymlinkConfig::default();
        structure_with_link.add_dependency_link(&dependency, &global_store, &config).unwrap();

        let full_path = structure_with_link.get_full_link_path("react");
        assert!(full_path.is_some());
        assert_eq!(full_path.unwrap(), PathBuf::from("/project/node_modules/react"));

        let missing_path = structure_with_link.get_full_link_path("vue");
        assert!(missing_path.is_none());
    }

    #[test]
    fn test_ecosystem_support() {
        let js_structure = SymlinkStructure::new(
            PathBuf::from("/project/node_modules"),
            Ecosystem::JavaScript,
        );

        assert!(js_structure.supports_ecosystem(&Ecosystem::JavaScript));
        assert!(!js_structure.supports_ecosystem(&Ecosystem::Python));
        assert_eq!(js_structure.identifier(), "javascript:/project/node_modules");
    }

    #[test]
    fn test_link_type_determination() {
        let structure = SymlinkStructure::new(
            PathBuf::from("/project/node_modules"),
            Ecosystem::JavaScript,
        );

        let windows_config = SymlinkConfig::windows_optimized();
        let unix_config = SymlinkConfig::unix_optimized();

        #[cfg(windows)]
        {
            let link_type = structure.determine_link_type(&windows_config);
            assert_eq!(link_type, SymlinkType::Junction);
        }

        let link_type = structure.determine_link_type(&unix_config);
        assert_eq!(link_type, SymlinkType::Directory);
    }

    #[test]
    fn test_links_by_type() {
        let mut structure = SymlinkStructure::new(
            PathBuf::from("/project/node_modules"),
            Ecosystem::JavaScript,
        );
        let dependency = sample_dependency();
        let global_store = PathBuf::from("/global/store");
        let config = SymlinkConfig::default();

        structure.add_dependency_link(&dependency, &global_store, &config).unwrap();
        
        let directory_links = structure.get_links_by_type(&SymlinkType::Directory);
        // The count depends on platform - on Unix it should be 1, on Windows it might be 0 if junctions are used
        assert!(directory_links.len() <= 1);
        
        let file_links = structure.get_links_by_type(&SymlinkType::File);
        assert_eq!(file_links.len(), 0);
    }
}
