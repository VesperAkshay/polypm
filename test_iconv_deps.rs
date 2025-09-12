use std::collections::HashMap;
use serde_json::Value;

use ppm::models::dependency::Dependency;
use ppm::models::ecosystem::Ecosystem;
use ppm::services::npm_client::NpmClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing iconv-lite dependencies ===");
    
    let npm_client = NpmClient::new();
    
    // Get iconv-lite package info
    let package_info = npm_client.get_package_info("iconv-lite").await?;
    
    // Get version 0.4.24 specifically
    if let Some(version_info) = package_info.versions.get("0.4.24") {
        println!("📦 iconv-lite 0.4.24 dependencies:");
        
        if let Some(deps) = &version_info.dependencies {
            for (name, version_spec) in deps {
                println!("  - {} -> {}", name, version_spec);
                
                if name == "safer-buffer" {
                    println!("🔍 Found safer-buffer dependency: {}", version_spec);
                    
                    // Test resolving this exact version spec
                    match npm_client.resolve_version("safer-buffer", version_spec).await {
                        Ok(resolved) => println!("✅ Resolved to: {}", resolved),
                        Err(e) => println!("❌ Failed to resolve: {}", e),
                    }
                }
            }
        }
        
        println!("\n🔍 Optional dependencies:");
        if let Some(opt_deps) = &version_info.optional_dependencies {
            for (name, version_spec) in opt_deps {
                println!("  - {} -> {}", name, version_spec);
            }
        }
    }
    
    Ok(())
}
