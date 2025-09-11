use std::collections::HashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::models::package::Package;
use crate::models::ecosystem::Ecosystem;
use crate::models::dependency::Dependency;
use crate::models::global_store::{CachedPackageInfo, RegistryCache};

/// PyPI registry API client for Python package management
#[derive(Debug, Clone)]
pub struct PypiClient {
    /// HTTP client for registry requests
    client: Client,
    /// Base URL for PyPI registry (configurable for testing)
    registry_url: String,
    /// Simple API URL for PyPI (used for package discovery)
    simple_url: String,
    /// User agent string for requests
    user_agent: String,
}

/// Response from PyPI JSON API package endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PypiPackageResponse {
    /// Package information
    pub info: PypiPackageInfo,
    /// Last serial number (for caching)
    pub last_serial: u64,
    /// All releases with files
    pub releases: HashMap<String, Vec<PypiReleaseFile>>,
    /// URLs for package resources
    pub urls: Vec<PypiReleaseFile>,
    /// Vulnerabilities information (if available)
    pub vulnerabilities: Option<Vec<PypiVulnerability>>,
}

/// Package metadata from PyPI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PypiPackageInfo {
    /// Package name
    pub name: String,
    /// Package version (latest)
    pub version: String,
    /// Package summary/description
    pub summary: Option<String>,
    /// Long description
    pub description: Option<String>,
    /// Content type of description (text/markdown, text/x-rst, etc.)
    pub description_content_type: Option<String>,
    /// Package author
    pub author: Option<String>,
    /// Author email
    pub author_email: Option<String>,
    /// Package maintainer
    pub maintainer: Option<String>,
    /// Maintainer email
    pub maintainer_email: Option<String>,
    /// Package license
    pub license: Option<String>,
    /// Package keywords
    pub keywords: Option<String>,
    /// Package classifiers
    pub classifiers: Option<Vec<String>>,
    /// Project URLs
    pub project_urls: Option<HashMap<String, String>>,
    /// Package homepage
    pub home_page: Option<String>,
    /// Download URL
    pub download_url: Option<String>,
    /// Platform compatibility
    pub platform: Option<String>,
    /// Python version requirements
    pub requires_python: Option<String>,
    /// Package dependencies
    pub requires_dist: Option<Vec<String>>,
    /// Extra dependencies
    pub provides_extra: Option<Vec<String>>,
}

/// Individual file/release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PypiReleaseFile {
    /// Filename
    pub filename: String,
    /// Package type (sdist, bdist_wheel, etc.)
    pub packagetype: String,
    /// Python version
    pub python_version: Option<String>,
    /// File size in bytes
    pub size: u64,
    /// Upload timestamp
    pub upload_time: String,
    /// Upload timestamp (ISO format)
    pub upload_time_iso_8601: String,
    /// Download URL
    pub url: String,
    /// MD5 hash (deprecated but still provided)
    pub md5_digest: String,
    /// SHA256 hash
    pub digests: PypiDigests,
    /// Whether file requires Python
    pub requires_python: Option<String>,
    /// Whether file was yanked
    pub yanked: bool,
    /// Yank reason if applicable
    pub yanked_reason: Option<String>,
}

/// Hash digests for a release file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PypiDigests {
    /// SHA256 hash
    pub sha256: String,
    /// MD5 hash (deprecated)
    pub md5: Option<String>,
}

/// Vulnerability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PypiVulnerability {
    /// Vulnerability ID
    pub id: String,
    /// Link to vulnerability details
    pub link: String,
    /// Source of vulnerability information
    pub source: String,
}

/// Search response from PyPI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PypiSearchResponse {
    /// Search metadata
    pub meta: PypiSearchMeta,
    /// Search results
    pub projects: Vec<PypiSearchResult>,
}

/// Search metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PypiSearchMeta {
    /// API version
    pub api_version: String,
}

/// Individual search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PypiSearchResult {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package description
    pub description: String,
}

/// Simple API response for package discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PypiSimpleResponse {
    /// Available files
    pub files: Vec<PypiSimpleFile>,
    /// Package metadata
    pub meta: Option<PypiSimpleMeta>,
}

/// Simple API file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PypiSimpleFile {
    /// Filename
    pub filename: String,
    /// Download URL
    pub url: String,
    /// Hash information
    pub hashes: Option<HashMap<String, String>>,
    /// Whether file requires Python
    pub requires_python: Option<String>,
    /// Upload timestamp
    pub upload_time: Option<String>,
}

/// Simple API metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PypiSimpleMeta {
    /// API version
    pub api_version: String,
}

/// PyPI client errors
#[derive(Debug, thiserror::Error)]
pub enum PypiError {
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
    
    /// Invalid Python version specification
    #[error("Invalid Python version specification: {0}")]
    InvalidPythonVersion(String),
}

impl PypiClient {
    /// Create a new PyPI registry client
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            registry_url: "https://pypi.org".to_string(),
            simple_url: "https://pypi.org/simple".to_string(),
            user_agent: format!("ppm/{}", env!("CARGO_PKG_VERSION")),
        }
    }
    
    /// Create a new PyPI client with custom registry URL (for testing)
    pub fn with_registry_url(registry_url: String) -> Self {
        let simple_url = format!("{}/simple", registry_url);
        Self {
            client: Client::new(),
            registry_url,
            simple_url,
            user_agent: format!("ppm/{}", env!("CARGO_PKG_VERSION")),
        }
    }
    
    /// Create a new PyPI client with custom HTTP client (for testing)
    pub fn with_client(client: Client, registry_url: String) -> Self {
        let simple_url = format!("{}/simple", registry_url);
        Self {
            client,
            registry_url,
            simple_url,
            user_agent: format!("ppm/{}", env!("CARGO_PKG_VERSION")),
        }
    }
    
    /// Get package information from PyPI JSON API
    pub async fn get_package_info(&self, package_name: &str) -> Result<PypiPackageResponse, PypiError> {
        // Validate package name
        Ecosystem::Python.validate_package_name(package_name)
            .map_err(|_| PypiError::InvalidPackageName(package_name.to_string()))?;
        
        let url = format!("{}/pypi/{}/json", self.registry_url, package_name);
        
        let response = self.client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .header("Accept", "application/json")
            .send()
            .await?;
        
        if response.status() == 404 {
            return Err(PypiError::PackageNotFound(package_name.to_string()));
        }
        
        if response.status() == 429 {
            return Err(PypiError::RateLimited);
        }
        
        if !response.status().is_success() {
            return Err(PypiError::RequestFailed(
                response.error_for_status().unwrap_err()
            ));
        }
        
        let package_info: PypiPackageResponse = response.json().await
            .map_err(|e| PypiError::ParseError(e.to_string()))?;
        
        Ok(package_info)
    }
    
    /// Get specific version information for a package
    pub async fn get_version_info(&self, package_name: &str, version: &str) -> Result<PypiPackageInfo, PypiError> {
        let url = format!("{}/pypi/{}/{}/json", self.registry_url, package_name, version);
        
        let response = self.client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .header("Accept", "application/json")
            .send()
            .await?;
        
        if response.status() == 404 {
            return Err(PypiError::VersionNotFound(package_name.to_string(), version.to_string()));
        }
        
        if !response.status().is_success() {
            return Err(PypiError::RequestFailed(
                response.error_for_status().unwrap_err()
            ));
        }
        
        let package_response: PypiPackageResponse = response.json().await
            .map_err(|e| PypiError::ParseError(e.to_string()))?;
        
        Ok(package_response.info)
    }
    
    /// Get the latest version of a package
    pub async fn get_latest_version(&self, package_name: &str) -> Result<String, PypiError> {
        let package_info = self.get_package_info(package_name).await?;
        Ok(package_info.info.version)
    }
    
    /// Get all available versions for a package
    pub async fn get_available_versions(&self, package_name: &str) -> Result<Vec<String>, PypiError> {
        let package_info = self.get_package_info(package_name).await?;
        
        let mut versions: Vec<String> = package_info.releases.keys().cloned().collect();
        // Sort versions (simple lexicographic for now, could implement proper PEP 440 sorting)
        versions.sort();
        
        Ok(versions)
    }
    
    /// Resolve version specification to exact version
    pub async fn resolve_version(&self, package_name: &str, version_spec: &str) -> Result<String, PypiError> {
        // For now, implement basic version resolution
        // In a full implementation, this would handle PEP 440 version specifiers
        
        if version_spec == "latest" || version_spec == "*" {
            return self.get_latest_version(package_name).await;
        }
        
        // If version_spec is exact version, verify it exists
        let versions = self.get_available_versions(package_name).await?;
        
        // Handle common Python version prefixes
        let clean_version = version_spec.trim_start_matches("==")
                                      .trim_start_matches(">=")
                                      .trim_start_matches("<=")
                                      .trim_start_matches("!=")
                                      .trim_start_matches("~=")
                                      .trim_start_matches('>')
                                      .trim_start_matches('<');
        
        if versions.contains(&clean_version.to_string()) {
            Ok(clean_version.to_string())
        } else {
            // Find best matching version (simplified PEP 440 logic)
            for version in versions.iter().rev() {
                if version.starts_with(clean_version) {
                    return Ok(version.clone());
                }
            }
            
            Err(PypiError::VersionNotFound(package_name.to_string(), version_spec.to_string()))
        }
    }
    
    /// Convert PyPI package information to our Package model
    pub fn pypi_to_package(&self, pypi_info: &PypiPackageInfo, store_path: std::path::PathBuf) -> Result<Package> {
        // Create package metadata
        let mut metadata = crate::models::package::PackageMetadata::default();
        metadata.description = pypi_info.summary.clone().or_else(|| pypi_info.description.clone());
        metadata.license = pypi_info.license.clone();
        
        // Parse keywords from string
        if let Some(keywords_str) = &pypi_info.keywords {
            metadata.keywords = keywords_str
                .split(',')
                .map(|k| k.trim().to_string())
                .filter(|k| !k.is_empty())
                .collect();
        }
        
        // Extract author information
        if let Some(author) = &pypi_info.author {
            metadata.author = Some(if let Some(email) = &pypi_info.author_email {
                format!("{} <{}>", author, email)
            } else {
                author.clone()
            });
        } else if let Some(maintainer) = &pypi_info.maintainer {
            metadata.author = Some(if let Some(email) = &pypi_info.maintainer_email {
                format!("{} <{}>", maintainer, email)
            } else {
                maintainer.clone()
            });
        }
        
        // For now, use a placeholder hash - in production this would be calculated from the actual package content
        let hash = "placeholder_hash_for_pypi_package".to_string();
        
        // Convert dependencies from requires_dist
        let mut dependencies = Vec::new();
        if let Some(requires_dist) = &pypi_info.requires_dist {
            for requirement in requires_dist {
                // Parse requirement string (simplified - real implementation would use a proper parser)
                if let Some(package_name) = requirement.split_whitespace().next() {
                    // Extract version specification if present
                    let version_spec = if requirement.contains(">=") || requirement.contains("<=") || 
                                         requirement.contains("==") || requirement.contains("!=") || 
                                         requirement.contains("~=") || requirement.contains('>') || 
                                         requirement.contains('<') {
                        requirement.clone()
                    } else {
                        "*".to_string()
                    };
                    
                    dependencies.push(Dependency::production(
                        package_name.to_string(),
                        version_spec,
                        Ecosystem::Python,
                    ));
                }
            }
        }
        
        let mut package = Package::with_metadata(
            pypi_info.name.clone(),
            pypi_info.version.clone(),
            Ecosystem::Python,
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
    pub fn update_cache(&self, cache: &mut RegistryCache, package_info: &PypiPackageResponse) {
        let versions: Vec<String> = package_info.releases.keys().cloned().collect();
        let latest_version = package_info.info.version.clone();
        
        let cached_info = CachedPackageInfo::new(
            package_info.info.name.clone(),
            versions,
            latest_version,
        );
        
        cache.update_package(cached_info);
    }
    
    /// Download package file (returns raw bytes)
    pub async fn download_package(&self, download_url: &str) -> Result<Vec<u8>, PypiError> {
        let response = self.client
            .get(download_url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(PypiError::RequestFailed(
                response.error_for_status().unwrap_err()
            ));
        }
        
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
    
    /// Get best download file for a package version (prefers wheels over source)
    pub async fn get_best_download_file(&self, package_name: &str, version: &str) -> Result<PypiReleaseFile, PypiError> {
        let package_info = self.get_package_info(package_name).await?;
        
        let files = package_info.releases.get(version)
            .ok_or_else(|| PypiError::VersionNotFound(package_name.to_string(), version.to_string()))?;
        
        // Prefer wheels over source distributions
        let best_file = files.iter()
            .find(|f| f.packagetype == "bdist_wheel" && !f.yanked)
            .or_else(|| files.iter().find(|f| f.packagetype == "sdist" && !f.yanked))
            .or_else(|| files.first()) // Fallback to any file
            .ok_or_else(|| PypiError::VersionNotFound(package_name.to_string(), version.to_string()))?;
        
        Ok(best_file.clone())
    }
    
    /// Search for packages in PyPI (Note: PyPI deprecated search, this would use a third-party service)
    pub async fn search_packages(&self, query: &str, limit: Option<usize>) -> Result<Vec<PypiSearchResult>, PypiError> {
        // Note: PyPI's search API was deprecated. In a real implementation, you might use:
        // - Third-party search services
        // - Package name matching from the simple API
        // - External search engines
        
        // For now, return a placeholder implementation
        let _limit = limit.unwrap_or(20);
        
        // This is a simplified mock - real implementation would use alternative search methods
        Ok(vec![
            PypiSearchResult {
                name: format!("mock-search-result-for-{}", query),
                version: "1.0.0".to_string(),
                description: format!("Mock search result for query: {}", query),
            }
        ])
    }
    
    /// Check if a package exists in the registry
    pub async fn package_exists(&self, package_name: &str) -> Result<bool, PypiError> {
        match self.get_package_info(package_name).await {
            Ok(_) => Ok(true),
            Err(PypiError::PackageNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
    
    /// Get package files using Simple API (useful for dependency resolution)
    pub async fn get_simple_package_files(&self, package_name: &str) -> Result<Vec<PypiSimpleFile>, PypiError> {
        let url = format!("{}/{}/", self.simple_url, package_name);
        
        let response = self.client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .header("Accept", "application/vnd.pypi.simple.v1+json")
            .send()
            .await?;
        
        if response.status() == 404 {
            return Err(PypiError::PackageNotFound(package_name.to_string()));
        }
        
        if !response.status().is_success() {
            return Err(PypiError::RequestFailed(
                response.error_for_status().unwrap_err()
            ));
        }
        
        let simple_response: PypiSimpleResponse = response.json().await
            .map_err(|e| PypiError::ParseError(e.to_string()))?;
        
        Ok(simple_response.files)
    }
    
    /// Verify package integrity using SHA256 hash
    pub fn verify_package_integrity(&self, content: &[u8], expected_sha256: &str) -> bool {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = hasher.finalize();
        let calculated_hash = format!("{:x}", hash);
        
        calculated_hash == expected_sha256
    }
}

impl Default for PypiClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_pypi_client_creation() {
        let client = PypiClient::new();
        assert_eq!(client.registry_url, "https://pypi.org");
        assert_eq!(client.simple_url, "https://pypi.org/simple");
        assert!(client.user_agent.starts_with("ppm/"));
    }
    
    #[test]
    fn test_pypi_client_with_custom_registry() {
        let custom_url = "https://test-pypi.org".to_string();
        let client = PypiClient::with_registry_url(custom_url.clone());
        assert_eq!(client.registry_url, custom_url);
        assert_eq!(client.simple_url, "https://test-pypi.org/simple");
    }
    
    #[test]
    fn test_pypi_to_package_conversion() {
        let client = PypiClient::new();
        
        let pypi_info = PypiPackageInfo {
            name: "flask".to_string(),
            version: "2.3.2".to_string(),
            summary: Some("A simple framework for building complex web applications.".to_string()),
            description: Some("Flask is a lightweight web application framework...".to_string()),
            description_content_type: Some("text/x-rst".to_string()),
            author: Some("Armin Ronacher".to_string()),
            author_email: Some("armin.ronacher@active-4.com".to_string()),
            maintainer: None,
            maintainer_email: None,
            license: Some("BSD-3-Clause".to_string()),
            keywords: Some("web,framework,application".to_string()),
            classifiers: Some(vec![
                "Development Status :: 5 - Production/Stable".to_string(),
                "Environment :: Web Environment".to_string(),
            ]),
            project_urls: None,
            home_page: Some("https://flask.palletsprojects.com/".to_string()),
            download_url: None,
            platform: None,
            requires_python: Some(">=3.8".to_string()),
            requires_dist: Some(vec![
                "Werkzeug >= 2.3.0".to_string(),
                "Jinja2 >= 3.1.2".to_string(),
                "itsdangerous >= 2.1.2".to_string(),
                "click >= 8.1.3".to_string(),
            ]),
            provides_extra: Some(vec!["async".to_string(), "dotenv".to_string()]),
        };
        
        let store_path = PathBuf::from("/store/packages/flask123");
        let package = client.pypi_to_package(&pypi_info, store_path.clone()).unwrap();
        
        assert_eq!(package.name, "flask");
        assert_eq!(package.version, "2.3.2");
        assert_eq!(package.ecosystem, Ecosystem::Python);
        assert_eq!(package.store_path, store_path);
        assert_eq!(package.metadata.description, Some("A simple framework for building complex web applications.".to_string()));
        assert_eq!(package.metadata.author, Some("Armin Ronacher <armin.ronacher@active-4.com>".to_string()));
        assert_eq!(package.metadata.license, Some("BSD-3-Clause".to_string()));
        assert_eq!(package.metadata.keywords, vec!["web".to_string(), "framework".to_string(), "application".to_string()]);
        
        // Check dependencies
        assert_eq!(package.dependencies.len(), 4);
        
        let dep_names: Vec<_> = package.dependencies.iter()
            .map(|d| d.name.clone())
            .collect();
        assert!(dep_names.contains(&"Werkzeug".to_string()));
        assert!(dep_names.contains(&"Jinja2".to_string()));
        assert!(dep_names.contains(&"itsdangerous".to_string()));
        assert!(dep_names.contains(&"click".to_string()));
    }
    
    #[test]
    fn test_cache_update() {
        let client = PypiClient::new();
        let mut cache = RegistryCache::new(Ecosystem::Python, 3600);
        
        let pypi_response = PypiPackageResponse {
            info: PypiPackageInfo {
                name: "requests".to_string(),
                version: "2.31.0".to_string(),
                summary: Some("Python HTTP for Humans.".to_string()),
                description: None,
                description_content_type: None,
                author: Some("Kenneth Reitz".to_string()),
                author_email: None,
                maintainer: None,
                maintainer_email: None,
                license: Some("Apache 2.0".to_string()),
                keywords: None,
                classifiers: None,
                project_urls: None,
                home_page: None,
                download_url: None,
                platform: None,
                requires_python: Some(">=3.7".to_string()),
                requires_dist: None,
                provides_extra: None,
            },
            last_serial: 12345,
            releases: [
                ("2.29.0".to_string(), vec![]),
                ("2.30.0".to_string(), vec![]),
                ("2.31.0".to_string(), vec![]),
            ].into(),
            urls: vec![],
            vulnerabilities: None,
        };
        
        client.update_cache(&mut cache, &pypi_response);
        
        let cached_info = cache.get_package("requests").unwrap();
        assert_eq!(cached_info.name, "requests");
        assert_eq!(cached_info.latest_version, "2.31.0");
        assert_eq!(cached_info.versions.len(), 3);
        assert!(cached_info.has_version("2.30.0"));
        assert!(!cached_info.has_version("2.28.0"));
    }
    
    #[test]
    fn test_author_parsing() {
        let client = PypiClient::new();
        
        // Test with author and email
        let pypi_info_with_email = PypiPackageInfo {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            summary: None,
            description: None,
            description_content_type: None,
            author: Some("John Doe".to_string()),
            author_email: Some("john@example.com".to_string()),
            maintainer: None,
            maintainer_email: None,
            license: None,
            keywords: None,
            classifiers: None,
            project_urls: None,
            home_page: None,
            download_url: None,
            platform: None,
            requires_python: None,
            requires_dist: None,
            provides_extra: None,
        };
        
        let package = client.pypi_to_package(&pypi_info_with_email, PathBuf::from("/tmp")).unwrap();
        assert_eq!(package.metadata.author, Some("John Doe <john@example.com>".to_string()));
        
        // Test with maintainer fallback
        let pypi_info_maintainer = PypiPackageInfo {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            summary: None,
            description: None,
            description_content_type: None,
            author: None,
            author_email: None,
            maintainer: Some("Jane Smith".to_string()),
            maintainer_email: Some("jane@example.com".to_string()),
            license: None,
            keywords: None,
            classifiers: None,
            project_urls: None,
            home_page: None,
            download_url: None,
            platform: None,
            requires_python: None,
            requires_dist: None,
            provides_extra: None,
        };
        
        let package = client.pypi_to_package(&pypi_info_maintainer, PathBuf::from("/tmp")).unwrap();
        assert_eq!(package.metadata.author, Some("Jane Smith <jane@example.com>".to_string()));
    }
    
    #[test]
    fn test_keywords_parsing() {
        let client = PypiClient::new();
        
        let pypi_info = PypiPackageInfo {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            summary: None,
            description: None,
            description_content_type: None,
            author: None,
            author_email: None,
            maintainer: None,
            maintainer_email: None,
            license: None,
            keywords: Some("web, api, http, client".to_string()),
            classifiers: None,
            project_urls: None,
            home_page: None,
            download_url: None,
            platform: None,
            requires_python: None,
            requires_dist: None,
            provides_extra: None,
        };
        
        let package = client.pypi_to_package(&pypi_info, PathBuf::from("/tmp")).unwrap();
        assert_eq!(package.metadata.keywords, vec!["web", "api", "http", "client"]);
    }
    
    #[test]
    fn test_package_integrity_verification() {
        let client = PypiClient::new();
        
        // Test with known content and hash
        let content = b"hello world";
        let expected_sha256 = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        
        assert!(client.verify_package_integrity(content, expected_sha256));
        assert!(!client.verify_package_integrity(content, "invalid_hash"));
    }
    
    // Note: Integration tests would require HTTP mocking or actual registry access
    // These would be in tests/integration/ directory
}
