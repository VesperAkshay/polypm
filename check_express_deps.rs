use ppm::services::npm_client::NpmClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Checking Express Dependencies ===\n");
    
    let client = NpmClient::new();
    
    // Get Express package info
    match client.get_package_info("express").await {
        Ok(package_info) => {
            if let Some(latest_version) = package_info.dist_tags.get("latest") {
                println!("Latest Express version: {}", latest_version);
                
                if let Some(version_info) = package_info.versions.get(latest_version) {
                    println!("\nDependencies of Express {}:", latest_version);
                    
                    if let Some(deps) = &version_info.dependencies {
                        for (name, version) in deps {
                            println!("  • {} {}", name, version);
                        }
                        println!("\nTotal dependencies: {}", deps.len());
                        
                        // Check if array-flatten is in the dependencies
                        if deps.contains_key("array-flatten") {
                            println!("\n✅ array-flatten is listed as a dependency: {}", deps["array-flatten"]);
                        } else {
                            println!("\n❌ array-flatten is NOT in Express dependencies");
                        }
                    } else {
                        println!("No dependencies listed");
                    }
                    
                    if let Some(dev_deps) = &version_info.dev_dependencies {
                        println!("\nDev dependencies: {}", dev_deps.len());
                        for (name, version) in dev_deps {
                            println!("  • {} {}", name, version);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to get Express package info: {}", e);
        }
    }
    
    // Also check array-flatten separately
    println!("\n=== Checking array-flatten package ===");
    match client.get_package_info("array-flatten").await {
        Ok(package_info) => {
            if let Some(latest_version) = package_info.dist_tags.get("latest") {
                println!("✅ array-flatten exists, latest version: {}", latest_version);
            }
        }
        Err(e) => {
            println!("❌ array-flatten not found: {}", e);
        }
    }
    
    Ok(())
}
