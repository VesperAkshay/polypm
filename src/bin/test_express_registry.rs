use ppm::services::npm_client::NpmClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let npm_client = NpmClient::new();
    
    // Test problematic packages
    let problematic_packages = ["inherits", "ipaddr.js", "safer-buffer"];
    
    for package_name in &problematic_packages {
        println!("\n=== Testing {} ===", package_name);
        
        match npm_client.get_package_info(package_name).await {
            Ok(package_info) => {
                println!("✅ Successfully fetched {} package info", package_name);
                
                if let Some(latest_version) = package_info.dist_tags.get("latest") {
                    println!("Latest version: {}", latest_version);
                } else {
                    println!("❌ No 'latest' tag found");
                }
                
                println!("Available versions: {}", package_info.versions.len());
            }
            Err(e) => {
                println!("❌ Failed to fetch {} package info: {}", package_name, e);
            }
        }
    }
    
    Ok(())
}
