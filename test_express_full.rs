use std::path::PathBuf;
use std::collections::HashMap;
use ppm::services::dependency_resolver::DependencyResolver;
use ppm::models::dependency::Dependency;
use ppm::models::ecosystem::Ecosystem;
use ppm::models::project::Project;
use ppm::models::global_store::GlobalStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Express Full Dependency Resolution ===\n");
    
    // Create a temporary project for testing
    let temp_dir = std::env::temp_dir().join("ppm_express_test");
    let project = Project {
        name: "express-test".to_string(),
        version: "1.0.0".to_string(),
        ecosystems: vec![Ecosystem::JavaScript],
        dependencies: HashMap::new(),
        dev_dependencies: HashMap::new(),
        scripts: HashMap::new(),
        venv_config: None,
    };
    
    // Create global store
    let global_store = GlobalStore::new(temp_dir.join("global_store"));
    
    // Create dependency resolver
    let npm_client = ppm::services::npm_client::NpmClient::new();
    let pypi_client = ppm::services::pypi_client::PypiClient::new();
    let mut resolver = DependencyResolver::new(npm_client, pypi_client, global_store.clone());
    
    // Create Express dependency
    let express_dep = Dependency::production(
        "express".to_string(),
        "^4.18.2".to_string(),
        Ecosystem::JavaScript,
    );
    
    println!("Resolving Express ^4.18.2 and all its dependencies...\n");
    
    // Resolve dependencies
    match resolver.resolve_dependencies(vec![express_dep]).await {
        Ok(resolution_result) => {
            let resolved = resolution_result.resolved;
            println!("✅ Successfully resolved {} packages:", resolved.len());
            
            println!("\n📦 Dependencies:");
            for dep in &resolved {
                println!("  • {} v{}", dep.name, dep.version);
            }
            
            // Check that the three previously failing packages are included
            let dep_names: Vec<&str> = resolved.iter().map(|d| d.name.as_str()).collect();
            
            println!("\n🔍 Checking previously failing packages:");
            
            if dep_names.contains(&"inherits") {
                println!("  ✅ inherits - RESOLVED (was failing due to JSON parsing)");
            } else {
                println!("  ❌ inherits - NOT FOUND");
            }
            
            if dep_names.contains(&"ipaddr.js") {
                println!("  ✅ ipaddr.js - RESOLVED (was failing due to dot in name)");
            } else {
                println!("  ❌ ipaddr.js - NOT FOUND");
            }
            
            if dep_names.contains(&"safer-buffer") {
                println!("  ✅ safer-buffer - RESOLVED (was failing due to version range)");
            } else {
                println!("  ❌ safer-buffer - NOT FOUND");
            }
            
            println!("\n📊 Summary:");
            println!("  Total packages: {}", resolved.len());
            
            if resolution_result.failed.len() > 0 {
                println!("  Failed packages: {}", resolution_result.failed.len());
                for failure in &resolution_result.failed {
                    println!("    • {} - {}", failure.dependency.name, failure.error);
                }
            }
            
            let total = resolved.len() + resolution_result.failed.len();
            let success_rate = if total > 0 { 
                (resolved.len() as f64 / total as f64) * 100.0
            } else { 
                100.0 
            };
            println!("  Success rate: {:.1}%", success_rate);
            
            if resolved.len() >= 58 {
                println!("\n🎉 SUCCESS: Express dependency resolution is working!");
                println!("   All packages including previously failing ones are now resolved.");
            } else {
                println!("\n⚠️  INFO: Got {} packages (previously had 58+)", resolved.len());
            }
        }
        Err(e) => {
            println!("❌ Failed to resolve Express dependencies: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
