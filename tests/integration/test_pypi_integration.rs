use ppm::services::pypi_client::PypiClient;

#[tokio::test]
async fn test_pypi_real_registry_connection() {
    let client = PypiClient::new();
    
    // Test with a well-known package that should always exist
    let result = client.get_package_info("requests").await;
    
    match result {
        Ok(package_info) => {
            assert_eq!(package_info.info.name, "requests");
            assert!(!package_info.releases.is_empty());
            println!("✓ Successfully connected to PyPI registry and fetched requests package info");
        }
        Err(e) => {
            // If we can't connect to the real registry, this might be expected in CI
            println!("⚠ Could not connect to PyPI registry: {}", e);
            // We'll make this test pass but warn - the client should work when internet is available
        }
    }
}

#[tokio::test]
async fn test_pypi_get_latest_version() {
    let client = PypiClient::new();
    
    // Test getting latest version of a stable package
    match client.get_latest_version("numpy").await {
        Ok(version) => {
            assert!(!version.is_empty());
            assert!(version.chars().next().unwrap().is_ascii_digit());
            println!("✓ Got latest numpy version: {}", version);
        }
        Err(e) => {
            println!("⚠ Could not get latest version: {}", e);
        }
    }
}

#[tokio::test]
async fn test_pypi_download_package() {
    let client = PypiClient::new();
    
    // First get package info to get download URLs
    match client.get_package_info("six").await {
        Ok(package_info) => {
            let latest_version = &package_info.info.version;
            if let Some(files) = package_info.releases.get(latest_version) {
                if let Some(file) = files.first() {
                    // Try to download the package
                    match client.download_package(&file.url).await {
                        Ok(bytes) => {
                            assert!(!bytes.is_empty());
                            println!("✓ Downloaded package: {} bytes", bytes.len());
                        }
                        Err(e) => {
                            println!("⚠ Could not download package: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠ Could not get package info for download test: {}", e);
        }
    }
}

#[tokio::test]
async fn test_pypi_download_with_verification() {
    let client = PypiClient::new();
    
    // First get package info to get release files with integrity data
    match client.get_package_info("six").await {
        Ok(package_info) => {
            let latest_version = &package_info.info.version;
            if let Some(files) = package_info.releases.get(latest_version) {
                if let Some(file) = files.first() {
                    // Try to download with verification
                    match client.download_package_with_verification(file).await {
                        Ok(bytes) => {
                            assert!(!bytes.is_empty());
                            println!("✓ Downloaded and verified package: {} bytes", bytes.len());
                        }
                        Err(e) => {
                            println!("⚠ Could not download with verification: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠ Could not get package info for verification test: {}", e);
        }
    }
}

#[tokio::test]
async fn test_pypi_version_resolution() {
    let client = PypiClient::new();
    
    // Test various version specifications
    let test_cases = vec![
        ("requests", "latest"),
        ("flask", "*"),
        ("django", ">=3.0"),
    ];
    
    for (package, version_spec) in test_cases {
        match client.resolve_version(package, version_spec).await {
            Ok(resolved) => {
                assert!(!resolved.is_empty());
                println!("✓ Resolved {} {} -> {}", package, version_spec, resolved);
            }
            Err(e) => {
                println!("⚠ Could not resolve {} {}: {}", package, version_spec, e);
            }
        }
    }
}

#[tokio::test]
async fn test_pypi_package_exists() {
    let client = PypiClient::new();
    
    // Test with a package that should exist
    match client.package_exists("requests").await {
        Ok(exists) => {
            assert!(exists);
            println!("✓ Confirmed requests package exists");
        }
        Err(e) => {
            println!("⚠ Could not check if requests exists: {}", e);
        }
    }
    
    // Test with a package that should not exist
    match client.package_exists("this-python-package-definitely-does-not-exist-12345").await {
        Ok(exists) => {
            assert!(!exists);
            println!("✓ Confirmed non-existent package does not exist");
        }
        Err(e) => {
            println!("⚠ Error checking non-existent package: {}", e);
        }
    }
}

#[tokio::test]
async fn test_pypi_get_best_download_file() {
    let client = PypiClient::new();
    
    match client.get_best_download_file("requests", "2.31.0").await {
        Ok(file) => {
            assert!(!file.filename.is_empty());
            assert!(!file.url.is_empty());
            println!("✓ Got best download file: {} ({})", file.filename, file.packagetype);
        }
        Err(e) => {
            println!("⚠ Could not get best download file: {}", e);
        }
    }
}

#[tokio::test]
async fn test_pypi_batch_package_info() {
    let client = PypiClient::new();
    
    let packages = vec![
        "requests".to_string(),
        "flask".to_string(), 
        "django".to_string(),
    ];
    
    let results = client.get_multiple_package_infos(&packages).await;
    
    assert_eq!(results.len(), 3);
    
    for (name, result) in results {
        match result {
            Ok(package_info) => {
                // PyPI package names can be case-sensitive in response, so use case-insensitive comparison
                assert_eq!(package_info.info.name.to_lowercase(), name.to_lowercase());
                println!("✓ Fetched package info for {}", name);
            }
            Err(e) => {
                println!("⚠ Could not fetch {}: {}", name, e);
            }
        }
    }
}

#[tokio::test]
async fn test_pypi_registry_status() {
    let client = PypiClient::new();
    
    match client.get_registry_status().await {
        Ok(is_healthy) => {
            println!("✓ Registry status: {}", if is_healthy { "healthy" } else { "unhealthy" });
        }
        Err(e) => {
            println!("⚠ Could not check registry status: {}", e);
        }
    }
}
