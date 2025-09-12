use ppm::models::{
    global_store::GlobalStore,
};
use ppm::services::{
    package_installer::{InstallConfig, PackageInstaller},
};
use tempfile::TempDir;
use tokio;

/// Test package installer creation and configuration
#[tokio::test]
async fn test_package_installer_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let global_store = GlobalStore::new(temp_dir.path().to_path_buf());
    
    let config = InstallConfig {
        include_dev: false,
        skip_verification: false,
        force_update: false,
        max_concurrent: 1,
        download_timeout: 30,
    };
    
    let installer = PackageInstaller::new(global_store, Some(config));
    assert!(installer.is_ok(), "Failed to create package installer: {:?}", installer);
}

/// Test InstallConfig builder pattern
#[tokio::test]
async fn test_install_config_builder() {
    let config = InstallConfig::new()
        .with_dev_dependencies(true)
        .with_verification(false)
        .with_force_update(true)
        .with_concurrency(8)
        .with_timeout(600);
    
    assert_eq!(config.include_dev, true);
    assert_eq!(config.skip_verification, true);
    assert_eq!(config.force_update, true);
    assert_eq!(config.max_concurrent, 8);
    assert_eq!(config.download_timeout, 600);
}

/// Test default InstallConfig values
#[tokio::test]
async fn test_install_config_defaults() {
    let config = InstallConfig::default();
    
    assert_eq!(config.include_dev, false);
    assert_eq!(config.skip_verification, false);
    assert_eq!(config.force_update, false);
    assert_eq!(config.max_concurrent, 4);
    assert_eq!(config.download_timeout, 300);
}

/// Test InstallConfig concurrency limits
#[tokio::test]
async fn test_install_config_concurrency_limits() {
    let config = InstallConfig::new().with_concurrency(0);
    assert_eq!(config.max_concurrent, 1, "Concurrency should be at least 1");
    
    let config = InstallConfig::new().with_concurrency(16);
    assert_eq!(config.max_concurrent, 16);
}

/// Test package installer with different configurations
#[tokio::test]
async fn test_package_installer_with_different_configs() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let global_store = GlobalStore::new(temp_dir.path().to_path_buf());
    
    // Test with None config (should use defaults)
    let installer1 = PackageInstaller::new(global_store.clone(), None);
    assert!(installer1.is_ok(), "Failed to create installer with None config");
    
    // Test with custom config
    let config = InstallConfig::new()
        .with_dev_dependencies(true)
        .with_timeout(60);
    
    let installer2 = PackageInstaller::new(global_store, Some(config));
    assert!(installer2.is_ok(), "Failed to create installer with custom config");
}
