use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::models::ecosystem::Ecosystem;
use crate::models::package::Package;

/// Content-addressable storage system for packages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalStore {
    /// Root path to the global store directory (typically ~/.ppm-store)
    pub root_path: PathBuf,
    /// Map of package hashes to their storage entries
    pub packages: HashMap<String, PackageEntry>,
    /// Registry cache by ecosystem for faster lookups
    pub registry_cache: HashMap<Ecosystem, RegistryCache>,
}

/// Entry for a package stored in the global store
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageEntry {
    /// Content hash of the package (SHA-256)
    pub hash: String,
    /// Relative path within the global store
    pub store_path: String,
    /// Size of the package in bytes
    pub size: u64,
    /// When this package was first stored
    pub stored_at: String, // RFC 3339 timestamp
    /// Number of projects referencing this package
    pub reference_count: u32,
    /// Last time this package was accessed
    pub last_accessed: String, // RFC 3339 timestamp
    /// Which ecosystem this package belongs to
    pub ecosystem: Ecosystem,
    /// Package name for easier lookups
    pub name: String,
    /// Package version for easier lookups
    pub version: String,
}

/// Cache of registry metadata per ecosystem
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistryCache {
    /// Ecosystem this cache represents
    pub ecosystem: Ecosystem,
    /// Cached package metadata by package name
    pub packages: HashMap<String, CachedPackageInfo>,
    /// When this cache was last updated
    pub last_updated: String, // RFC 3339 timestamp
    /// Cache expiration time in seconds
    pub cache_ttl: u64,
}

/// Cached information about a package from registry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedPackageInfo {
    /// Package name
    pub name: String,
    /// Available versions
    pub versions: Vec<String>,
    /// Latest version
    pub latest_version: String,
    /// When this package info was cached
    pub cached_at: String, // RFC 3339 timestamp
}

impl GlobalStore {
    /// Create a new GlobalStore with the given root path
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path,
            packages: HashMap::new(),
            registry_cache: HashMap::new(),
        }
    }

    /// Create a GlobalStore with default location (~/.ppm-store)
    pub fn default_location() -> Result<Self, String> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| "Could not determine home directory".to_string())?;
        
        let mut store_path = PathBuf::from(home);
        store_path.push(".ppm-store");
        
        Ok(Self::new(store_path))
    }

    /// Validate the global store configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate root path is not empty
        if self.root_path.as_os_str().is_empty() {
            return Err("Global store root path cannot be empty".to_string());
        }

        // Validate all package entries
        for (hash, entry) in &self.packages {
            if hash != &entry.hash {
                return Err(format!(
                    "Package entry hash mismatch: key '{}' vs entry '{}'",
                    hash, entry.hash
                ));
            }
            entry.validate()?;
        }

        // Validate registry caches
        for (ecosystem, cache) in &self.registry_cache {
            if ecosystem != &cache.ecosystem {
                return Err(format!(
                    "Registry cache ecosystem mismatch: key '{:?}' vs cache '{:?}'",
                    ecosystem, cache.ecosystem
                ));
            }
            cache.validate()?;
        }

        Ok(())
    }

    /// Store a package in the global store and return its hash
    pub fn store_package(&mut self, package: &Package) -> Result<String, String> {
        // Validate package first
        package.validate()?;

        let hash = package.hash.clone();
        
        // Check if package already exists
        if self.packages.contains_key(&hash) {
            // Update reference count and access time
            if let Some(entry) = self.packages.get_mut(&hash) {
                entry.reference_count += 1;
                entry.last_accessed = current_timestamp();
            }
            return Ok(hash);
        }

        // Create new package entry
        let entry = PackageEntry {
            hash: hash.clone(),
            store_path: self.generate_store_path(&hash),
            size: self.calculate_package_size(package),
            stored_at: current_timestamp(),
            reference_count: 1,
            last_accessed: current_timestamp(),
            ecosystem: package.ecosystem,
            name: package.name.clone(),
            version: package.version.clone(),
        };

        self.packages.insert(hash.clone(), entry);
        Ok(hash)
    }

    /// Get a package by its hash
    pub fn get_package(&self, hash: &str) -> Option<&PackageEntry> {
        self.packages.get(hash)
    }

    /// Get a mutable reference to a package entry
    pub fn get_package_mut(&mut self, hash: &str) -> Option<&mut PackageEntry> {
        self.packages.get_mut(hash)
    }

    /// Find packages by name and ecosystem
    pub fn find_packages(&self, name: &str, ecosystem: &Ecosystem) -> Vec<&PackageEntry> {
        self.packages
            .values()
            .filter(|entry| entry.name == name && &entry.ecosystem == ecosystem)
            .collect()
    }

    /// Remove a package from the store (if reference count reaches zero)
    pub fn remove_package(&mut self, hash: &str) -> Result<bool, String> {
        if let Some(entry) = self.packages.get_mut(hash) {
            if entry.reference_count > 1 {
                entry.reference_count -= 1;
                Ok(false) // Not removed, still has references
            } else {
                self.packages.remove(hash);
                Ok(true) // Removed
            }
        } else {
            Err(format!("Package with hash '{}' not found", hash))
        }
    }

    /// Clean up orphaned packages (with zero references)
    pub fn cleanup_orphaned(&mut self) -> Vec<String> {
        let mut removed = Vec::new();
        
        self.packages.retain(|hash, entry| {
            if entry.reference_count == 0 {
                removed.push(hash.clone());
                false
            } else {
                true
            }
        });
        
        removed
    }

    /// Get packages by ecosystem
    pub fn get_packages_by_ecosystem(&self, ecosystem: &Ecosystem) -> Vec<&PackageEntry> {
        self.packages
            .values()
            .filter(|entry| &entry.ecosystem == ecosystem)
            .collect()
    }

    /// Get total storage size
    pub fn total_size(&self) -> u64 {
        self.packages.values().map(|entry| entry.size).sum()
    }

    /// Get package count
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }

    /// Generate store path for a package hash
    fn generate_store_path(&self, hash: &str) -> String {
        // Use content-addressable storage pattern: first 2 chars / next 2 chars / full hash
        let prefix1 = &hash[0..2];
        let prefix2 = &hash[2..4];
        format!("packages/{}/{}/{}", prefix1, prefix2, hash)
    }

    /// Calculate package size (simplified estimation)
    fn calculate_package_size(&self, package: &Package) -> u64 {
        // Simple estimation based on package name and version length
        // In a real implementation, this would calculate actual file size
        (package.name.len() + package.version.len()) as u64 * 1024
    }

    /// Get registry cache for an ecosystem
    pub fn get_registry_cache(&self, ecosystem: &Ecosystem) -> Option<&RegistryCache> {
        self.registry_cache.get(ecosystem)
    }

    /// Update registry cache for an ecosystem
    pub fn update_registry_cache(&mut self, ecosystem: Ecosystem, cache: RegistryCache) {
        self.registry_cache.insert(ecosystem, cache);
    }

    /// Check if registry cache is expired for an ecosystem
    pub fn is_cache_expired(&self, ecosystem: &Ecosystem) -> bool {
        if let Some(cache) = self.registry_cache.get(ecosystem) {
            cache.is_expired()
        } else {
            true // No cache means expired
        }
    }

    /// Get full file path for a package
    pub fn get_package_path(&self, hash: &str) -> Option<PathBuf> {
        self.packages.get(hash).map(|entry| {
            let mut path = self.root_path.clone();
            path.push(&entry.store_path);
            path
        })
    }

    /// Update package access time
    pub fn update_access_time(&mut self, hash: &str) {
        if let Some(entry) = self.packages.get_mut(hash) {
            entry.last_accessed = current_timestamp();
        }
    }
}

impl PackageEntry {
    /// Validate the package entry
    pub fn validate(&self) -> Result<(), String> {
        // Validate hash
        if !self.is_valid_sha256(&self.hash) {
            return Err("Hash must be a valid SHA-256 (64 hex characters)".to_string());
        }

        // Validate store path is not empty
        if self.store_path.is_empty() {
            return Err("Store path cannot be empty".to_string());
        }

        // Validate name and version are not empty
        if self.name.is_empty() {
            return Err("Package name cannot be empty".to_string());
        }

        if self.version.is_empty() {
            return Err("Package version cannot be empty".to_string());
        }

        // Validate timestamps are not empty
        if self.stored_at.is_empty() {
            return Err("Stored timestamp cannot be empty".to_string());
        }

        if self.last_accessed.is_empty() {
            return Err("Last accessed timestamp cannot be empty".to_string());
        }

        Ok(())
    }

    /// Check if a string is a valid SHA-256 hash
    fn is_valid_sha256(&self, hash: &str) -> bool {
        hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Get age of package entry in seconds
    pub fn age_seconds(&self) -> u64 {
        // Simplified implementation - in real code would parse timestamps
        0
    }

    /// Check if package is recently accessed (within 30 days)
    pub fn is_recently_accessed(&self) -> bool {
        self.age_seconds() < 30 * 24 * 60 * 60 // 30 days
    }
}

impl RegistryCache {
    /// Create a new registry cache
    pub fn new(ecosystem: Ecosystem, cache_ttl: u64) -> Self {
        Self {
            ecosystem,
            packages: HashMap::new(),
            last_updated: current_timestamp(),
            cache_ttl,
        }
    }

    /// Validate the registry cache
    pub fn validate(&self) -> Result<(), String> {
        // Validate cache TTL is reasonable
        if self.cache_ttl == 0 {
            return Err("Cache TTL must be greater than 0".to_string());
        }

        // Validate timestamp is not empty
        if self.last_updated.is_empty() {
            return Err("Last updated timestamp cannot be empty".to_string());
        }

        // Validate cached packages
        for (name, info) in &self.packages {
            if name != &info.name {
                return Err(format!(
                    "Cached package name mismatch: key '{}' vs info '{}'",
                    name, info.name
                ));
            }
            info.validate()?;
        }

        Ok(())
    }

    /// Check if cache is expired
    pub fn is_expired(&self) -> bool {
        // Simplified implementation - in real code would parse timestamps
        false
    }

    /// Add or update cached package info
    pub fn update_package(&mut self, info: CachedPackageInfo) {
        self.packages.insert(info.name.clone(), info);
        self.last_updated = current_timestamp();
    }

    /// Get cached package info
    pub fn get_package(&self, name: &str) -> Option<&CachedPackageInfo> {
        self.packages.get(name)
    }
}

impl CachedPackageInfo {
    /// Create new cached package info
    pub fn new(name: String, versions: Vec<String>, latest_version: String) -> Self {
        Self {
            name,
            versions,
            latest_version,
            cached_at: current_timestamp(),
        }
    }

    /// Validate cached package info
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Cached package name cannot be empty".to_string());
        }

        if self.versions.is_empty() {
            return Err("Cached package must have at least one version".to_string());
        }

        if self.latest_version.is_empty() {
            return Err("Latest version cannot be empty".to_string());
        }

        if !self.versions.contains(&self.latest_version) {
            return Err("Latest version must be in versions list".to_string());
        }

        if self.cached_at.is_empty() {
            return Err("Cached timestamp cannot be empty".to_string());
        }

        Ok(())
    }

    /// Check if this package has a specific version
    pub fn has_version(&self, version: &str) -> bool {
        self.versions.contains(&version.to_string())
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

    fn sample_package() -> Package {
        Package::new(
            "react".to_string(),
            "18.2.0".to_string(),
            Ecosystem::JavaScript,
            "a".repeat(64),
            std::path::PathBuf::from("packages/react-18.2.0"),
        )
    }

    #[test]
    fn test_global_store_creation() {
        let store_path = PathBuf::from("/tmp/ppm-store");
        let store = GlobalStore::new(store_path.clone());

        assert_eq!(store.root_path, store_path);
        assert!(store.packages.is_empty());
        assert!(store.registry_cache.is_empty());
    }

    #[test]
    fn test_global_store_default_location() {
        // This test depends on environment variables, so we'll just check it doesn't panic
        let _result = GlobalStore::default_location();
    }

    #[test]
    fn test_global_store_validation_success() {
        let store = GlobalStore::new(PathBuf::from("/tmp/ppm-store"));
        assert!(store.validate().is_ok());
    }

    #[test]
    fn test_global_store_validation_empty_path() {
        let store = GlobalStore::new(PathBuf::new());
        assert!(store.validate().is_err());
        assert!(store.validate().unwrap_err().contains("root path cannot be empty"));
    }

    #[test]
    fn test_store_package() {
        let mut store = GlobalStore::new(PathBuf::from("/tmp/ppm-store"));
        let package = sample_package();
        
        let result = store.store_package(&package);
        assert!(result.is_ok());
        
        let hash = result.unwrap();
        assert_eq!(hash, package.hash);
        assert!(store.packages.contains_key(&hash));
        
        let entry = store.get_package(&hash).unwrap();
        assert_eq!(entry.name, "react");
        assert_eq!(entry.version, "18.2.0");
        assert_eq!(entry.ecosystem, Ecosystem::JavaScript);
        assert_eq!(entry.reference_count, 1);
    }

    #[test]
    fn test_store_duplicate_package() {
        let mut store = GlobalStore::new(PathBuf::from("/tmp/ppm-store"));
        let package = sample_package();
        
        // Store first time
        let hash1 = store.store_package(&package).unwrap();
        assert_eq!(store.get_package(&hash1).unwrap().reference_count, 1);
        
        // Store second time - should increment reference count
        let hash2 = store.store_package(&package).unwrap();
        assert_eq!(hash1, hash2);
        assert_eq!(store.get_package(&hash1).unwrap().reference_count, 2);
    }

    #[test]
    fn test_find_packages() {
        let mut store = GlobalStore::new(PathBuf::from("/tmp/ppm-store"));
        let package = sample_package();
        
        store.store_package(&package).unwrap();
        
        let found = store.find_packages("react", &Ecosystem::JavaScript);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "react");
        
        let not_found = store.find_packages("vue", &Ecosystem::JavaScript);
        assert!(not_found.is_empty());
        
        let wrong_ecosystem = store.find_packages("react", &Ecosystem::Python);
        assert!(wrong_ecosystem.is_empty());
    }

    #[test]
    fn test_remove_package() {
        let mut store = GlobalStore::new(PathBuf::from("/tmp/ppm-store"));
        let package = sample_package();
        let hash = store.store_package(&package).unwrap();
        
        // Store twice to get reference count of 2
        store.store_package(&package).unwrap();
        assert_eq!(store.get_package(&hash).unwrap().reference_count, 2);
        
        // First removal decrements count
        let removed = store.remove_package(&hash).unwrap();
        assert!(!removed);
        assert_eq!(store.get_package(&hash).unwrap().reference_count, 1);
        
        // Second removal actually removes
        let removed = store.remove_package(&hash).unwrap();
        assert!(removed);
        assert!(store.get_package(&hash).is_none());
    }

    #[test]
    fn test_cleanup_orphaned() {
        let mut store = GlobalStore::new(PathBuf::from("/tmp/ppm-store"));
        let package = sample_package();
        let hash = store.store_package(&package).unwrap();
        
        // Manually set reference count to 0
        store.packages.get_mut(&hash).unwrap().reference_count = 0;
        
        let removed = store.cleanup_orphaned();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], hash);
        assert!(store.get_package(&hash).is_none());
    }

    #[test]
    fn test_package_entry_validation() {
        let entry = PackageEntry {
            hash: "a".repeat(64),
            store_path: "packages/aa/aaa.../content".to_string(),
            size: 1024,
            stored_at: "2025-09-11T10:00:00Z".to_string(),
            reference_count: 1,
            last_accessed: "2025-09-11T10:00:00Z".to_string(),
            ecosystem: Ecosystem::JavaScript,
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
        };

        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_package_entry_validation_invalid_hash() {
        let entry = PackageEntry {
            hash: "invalid-hash".to_string(),
            store_path: "packages/test".to_string(),
            size: 1024,
            stored_at: "2025-09-11T10:00:00Z".to_string(),
            reference_count: 1,
            last_accessed: "2025-09-11T10:00:00Z".to_string(),
            ecosystem: Ecosystem::JavaScript,
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
        };

        assert!(entry.validate().is_err());
        assert!(entry.validate().unwrap_err().contains("valid SHA-256"));
    }

    #[test]
    fn test_registry_cache() {
        let mut cache = RegistryCache::new(Ecosystem::JavaScript, 3600);
        
        let package_info = CachedPackageInfo::new(
            "react".to_string(),
            vec!["18.0.0".to_string(), "18.1.0".to_string(), "18.2.0".to_string()],
            "18.2.0".to_string(),
        );
        
        cache.update_package(package_info);
        
        let retrieved = cache.get_package("react").unwrap();
        assert_eq!(retrieved.name, "react");
        assert_eq!(retrieved.latest_version, "18.2.0");
        assert!(retrieved.has_version("18.1.0"));
        assert!(!retrieved.has_version("17.0.0"));
    }

    #[test]
    fn test_cached_package_info_validation() {
        let info = CachedPackageInfo::new(
            "test-package".to_string(),
            vec!["1.0.0".to_string(), "1.1.0".to_string()],
            "1.1.0".to_string(),
        );

        assert!(info.validate().is_ok());
    }

    #[test]
    fn test_store_path_generation() {
        let store = GlobalStore::new(PathBuf::from("/tmp/ppm-store"));
        let hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let path = store.generate_store_path(hash);
        
        assert_eq!(path, "packages/ab/cd/abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890");
    }
}
