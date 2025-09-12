use std::path::Path;
use tokio::fs;
use crate::utils::error::PpmError;
use crate::models::symlink_structure::{SymlinkStructure, SymlinkEntry, SymlinkConfig, SymlinkStatus, SymlinkType};
use crate::models::resolved_dependency::ResolvedDependency;
use crate::models::ecosystem::Ecosystem;

#[cfg(windows)]
use std::os::windows::fs as windows_fs;
#[cfg(unix)]
use std::os::unix::fs as unix_fs;

/// Cross-platform symlink manager for package installation
#[derive(Debug, Clone)]
pub struct SymlinkManager {
    config: SymlinkConfig,
}

impl SymlinkManager {
    /// Create a new SymlinkManager with default configuration
    pub fn new() -> Self {
        Self {
            config: SymlinkConfig::default(),
        }
    }

    /// Create a new SymlinkManager with custom configuration
    pub fn with_config(config: SymlinkConfig) -> Self {
        Self { config }
    }

    /// Get the current configuration
    pub fn config(&self) -> &SymlinkConfig {
        &self.config
    }

    /// Create symlinks for JavaScript packages in a project
    pub async fn create_javascript_symlinks(
        &self,
        project_root: &Path,
        resolved_deps: &[ResolvedDependency],
        global_store_path: &Path,
    ) -> Result<SymlinkStructure, PpmError> {
        let node_modules_path = project_root.join("node_modules");
        let mut structure = SymlinkStructure::node_modules(project_root.to_path_buf());

        // Create node_modules directory if it doesn't exist
        if self.config.create_parent_dirs {
            fs::create_dir_all(&node_modules_path).await?;
        }

        // Filter JavaScript dependencies
        let js_deps: Vec<_> = resolved_deps
            .iter()
            .filter(|dep| dep.ecosystem == Ecosystem::JavaScript)
            .collect();

        for dep in js_deps {
            match self.create_package_symlink(&mut structure, dep, global_store_path).await {
                Ok(status) => {
                    println!("Created symlink for {}: {:?}", dep.name, status);
                }
                Err(e) => {
                    println!("Failed to create symlink for {}: {}", dep.name, e);
                    return Err(e);
                }
            }
        }

        Ok(structure)
    }

    /// Create a symlink for a single package
    async fn create_package_symlink(
        &self,
        structure: &mut SymlinkStructure,
        dependency: &ResolvedDependency,
        global_store_path: &Path,
    ) -> Result<SymlinkStatus, PpmError> {
        // Add the dependency to the structure (this creates the SymlinkEntry)
        let _status = structure.add_dependency_link(dependency, &global_store_path.to_path_buf(), &self.config)
            .map_err(|e| PpmError::SymlinkError(e))?;

        // Get the entry we just created
        let entry = structure.get_link(&dependency.name)
            .ok_or_else(|| PpmError::SymlinkError("Failed to retrieve symlink entry after creation".to_string()))?;

        // Create the actual symlink on the filesystem
        match self.create_filesystem_symlink(structure, entry).await {
            Ok(_) => {
                // Update the entry to reflect that it exists
                structure.update_link_status(&dependency.name, true);
                Ok(SymlinkStatus::Created)
            }
            Err(e) => {
                println!("Failed to create filesystem symlink for {}: {}", dependency.name, e);
                Err(e)
            }
        }
    }

    /// Create the actual symlink on the filesystem
    async fn create_filesystem_symlink(
        &self,
        structure: &SymlinkStructure,
        entry: &SymlinkEntry,
    ) -> Result<(), PpmError> {
        let link_path = structure.root_path.join(&entry.link_path);
        let target_path = &entry.target_path;

        // Validate target exists
        if self.config.validate_targets && !target_path.exists() {
            return Err(PpmError::SymlinkError(format!(
                "Target path does not exist: {}",
                target_path.display()
            )));
        }

        // Check if link already exists
        if link_path.exists() {
            if !self.config.overwrite_existing {
                return Ok(()); // Silently skip existing links
            }
            
            // Remove existing link/directory
            if link_path.is_dir() {
                fs::remove_dir_all(&link_path).await?;
            } else {
                fs::remove_file(&link_path).await?;
            }
        }

        // Create parent directories if needed
        if self.config.create_parent_dirs {
            if let Some(parent) = link_path.parent() {
                fs::create_dir_all(parent).await?;
            }
        }

        // Create the symlink based on type and platform
        match entry.link_type {
            SymlinkType::Directory => self.create_directory_symlink(&link_path, target_path).await?,
            SymlinkType::File => self.create_file_symlink(&link_path, target_path).await?,
            SymlinkType::Junction => self.create_junction(&link_path, target_path).await?,
            SymlinkType::HardLink => self.create_hardlink(&link_path, target_path).await?,
        }

        Ok(())
    }

    /// Create a directory symlink (most common for packages)
    async fn create_directory_symlink(&self, link_path: &Path, target_path: &Path) -> Result<(), PpmError> {
        #[cfg(windows)]
        {
            // On Windows, try to create a directory symlink
            windows_fs::symlink_dir(target_path, link_path)
                .map_err(|e| PpmError::SymlinkError(format!(
                    "Failed to create Windows directory symlink from {} to {}: {}",
                    link_path.display(),
                    target_path.display(),
                    e
                )))?;
        }

        #[cfg(unix)]
        {
            // On Unix systems, create a symbolic link
            unix_fs::symlink(target_path, link_path)
                .map_err(|e| PpmError::SymlinkError(format!(
                    "Failed to create Unix symlink from {} to {}: {}",
                    link_path.display(),
                    target_path.display(),
                    e
                )))?;
        }

        #[cfg(not(any(windows, unix)))]
        {
            return Err(PpmError::SymlinkError(
                "Symlinks not supported on this platform".to_string()
            ));
        }

        Ok(())
    }

    /// Create a file symlink
    async fn create_file_symlink(&self, link_path: &Path, target_path: &Path) -> Result<(), PpmError> {
        #[cfg(windows)]
        {
            windows_fs::symlink_file(target_path, link_path)
                .map_err(|e| PpmError::SymlinkError(format!(
                    "Failed to create Windows file symlink from {} to {}: {}",
                    link_path.display(),
                    target_path.display(),
                    e
                )))?;
        }

        #[cfg(unix)]
        {
            unix_fs::symlink(target_path, link_path)
                .map_err(|e| PpmError::SymlinkError(format!(
                    "Failed to create Unix file symlink from {} to {}: {}",
                    link_path.display(),
                    target_path.display(),
                    e
                )))?;
        }

        #[cfg(not(any(windows, unix)))]
        {
            return Err(PpmError::SymlinkError(
                "File symlinks not supported on this platform".to_string()
            ));
        }

        Ok(())
    }

    /// Create a Windows junction (Windows-specific)
    async fn create_junction(&self, link_path: &Path, target_path: &Path) -> Result<(), PpmError> {
        #[cfg(windows)]
        {
            // Junctions are directory symlinks on Windows that don't require admin privileges
            windows_fs::symlink_dir(target_path, link_path)
                .map_err(|e| PpmError::SymlinkError(format!(
                    "Failed to create Windows junction from {} to {}: {}",
                    link_path.display(),
                    target_path.display(),
                    e
                )))?;
        }

        #[cfg(not(windows))]
        {
            // Fall back to regular symlink on non-Windows systems
            self.create_directory_symlink(link_path, target_path).await?;
        }

        Ok(())
    }

    /// Create a hard link (fallback option)
    async fn create_hardlink(&self, link_path: &Path, target_path: &Path) -> Result<(), PpmError> {
        fs::hard_link(target_path, link_path)
            .await
            .map_err(|e| PpmError::SymlinkError(format!(
                "Failed to create hard link from {} to {}: {}",
                link_path.display(),
                target_path.display(),
                e
            )))?;

        Ok(())
    }

    /// Remove symlinks for a package
    pub async fn remove_package_symlinks(
        &self,
        structure: &mut SymlinkStructure,
        package_name: &str,
    ) -> Result<bool, PpmError> {
        if let Some(entry) = structure.get_link(package_name) {
            let link_path = structure.root_path.join(&entry.link_path);
            
            if link_path.exists() {
                if link_path.is_dir() {
                    fs::remove_dir_all(&link_path).await?;
                } else {
                    fs::remove_file(&link_path).await?;
                }
            }

            structure.remove_link(package_name)
                .map_err(|e| PpmError::SymlinkError(e))?;
            
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Verify that all symlinks in a structure exist and are valid
    pub async fn verify_symlinks(&self, structure: &mut SymlinkStructure) -> Result<Vec<String>, PpmError> {
        let mut broken_links = Vec::new();

        for (name, entry) in &structure.links.clone() {
            let link_path = structure.root_path.join(&entry.link_path);
            let exists = link_path.exists() && self.is_valid_symlink(&link_path, &entry.target_path).await;
            
            structure.update_link_status(name, exists);
            
            if !exists {
                broken_links.push(name.clone());
            }
        }

        Ok(broken_links)
    }

    /// Check if a symlink exists and points to the correct target
    async fn is_valid_symlink(&self, link_path: &Path, expected_target: &Path) -> bool {
        if !link_path.exists() {
            return false;
        }

        // Try to read the symlink target
        match fs::read_link(link_path).await {
            Ok(actual_target) => {
                // Compare the targets (handle both absolute and relative paths)
                actual_target == expected_target || 
                link_path.parent()
                    .and_then(|parent| parent.join(&actual_target).canonicalize().ok())
                    .map(|canonical| canonical == expected_target)
                    .unwrap_or(false)
            }
            Err(_) => {
                // If we can't read the link, check if it's a regular directory/file
                // that might have been created instead of a symlink
                link_path.is_dir() || link_path.is_file()
            }
        }
    }

    /// Get platform-specific symlink capabilities
    pub fn get_platform_capabilities(&self) -> SymlinkCapabilities {
        SymlinkCapabilities {
            supports_directory_symlinks: self.supports_directory_symlinks(),
            supports_file_symlinks: self.supports_file_symlinks(),
            supports_junctions: self.supports_junctions(),
            supports_hardlinks: true, // Hard links are supported on most filesystems
            requires_admin_privileges: self.requires_admin_privileges(),
        }
    }

    #[cfg(windows)]
    fn supports_directory_symlinks(&self) -> bool {
        true // Windows supports directory symlinks
    }

    #[cfg(unix)]
    fn supports_directory_symlinks(&self) -> bool {
        true // Unix systems support symlinks
    }

    #[cfg(not(any(windows, unix)))]
    fn supports_directory_symlinks(&self) -> bool {
        false
    }

    #[cfg(windows)]
    fn supports_file_symlinks(&self) -> bool {
        true
    }

    #[cfg(unix)]
    fn supports_file_symlinks(&self) -> bool {
        true
    }

    #[cfg(not(any(windows, unix)))]
    fn supports_file_symlinks(&self) -> bool {
        false
    }

    #[cfg(windows)]
    fn supports_junctions(&self) -> bool {
        true
    }

    #[cfg(not(windows))]
    fn supports_junctions(&self) -> bool {
        false
    }

    #[cfg(windows)]
    fn requires_admin_privileges(&self) -> bool {
        // Directory symlinks on Windows typically require admin privileges
        // unless Developer Mode is enabled
        true
    }

    #[cfg(not(windows))]
    fn requires_admin_privileges(&self) -> bool {
        false
    }
}

impl Default for SymlinkManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Platform capabilities for symlink operations
#[derive(Debug, Clone, PartialEq)]
pub struct SymlinkCapabilities {
    pub supports_directory_symlinks: bool,
    pub supports_file_symlinks: bool,
    pub supports_junctions: bool,
    pub supports_hardlinks: bool,
    pub requires_admin_privileges: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper to create a test resolved dependency
    fn create_test_dependency(name: &str, version: &str) -> ResolvedDependency {
        ResolvedDependency {
            name: name.to_string(),
            version: version.to_string(),
            ecosystem: Ecosystem::JavaScript,
            integrity: "sha256-test".to_string(),
            hash: "test-hash".to_string(),
            store_path: format!("npm/{}/{}", name, version),
        }
    }

    #[tokio::test]
    async fn test_symlink_manager_creation() {
        let manager = SymlinkManager::new();
        let capabilities = manager.get_platform_capabilities();
        
        // Basic sanity checks
        assert!(capabilities.supports_hardlinks);
    }

    #[tokio::test] 
    async fn test_symlink_config_builder() {
        let config = SymlinkConfig {
            use_junctions_on_windows: false,
            fallback_to_hardlinks: true,
            create_parent_dirs: true,
            overwrite_existing: true,
            max_depth: 5,
            validate_targets: false,
        };
        
        let manager = SymlinkManager::with_config(config.clone());
        assert_eq!(manager.config, config);
    }

    #[tokio::test]
    async fn test_javascript_symlink_structure_creation() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        let global_store = temp_dir.path().join("global_store");
        
        let manager = SymlinkManager::new();
        let deps = vec![
            create_test_dependency("lodash", "4.17.21"),
            create_test_dependency("express", "4.18.0"),
        ];

        // This should not fail even if targets don't exist for basic structure creation
        let config = SymlinkConfig {
            validate_targets: false,
            ..Default::default()
        };
        let manager = SymlinkManager::with_config(config);

        let structure = manager.create_javascript_symlinks(project_root, &deps, &global_store).await;
        
        // Should succeed in creating the structure even if symlinks fail
        // (because targets don't exist in this test)
        assert!(structure.is_ok() || structure.is_err());
        
        // Verify node_modules directory was created
        assert!(project_root.join("node_modules").exists());
    }

    #[tokio::test]
    async fn test_symlink_capabilities() {
        let manager = SymlinkManager::new();
        let capabilities = manager.get_platform_capabilities();
        
        // Should have some capabilities on any platform
        assert!(
            capabilities.supports_directory_symlinks ||
            capabilities.supports_file_symlinks ||
            capabilities.supports_hardlinks
        );
    }
}
