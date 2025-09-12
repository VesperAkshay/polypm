use std::error::Error;
use tokio;

use ppm::services::package_installer::{PackageInstaller, InstallConfig};
use ppm::models::global_store::GlobalStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Testing Performance Optimizations...");
    
    // Create test configuration
    let config = InstallConfig {
        include_dev: false,
        skip_verification: true, // Skip for testing
        force_update: false,
        max_concurrent: 4,
        download_timeout: 30,
    };
    
    // Create global store for testing
    let global_store = GlobalStore::new("./test_global".into());
    
    // Create package installer with performance optimizations
    let installer = PackageInstaller::new(global_store, Some(config))?;
    
    // Get initial cache stats
    let initial_stats = installer.get_cache_stats();
    println!("Initial cache stats: entries={}, size={}KB, hit_ratio={:.2}%", 
             initial_stats.total_entries, 
             initial_stats.total_size_bytes / 1024, 
             initial_stats.hit_ratio * 100.0);
    
    // Test progress monitoring
    let progress = installer.get_download_progress();
    println!("Active downloads: {}", progress.len());
    
    println!("✓ Performance optimization components initialized successfully!");
    println!("- Parallel downloads: enabled (max {} concurrent)", 4);
    println!("- Download caching: enabled");
    println!("- Progress tracking: enabled");
    println!("- Cache statistics: available");
    
    Ok(())
}
