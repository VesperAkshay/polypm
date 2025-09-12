use ppm::services::npm_client::NpmClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing safer-buffer version range resolution ===");
    
    let npm_client = NpmClient::new();
    
    // Get all versions first
    let versions = npm_client.get_available_versions("safer-buffer").await?;
    println!("📦 Available versions: {:?}", versions);
    
    // Test the exact problematic version range
    let version_spec = ">= 2.1.2 < 3";
    println!("🔍 Testing version spec: {}", version_spec);
    
    match npm_client.resolve_version("safer-buffer", version_spec).await {
        Ok(resolved) => println!("✅ Resolved to: {}", resolved),
        Err(e) => {
            println!("❌ Failed to resolve: {}", e);
            
            // Debug: try parsing manually
            println!("🐛 Debug: Let's check the range parsing logic");
            
            // Manually check if 2.1.2 is in the versions
            if versions.contains(&"2.1.2".to_string()) {
                println!("✅ Version 2.1.2 exists in available versions");
            } else {
                println!("❌ Version 2.1.2 NOT found in available versions");
            }
        }
    }
    
    Ok(())
}
