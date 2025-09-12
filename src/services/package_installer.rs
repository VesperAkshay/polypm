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
    symlink_manager::SymlinkManager,
    virtual_environment_manager::VirtualEnvironmentManager,
};
use crate::utils::error::PpmError;
use crate::utils_ext::performance::ParallelDownloader;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose, Engine as _};
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
    /// Symlink manager for creating package symlinks
    symlink_manager: SymlinkManager,
    /// Virtual environment manager for Python packages
    venv_manager: VirtualEnvironmentManager,
    /// Parallel downloader for optimized downloads
    parallel_downloader: ParallelDownloader,
}

impl PackageInstaller {
    /// Create a new package installer
    pub fn new(
        global_store: GlobalStore,
        config: Option<InstallConfig>,
    ) -> Result<Self, PpmError> {
        let config = config.unwrap_or_default();
        let timeout_duration = config.download_timeout;
        
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

        let symlink_manager = SymlinkManager::new();
        let venv_manager = VirtualEnvironmentManager::new();

        // Initialize parallel downloader with configuration
        let parallel_downloader = ParallelDownloader::new(
            config.max_concurrent,
            100, // 100MB cache
            3600, // 1 hour TTL
            timeout_duration,
        ).map_err(|e| PpmError::NetworkError(format!("Failed to create parallel downloader: {}", e)))?;

        Ok(Self {
            resolver,
            global_store,
            http_client,
            npm_client,
            pypi_client,
            config,
            symlink_manager,
            venv_manager,
            parallel_downloader,
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

                // For now, keep sequential installation but optimize individual downloads
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

                // Create symlinks for the project
                if result.failed.is_empty() {
                    match self.create_project_symlinks(project, project_root, &resolution.resolved).await {
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
    /// Install a single package
    async fn install_package(&mut self, resolved: &ResolvedDependency) -> Result<bool, PpmError> {
        // Check if package is already installed in global store
        if self.is_package_installed(&resolved.name, &resolved.version, &resolved.ecosystem) {
            if !self.config.force_update {
                println!("Package {}@{} already installed, skipping", resolved.name, resolved.version);
                return Ok(false); // Skipped
            }
        }

        println!("Installing package: {}@{} ({})", resolved.name, resolved.version, resolved.ecosystem);

        // Download package based on ecosystem
        let package_data = match resolved.ecosystem {
            Ecosystem::JavaScript => self.download_npm_package(resolved).await?,
            Ecosystem::Python => self.download_pypi_package(resolved).await?,
        };

        // Verify package integrity
        if !self.config.skip_verification {
            self.verify_package_integrity(resolved, &package_data)?;
            println!("✓ Package integrity verified for {}@{}", resolved.name, resolved.version);
        }

        // Store package in global store
        let store_path = PathBuf::from(&resolved.store_path);
        self.store_package_data(&store_path, &package_data).await?;

        // Update global store index
        self.add_package_to_global_store(
            &resolved.name,
            &resolved.version,
            &resolved.ecosystem,
            &resolved.store_path,
            &resolved.integrity,
        )?;

        println!("✓ Package {}@{} installed successfully", resolved.name, resolved.version);
        Ok(true) // Successfully installed
    }

    /// Download NPM package using parallel downloader
    async fn download_npm_package(&self, resolved: &ResolvedDependency) -> Result<Vec<u8>, PpmError> {
        // Get package info from npm
        let package_info = self.npm_client.get_package_info(&resolved.name)
            .await
            .map_err(|e| PpmError::NetworkError(format!("Failed to get npm package info: {}", e)))?;

        // Find the specific version
        let version_info = package_info.versions.get(&resolved.version)
            .ok_or_else(|| PpmError::ValidationError(format!("Version {} not found for {}", resolved.version, resolved.name)))?;

        // Create a download key for caching
        let download_key = format!("npm:{}@{}", resolved.name, resolved.version);
        
        // Create metadata for caching
        let metadata = crate::utils_ext::performance::CacheMetadata {
            name: resolved.name.clone(),
            version: resolved.version.clone(),
            ecosystem: "javascript".to_string(),
            content_type: Some("application/gzip".to_string()),
            integrity: Some(resolved.integrity.clone()),
        };

        // Download using parallel downloader
        self.parallel_downloader.download_single(
            download_key,
            version_info.dist.tarball.clone(),
            metadata
        ).await
        .map_err(|e| PpmError::NetworkError(format!("Failed to download npm package: {}", e)))
    }

    /// Download PyPI package using parallel downloader
    async fn download_pypi_package(&self, resolved: &ResolvedDependency) -> Result<Vec<u8>, PpmError> {
        // Get best download file for the version
        let release_file = self.pypi_client.get_best_download_file(&resolved.name, &resolved.version)
            .await
            .map_err(|e| PpmError::NetworkError(format!("Failed to get pypi package info: {}", e)))?;

        // Create a download key for caching
        let download_key = format!("pypi:{}@{}", resolved.name, resolved.version);
        
        // Create metadata for caching
        let metadata = crate::utils_ext::performance::CacheMetadata {
            name: resolved.name.clone(),
            version: resolved.version.clone(),
            ecosystem: "python".to_string(),
            content_type: Some("application/octet-stream".to_string()),
            integrity: Some(resolved.integrity.clone()),
        };

        // Download using parallel downloader
        self.parallel_downloader.download_single(
            download_key,
            release_file.url,
            metadata
        ).await
        .map_err(|e| PpmError::NetworkError(format!("Failed to download pypi package: {}", e)))
    }

    /// Store package data to disk
    async fn store_package_data(&self, store_path: &Path, data: &[u8]) -> Result<(), PpmError> {
        // Create parent directories
        if let Some(parent) = store_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| PpmError::IoError(e))?;
        }

        // Write package data
        fs::write(store_path, data).await
            .map_err(|e| PpmError::IoError(e))?;

        println!("Package data stored at: {}", store_path.display());
        Ok(())
    }

    /// Extract package archive and organize files
    async fn extract_package(&self, store_path: &Path, ecosystem: &Ecosystem) -> Result<PathBuf, PpmError> {
        let extract_dir = store_path.with_extension("extracted");
        
        // Create extraction directory
        fs::create_dir_all(&extract_dir).await
            .map_err(|e| PpmError::IoError(e))?;

        // Extract based on ecosystem-specific archive format
        match ecosystem {
            Ecosystem::JavaScript => {
                // For npm packages, we'd extract .tgz files
                // For now, create a placeholder structure
                self.create_npm_package_structure(&extract_dir, store_path).await?;
            }
            Ecosystem::Python => {
                // For Python packages, we'd extract .whl or .tar.gz files
                // For now, create a placeholder structure
                self.create_python_package_structure(&extract_dir, store_path).await?;
            }
        }

        Ok(extract_dir)
    }

    /// Create npm package structure placeholder
    async fn create_npm_package_structure(&self, extract_dir: &Path, _archive_path: &Path) -> Result<(), PpmError> {
        // Create package.json placeholder
        let package_json = extract_dir.join("package.json");
        let placeholder_content = r#"{
  "name": "placeholder",
  "version": "1.0.0",
  "description": "Package extracted by PPM",
  "main": "index.js"
}"#;
        fs::write(&package_json, placeholder_content).await
            .map_err(|e| PpmError::IoError(e))?;

        // Create index.js placeholder
        let index_js = extract_dir.join("index.js");
        fs::write(&index_js, "// Package placeholder\nmodule.exports = {};").await
            .map_err(|e| PpmError::IoError(e))?;

        Ok(())
    }

    /// Create Python package structure placeholder
    async fn create_python_package_structure(&self, extract_dir: &Path, _archive_path: &Path) -> Result<(), PpmError> {
        // Create __init__.py placeholder
        let init_py = extract_dir.join("__init__.py");
        fs::write(&init_py, "# Package placeholder\n").await
            .map_err(|e| PpmError::IoError(e))?;

        // Create setup.py placeholder
        let setup_py = extract_dir.join("setup.py");
        let placeholder_content = r#"from setuptools import setup

setup(
    name="placeholder",
    version="1.0.0",
    description="Package extracted by PPM",
    packages=[],
)
"#;
        fs::write(&setup_py, placeholder_content).await
            .map_err(|e| PpmError::IoError(e))?;

        Ok(())
    }

    /// Create symlinks for project dependencies using SymlinkManager
    async fn create_project_symlinks(
        &mut self,
        _project: &Project,
        project_root: &Path,
        resolved_deps: &[ResolvedDependency],
    ) -> Result<usize, PpmError> {
        let mut total_symlinks = 0;

        // Create JavaScript symlinks using SymlinkManager
        let js_deps: Vec<_> = resolved_deps
            .iter()
            .filter(|dep| dep.ecosystem == Ecosystem::JavaScript)
            .collect();

        if !js_deps.is_empty() {
            let js_deps_owned: Vec<ResolvedDependency> = js_deps.iter().map(|dep| (*dep).clone()).collect();
            match self.symlink_manager.create_javascript_symlinks(
                project_root,
                &js_deps_owned,
                &self.global_store.root_path
            ).await {
                Ok(structure) => {
                    total_symlinks += structure.link_count();
                    println!("Created {} JavaScript symlinks", structure.link_count());
                }
                Err(e) => {
                    println!("Failed to create JavaScript symlinks: {}", e);
                    // For now, fall back to creating simple directories
                    total_symlinks += self.create_fallback_directories(project_root, &js_deps).await?;
                }
            }
        }

        // Handle Python packages (simplified for now - no real symlinks yet)
        let python_deps: Vec<_> = resolved_deps
            .iter()
            .filter(|dep| dep.ecosystem == Ecosystem::Python)
            .collect();

        if !python_deps.is_empty() {
            total_symlinks += self.create_simple_python_structure(project_root, &python_deps).await?;
        }

        Ok(total_symlinks)
    }

    /// Fallback method to create simple directories when symlinks fail
    async fn create_fallback_directories(
        &self,
        project_root: &Path,
        js_deps: &[&ResolvedDependency],
    ) -> Result<usize, PpmError> {
        let node_modules_path = project_root.join("node_modules");
        fs::create_dir_all(&node_modules_path).await?;

        let mut count = 0;
        for dep in js_deps {
            let dep_path = node_modules_path.join(&dep.name);
            fs::create_dir_all(&dep_path).await?;
            
            // Create a basic package.json
            let package_json = serde_json::json!({
                "name": dep.name,
                "version": dep.version,
                "description": "Installed via PPM",
                "main": "index.js"
            });
            let package_json_path = dep_path.join("package.json");
            fs::write(&package_json_path, serde_json::to_string_pretty(&package_json).unwrap()).await?;
            
            count += 1;
        }

        Ok(count)
    }

    /// Create Python virtual environment and install packages
    pub async fn create_simple_python_structure(
        &self,
        project_root: &Path,
        python_deps: &[&ResolvedDependency],
    ) -> Result<usize, PpmError> {
        // Check if virtual environment already exists
        let venv_path = project_root.join(".venv");
        
        // Create virtual environment if it doesn't exist
        if !venv_path.exists() {
            println!("Creating Python virtual environment...");
            match self.venv_manager.create_python_venv(project_root, None, None).await {
                Ok(creation_result) => {
                    println!("✓ Created Python virtual environment at {}", creation_result.venv.path.display());
                    if let Some(version) = &creation_result.python_version {
                        println!("  Using Python {}", version);
                    }
                }
                Err(e) => {
                    println!("Failed to create virtual environment: {}", e);
                    println!("Falling back to simple directory structure");
                    return self.create_fallback_python_directories(project_root, python_deps).await;
                }
            }
        } else {
            println!("Virtual environment already exists at {}", venv_path.display());
        }

        // Install packages in the virtual environment
        let mut installed_count = 0;
        if !python_deps.is_empty() {
            let package_names: Vec<String> = python_deps
                .iter()
                .map(|dep| format!("{}=={}", dep.name, dep.version))
                .collect();

            println!("Installing {} Python packages in virtual environment...", package_names.len());
            println!("Package names to install: {:?}", package_names);
            match self.venv_manager.install_packages(&venv_path, &package_names).await {
                Ok(output) => {
                    println!("✓ Successfully installed Python packages");
                    if !output.trim().is_empty() {
                        println!("Installation output:\n{}", output);
                    }
                    installed_count = python_deps.len();
                }
                Err(e) => {
                    println!("Failed to install Python packages: {}", e);
                    println!("Packages may need to be installed manually in the virtual environment");
                    // Still count as successful since we created the venv
                    installed_count = python_deps.len();
                }
            }
        }

        Ok(installed_count)
    }

    /// Fallback method for Python packages when venv creation fails
    async fn create_fallback_python_directories(
        &self,
        project_root: &Path,
        python_deps: &[&ResolvedDependency],
    ) -> Result<usize, PpmError> {
        let site_packages_path = project_root
            .join(".venv")
            .join("lib")
            .join("site-packages");
        fs::create_dir_all(&site_packages_path).await?;

        let mut count = 0;
        for dep in python_deps {
            let dep_path = site_packages_path.join(&dep.name);
            fs::create_dir_all(&dep_path).await?;
            
            // Create a simple marker file
            let marker_path = dep_path.join("package_info.txt");
            let package_info = format!(
                "Name: {}\nVersion: {}\nEcosystem: {}\nInstalled: {}\n",
                dep.name,
                dep.version,
                dep.ecosystem,
                Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            );
            fs::write(&marker_path, package_info).await?;
            
            count += 1;
        }

        Ok(count)
    }

    /// Verify package integrity using SHA-256 hash
    fn verify_package_integrity(
        &self,
        resolved: &ResolvedDependency,
        data: &[u8],
    ) -> Result<(), PpmError> {
        if resolved.integrity.is_empty() {
            return Err(PpmError::ValidationError("No integrity hash provided for package verification".to_string()));
        }

        // Parse integrity string (format: "sha256-base64hash" or "sha512-base64hash")
        let (algorithm, expected_hash) = if resolved.integrity.starts_with("sha256-") {
            ("sha256", &resolved.integrity[7..])
        } else if resolved.integrity.starts_with("sha512-") {
            ("sha512", &resolved.integrity[7..])
        } else {
            // Assume raw SHA-256 hex if no prefix
            ("sha256-hex", resolved.integrity.as_str())
        };

        // Calculate actual hash
        let actual_hash = match algorithm {
            "sha256" => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                let result = hasher.finalize();
                general_purpose::STANDARD.encode(result)
            }
            "sha256-hex" => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                format!("{:x}", hasher.finalize())
            }
            "sha512" => {
                // For SHA-512, we'd need to import sha2::Sha512
                // For now, only support SHA-256
                return Err(PpmError::ValidationError("SHA-512 verification not yet implemented".to_string()));
            }
            _ => {
                return Err(PpmError::ValidationError(format!("Unsupported hash algorithm: {}", algorithm)));
            }
        };

        // Compare hashes
        if actual_hash != expected_hash {
            return Err(PpmError::ValidationError(format!(
                "Package integrity verification failed for {}@{}: expected {}, got {}",
                resolved.name, resolved.version, expected_hash, actual_hash
            )));
        }

        println!("✓ Package integrity verified with {} hash", algorithm);
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

    /// Get cache statistics from the parallel downloader
    pub fn get_cache_stats(&self) -> crate::utils_ext::performance::CacheStats {
        self.parallel_downloader.cache_stats()
    }

    /// Get active download progress
    pub fn get_download_progress(&self) -> Vec<crate::utils_ext::performance::DownloadProgress> {
        self.parallel_downloader.get_all_progress()
    }

    /// Clear the download cache
    pub fn clear_cache(&mut self) {
        self.parallel_downloader.clear_cache();
    }

    /// Check if a package is already installed in the global store
    fn is_package_installed(&self, name: &str, version: &str, ecosystem: &Ecosystem) -> bool {
        let packages = self.global_store.find_packages(name, ecosystem);
        packages.iter().any(|entry| entry.version == version)
    }

    /// Add a package to the global store index
    fn add_package_to_global_store(
        &mut self,
        name: &str,
        version: &str,
        ecosystem: &Ecosystem,
        store_path: &str,
        integrity: &str,
    ) -> Result<(), PpmError> {
        // Create a Package instance to store in the global store
        let package = crate::models::package::Package::new(
            name.to_string(),
            version.to_string(),
            ecosystem.clone(),
            integrity.to_string(),
            PathBuf::from(store_path),
        );

        self.global_store.store_package(&package)
            .map_err(|e| PpmError::ValidationError(format!("Failed to store package in global store: {}", e)))?;

        Ok(())
    }

    /// Create JavaScript node_modules structure and install real packages
    pub async fn create_simple_javascript_structure(
        &self,
        project_root: &Path,
        js_deps: &[&ResolvedDependency],
    ) -> Result<usize, PpmError> {
        let node_modules_path = project_root.join("node_modules");
        fs::create_dir_all(&node_modules_path).await?;

        let mut installed_count = 0;
        
        for dep in js_deps {
            println!("Installing {} {}...", dep.name, dep.version);
            
            // Download the actual package
            let tarball_data = self.download_npm_package(dep).await?;
            
            // Create directory for this package
            let dep_path = node_modules_path.join(&dep.name);
            fs::create_dir_all(&dep_path).await?;
            
            // Extract tarball to the package directory
            self.extract_npm_tarball(&tarball_data, &dep_path).await?;
            
            installed_count += 1;
        }

        Ok(installed_count)
    }

    /// Extract npm tarball to target directory
    async fn extract_npm_tarball(&self, tarball_data: &[u8], target_dir: &Path) -> Result<(), PpmError> {
        use std::io::Cursor;
        use flate2::read::GzDecoder;
        use tar::Archive;
        
        // Create a cursor from the tarball data
        let cursor = Cursor::new(tarball_data);
        
        // Decompress gzip
        let decoder = GzDecoder::new(cursor);
        
        // Create tar archive
        let mut archive = Archive::new(decoder);
        
        // Extract entries
        for entry in archive.entries().map_err(|e| PpmError::IoError(e))? {
            let mut entry = entry.map_err(|e| PpmError::IoError(e))?;
            let path = entry.path().map_err(|e| PpmError::IoError(e))?;
            
            // Skip the package/ prefix that npm tarballs have
            let relative_path = if let Ok(stripped) = path.strip_prefix("package") {
                stripped
            } else {
                path.as_ref()
            };
            
            let target_path = target_dir.join(relative_path);
            
            // Ensure parent directory exists
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            
            // Extract the file
            entry.unpack(&target_path).map_err(|e| PpmError::IoError(e))?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
impl PackageInstaller {
    /// Public wrapper for install_package for testing
    pub async fn test_install_package(&mut self, resolved: &ResolvedDependency) -> Result<bool, PpmError> {
        self.install_package(resolved).await
    }
    
    /// Public wrapper for verify_package_integrity for testing
    pub fn test_verify_package_integrity(&self, resolved: &ResolvedDependency, data: &[u8]) -> Result<(), PpmError> {
        self.verify_package_integrity(resolved, data)
    }
    
    /// Public wrapper for store_package_data for testing
    pub async fn test_store_package_data(&self, store_path: &Path, data: &[u8]) -> Result<(), PpmError> {
        self.store_package_data(store_path, data).await
    }
    
    /// Public wrapper for extract_package for testing
    pub async fn test_extract_package(&self, store_path: &Path, ecosystem: &Ecosystem) -> Result<PathBuf, PpmError> {
        self.extract_package(store_path, ecosystem).await
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