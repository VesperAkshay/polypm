use ppm::models::{dependency::Dependency, ecosystem::Ecosystem, global_store::GlobalStore};
use ppm::services::{dependency_resolver::DependencyResolver, npm_client::NpmClient, pypi_client::PypiClient};
use std::path::Path;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up directory
    let test_dir = Path::new("debug_test");
    if test_dir.exists() {
        fs::remove_dir_all(test_dir)?;
    }
    fs::create_dir_all(test_dir)?;
    std::env::set_current_dir(test_dir)?;
    
    // Create project.toml
    let project_toml = r#"
[project]
name = "debug-test"
version = "1.0.0"

[dependencies.javascript]
express = "4.18.0"
"#;
    fs::write("project.toml", project_toml)?;
    
    // Test dependency resolution directly
    println!("Testing dependency resolution for Express 4.18.0...");
    
    let npm_client = NpmClient::new();
    let pypi_client = PypiClient::new();
    let global_store = GlobalStore::new(Path::new(".ppm").to_path_buf());
    
    let mut resolver = DependencyResolver::new(npm_client, pypi_client, global_store);
    
    let express_dep = Dependency::production(
        "express".to_string(),
        "4.18.0".to_string(),
        Ecosystem::JavaScript,
    );
    
    println!("Resolving express dependency...");
    match resolver.resolve_dependencies(vec![express_dep]).await {
        Ok(result) => {
            println!("✅ Resolution successful!");
            println!("Resolved {} dependencies:", result.resolved.len());
            for dep in &result.resolved {
                println!("  - {} {} ({})", dep.name, dep.version, dep.ecosystem);
            }
            
            if !result.failed.is_empty() {
                println!("❌ Failed to resolve {} dependencies:", result.failed.len());
                for failure in &result.failed {
                    println!("  - {}: {}", failure.dependency.name, failure.error);
                }
            }
            
            println!("Total processed: {}", result.total_processed);
            println!("Max depth reached: {}", result.max_depth_reached);
            println!("Resolution time: {}ms", result.resolution_time_ms);
        }
        Err(e) => {
            println!("❌ Resolution failed: {}", e);
        }
    }
    
    Ok(())
}
