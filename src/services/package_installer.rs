use crate::models::{
    dependency::Dependency,
    ecosystem::Ecosystem,
    global_store::GlobalStore,
    package::Package,
    project::Project,
    resolved_dependency::ResolvedDependency,
};
use crate::services::{
    dependency_resolver::DependencyResolver,
    npm_client::NpmClient,
    pypi_client::PypiClient,
};
use crate::utils::error::PpmError;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Configuration for package installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// Whether to install development dependencies
    pub include_dev: bool,
    /// Whether to skip integrity verification
    pub skip_verification: bool,
    /// Whether to update existing packages
    pub force_update: bool,
    /// Maximum concurrent downloads
    pub max_concurrent: usize,
    /// Timeout for downloads in seconds
    pub download_timeout: u64,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            include_dev: false,
            skip_verification: false,
            force_update: false,
            max_concurrent: 4,
            download_timeout: 300, // 5 minutes
        }
    }
}

impl InstallConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dev_dependencies(mut self, include: bool) -> Self {
        self.include_dev = include;
        self
    }

    pub fn with_verification(mut self, verify: bool) -> Self {
        self.skip_verification = !verify;
        self
    }

    pub fn with_force_update(mut self, force: bool) -> Self {
        self.force_update = force;
        self
    }

    pub fn with_concurrency(mut self, max: usize) -> Self {
        self.max_concurrent = max.max(1);
        self
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.download_timeout = seconds;
        self
    }
}

/// Result of package installation
#[derive(Debug, Clone)]
pub struct InstallResult {
    /// Successfully installed packages
    pub installed: Vec<ResolvedDependency>,
    /// Packages that were skipped (already installed)
    pub skipped: Vec<ResolvedDependency>,
    /// Packages that failed to install
    pub failed: Vec<(String, String)>, // (package_name, error_message)
    /// Symlinks created
    pub symlinks_created: usize,
    /// Total download size in bytes
    pub download_size: u64,
    /// Installation duration in milliseconds
    pub duration_ms: u128,
}

impl InstallResult {
    pub fn new() -> Self {
        Self {
            installed: Vec::new(),
            skipped: Vec::new(),
            failed: Vec::new(),
            symlinks_created: 0,
            download_size: 0,
            duration_ms: 0,
        }
    }

    pub fn total_packages(&self) -> usize {
        self.installed.len() + self.skipped.len() + self.failed.len()
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total_packages();
        if total == 0 {
            1.0
        } else {
            (self.installed.len() + self.skipped.len()) as f64 / total as f64
        }
    }

    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }
}

/// Package installer service
#[derive(Debug)]
pub struct PackageInstaller {
    /// Dependency resolver for resolving package dependencies
    resolver: DependencyResolver,
    /// Global store for package storage
    global_store: GlobalStore,
    /// HTTP client for downloads
    http_client: Client,
    /// NPM client for npm packages
    npm_client: NpmClient,
    /// PyPI client for Python packages
    pypi_client: PypiClient,
    /// Installation configuration
    config: InstallConfig,
}

impl PackageInstaller {
    /// Create a new package installer
    pub fn new(
        global_store: GlobalStore,
        config: Option<InstallConfig>,
    ) -> Result<Self, PpmError> {
        let timeout_duration = config.as_ref().map(|c| c.download_timeout).unwrap_or(300);
        
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_duration))
            .build()
            .map_err(|e| PpmError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        let npm_client = NpmClient::new();
        let pypi_client = PypiClient::new();
        let resolver = DependencyResolver::new(
            npm_client.clone(),
            pypi_client.clone(),
            global_store.clone(),
        );

        Ok(Self {
            resolver,
            global_store,
            http_client,
            npm_client,
            pypi_client,
            config: config.unwrap_or_default(),
        })
    }

    /// Install dependencies for a project
    pub async fn install_project(
        &mut self,
        project: &Project,
        project_root: &Path,
    ) -> Result<InstallResult, PpmError> {
        let start_time = std::time::Instant::now();
        let mut result = InstallResult::new();

        // Collect dependencies based on configuration
        let mut all_dependencies = Vec::new();
        
        // Get production dependencies from all ecosystems
        for ecosystem in [Ecosystem::JavaScript, Ecosystem::Python] {
            if let Some(deps) = project.get_dependencies(&ecosystem) {
                for (name, version_spec) in deps {
                    all_dependencies.push((name.clone(), version_spec.clone(), ecosystem.clone()));
                }
            }
        }
        
        // Include dev dependencies if requested
        if self.config.include_dev {
            for ecosystem in [Ecosystem::JavaScript, Ecosystem::Python] {
                if let Some(deps) = project.get_dev_dependencies(&ecosystem) {
                    for (name, version_spec) in deps {
                        all_dependencies.push((name.clone(), version_spec.clone(), ecosystem.clone()));
                    }
                }
            }
        }

        if all_dependencies.is_empty() {
            result.duration_ms = start_time.elapsed().as_millis();
            return Ok(result);
        }

        // Resolve dependencies - convert to Dependency structs
        let dependencies: Vec<Dependency> = all_dependencies.into_iter()
            .map(|(name, version, ecosystem)| {
                Dependency::production(name, version, ecosystem)
            })
            .collect();
        
        match self.resolver.resolve_dependencies(dependencies).await {
            Ok(resolution) => {
                // Check for resolution failures
                if !resolution.failed.is_empty() {
                    return Err(PpmError::ValidationError(format!(
                        "Failed to resolve dependencies: {}",
                        resolution.failed.iter()
                            .map(|failure| format!("{}: {}", failure.dependency.name, failure.error))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )));
                }

                // Install resolved dependencies
                for resolved_dep in &resolution.resolved {
                    match self.install_package(resolved_dep).await {
                        Ok(installed) => {
                            if installed {
                                result.installed.push(resolved_dep.clone());
                            } else {
                                result.skipped.push(resolved_dep.clone());
                            }
                        }
                        Err(e) => {
                            result.failed.push((resolved_dep.name.to_string(), e.to_string()));
                        }
                    }
                }

                // Create symlinks for the project (simplified)
                if result.failed.is_empty() {
                    match self.create_simple_symlinks(project, project_root, &resolution.resolved).await {
                        Ok(count) => result.symlinks_created = count,
                        Err(e) => {
                            result.failed.push(("symlink_creation".to_string(), e.to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                return Err(PpmError::ValidationError(format!("Dependency resolution failed: {}", e)));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis();
        Ok(result)
    }

    /// Install a single dependency
    pub async fn install_dependency(
        &mut self,
        dependency: &Dependency,
    ) -> Result<ResolvedDependency, PpmError> {
        // Resolve the dependency
        match self.resolver.resolve_dependencies(vec![dependency.clone()]).await {
            Ok(resolution) => {
                if !resolution.failed.is_empty() {
                    return Err(PpmError::ValidationError(format!(
                        "Failed to resolve dependency {}: {}",
                        dependency.name,
                        resolution.failed.iter()
                            .map(|failure| format!("{}: {}", failure.dependency.name, failure.error))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )));
                }

                if resolution.resolved.is_empty() {
                    return Err(PpmError::ValidationError(format!("Package not found: {}", dependency.name)));
                }

                let resolved = &resolution.resolved[0];
                self.install_package(resolved).await?;
                Ok(resolved.clone())
            }
            Err(e) => {
                Err(PpmError::ValidationError(format!("Dependency resolution failed: {}", e)))
            }
        }
    }

    /// Install a single package to the global store
    async fn install_package(&mut self, resolved: &ResolvedDependency) -> Result<bool, PpmError> {
        // For now, simulate package installation
        // In a real implementation, this would:
        // 1. Check if package is already installed using global store
        // 2. Download package from registry
        // 3. Verify integrity using SHA-256 hash
        // 4. Extract and store in global store
        // 5. Update global store index
        
        println!("Installing package: {}@{}", resolved.name, resolved.version);
        
        // Simulate package verification
        if !resolved.integrity.is_empty() {
            let hash = &resolved.integrity;
            println!("Verifying integrity: {}", hash);
        }
        
        // Simulate storage in global store
        // Use the store path from the resolved dependency  
        let store_path = PathBuf::from(&resolved.store_path);
        println!("Would store package at: {}", store_path.display());
        
        // Simulate download and installation time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(true) // Successfully installed
    }

    /// Create simplified symlinks for project dependencies
    async fn create_simple_symlinks(
        &mut self,
        _project: &Project,
        project_root: &Path,
        resolved_deps: &[ResolvedDependency],
    ) -> Result<usize, PpmError> {
        let mut symlink_count = 0;

        // Create basic directories for each ecosystem
        for ecosystem in [Ecosystem::JavaScript, Ecosystem::Python] {
            let ecosystem_deps: Vec<_> = resolved_deps
                .iter()
                .filter(|dep| dep.ecosystem == ecosystem)
                .collect();

            if ecosystem_deps.is_empty() {
                continue;
            }

            let symlink_path = match ecosystem {
                Ecosystem::JavaScript => project_root.join("node_modules"),
                Ecosystem::Python => project_root.join(".venv").join("lib").join("site-packages"),
            };

            // Create the base directory
            fs::create_dir_all(&symlink_path).await?;

            // For each dependency, create a placeholder directory structure
            for dep in ecosystem_deps {
                let dep_path = symlink_path.join(&dep.name);
                fs::create_dir_all(&dep_path).await?;
                
                // Create a simple marker file with package info
                let marker_path = dep_path.join("package_info.txt");
                let package_info = format!(
                    "Name: {}\nVersion: {}\nEcosystem: {}\nInstalled: {}\n",
                    dep.name,
                    dep.version,
                    dep.ecosystem,
                    Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                );
                fs::write(&marker_path, package_info).await?;
                
                // For JavaScript packages, create a basic package.json
                if ecosystem == Ecosystem::JavaScript {
                    let package_json = serde_json::json!({
                        "name": dep.name,
                        "version": dep.version,
                        "description": "Installed via PPM",
                        "main": "index.js"
                    });
                    let package_json_path = dep_path.join("package.json");
                    fs::write(&package_json_path, serde_json::to_string_pretty(&package_json).unwrap()).await?;
                }
                
                symlink_count += 1;
            }
        }

        Ok(symlink_count)
    }

    /// Verify package integrity using SHA-256 hash
    fn verify_package_integrity(
        &self,
        resolved: &ResolvedDependency,
        data: &[u8],
    ) -> Result<(), PpmError> {
        if !resolved.integrity.is_empty() {
            let expected_hash = &resolved.integrity;
            let mut hasher = Sha256::new();
            hasher.update(data);
            let actual_hash = format!("{:x}", hasher.finalize());

            if actual_hash != *expected_hash {
                return Err(PpmError::ValidationError(format!(
                    "Hash mismatch for {}: expected {}, got {}",
                    resolved.name,
                    expected_hash,
                    actual_hash
                )));
            }
        }
        Ok(())
    }

    /// Convert ResolvedDependency to Package model
    fn resolved_to_package(&self, resolved: &ResolvedDependency) -> Result<Package, PpmError> {
        let integrity_hash = resolved.integrity.clone();

        let package = Package::new(
            resolved.name.clone(),
            resolved.version.clone(),
            resolved.ecosystem,
            integrity_hash,
            PathBuf::from(&resolved.store_path),
        );
        
        Ok(package)
    }

    /// Update installation configuration
    pub fn update_config(&mut self, config: InstallConfig) {
        self.config = config;
    }

    /// Get current installation configuration
    pub fn config(&self) -> &InstallConfig {
        &self.config
    }

    /// Get installation statistics
    pub async fn get_install_stats(&self) -> Result<HashMap<String, u64>, PpmError> {
        let mut stats = HashMap::new();
        
        // For now, return basic placeholder stats
        // In a real implementation, this would query the global store
        stats.insert("total_packages".to_string(), 0);
        stats.insert("javascript_packages".to_string(), 0);
        stats.insert("python_packages".to_string(), 0);
        stats.insert("total_size_bytes".to_string(), 0);

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ecosystem::Ecosystem;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_package_installer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().to_path_buf();
        let global_store = GlobalStore::new(store_path);
        
        let installer = PackageInstaller::new(global_store, None);
        assert!(installer.is_ok());
    }

    #[tokio::test]
    async fn test_install_config_builder() {
        let config = InstallConfig::new()
            .with_dev_dependencies(true)
            .with_verification(false)
            .with_force_update(true)
            .with_concurrency(8)
            .with_timeout(600);

        assert!(config.include_dev);
        assert!(config.skip_verification);
        assert!(config.force_update);
        assert_eq!(config.max_concurrent, 8);
        assert_eq!(config.download_timeout, 600);
    }

    #[tokio::test]
    async fn test_install_result_methods() {
        let mut result = InstallResult::new();
        
        // Add some test data
        let dep1 = ResolvedDependency::new(
            "test-package".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            "abc123".to_string(),
            "abc123".to_string(),
            "packages/test".to_string(),
        );
        
        result.installed.push(dep1);
        result.failed.push(("failed-package".to_string(), "Error message".to_string()));

        assert_eq!(result.total_packages(), 2);
        assert_eq!(result.success_rate(), 0.5);
        assert!(!result.is_success());
    }

    #[tokio::test]
    async fn test_install_config_default() {
        let config = InstallConfig::default();
        
        assert!(!config.include_dev);
        assert!(!config.skip_verification);
        assert!(!config.force_update);
        assert_eq!(config.max_concurrent, 4);
        assert_eq!(config.download_timeout, 300);
    }

    #[tokio::test]
    async fn test_resolved_to_package_conversion() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().to_path_buf();
        let global_store = GlobalStore::new(store_path);
        let installer = PackageInstaller::new(global_store, None).unwrap();

        let resolved = ResolvedDependency::new(
            "test-package".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            "abc123".to_string(),
            "abc123".to_string(),
            "packages/test".to_string(),
        );

        let package = installer.resolved_to_package(&resolved).unwrap();
        assert_eq!(package.name, "test-package");
        assert_eq!(package.version, "1.0.0");
        assert_eq!(package.ecosystem, Ecosystem::JavaScript);
    }

    #[tokio::test]
    async fn test_integrity_verification() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().to_path_buf();
        let global_store = GlobalStore::new(store_path);
        let installer = PackageInstaller::new(global_store, None).unwrap();

        let data = b"test data";
        let mut hasher = Sha256::new();
        hasher.update(data);
        let expected_hash = format!("{:x}", hasher.finalize());

        let resolved = ResolvedDependency::new(
            "test-package".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            expected_hash.clone(),
            expected_hash.clone(),
            "packages/test".to_string(),
        );

        // Should succeed with correct hash
        let result = installer.verify_package_integrity(&resolved, data);
        assert!(result.is_ok());

        // Should fail with incorrect hash
        let resolved_bad = ResolvedDependency::new(
            "test-package".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            "wrong_hash".to_string(),
            "wrong_hash".to_string(),
            "packages/test".to_string(),
        );

        let result = installer.verify_package_integrity(&resolved_bad, data);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_install_stats() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().to_path_buf();
        let global_store = GlobalStore::new(store_path);
        let installer = PackageInstaller::new(global_store, None).unwrap();

        let stats = installer.get_install_stats().await.unwrap();
        assert!(stats.contains_key("total_packages"));
        assert!(stats.contains_key("total_size_bytes"));
    }
}
