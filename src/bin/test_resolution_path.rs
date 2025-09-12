use ppm::models::dependency::Dependency;
use ppm::models::ecosystem::Ecosystem;
use ppm::models::global_store::GlobalStore;
use ppm::services::dependency_resolver::DependencyResolver;
use ppm::services::npm_client::NpmClient;
use ppm::services::pypi_client::PypiClient;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing dependency resolution path for safer-buffer ===");
    
    // Create the exact same resolver setup as in installation
    let npm_client = NpmClient::new();
    let pypi_client = PypiClient::new();
    let global_store = GlobalStore::new(PathBuf::from(".ppm/global"));
    
    let mut resolver = DependencyResolver::new(
        npm_client,
        pypi_client,
        global_store,
    );
    
    // Create the iconv-lite dependency which depends on safer-buffer
    let iconv_lite_dep = Dependency {
        name: "iconv-lite".to_string(),
        version_spec: "0.4.24".to_string(),
        resolved_version: None,
        ecosystem: Ecosystem::JavaScript,
        dev_only: false,
    };
    
    println!("🔍 Resolving iconv-lite and its dependencies...");
    
    // Resolve dependencies
    match resolver.resolve_dependencies(vec![iconv_lite_dep]).await {
        Ok(result) => {
            println!("✅ Resolution successful!");
            println!("📦 Resolved packages: {}", result.resolved.len());
            
            // Check if safer-buffer is in resolved
            let safer_buffer = result.resolved.iter()
                .find(|dep| dep.name == "safer-buffer");
            
            if let Some(sb) = safer_buffer {
                println!("✅ safer-buffer resolved: {} v{}", sb.name, sb.version);
            } else {
                println!("❌ safer-buffer NOT found in resolved dependencies");
            }
            
            // Check failed dependencies
            if !result.failed.is_empty() {
                println!("⚠️  Failed dependencies:");
                for failure in &result.failed {
                    println!("   - {}: {}", failure.dependency.name, failure.error);
                }
            }
        }
        Err(e) => {
            println!("❌ Resolution failed: {}", e);
        }
    }
    
    Ok(())
}
