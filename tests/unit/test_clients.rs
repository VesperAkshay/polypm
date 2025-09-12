use std::collections::HashMap;
use reqwest::Client;
use serde_json::json;
use mockito::Server;
use tokio;

use ppm::services::npm_client::{NpmClient, NpmPackageResponse, NpmVersionInfo, NpmDistInfo, NpmAuthor, NpmError};
use ppm::services::pypi_client::{PypiClient, PypiPackageResponse, PypiPackageInfo, PypiError};
use ppm::models::ecosystem::Ecosystem;

/// Test module for NPM registry client
#[cfg(test)]
mod npm_client_tests {
    use super::*;

    /// Test NPM client creation with default registry
    #[test]
    fn test_npm_client_new() {
        let client = NpmClient::new();
        // We can't directly test private fields, but we can test the client works
        assert!(true); // Client creation should not panic
    }

    /// Test NPM client creation with custom registry URL
    #[test]
    fn test_npm_client_with_registry_url() {
        let custom_url = "https://custom-registry.npmjs.org".to_string();
        let client = NpmClient::with_registry_url(custom_url);
        assert!(true); // Client creation should not panic
    }

    /// Test NPM client creation with custom HTTP client
    #[test]
    fn test_npm_client_with_client() {
        let http_client = Client::new();
        let registry_url = "https://registry.npmjs.org".to_string();
        let client = NpmClient::with_client(http_client, registry_url);
        assert!(true); // Client creation should not panic
    }

    /// Test NPM package info parsing from JSON response
    #[test]
    fn test_npm_package_response_parsing() {
        let json_data = json!({
            "name": "test-package",
            "versions": {
                "1.0.0": {
                    "name": "test-package",
                    "version": "1.0.0",
                    "description": "A test package",
                    "dist": {
                        "tarball": "https://registry.npmjs.org/test-package/-/test-package-1.0.0.tgz",
                        "shasum": "d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2",
                        "integrity": "sha512-abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                        "fileCount": 10,
                        "unpackedSize": 50000
                    },
                    "dependencies": {
                        "lodash": "^4.17.21"
                    },
                    "devDependencies": {
                        "jest": "^27.0.0"
                    },
                    "author": {
                        "name": "Test Author",
                        "email": "test@example.com"
                    },
                    "license": "MIT",
                    "keywords": ["test", "package"]
                }
            },
            "dist-tags": {
                "latest": "1.0.0"
            },
            "description": "A test package",
            "author": {
                "name": "Test Author",
                "email": "test@example.com"
            },
            "license": "MIT",
            "keywords": ["test", "package"],
            "time": {
                "1.0.0": "2023-01-01T00:00:00.000Z"
            }
        });

        let response: Result<NpmPackageResponse, _> = serde_json::from_value(json_data);
        assert!(response.is_ok());

        let package_response = response.unwrap();
        assert_eq!(package_response.name, "test-package");
        assert_eq!(package_response.description, Some("A test package".to_string()));
        assert_eq!(package_response.license, Some("MIT".to_string()));
        assert!(package_response.versions.contains_key("1.0.0"));
        assert_eq!(package_response.dist_tags.get("latest"), Some(&"1.0.0".to_string()));
        
        let version_info = package_response.versions.get("1.0.0").unwrap();
        assert_eq!(version_info.name, "test-package");
        assert_eq!(version_info.version, "1.0.0");
        assert_eq!(version_info.description, Some("A test package".to_string()));
        assert_eq!(version_info.dist.tarball, "https://registry.npmjs.org/test-package/-/test-package-1.0.0.tgz");
        assert_eq!(version_info.dist.shasum, "d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2");
        
        // Test dependencies
        let deps = version_info.dependencies.as_ref().unwrap();
        assert_eq!(deps.get("lodash"), Some(&"^4.17.21".to_string()));
        
        let dev_deps = version_info.dev_dependencies.as_ref().unwrap();
        assert_eq!(dev_deps.get("jest"), Some(&"^27.0.0".to_string()));
    }

    /// Test NPM author parsing (string format)
    #[test]
    fn test_npm_author_string_format() {
        let json_data = json!("Test Author <test@example.com>");
        let author: Result<NpmAuthor, _> = serde_json::from_value(json_data);
        assert!(author.is_ok());
        
        match author.unwrap() {
            NpmAuthor::String(name) => assert_eq!(name, "Test Author <test@example.com>"),
            _ => panic!("Expected string format author"),
        }
    }

    /// Test NPM author parsing (object format)
    #[test]
    fn test_npm_author_object_format() {
        let json_data = json!({
            "name": "Test Author",
            "email": "test@example.com",
            "url": "https://example.com"
        });
        
        let author: Result<NpmAuthor, _> = serde_json::from_value(json_data);
        assert!(author.is_ok());
        
        match author.unwrap() {
            NpmAuthor::Object { name, email, url } => {
                assert_eq!(name, "Test Author");
                assert_eq!(email, Some("test@example.com".to_string()));
                assert_eq!(url, Some("https://example.com".to_string()));
            },
            _ => panic!("Expected object format author"),
        }
    }

    /// Test NPM error types
    #[test]
    fn test_npm_error_types() {
        let package_not_found = NpmError::PackageNotFound("test-package".to_string());
        assert_eq!(package_not_found.to_string(), "Package 'test-package' not found");
        
        let version_not_found = NpmError::VersionNotFound("test-package".to_string(), "1.0.0".to_string());
        assert_eq!(version_not_found.to_string(), "Version '1.0.0' not found for package 'test-package'");
        
        let invalid_name = NpmError::InvalidPackageName("invalid..name".to_string());
        assert_eq!(invalid_name.to_string(), "Invalid package name: invalid..name");
        
        let parse_error = NpmError::ParseError("Invalid JSON".to_string());
        assert_eq!(parse_error.to_string(), "Failed to parse registry response: Invalid JSON");
        
        let timeout = NpmError::Timeout;
        assert_eq!(timeout.to_string(), "Request timeout");
        
        let rate_limited = NpmError::RateLimited;
        assert_eq!(rate_limited.to_string(), "Rate limited by registry");
    }

    /// Test NPM package conversion to internal Package model
    #[tokio::test]
    async fn test_npm_to_package_conversion() {
        let client = NpmClient::new();
        
        let npm_version = NpmVersionInfo {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test package".to_string()),
            dist: NpmDistInfo {
                tarball: "https://registry.npmjs.org/test-package/-/test-package-1.0.0.tgz".to_string(),
                shasum: "d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2".to_string(),
                integrity: Some("sha512-abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string()),
                file_count: Some(10),
                unpacked_size: Some(50000),
                extra: HashMap::new(),
            },
            dependencies: Some({
                let mut deps = HashMap::new();
                deps.insert("lodash".to_string(), "^4.17.21".to_string());
                deps
            }),
            dev_dependencies: Some({
                let mut dev_deps = HashMap::new();
                dev_deps.insert("jest".to_string(), "^27.0.0".to_string());
                dev_deps
            }),
            author: Some(NpmAuthor::Object {
                name: "Test Author".to_string(),
                email: Some("test@example.com".to_string()),
                url: None,
            }),
            license: Some(serde_json::Value::String("MIT".to_string())),
            keywords: Some(vec!["test".to_string(), "package".to_string()]),
            extra: HashMap::new(),
        };
        
        let store_path = std::path::PathBuf::from("/tmp/test-store");
        let package = client.npm_to_package(&npm_version, store_path);
        
        assert!(package.is_ok());
        let package = package.unwrap();
        
        assert_eq!(package.name, "test-package");
        assert_eq!(package.version, "1.0.0");
        assert_eq!(package.ecosystem, Ecosystem::JavaScript);
        assert_eq!(package.metadata.description, Some("A test package".to_string()));
        assert_eq!(package.metadata.license, Some("MIT".to_string()));
        assert_eq!(package.metadata.keywords, vec!["test".to_string(), "package".to_string()]);
        assert_eq!(package.metadata.author, Some("Test Author <test@example.com>".to_string()));
        
        // Check dependencies
        assert_eq!(package.dependencies.len(), 2); // prod + dev
        let lodash_dep = package.dependencies.iter().find(|d| d.name == "lodash").unwrap();
        assert_eq!(lodash_dep.version_spec, "^4.17.21");
        assert!(!lodash_dep.dev_only);
        
        let jest_dep = package.dependencies.iter().find(|d| d.name == "jest").unwrap();
        assert_eq!(jest_dep.version_spec, "^27.0.0");
        assert!(jest_dep.dev_only);
    }

    /// Test NPM cache update functionality
    #[test]
    fn test_npm_cache_update() {
        use ppm::models::global_store::RegistryCache;
        
        let client = NpmClient::new();
        let mut cache = RegistryCache::new(Ecosystem::JavaScript, 3600); // 1 hour TTL
        
        let npm_response = NpmPackageResponse {
            name: "test-package".to_string(),
            versions: {
                let mut versions = HashMap::new();
                versions.insert("1.0.0".to_string(), NpmVersionInfo {
                    name: "test-package".to_string(),
                    version: "1.0.0".to_string(),
                    description: Some("A test package".to_string()),
                    dist: NpmDistInfo {
                        tarball: "https://registry.npmjs.org/test-package/-/test-package-1.0.0.tgz".to_string(),
                        shasum: "d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2".to_string(),
                        integrity: None,
                        file_count: None,
                        unpacked_size: None,
                        extra: HashMap::new(),
                    },
                    dependencies: None,
                    dev_dependencies: None,
                    author: None,
                    license: None,
                    keywords: None,
                    extra: HashMap::new(),
                });
                versions
            },
            dist_tags: {
                let mut tags = HashMap::new();
                tags.insert("latest".to_string(), "1.0.0".to_string());
                tags
            },
            description: Some("A test package".to_string()),
            author: None,
            license: None,
            keywords: None,
            time: None,
        };
        
        client.update_cache(&mut cache, &npm_response);
        
        // Verify cache was updated
        let cached_info = cache.packages.get("test-package");
        assert!(cached_info.is_some());
        
        let cached_info = cached_info.unwrap();
        assert_eq!(cached_info.name, "test-package");
        assert_eq!(cached_info.latest_version, "1.0.0");
        assert!(cached_info.versions.contains(&"1.0.0".to_string()));
    }
}

/// Test module for PyPI registry client
#[cfg(test)]
mod pypi_client_tests {
    use super::*;

    /// Test PyPI client creation with default registry
    #[test]
    fn test_pypi_client_new() {
        let client = PypiClient::new();
        // We can't directly test private fields, but we can test the client works
        assert!(true); // Client creation should not panic
    }

    /// Test PyPI client creation with custom registry URL
    #[test]
    fn test_pypi_client_with_registry_url() {
        let custom_url = "https://test.pypi.org".to_string();
        let client = PypiClient::with_registry_url(custom_url);
        assert!(true); // Client creation should not panic
    }

    /// Test PyPI client creation with custom HTTP client
    #[test]
    fn test_pypi_client_with_client() {
        let http_client = Client::new();
        let registry_url = "https://pypi.org".to_string();
        let client = PypiClient::with_client(http_client, registry_url);
        assert!(true); // Client creation should not panic
    }

    /// Test PyPI package info parsing from JSON response
    #[test]
    fn test_pypi_package_response_parsing() {
        let json_data = json!({
            "info": {
                "name": "test-package",
                "version": "1.0.0",
                "summary": "A test package for Python",
                "description": "A longer description of the test package",
                "description_content_type": "text/markdown",
                "author": "Test Author",
                "author_email": "test@example.com",
                "maintainer": "Test Maintainer",
                "maintainer_email": "maintainer@example.com",
                "license": "MIT",
                "keywords": "test,package,python",
                "classifiers": [
                    "Development Status :: 4 - Beta",
                    "Intended Audience :: Developers",
                    "License :: OSI Approved :: MIT License",
                    "Programming Language :: Python :: 3",
                    "Programming Language :: Python :: 3.8",
                    "Programming Language :: Python :: 3.9"
                ],
                "project_urls": {
                    "Homepage": "https://github.com/example/test-package",
                    "Bug Reports": "https://github.com/example/test-package/issues"
                },
                "home_page": "https://github.com/example/test-package",
                "download_url": "https://github.com/example/test-package/archive/v1.0.0.tar.gz",
                "platform": "any",
                "requires_python": ">=3.8",
                "requires_dist": [
                    "requests>=2.25.0",
                    "pydantic>=1.8.0"
                ],
                "provides_extra": ["dev", "test"]
            },
            "last_serial": 12345678,
            "releases": {
                "1.0.0": [
                    {
                        "filename": "test_package-1.0.0-py3-none-any.whl",
                        "packagetype": "bdist_wheel",
                        "python_version": "py3",
                        "size": 15000,
                        "upload_time": "2023-01-01T12:00:00",
                        "upload_time_iso_8601": "2023-01-01T12:00:00.000000Z",
                        "url": "https://files.pythonhosted.org/packages/test_package-1.0.0-py3-none-any.whl",
                        "md5_digest": "abcdef1234567890abcdef1234567890",
                        "digests": {
                            "sha256": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
                            "md5": "abcdef1234567890abcdef1234567890"
                        },
                        "requires_python": ">=3.8",
                        "yanked": false,
                        "yanked_reason": null
                    }
                ]
            },
            "urls": [
                {
                    "filename": "test_package-1.0.0-py3-none-any.whl",
                    "packagetype": "bdist_wheel",
                    "python_version": "py3",
                    "size": 15000,
                    "upload_time": "2023-01-01T12:00:00",
                    "upload_time_iso_8601": "2023-01-01T12:00:00.000000Z",
                    "url": "https://files.pythonhosted.org/packages/test_package-1.0.0-py3-none-any.whl",
                    "md5_digest": "abcdef1234567890abcdef1234567890",
                    "digests": {
                        "sha256": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
                        "md5": "abcdef1234567890abcdef1234567890"
                    },
                    "requires_python": ">=3.8",
                    "yanked": false,
                    "yanked_reason": null
                }
            ],
            "vulnerabilities": []
        });

        let response: Result<PypiPackageResponse, _> = serde_json::from_value(json_data);
        assert!(response.is_ok());

        let package_response = response.unwrap();
        assert_eq!(package_response.info.name, "test-package");
        assert_eq!(package_response.info.version, "1.0.0");
        assert_eq!(package_response.info.summary, Some("A test package for Python".to_string()));
        assert_eq!(package_response.info.description, Some("A longer description of the test package".to_string()));
        assert_eq!(package_response.info.author, Some("Test Author".to_string()));
        assert_eq!(package_response.info.author_email, Some("test@example.com".to_string()));
        assert_eq!(package_response.info.license, Some("MIT".to_string()));
        assert_eq!(package_response.info.requires_python, Some(">=3.8".to_string()));
        assert_eq!(package_response.last_serial, 12345678);
        
        // Test release files
        let releases = package_response.releases.get("1.0.0").unwrap();
        assert_eq!(releases.len(), 1);
        
        let release_file = &releases[0];
        assert_eq!(release_file.filename, "test_package-1.0.0-py3-none-any.whl");
        assert_eq!(release_file.packagetype, "bdist_wheel");
        assert_eq!(release_file.python_version, Some("py3".to_string()));
        assert_eq!(release_file.size, 15000);
        assert_eq!(release_file.digests.sha256, "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2");
        assert!(!release_file.yanked);
    }

    /// Test PyPI error types
    #[test]
    fn test_pypi_error_types() {
        let package_not_found = PypiError::PackageNotFound("test-package".to_string());
        assert_eq!(package_not_found.to_string(), "Package 'test-package' not found");
        
        let version_not_found = PypiError::VersionNotFound("test-package".to_string(), "1.0.0".to_string());
        assert_eq!(version_not_found.to_string(), "Version '1.0.0' not found for package 'test-package'");
        
        let invalid_name = PypiError::InvalidPackageName("invalid..name".to_string());
        assert_eq!(invalid_name.to_string(), "Invalid package name: invalid..name");
        
        let parse_error = PypiError::ParseError("Invalid JSON".to_string());
        assert_eq!(parse_error.to_string(), "Failed to parse registry response: Invalid JSON");
        
        let timeout = PypiError::Timeout;
        assert_eq!(timeout.to_string(), "Request timeout");
        
        let rate_limited = PypiError::RateLimited;
        assert_eq!(rate_limited.to_string(), "Rate limited by registry");
    }

    /// Test PyPI package conversion to internal Package model
    #[tokio::test]
    async fn test_pypi_to_package_conversion() {
        let client = PypiClient::new();
        
        let pypi_info = PypiPackageInfo {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            summary: Some("A test package for Python".to_string()),
            description: Some("A longer description".to_string()),
            description_content_type: Some("text/markdown".to_string()),
            author: Some("Test Author".to_string()),
            author_email: Some("test@example.com".to_string()),
            maintainer: None,
            maintainer_email: None,
            license: Some("MIT".to_string()),
            keywords: Some("test,package,python".to_string()),
            classifiers: Some(vec![
                "Development Status :: 4 - Beta".to_string(),
                "Programming Language :: Python :: 3".to_string(),
            ]),
            project_urls: None,
            home_page: Some("https://github.com/example/test-package".to_string()),
            download_url: None,
            platform: Some("any".to_string()),
            requires_python: Some(">=3.8".to_string()),
            requires_dist: Some(vec![
                "requests>=2.25.0".to_string(),
                "pydantic>=1.8.0".to_string(),
            ]),
            provides_extra: Some(vec!["dev".to_string(), "test".to_string()]),
        };
        
        let store_path = std::path::PathBuf::from("/tmp/test-store");
        let package = client.pypi_to_package(&pypi_info, store_path);
        
        assert!(package.is_ok());
        let package = package.unwrap();
        
        assert_eq!(package.name, "test-package");
        assert_eq!(package.version, "1.0.0");
        assert_eq!(package.ecosystem, Ecosystem::Python);
        assert_eq!(package.metadata.description, Some("A test package for Python".to_string()));
        assert_eq!(package.metadata.license, Some("MIT".to_string()));
        assert_eq!(package.metadata.keywords, vec!["test".to_string(), "package".to_string(), "python".to_string()]);
        assert_eq!(package.metadata.author, Some("Test Author <test@example.com>".to_string()));
        
        // Check dependencies (PyPI client stores full requirement as name for now)
        assert_eq!(package.dependencies.len(), 2);
        let requests_dep = package.dependencies.iter().find(|d| d.name == "requests>=2.25.0").unwrap();
        assert_eq!(requests_dep.version_spec, "requests>=2.25.0");
        assert!(!requests_dep.dev_only);
        
        let pydantic_dep = package.dependencies.iter().find(|d| d.name == "pydantic>=1.8.0").unwrap();
        assert_eq!(pydantic_dep.version_spec, "pydantic>=1.8.0");
        assert!(!pydantic_dep.dev_only);
    }

    /// Test PyPI cache update functionality
    #[test]
    fn test_pypi_cache_update() {
        use ppm::models::global_store::RegistryCache;
        
        let client = PypiClient::new();
        let mut cache = RegistryCache::new(Ecosystem::Python, 3600); // 1 hour TTL
        
        let pypi_response = PypiPackageResponse {
            info: PypiPackageInfo {
                name: "test-package".to_string(),
                version: "1.0.0".to_string(),
                summary: Some("A test package".to_string()),
                description: None,
                description_content_type: None,
                author: None,
                author_email: None,
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
            },
            last_serial: 12345,
            releases: {
                let mut releases = HashMap::new();
                releases.insert("1.0.0".to_string(), vec![]);
                releases
            },
            urls: vec![],
            vulnerabilities: None,
        };
        
        client.update_cache(&mut cache, &pypi_response);
        
        // Verify cache was updated
        let cached_info = cache.packages.get("test-package");
        assert!(cached_info.is_some());
        
        let cached_info = cached_info.unwrap();
        assert_eq!(cached_info.name, "test-package");
        assert_eq!(cached_info.latest_version, "1.0.0");
        assert!(cached_info.versions.contains(&"1.0.0".to_string()));
    }

    /// Test PyPI package integrity verification
    #[test]
    fn test_pypi_package_integrity_verification() {
        let client = PypiClient::new();
        
        // Test data and its SHA-256 hash
        let test_data = b"Hello, world!";
        let expected_sha256 = "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3";
        
        let is_valid = client.verify_package_integrity(test_data, expected_sha256);
        assert!(is_valid);
        
        // Test with wrong hash
        let wrong_sha256 = "0000000000000000000000000000000000000000000000000000000000000000";
        let is_invalid = client.verify_package_integrity(test_data, wrong_sha256);
        assert!(!is_invalid);
    }
}

/// Integration tests for registry clients (mock HTTP responses)
#[cfg(test)]
mod client_integration_tests {
    use super::*;

    /// Test NPM client with mocked HTTP response
    #[tokio::test]
    async fn test_npm_client_mock_response() {
        let mut server = Server::new_async().await;
        
        let mock_response = json!({
            "name": "lodash",
            "versions": {
                "4.17.21": {
                    "name": "lodash",
                    "version": "4.17.21",
                    "description": "Lodash modular utilities.",
                    "dist": {
                        "tarball": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz",
                        "shasum": "679591c564c3bffaae8454cf0b3df370c3d6911c",
                        "integrity": "sha512-v2kDEe57lecTulaDIuNTPy3Ry4gLGJ6Z1O3vE1krgXZNrsQ+LFTGHVxVjcXPs17LhbZVGedAJv8XZ1tvj5FvSg=="
                    },
                    "dependencies": {},
                    "license": "MIT"
                }
            },
            "dist-tags": {
                "latest": "4.17.21"
            },
            "description": "Lodash modular utilities.",
            "license": "MIT"
        });
        
        let mock = server.mock("GET", "/lodash")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;
        
        let client = NpmClient::with_registry_url(server.url());
        let result = client.get_package_info("lodash").await;
        
        mock.assert_async().await;
        assert!(result.is_ok());
        
        let package_info = result.unwrap();
        assert_eq!(package_info.name, "lodash");
        assert!(package_info.versions.contains_key("4.17.21"));
    }

    /// Test PyPI client with mocked HTTP response
    #[tokio::test]
    async fn test_pypi_client_mock_response() {
        let mut server = Server::new_async().await;
        
        let mock_response = json!({
            "info": {
                "name": "requests",
                "version": "2.28.1",
                "summary": "Python HTTP for Humans.",
                "description": "A simple, yet elegant HTTP library.",
                "author": "Kenneth Reitz",
                "author_email": "me@kennethreitz.org",
                "license": "Apache 2.0",
                "requires_python": ">=3.7, <4"
            },
            "last_serial": 15000000,
            "releases": {
                "2.28.1": [
                    {
                        "filename": "requests-2.28.1-py3-none-any.whl",
                        "packagetype": "bdist_wheel",
                        "python_version": "py3",
                        "size": 62317,
                        "upload_time": "2022-07-13T18:30:00",
                        "upload_time_iso_8601": "2022-07-13T18:30:00.000000Z",
                        "url": "https://files.pythonhosted.org/packages/requests-2.28.1-py3-none-any.whl",
                        "md5_digest": "abcdef1234567890abcdef1234567890",
                        "digests": {
                            "sha256": "8fefa2a1a1365bf5520aac41836fbee479da67864514bdb821f31ce07ce65349"
                        },
                        "yanked": false
                    }
                ]
            },
            "urls": [],
            "vulnerabilities": []
        });
        
        let mock = server.mock("GET", "/pypi/requests/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;
        
        let client = PypiClient::with_registry_url(server.url());
        let result = client.get_package_info("requests").await;
        
        mock.assert_async().await;
        assert!(result.is_ok());
        
        let package_info = result.unwrap();
        assert_eq!(package_info.info.name, "requests");
        assert_eq!(package_info.info.version, "2.28.1");
        assert!(package_info.releases.contains_key("2.28.1"));
    }

    /// Test error handling for 404 responses
    #[tokio::test]
    async fn test_package_not_found_errors() {
        let mut server = Server::new_async().await;
        
        // Test NPM 404
        let npm_mock = server.mock("GET", "/nonexistent-package")
            .with_status(404)
            .create_async()
            .await;
        
        let npm_client = NpmClient::with_registry_url(server.url());
        let npm_result = npm_client.get_package_info("nonexistent-package").await;
        
        npm_mock.assert_async().await;
        assert!(npm_result.is_err());
        match npm_result.unwrap_err() {
            NpmError::PackageNotFound(name) => assert_eq!(name, "nonexistent-package"),
            _ => panic!("Expected PackageNotFound error"),
        }
        
        // Test PyPI 404
        let pypi_mock = server.mock("GET", "/pypi/nonexistent-package/json")
            .with_status(404)
            .create_async()
            .await;
        
        let pypi_client = PypiClient::with_registry_url(server.url());
        let pypi_result = pypi_client.get_package_info("nonexistent-package").await;
        
        pypi_mock.assert_async().await;
        assert!(pypi_result.is_err());
        match pypi_result.unwrap_err() {
            PypiError::PackageNotFound(name) => assert_eq!(name, "nonexistent-package"),
            _ => panic!("Expected PackageNotFound error"),
        }
    }
}
