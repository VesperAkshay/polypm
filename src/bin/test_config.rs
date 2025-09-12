// Test program for T040 TOML configuration parsing and validation

use std::env;
use ppm::utils::config::ConfigParser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <toml-file>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    
    println!("Testing TOML configuration parsing and validation for: {}", file_path);
    
    match ConfigParser::load_project_config(file_path) {
        Ok(project) => {
            println!("✅ Configuration loaded successfully!");
            println!("Project: {} v{}", project.name, project.version);
            println!("Dependencies: {} ecosystems", project.dependencies.len());
            println!("Scripts: {} defined", project.scripts.len());
            if let Some(venv_config) = &project.venv_config {
                println!("Virtual environment: {} (auto_create: {})", venv_config.path, venv_config.auto_create);
            }
        }
        Err(e) => {
            println!("❌ Configuration validation failed:");
            println!("Error: {}", e);
        }
    }
}
