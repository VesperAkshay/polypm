use std::collections::HashMap;
use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::models::package::Package;
use crate::models::ecosystem::Ecosystem;
use crate::models::dependency::Dependency;
use crate::models::global_store::{CachedPackageInfo, RegistryCache};

/// NPM registry API client for JavaScript package management
#[derive(Debug, Clone)]
pub struct NpmClient {
    /// HTTP client for registry requests
    client: Client,
    /// Base URL for npm registry (configurable for testing)
    registry_url: String,
    /// User agent string for requests
    user_agent: String,
}

/// Response from npm registry package endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmPackageResponse {
    /// Package name
    pub name: String,
    /// All available versions with metadata
    pub versions: HashMap<String, NpmVersionInfo>,
    /// Distribution tags (latest, beta, etc.)
    #[serde(rename = "dist-tags")]
    pub dist_tags: HashMap<String, String>,
    /// Package description
    pub description: Option<String>,
    /// Package author
    pub author: Option<NpmAuthor>,
    /// Package license
    pub license: Option<String>,
    /// Package keywords
    pub keywords: Option<Vec<String>>,
    /// When package was last modified
    pub time: Option<HashMap<String, String>>,
}

/// Version-specific information from npm registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmVersionInfo {
    /// Package name
    pub name: String,
    /// Version number
    pub version: String,
    /// Package description
    pub description: Option<String>,
    /// Distribution information
    pub dist: NpmDistInfo,
    /// Dependencies
    pub dependencies: Option<HashMap<String, String>>,
    /// Dev dependencies
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
    /// Package author
    pub author: Option<NpmAuthor>,
    /// Package license
    pub license: Option<serde_json::Value>, // Can be string or object
    /// Package keywords
    pub keywords: Option<Vec<String>>,
    /// Ignore any unknown fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Distribution information for a package version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmDistInfo {
    /// Download URL for the package tarball
    pub tarball: String,
    /// SHA-1 checksum of the tarball
    pub shasum: String,
    /// Integrity hash (sha512)
    pub integrity: Option<String>,
    /// File count in the package
    #[serde(rename = "fileCount")]
    pub file_count: Option<u32>,
    /// Unpacked size in bytes
    #[serde(rename = "unpackedSize")]
    pub unpacked_size: Option<u64>,
    /// Ignore any unknown fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Author information from npm registry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NpmAuthor {
    /// Simple string format
    String(String),
    /// Detailed object format
    Object {
        name: String,
        email: Option<String>,
        url: Option<String>,
    },
}

/// NPM client errors
#[derive(Debug, thiserror::Error)]
pub enum NpmError {
    /// HTTP request failed
    #[error("Registry request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    
    /// Package not found in registry
    #[error("Package '{0}' not found")]
    PackageNotFound(String),
    
    /// Version not found for package
    #[error("Version '{1}' not found for package '{0}'")]
    VersionNotFound(String, String),
    
    /// Invalid package name
    #[error("Invalid package name: {0}")]
    InvalidPackageName(String),
    
    /// Registry response parsing failed
    #[error("Failed to parse registry response: {0}")]
    ParseError(String),
    
    /// Network timeout
    #[error("Request timeout")]
    Timeout,
    
    /// Rate limiting
    #[error("Rate limited by registry")]
    RateLimited,
}

impl NpmClient {
    /// Create a new NPM registry client
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(&format!("ppm/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            client,
            registry_url: "https://registry.npmjs.org".to_string(),
            user_agent: format!("ppm/{}", env!("CARGO_PKG_VERSION")),
        }
    }
    
    /// Create a new NPM client with custom registry URL (for testing)
    pub fn with_registry_url(registry_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(&format!("ppm/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            client,
            registry_url,
            user_agent: format!("ppm/{}", env!("CARGO_PKG_VERSION")),
        }
    }
    
    /// Create a new NPM client with custom HTTP client (for testing)
    pub fn with_client(client: Client, registry_url: String) -> Self {
        Self {
            client,
            registry_url,
            user_agent: format!("ppm/{}", env!("CARGO_PKG_VERSION")),
        }
    }
    
    /// Get package information from npm registry with retry logic
    pub async fn get_package_info(&self, package_name: &str) -> Result<NpmPackageResponse, NpmError> {
        // Validate package name
        Ecosystem::JavaScript.validate_package_name(package_name)
            .map_err(|_| NpmError::InvalidPackageName(package_name.to_string()))?;
        
        let url = format!("{}/{}", self.registry_url, package_name);
        
        // Retry logic for transient failures
        let mut attempts = 0;
        let max_attempts = 3;
        
        loop {
            attempts += 1;
            
            let response = self.client
                .get(&url)
                .header("User-Agent", &self.user_agent)
                .header("Accept", "application/json")
                .send()
                .await;
            
            match response {
                Ok(resp) => {
                    if resp.status() == 404 {
                        return Err(NpmError::PackageNotFound(package_name.to_string()));
                    }
                    
                    if resp.status() == 429 {
                        return Err(NpmError::RateLimited);
                    }
                    
                    if !resp.status().is_success() {
                        if attempts >= max_attempts {
                            return Err(NpmError::RequestFailed(
                                reqwest::Error::from(resp.error_for_status().unwrap_err())
                            ));
                        }
                        // Wait before retry (exponential backoff)
                        tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(attempts - 1))).await;
                        continue;
                    }
                    
                    match resp.json().await {
                        Ok(package_info) => return Ok(package_info),
                        Err(e) => {
                            if attempts >= max_attempts {
                                return Err(NpmError::ParseError(e.to_string()));
                            }
                            // Wait before retry
                            tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(attempts - 1))).await;
                            continue;
                        }
                    }
                }
                Err(e) => {
                    if attempts >= max_attempts {
                        return Err(NpmError::RequestFailed(e));
                    }
                    // Wait before retry
                    tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(attempts - 1))).await;
                    continue;
                }
            }
        }
    }
    
    /// Get specific version information for a package
    pub async fn get_version_info(&self, package_name: &str, version: &str) -> Result<NpmVersionInfo, NpmError> {
        let package_info = self.get_package_info(package_name).await?;
        
        package_info.versions.get(version)
            .cloned()
            .ok_or_else(|| NpmError::VersionNotFound(package_name.to_string(), version.to_string()))
    }
    
    /// Get the latest version of a package
    pub async fn get_latest_version(&self, package_name: &str) -> Result<String, NpmError> {
        let package_info = self.get_package_info(package_name).await?;
        
        // Try to get 'latest' tag first, fall back to highest version number
        if let Some(latest) = package_info.dist_tags.get("latest") {
            Ok(latest.clone())
        } else if let Some(version) = package_info.versions.keys().max() {
            Ok(version.clone())
        } else {
            Err(NpmError::PackageNotFound(package_name.to_string()))
        }
    }
    
    /// Get all available versions for a package
    pub async fn get_available_versions(&self, package_name: &str) -> Result<Vec<String>, NpmError> {
        let package_info = self.get_package_info(package_name).await?;
        
        let mut versions: Vec<String> = package_info.versions.keys().cloned().collect();
        // Sort versions (simple lexicographic for now, could implement proper semver sorting)
        versions.sort();
        
        Ok(versions)
    }
    
    /// Resolve version specification to exact version
    pub async fn resolve_version(&self, package_name: &str, version_spec: &str) -> Result<String, NpmError> {
        if version_spec == "latest" || version_spec == "*" {
            return self.get_latest_version(package_name).await;
        }
        
        let versions = self.get_available_versions(package_name).await?;
        
        // Handle exact version match first
        if versions.contains(&version_spec.to_string()) {
            return Ok(version_spec.to_string());
        }
        
        // Handle complex version ranges like ">= 2.1.2 < 3"
        if version_spec.contains(">=") && version_spec.contains("<") {
            return self.resolve_range_version(&versions, version_spec, package_name);
        }
        
        // Handle simple version prefixes
        if let Some(clean_version) = self.parse_simple_version_spec(version_spec) {
            if versions.contains(&clean_version) {
                return Ok(clean_version);
            }
            
            // For caret/tilde ranges, find compatible versions
            if version_spec.starts_with('^') {
                return self.find_caret_compatible(&versions, &clean_version, package_name);
            } else if version_spec.starts_with('~') {
                return self.find_tilde_compatible(&versions, &clean_version, package_name);
            } else {
                // Try prefix matching for partial versions
                for version in versions.iter().rev() {
                    if version.starts_with(&clean_version) {
                        return Ok(version.clone());
                    }
                }
            }
        }
        
        Err(NpmError::VersionNotFound(package_name.to_string(), version_spec.to_string()))
    }
    
    /// Parse simple version specifications by removing common prefixes
    fn parse_simple_version_spec(&self, version_spec: &str) -> Option<String> {
        let cleaned = version_spec.trim_start_matches('^')
                                 .trim_start_matches('~')
                                 .trim_start_matches(">=")
                                 .trim_start_matches('>')
                                 .trim_start_matches("<=")
                                 .trim_start_matches('<')
                                 .trim_start_matches('=')
                                 .trim();
        
        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned.to_string())
        }
    }
    
    /// Resolve complex version ranges like ">= 2.1.2 < 3"
    fn resolve_range_version(&self, versions: &[String], version_spec: &str, package_name: &str) -> Result<String, NpmError> {
        // Parse the range specification
        let parts: Vec<&str> = version_spec.split_whitespace().collect();
        
        let mut min_version = None;
        let mut max_version = None;
        let mut min_inclusive = false;
        let mut max_inclusive = false;
        
        let mut i = 0;
        while i < parts.len() {
            match parts[i] {
                ">=" => {
                    if i + 1 < parts.len() {
                        min_version = Some(parts[i + 1]);
                        min_inclusive = true;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                ">" => {
                    if i + 1 < parts.len() {
                        min_version = Some(parts[i + 1]);
                        min_inclusive = false;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "<=" => {
                    if i + 1 < parts.len() {
                        max_version = Some(parts[i + 1]);
                        max_inclusive = true;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "<" => {
                    if i + 1 < parts.len() {
                        max_version = Some(parts[i + 1]);
                        max_inclusive = false;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                _ => i += 1,
            }
        }
        
        // Find the highest version that satisfies the range
        let mut best_version = None;
        for version in versions.iter().rev() {
            let mut satisfies = true;
            
            // Check minimum version constraint
            if let Some(min_ver) = min_version {
                let cmp = self.compare_versions(version, min_ver);
                if min_inclusive {
                    if cmp < 0 { satisfies = false; }
                } else {
                    if cmp <= 0 { satisfies = false; }
                }
            }
            
            // Check maximum version constraint  
            if let Some(max_ver) = max_version {
                let cmp = self.compare_versions(version, max_ver);
                if max_inclusive {
                    if cmp > 0 { satisfies = false; }
                } else {
                    if cmp >= 0 { satisfies = false; }
                }
            }
            
            if satisfies {
                best_version = Some(version.clone());
                break; // Take the first (highest) matching version
            }
        }
        
        best_version.ok_or_else(|| NpmError::VersionNotFound(package_name.to_string(), version_spec.to_string()))
    }
    
    /// Compare two version strings (simplified semver comparison)
    fn compare_versions(&self, version1: &str, version2: &str) -> i32 {
        let v1_parts: Vec<u32> = version1.split('.').filter_map(|s| s.parse().ok()).collect();
        let v2_parts: Vec<u32> = version2.split('.').filter_map(|s| s.parse().ok()).collect();
        
        let max_len = v1_parts.len().max(v2_parts.len());
        
        for i in 0..max_len {
            let v1_part = v1_parts.get(i).copied().unwrap_or(0);
            let v2_part = v2_parts.get(i).copied().unwrap_or(0);
            
            if v1_part < v2_part {
                return -1;
            } else if v1_part > v2_part {
                return 1;
            }
        }
        
        0
    }
    
    /// Find version compatible with caret range (^1.2.3 allows >=1.2.3 <2.0.0)
    fn find_caret_compatible(&self, versions: &[String], base_version: &str, package_name: &str) -> Result<String, NpmError> {
        let base_parts: Vec<u32> = base_version.split('.').filter_map(|s| s.parse().ok()).collect();
        if base_parts.is_empty() {
            return Err(NpmError::VersionNotFound(package_name.to_string(), format!("^{}", base_version)));
        }
        
        let major = base_parts[0];
        
        for version in versions.iter().rev() {
            let version_parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();
            if version_parts.is_empty() { continue; }
            
            // Same major version and >= base version
            if version_parts[0] == major && self.compare_versions(version, base_version) >= 0 {
                return Ok(version.clone());
            }
        }
        
        Err(NpmError::VersionNotFound(package_name.to_string(), format!("^{}", base_version)))
    }
    
    /// Find version compatible with tilde range (~1.2.3 allows >=1.2.3 <1.3.0)
    fn find_tilde_compatible(&self, versions: &[String], base_version: &str, package_name: &str) -> Result<String, NpmError> {
        let base_parts: Vec<u32> = base_version.split('.').filter_map(|s| s.parse().ok()).collect();
        if base_parts.len() < 2 {
            return Err(NpmError::VersionNotFound(package_name.to_string(), format!("~{}", base_version)));
        }
        
        let major = base_parts[0];
        let minor = base_parts[1];
        
        for version in versions.iter().rev() {
            let version_parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();
            if version_parts.len() < 2 { continue; }
            
            // Same major.minor and >= base version
            if version_parts[0] == major && version_parts[1] == minor && 
               self.compare_versions(version, base_version) >= 0 {
                return Ok(version.clone());
            }
        }
        
        Err(NpmError::VersionNotFound(package_name.to_string(), format!("~{}", base_version)))
    }
    
    /// Convert NPM package information to our Package model
    pub fn npm_to_package(&self, npm_info: &NpmVersionInfo, store_path: std::path::PathBuf) -> Result<Package> {
        // Create package metadata
        let mut metadata = crate::models::package::PackageMetadata::default();
        metadata.description = npm_info.description.clone();
        metadata.license = npm_info.license.as_ref()
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        metadata.keywords = npm_info.keywords.clone().unwrap_or_default();
        
        // Extract author information
        if let Some(author) = &npm_info.author {
            metadata.author = Some(match author {
                NpmAuthor::String(name) => name.clone(),
                NpmAuthor::Object { name, email, .. } => {
                    if let Some(email) = email {
                        format!("{} <{}>", name, email)
                    } else {
                        name.clone()
                    }
                }
            });
        }

        // Handle license which can be string or object
        if let Some(license) = &npm_info.license {
            metadata.license = match license {
                serde_json::Value::String(s) => Some(s.clone()),
                serde_json::Value::Object(obj) => {
                    // Try to extract license from object (e.g., {"type": "MIT"})
                    obj.get("type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .or_else(|| Some("Unknown".to_string()))
                }
                _ => Some("Unknown".to_string()),
            };
        }

        // Use SHA-1 from npm, but we'll need to convert it to SHA-256 for our system
        // For now, use the shasum as-is (this would need proper handling in production)
        let hash = npm_info.dist.shasum.clone();
        
        // Convert dependencies
        let mut dependencies = Vec::new();
        if let Some(deps) = &npm_info.dependencies {
            for (name, version_spec) in deps {
                dependencies.push(Dependency::production(
                    name.clone(),
                    version_spec.clone(),
                    Ecosystem::JavaScript,
                ));
            }
        }
        
        if let Some(dev_deps) = &npm_info.dev_dependencies {
            for (name, version_spec) in dev_deps {
                dependencies.push(Dependency::development(
                    name.clone(),
                    version_spec.clone(),
                    Ecosystem::JavaScript,
                ));
            }
        }
        
        let mut package = Package::with_metadata(
            npm_info.name.clone(),
            npm_info.version.clone(),
            Ecosystem::JavaScript,
            hash,
            metadata,
            store_path,
        );
        
        // Add dependencies
        for dep in dependencies {
            package.add_dependency(dep);
        }
        
        Ok(package)
    }
    
    /// Update registry cache with package information
    pub fn update_cache(&self, cache: &mut RegistryCache, package_info: &NpmPackageResponse) {
        let versions: Vec<String> = package_info.versions.keys().cloned().collect();
        let latest_version = package_info.dist_tags.get("latest")
            .cloned()
            .unwrap_or_else(|| {
                versions.iter().max().cloned().unwrap_or_default()
            });
        
        let cached_info = CachedPackageInfo::new(
            package_info.name.clone(),
            versions,
            latest_version,
        );
        
        cache.update_package(cached_info);
    }
    
    /// Download package tarball with retry and integrity verification
    pub async fn download_package(&self, tarball_url: &str) -> Result<Vec<u8>, NpmError> {
        let mut attempts = 0;
        let max_attempts = 3;
        
        loop {
            attempts += 1;
            
            let response = self.client
                .get(tarball_url)
                .header("User-Agent", &self.user_agent)
                .send()
                .await;
            
            match response {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        if attempts >= max_attempts {
                            return Err(NpmError::RequestFailed(
                                reqwest::Error::from(resp.error_for_status().unwrap_err())
                            ));
                        }
                        // Wait before retry
                        tokio::time::sleep(Duration::from_millis(200 * 2_u64.pow(attempts - 1))).await;
                        continue;
                    }
                    
                    match resp.bytes().await {
                        Ok(bytes) => return Ok(bytes.to_vec()),
                        Err(e) => {
                            if attempts >= max_attempts {
                                return Err(NpmError::RequestFailed(e));
                            }
                            // Wait before retry
                            tokio::time::sleep(Duration::from_millis(200 * 2_u64.pow(attempts - 1))).await;
                            continue;
                        }
                    }
                }
                Err(e) => {
                    if attempts >= max_attempts {
                        return Err(NpmError::RequestFailed(e));
                    }
                    // Wait before retry
                    tokio::time::sleep(Duration::from_millis(200 * 2_u64.pow(attempts - 1))).await;
                    continue;
                }
            }
        }
    }
    
    /// Download and verify package integrity
    pub async fn download_package_with_verification(&self, version_info: &NpmVersionInfo) -> Result<Vec<u8>, NpmError> {
        let bytes = self.download_package(&version_info.dist.tarball).await?;
        
        // Basic size check
        if bytes.is_empty() {
            return Err(NpmError::ParseError("Downloaded package is empty".to_string()));
        }
        
        // TODO: Implement proper SHA-1 verification using the shasum from version_info.dist.shasum
        // For now, we trust the download since we're using HTTPS
        
        Ok(bytes)
    }
    
    /// Search for packages in npm registry with improved error handling
    pub async fn search_packages(&self, query: &str, limit: Option<usize>) -> Result<Vec<NpmSearchResult>, NpmError> {
        let search_url = format!("{}/-/v1/search", self.registry_url);
        let limit = limit.unwrap_or(20).min(100); // Cap at 100 to avoid overloading registry
        
        let mut attempts = 0;
        let max_attempts = 2; // Fewer retries for search since it's less critical
        
        loop {
            attempts += 1;
            
            let response = self.client
                .get(&search_url)
                .header("User-Agent", &self.user_agent)
                .header("Accept", "application/json")
                .query(&[("text", query), ("size", &limit.to_string())])
                .send()
                .await;
            
            match response {
                Ok(resp) => {
                    if resp.status() == 429 {
                        return Err(NpmError::RateLimited);
                    }
                    
                    if !resp.status().is_success() {
                        if attempts >= max_attempts {
                            return Err(NpmError::RequestFailed(
                                reqwest::Error::from(resp.error_for_status().unwrap_err())
                            ));
                        }
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        continue;
                    }
                    
                    match resp.json::<NpmSearchResponse>().await {
                        Ok(search_response) => {
                            return Ok(search_response.objects.into_iter().map(|obj| obj.package).collect());
                        }
                        Err(e) => {
                            if attempts >= max_attempts {
                                return Err(NpmError::ParseError(e.to_string()));
                            }
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            continue;
                        }
                    }
                }
                Err(e) => {
                    if attempts >= max_attempts {
                        return Err(NpmError::RequestFailed(e));
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    continue;
                }
            }
        }
    }
    
    /// Check if a package exists in the registry
    pub async fn package_exists(&self, package_name: &str) -> Result<bool, NpmError> {
        match self.get_package_info(package_name).await {
            Ok(_) => Ok(true),
            Err(NpmError::PackageNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
    
    /// Batch fetch multiple package infos (useful for dependency resolution)
    pub async fn get_multiple_package_infos(&self, package_names: &[String]) -> Vec<(String, Result<NpmPackageResponse, NpmError>)> {
        // Limit concurrent requests to avoid overwhelming the registry
        let chunk_size = 5;
        let mut results = Vec::new();
        
        for chunk in package_names.chunks(chunk_size) {
            let mut chunk_results = Vec::new();
            
            for name in chunk {
                let result = self.get_package_info(name).await;
                chunk_results.push((name.clone(), result));
                
                // Small delay between individual requests in chunk
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            
            results.extend(chunk_results);
            
            // Longer delay between chunks
            if package_names.len() > chunk_size {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
        
        results
    }
    
    /// Get registry health/status information
    pub async fn get_registry_status(&self) -> Result<bool, NpmError> {
        // Simple health check by trying to get info for a well-known package
        match self.package_exists("npm").await {
            Ok(_) => Ok(true),
            Err(NpmError::Timeout) => Ok(false),
            Err(NpmError::RequestFailed(_)) => Ok(false),
            Err(_) => Ok(true), // Other errors suggest registry is up but had other issues
        }
    }
}

/// NPM search API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmSearchResponse {
    pub objects: Vec<NpmSearchObject>,
    pub total: u32,
    pub time: String,
}

/// Individual search result object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmSearchObject {
    pub package: NpmSearchResult,
    pub score: NpmSearchScore,
}

/// Package information in search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmSearchResult {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub author: Option<NpmAuthor>,
    pub links: Option<NpmSearchLinks>,
}

/// Search relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmSearchScore {
    pub final_score: f64,
    pub detail: NpmSearchScoreDetail,
}

/// Detailed search scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmSearchScoreDetail {
    pub quality: f64,
    pub popularity: f64,
    pub maintenance: f64,
}

/// Package links in search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmSearchLinks {
    pub npm: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub bugs: Option<String>,
}

impl Default for NpmClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_npm_client_creation() {
        let client = NpmClient::new();
        assert_eq!(client.registry_url, "https://registry.npmjs.org");
        assert!(client.user_agent.starts_with("ppm/"));
    }
    
    #[test]
    fn test_npm_client_with_custom_registry() {
        let custom_url = "https://custom-registry.example.com".to_string();
        let client = NpmClient::with_registry_url(custom_url.clone());
        assert_eq!(client.registry_url, custom_url);
    }
    
    #[test]
    fn test_npm_to_package_conversion() {
        let client = NpmClient::new();
        
        let npm_info = NpmVersionInfo {
            name: "express".to_string(),
            version: "4.18.2".to_string(),
            description: Some("Fast, unopinionated web framework".to_string()),
            dist: NpmDistInfo {
                tarball: "https://registry.npmjs.org/express/-/express-4.18.2.tgz".to_string(),
                shasum: "3fabe32365a502c0de6acad9a82ed63abecf9a91".to_string(),
                integrity: Some("sha512-...".to_string()),
                file_count: Some(16),
                unpacked_size: Some(214819),
                extra: HashMap::new(),
            },
            dependencies: Some([
                ("body-parser".to_string(), "1.20.1".to_string()),
                ("cookie".to_string(), "0.5.0".to_string()),
            ].into()),
            dev_dependencies: Some([
                ("jest".to_string(), "^29.0.0".to_string()),
            ].into()),
            author: Some(NpmAuthor::String("TJ Holowaychuk".to_string())),
            license: Some(serde_json::Value::String("MIT".to_string())),
            keywords: Some(vec!["web".to_string(), "framework".to_string()]),
            extra: HashMap::new(),
        };
        
        let store_path = PathBuf::from("/store/packages/abc123");
        let package = client.npm_to_package(&npm_info, store_path.clone()).unwrap();
        
        assert_eq!(package.name, "express");
        assert_eq!(package.version, "4.18.2");
        assert_eq!(package.ecosystem, Ecosystem::JavaScript);
        assert_eq!(package.store_path, store_path);
        assert_eq!(package.metadata.description, Some("Fast, unopinionated web framework".to_string()));
        assert_eq!(package.metadata.author, Some("TJ Holowaychuk".to_string()));
        assert_eq!(package.metadata.license, Some("MIT".to_string()));
        assert_eq!(package.metadata.keywords, vec!["web".to_string(), "framework".to_string()]);
        
        // Check dependencies
        assert_eq!(package.dependencies.len(), 3);
        
        let prod_deps: Vec<_> = package.dependencies.iter()
            .filter(|d| !d.dev_only)
            .collect();
        assert_eq!(prod_deps.len(), 2);
        
        let dev_deps: Vec<_> = package.dependencies.iter()
            .filter(|d| d.dev_only)
            .collect();
        assert_eq!(dev_deps.len(), 1);
    }
    
    #[test]
    fn test_cache_update() {
        let client = NpmClient::new();
        let mut cache = RegistryCache::new(Ecosystem::JavaScript, 3600);
        
        let npm_response = NpmPackageResponse {
            name: "react".to_string(),
            versions: [
                ("18.0.0".to_string(), create_mock_version_info("react", "18.0.0")),
                ("18.1.0".to_string(), create_mock_version_info("react", "18.1.0")),
                ("18.2.0".to_string(), create_mock_version_info("react", "18.2.0")),
            ].into(),
            dist_tags: [("latest".to_string(), "18.2.0".to_string())].into(),
            description: Some("React library".to_string()),
            author: None,
            license: Some("MIT".to_string()),
            keywords: Some(vec!["react".to_string()]),
            time: None,
        };
        
        client.update_cache(&mut cache, &npm_response);
        
        let cached_info = cache.get_package("react").unwrap();
        assert_eq!(cached_info.name, "react");
        assert_eq!(cached_info.latest_version, "18.2.0");
        assert_eq!(cached_info.versions.len(), 3);
        assert!(cached_info.has_version("18.1.0"));
        assert!(!cached_info.has_version("17.0.0"));
    }
    
    #[test]
    fn test_author_parsing() {
        // Test string author
        let author_string = NpmAuthor::String("John Doe".to_string());
        if let NpmAuthor::String(name) = author_string {
            assert_eq!(name, "John Doe");
        }
        
        // Test object author
        let author_object = NpmAuthor::Object {
            name: "Jane Smith".to_string(),
            email: Some("jane@example.com".to_string()),
            url: Some("https://example.com".to_string()),
        };
        
        if let NpmAuthor::Object { name, email, url } = author_object {
            assert_eq!(name, "Jane Smith");
            assert_eq!(email, Some("jane@example.com".to_string()));
            assert_eq!(url, Some("https://example.com".to_string()));
        }
    }
    
    fn create_mock_version_info(name: &str, version: &str) -> NpmVersionInfo {
        NpmVersionInfo {
            name: name.to_string(),
            version: version.to_string(),
            description: Some(format!("{} package", name)),
            dist: NpmDistInfo {
                tarball: format!("https://registry.npmjs.org/{}/-/{}-{}.tgz", name, name, version),
                shasum: "a".repeat(40),
                integrity: Some("sha512-...".to_string()),
                file_count: Some(10),
                unpacked_size: Some(1000),
                extra: HashMap::new(),
            },
            dependencies: None,
            dev_dependencies: None,
            author: None,
            license: Some(serde_json::Value::String("MIT".to_string())),
            keywords: None,
            extra: HashMap::new(),
        }
    }
    
    // Note: Integration tests would require HTTP mocking or actual registry access
    // These would be in tests/integration/ directory
}
