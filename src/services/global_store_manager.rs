use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tokio::fs as async_fs;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::models::{
    global_store::{GlobalStore, PackageEntry},
    package::Package,
    ecosystem::Ecosystem,
};
use crate::utils::error::PpmError;

/// Global store manager service for high-level store operations
#[derive(Debug)]
pub struct GlobalStoreManager {
    /// The global store instance
    store: GlobalStore,
    /// Configuration for store operations
    config: StoreConfig,
}

/// Configuration for global store operations
#[derive(Debug, Clone)]
pub struct StoreConfig {
    /// Automatically create store directory if it doesn't exist
    pub auto_create: bool,
    /// Maximum cache size in bytes (0 = unlimited)
    pub max_cache_size: u64,
    /// Default cache TTL in seconds
    pub cache_ttl: u64,
    /// Enable automatic cleanup of orphaned packages
    pub auto_cleanup: bool,
    /// Cleanup threshold - remove packages not accessed in this many days
    pub cleanup_threshold_days: u32,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            auto_create: true,
            max_cache_size: 5 * 1024 * 1024 * 1024, // 5GB
            cache_ttl: 3600, // 1 hour
            auto_cleanup: true,
            cleanup_threshold_days: 30,
        }
    }
}

/// Statistics about the global store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreStats {
    /// Total number of packages stored
    pub total_packages: usize,
    /// Total size of stored packages in bytes
    pub total_size: u64,
    /// Number of packages by ecosystem
    pub packages_by_ecosystem: HashMap<String, usize>,
    /// Cache statistics by ecosystem
    pub cache_stats: HashMap<String, CacheStats>,
    /// Number of orphaned packages (reference count = 0)
    pub orphaned_packages: usize,
}

/// Cache statistics for a specific ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cached packages
    pub package_count: usize,
    /// Cache age in seconds
    pub cache_age: u64,
    /// Whether cache is expired
    pub is_expired: bool,
}

/// Result of cleanup operations
#[derive(Debug, Clone)]
pub struct CleanupResult {
    /// Number of packages removed
    pub packages_removed: usize,
    /// Number of bytes freed
    pub bytes_freed: u64,
    /// List of removed package hashes
    pub removed_hashes: Vec<String>,
    /// Errors encountered during cleanup
    pub errors: Vec<String>,
}

impl GlobalStoreManager {
    /// Create a new GlobalStoreManager with default store location
    pub async fn new() -> Result<Self, PpmError> {
        let store = GlobalStore::default_location()
            .map_err(PpmError::ConfigError)?;
        let config = StoreConfig::default();
        
        let mut manager = Self { store, config };
        manager.initialize().await?;
        Ok(manager)
    }

    /// Create a new GlobalStoreManager with custom store path
    pub async fn with_path(store_path: PathBuf) -> Result<Self, PpmError> {
        let store = GlobalStore::new(store_path);
        let config = StoreConfig::default();
        
        let mut manager = Self { store, config };
        manager.initialize().await?;
        Ok(manager)
    }

    /// Create a new GlobalStoreManager with custom configuration
    pub async fn with_config(store_path: PathBuf, config: StoreConfig) -> Result<Self, PpmError> {
        let store = GlobalStore::new(store_path);
        let mut manager = Self { store, config };
        manager.initialize().await?;
        Ok(manager)
    }

    /// Initialize the global store (create directories, load metadata)
    pub async fn initialize(&mut self) -> Result<(), PpmError> {
        // Create store directory if it doesn't exist
        if self.config.auto_create {
            self.ensure_store_directory().await?;
        }

        // Load existing store metadata if available
        self.load_store_metadata().await?;

        // Save initial metadata if this is a new store
        self.save_store_metadata().await?;

        // Validate store consistency
        self.store.validate()
            .map_err(PpmError::ValidationError)?;

        Ok(())
    }

    /// Ensure the store directory exists
    async fn ensure_store_directory(&self) -> Result<(), PpmError> {
        let store_path = &self.store.root_path;
        
        if !store_path.exists() {
            async_fs::create_dir_all(store_path).await
                .map_err(PpmError::IoError)?;
        }

        // Create subdirectories
        let packages_dir = store_path.join("packages");
        if !packages_dir.exists() {
            async_fs::create_dir_all(packages_dir).await
                .map_err(PpmError::IoError)?;
        }

        let cache_dir = store_path.join("cache");
        if !cache_dir.exists() {
            async_fs::create_dir_all(cache_dir).await
                .map_err(PpmError::IoError)?;
        }

        Ok(())
    }

    /// Load store metadata from disk
    async fn load_store_metadata(&mut self) -> Result<(), PpmError> {
        let metadata_path = self.store.root_path.join("metadata.json");
        
        if metadata_path.exists() {
            let content = async_fs::read_to_string(&metadata_path).await
                .map_err(PpmError::IoError)?;

            let loaded_store: GlobalStore = serde_json::from_str(&content)
                .map_err(|e| PpmError::ValidationError(format!("Failed to parse store metadata: {}", e)))?;

            // Merge loaded data (keeping the root path from current store)
            self.store.packages = loaded_store.packages;
            self.store.registry_cache = loaded_store.registry_cache;
        }

        Ok(())
    }

    /// Save store metadata to disk
    pub async fn save_store_metadata(&self) -> Result<(), PpmError> {
        let metadata_path = self.store.root_path.join("metadata.json");
        
        let content = serde_json::to_string_pretty(&self.store)
            .map_err(|e| PpmError::ValidationError(format!("Failed to serialize store metadata: {}", e)))?;
        
        async_fs::write(&metadata_path, content).await
            .map_err(PpmError::IoError)?;

        Ok(())
    }

    /// Store a package in the global store
    pub async fn store_package(&mut self, package: &Package) -> Result<String, PpmError> {
        let hash = self.store.store_package(package)
            .map_err(PpmError::ValidationError)?;

        // Copy package files to store location
        if let Some(package_path) = self.store.get_package_path(&hash) {
            self.copy_package_to_store(package, &package_path).await?;
        }

        // Save updated metadata
        self.save_store_metadata().await?;

        Ok(hash)
    }

    /// Copy package files to the store
    async fn copy_package_to_store(&self, package: &Package, target_path: &Path) -> Result<(), PpmError> {
        // Ensure target directory exists
        if let Some(parent) = target_path.parent() {
            async_fs::create_dir_all(parent).await
                .map_err(PpmError::IoError)?;
        }

        // For now, we'll create a simple package structure
        // In a real implementation, this would copy the actual package files
        let package_info = serde_json::to_string_pretty(package)
            .map_err(|e| PpmError::ValidationError(format!("Failed to serialize package: {}", e)))?;
        
        async_fs::write(target_path.join("package.json"), package_info).await
            .map_err(PpmError::IoError)?;

        Ok(())
    }

    /// Remove a package from the global store
    pub async fn remove_package(&mut self, hash: &str) -> Result<bool, PpmError> {
        let removed = self.store.remove_package(hash)
            .map_err(PpmError::ValidationError)?;

        if removed {
            // Remove package files from disk
            if let Some(package_path) = self.store.get_package_path(hash) {
                if package_path.exists() {
                    async_fs::remove_dir_all(&package_path).await
                        .map_err(PpmError::IoError)?;
                }
            }

            // Save updated metadata
            self.save_store_metadata().await?;
        }

        Ok(removed)
    }

    /// Find packages by name and ecosystem
    pub fn find_packages(&self, name: &str, ecosystem: &Ecosystem) -> Vec<&PackageEntry> {
        self.store.find_packages(name, ecosystem)
    }

    /// Get package entry by hash
    pub fn get_package(&self, hash: &str) -> Option<&PackageEntry> {
        self.store.get_package(hash)
    }

    /// Get store statistics
    pub fn get_stats(&self) -> StoreStats {
        let mut packages_by_ecosystem = HashMap::new();
        let mut cache_stats = HashMap::new();

        // Count packages by ecosystem
        for entry in self.store.packages.values() {
            let ecosystem_str = entry.ecosystem.to_string();
            *packages_by_ecosystem.entry(ecosystem_str).or_insert(0) += 1;
        }

        // Get cache statistics
        for (ecosystem, cache) in &self.store.registry_cache {
            let ecosystem_str = ecosystem.to_string();
            cache_stats.insert(ecosystem_str, CacheStats {
                package_count: cache.packages.len(),
                cache_age: 0, // Simplified - would calculate actual age
                is_expired: cache.is_expired(),
            });
        }

        // Count orphaned packages
        let orphaned_packages = self.store.packages.values()
            .filter(|entry| entry.reference_count == 0)
            .count();

        StoreStats {
            total_packages: self.store.package_count(),
            total_size: self.store.total_size(),
            packages_by_ecosystem,
            cache_stats,
            orphaned_packages,
        }
    }

    /// Cleanup orphaned packages
    pub async fn cleanup_orphaned(&mut self) -> Result<CleanupResult, PpmError> {
        let mut result = CleanupResult {
            packages_removed: 0,
            bytes_freed: 0,
            removed_hashes: Vec::new(),
            errors: Vec::new(),
        };

        let orphaned_hashes = self.store.cleanup_orphaned();
        
        for hash in orphaned_hashes {
            if let Some(entry) = self.store.get_package(&hash) {
                let size = entry.size;

                // Remove package files
                if let Some(package_path) = self.store.get_package_path(&hash) {
                    if package_path.exists() {
                        match async_fs::remove_dir_all(&package_path).await {
                            Ok(()) => {
                                result.packages_removed += 1;
                                result.bytes_freed += size;
                                result.removed_hashes.push(hash.clone());
                            }
                            Err(e) => {
                                result.errors.push(format!("Failed to remove package {}: {}", hash, e));
                            }
                        }
                    }
                }
            }
        }

        // Save updated metadata
        if result.packages_removed > 0 {
            self.save_store_metadata().await?;
        }

        Ok(result)
    }

    /// Cleanup old cache entries
    pub async fn cleanup_cache(&mut self) -> Result<(), PpmError> {
        let mut registries_to_clean = Vec::new();

        for (ecosystem, cache) in &self.store.registry_cache {
            if cache.is_expired() {
                registries_to_clean.push(*ecosystem);
            }
        }

        for ecosystem in registries_to_clean {
            self.store.registry_cache.remove(&ecosystem);
        }

        // Save updated metadata
        self.save_store_metadata().await?;

        Ok(())
    }

    /// Get the underlying GlobalStore (read-only access)
    pub fn store(&self) -> &GlobalStore {
        &self.store
    }

    /// Get mutable access to the underlying GlobalStore
    pub fn store_mut(&mut self) -> &mut GlobalStore {
        &mut self.store
    }

    /// Get store configuration
    pub fn config(&self) -> &StoreConfig {
        &self.config
    }

    /// Update store configuration
    pub fn set_config(&mut self, config: StoreConfig) {
        self.config = config;
    }

    /// Verify store integrity
    pub async fn verify_integrity(&self) -> Result<Vec<String>, PpmError> {
        let mut issues = Vec::new();

        // Check if all package files exist
        for (hash, _entry) in &self.store.packages {
            if let Some(package_path) = self.store.get_package_path(hash) {
                if !package_path.exists() {
                    issues.push(format!("Package files missing for hash: {}", hash));
                }
            }
        }

        // Validate store model
        if let Err(e) = self.store.validate() {
            issues.push(format!("Store validation failed: {}", e));
        }

        Ok(issues)
    }

    /// Get total disk usage of the store
    pub async fn get_disk_usage(&self) -> Result<u64, PpmError> {
        let store_path = &self.store.root_path;
        
        if !store_path.exists() {
            return Ok(0);
        }

        let mut total_size = 0u64;
        let mut stack = vec![store_path.clone()];

        while let Some(path) = stack.pop() {
            let mut entries = async_fs::read_dir(&path).await
                .map_err(PpmError::IoError)?;

            while let Some(entry) = entries.next_entry().await
                .map_err(PpmError::IoError)? {

                let metadata = entry.metadata().await
                    .map_err(PpmError::IoError)?;

                if metadata.is_dir() {
                    stack.push(entry.path());
                } else {
                    total_size += metadata.len();
                }
            }
        }

        Ok(total_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_global_store_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().to_path_buf();
        
        let manager = GlobalStoreManager::with_path(store_path.clone()).await;
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        assert_eq!(manager.store().root_path, store_path);
    }

    #[tokio::test]
    async fn test_store_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("test-store");
        
        let _manager = GlobalStoreManager::with_path(store_path.clone()).await.unwrap();
        
        assert!(store_path.exists());
        assert!(store_path.join("packages").exists());
        assert!(store_path.join("cache").exists());
    }

    #[tokio::test]
    async fn test_store_config() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().to_path_buf();
        
        let config = StoreConfig {
            auto_create: false,
            max_cache_size: 1024,
            cache_ttl: 60,
            auto_cleanup: false,
            cleanup_threshold_days: 7,
        };
        
        let _manager = GlobalStoreManager::with_config(store_path, config.clone()).await;
        assert!(_manager.is_ok());
        
        let manager = _manager.unwrap();
        assert_eq!(manager.config().max_cache_size, 1024);
        assert_eq!(manager.config().cache_ttl, 60);
    }

    #[tokio::test]
    async fn test_store_stats() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().to_path_buf();
        
        let _manager = GlobalStoreManager::with_path(store_path).await.unwrap();
        let stats = _manager.get_stats();
        
        assert_eq!(stats.total_packages, 0);
        assert_eq!(stats.total_size, 0);
        assert_eq!(stats.orphaned_packages, 0);
    }

    #[tokio::test]
    async fn test_metadata_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().to_path_buf();
        
        let _manager = GlobalStoreManager::with_path(store_path.clone()).await.unwrap();
        
        // Check that metadata file was created during initialization
        let metadata_path = store_path.join("metadata.json");
        assert!(metadata_path.exists());
    }

    #[tokio::test]
    async fn test_integrity_verification() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().to_path_buf();
        
        let manager = GlobalStoreManager::with_path(store_path).await.unwrap();
        let issues = manager.verify_integrity().await.unwrap();
        
        // New store should have no integrity issues
        assert!(issues.is_empty());
    }

    #[tokio::test]
    async fn test_disk_usage() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().to_path_buf();
        
        let manager = GlobalStoreManager::with_path(store_path).await.unwrap();
        let usage = manager.get_disk_usage().await.unwrap();
        
        // Should have some usage due to created directories and metadata file
        assert!(usage > 0);
    }
}
