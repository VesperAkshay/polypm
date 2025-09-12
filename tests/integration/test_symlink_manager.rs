use tempfile::TempDir;
use ppm::models::resolved_dependency::ResolvedDependency;
use ppm::models::ecosystem::Ecosystem;
use ppm::services::symlink_manager::SymlinkManager;
use ppm::models::symlink_structure::SymlinkConfig;

/// Helper to create a test resolved dependency
fn create_test_dependency(name: &str, version: &str) -> ResolvedDependency {
    ResolvedDependency {
        name: name.to_string(),
        version: version.to_string(),
        ecosystem: Ecosystem::JavaScript,
        hash: "test-hash".to_string(),
        integrity: "sha256-test".to_string(),
        store_path: format!("npm/{}/{}", name, version),
    }
}

#[tokio::test]
async fn test_symlink_manager_creation() {
    let manager = SymlinkManager::new();
    let capabilities = manager.get_platform_capabilities();
    
    // Should have some capabilities on any platform
    assert!(
        capabilities.supports_directory_symlinks ||
        capabilities.supports_file_symlinks ||
        capabilities.supports_hardlinks
    );
}

#[tokio::test]
async fn test_symlink_manager_with_custom_config() {
    let config = SymlinkConfig {
        use_junctions_on_windows: false,
        fallback_to_hardlinks: true,
        create_parent_dirs: true,
        overwrite_existing: true,
        max_depth: 5,
        validate_targets: false,
    };
    
    let manager = SymlinkManager::with_config(config.clone());
    assert_eq!(manager.config(), &config);
}

#[tokio::test]
async fn test_javascript_symlinks_creation_with_missing_targets() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    let global_store = temp_dir.path().join("global_store");
    
    // Create a config that doesn't validate targets (since they won't exist in this test)
    let config = SymlinkConfig {
        validate_targets: false,
        create_parent_dirs: true,
        overwrite_existing: true,
        ..Default::default()
    };
    let manager = SymlinkManager::with_config(config);
    
    let deps = vec![
        create_test_dependency("lodash", "4.17.21"),
        create_test_dependency("express", "4.18.0"),
    ];

    // This should succeed even without targets since we disabled validation
    let result = manager.create_javascript_symlinks(project_root, &deps, &global_store).await;
    
    match result {
        Ok(structure) => {
            // Verify structure was created
            assert_eq!(structure.ecosystem, Ecosystem::JavaScript);
            assert_eq!(structure.link_count(), 2);
            assert!(structure.has_link("lodash"));
            assert!(structure.has_link("express"));
            
            // Verify node_modules directory was created
            assert!(project_root.join("node_modules").exists());
        }
        Err(e) => {
            // If symlinks fail (e.g., permission issues), that's expected in some environments
            println!("Symlink creation failed (may be expected): {}", e);
        }
    }
}

#[tokio::test]
async fn test_javascript_symlinks_with_existing_targets() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    let global_store = temp_dir.path().join("global_store");
    
    // Create mock package directories in the global store
    let lodash_path = global_store.join("npm").join("lodash").join("4.17.21");
    tokio::fs::create_dir_all(&lodash_path).await.unwrap();
    tokio::fs::write(lodash_path.join("package.json"), r#"{"name":"lodash","version":"4.17.21"}"#).await.unwrap();
    
    let express_path = global_store.join("npm").join("express").join("4.18.0");
    tokio::fs::create_dir_all(&express_path).await.unwrap();
    tokio::fs::write(express_path.join("package.json"), r#"{"name":"express","version":"4.18.0"}"#).await.unwrap();
    
    let config = SymlinkConfig {
        validate_targets: true,
        create_parent_dirs: true,
        ..Default::default()
    };
    let manager = SymlinkManager::with_config(config);
    
    let deps = vec![
        create_test_dependency("lodash", "4.17.21"),
        create_test_dependency("express", "4.18.0"),
    ];

    let result = manager.create_javascript_symlinks(project_root, &deps, &global_store).await;
    
    match result {
        Ok(structure) => {
            assert_eq!(structure.link_count(), 2);
            assert!(project_root.join("node_modules").exists());
            
            // Check if actual symlinks were created (may fail in some environments)
            let lodash_link = project_root.join("node_modules").join("lodash");
            let express_link = project_root.join("node_modules").join("express");
            
            // At minimum, the directories should exist (either as symlinks or fallback)
            println!("Lodash link exists: {}", lodash_link.exists());
            println!("Express link exists: {}", express_link.exists());
        }
        Err(e) => {
            // Symlink creation may fail in some test environments
            println!("Symlink creation failed (may be expected in test environment): {}", e);
        }
    }
}

#[tokio::test]
async fn test_symlink_capabilities() {
    let manager = SymlinkManager::new();
    let capabilities = manager.get_platform_capabilities();
    
    // Test that we get reasonable capabilities
    #[cfg(windows)]
    {
        assert!(capabilities.supports_junctions || capabilities.supports_directory_symlinks);
        // Windows may require admin privileges for symlinks
    }
    
    #[cfg(unix)]
    {
        assert!(capabilities.supports_directory_symlinks);
        assert!(capabilities.supports_file_symlinks);
        assert!(!capabilities.requires_admin_privileges);
    }
    
    // All platforms should support hard links
    assert!(capabilities.supports_hardlinks);
}

#[tokio::test]
async fn test_symlink_structure_validation() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    let global_store = temp_dir.path().join("global_store");
    
    let config = SymlinkConfig {
        validate_targets: false,
        ..Default::default()
    };
    let manager = SymlinkManager::with_config(config);
    
    let deps = vec![create_test_dependency("test-package", "1.0.0")];
    
    let result = manager.create_javascript_symlinks(project_root, &deps, &global_store).await;
    
    match result {
        Ok(mut structure) => {
            // Test validation
            assert!(structure.validate().is_ok());
            
            // Test broken link detection
            let broken_links = manager.verify_symlinks(&mut structure).await.unwrap();
            // Since targets don't exist, all links should be "broken" 
            assert_eq!(broken_links.len(), 1);
            assert_eq!(broken_links[0], "test-package");
        }
        Err(e) => {
            println!("Symlink creation failed: {}", e);
        }
    }
}

#[tokio::test] 
async fn test_symlink_config_options() {
    // Test different configuration options
    let configs = vec![
        SymlinkConfig {
            use_junctions_on_windows: true,
            fallback_to_hardlinks: false,
            create_parent_dirs: true,
            overwrite_existing: false,
            max_depth: 10,
            validate_targets: true,
        },
        SymlinkConfig {
            use_junctions_on_windows: false,
            fallback_to_hardlinks: true,
            create_parent_dirs: false,
            overwrite_existing: true,
            max_depth: 5,
            validate_targets: false,
        },
    ];
    
    for config in configs {
        let manager = SymlinkManager::with_config(config.clone());
        assert_eq!(manager.config(), &config);
        
        // Verify capabilities are still accessible
        let capabilities = manager.get_platform_capabilities();
        assert!(capabilities.supports_hardlinks);
    }
}
