use ppm::models::dependency::Dependency;
use ppm::models::ecosystem::Ecosystem;
use ppm::models::global_store::GlobalStore;
use ppm::services::dependency_resolver::DependencyResolver;
use ppm::services::npm_client::NpmClient;
use ppm::services::pypi_client::PypiClient;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Finding all packages that depend on safer-buffer ===");
    
    let npm_client = NpmClient::new();
    let pypi_client = PypiClient::new();
    let global_store = GlobalStore::new(PathBuf::from(".ppm/global"));
    
    let mut resolver = DependencyResolver::new(
        npm_client.clone(),
        pypi_client,
        global_store,
    );
    
    // Create the Express dependency that causes the full tree
    let express_dep = Dependency {
        name: "express".to_string(),
        version_spec: "^4.18.0".to_string(),
        resolved_version: None,
        ecosystem: Ecosystem::JavaScript,
        dev_only: false,
    };
    
    println!("🔍 Resolving Express and checking safer-buffer dependencies...");
    
    match resolver.resolve_dependencies(vec![express_dep]).await {
        Ok(result) => {
            println!("✅ Resolution successful!");
            println!("📦 Total resolved packages: {}", result.resolved.len());
            
            // Find all packages and check their dependencies for safer-buffer
            let mut safer_buffer_dependents = Vec::new();
            
            for resolved_dep in &result.resolved {
                // Check the dependencies of each resolved package
                if let Ok(package_info) = npm_client.get_package_info(&resolved_dep.name).await {
                    if let Some(version_info) = package_info.versions.get(&resolved_dep.version) {
                        if let Some(deps) = &version_info.dependencies {
                            if let Some(safer_buffer_spec) = deps.get("safer-buffer") {
                                safer_buffer_dependents.push((
                                    resolved_dep.name.clone(),
                                    resolved_dep.version.clone(),
                                    safer_buffer_spec.clone()
                                ));
                            }
                        }
                    }
                }
            }
            
            if safer_buffer_dependents.is_empty() {
                println!("🤔 No packages found that depend on safer-buffer");
            } else {
                println!("📋 Packages that depend on safer-buffer:");
                for (name, version, spec) in safer_buffer_dependents {
                    println!("   - {} v{} -> safer-buffer {}", name, version, spec);
                }
            }
            
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
