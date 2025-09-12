use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use std::fs;
use serde_json::json;

use crate::models::project::{Project, ProjectToml};
use crate::models::ecosystem::Ecosystem;
use crate::utils::error::{PpmError, Result};

/// Add new dependencies to project and install them
#[derive(Debug, Args)]
pub struct AddCommand {
    /// List of packages to add (required, at least one)
    pub packages: Vec<String>,

    /// Add to dev-dependencies instead of dependencies
    #[arg(long)]
    pub save_dev: bool,

    /// Force JavaScript ecosystem detection
    #[arg(long)]
    pub javascript: bool,

    /// Force Python ecosystem detection
    #[arg(long)]
    pub python: bool,

    /// Specify version constraint for single package
    #[arg(long)]
    pub version: Option<String>,

    /// Output results in JSON format
    #[arg(long)]
    pub json: bool,
}

impl AddCommand {
    /// Execute the add command
    pub async fn execute(&self) -> Result<()> {
        let start_time = Instant::now();
        
        // Validate command arguments
        self.validate_arguments()?;
        
        // Load project configuration
        let mut project = self.load_project()?;
        
        // Parse package specifications
        let packages_to_add = self.parse_packages()?;
        
        // Resolve package ecosystems
        let resolved_packages = self.resolve_package_ecosystems(&packages_to_add).await?;
        
        // Check for existing packages
        self.check_existing_packages(&project, &resolved_packages)?;
        
        // Update project configuration
        self.update_project_config(&mut project, &resolved_packages)?;
        
        // Save updated project
        self.save_project(&project)?;
        
        // Install packages (simulated)
        self.install_packages(&project, &resolved_packages).await?;
        
        // Output results
        let duration_ms = start_time.elapsed().as_millis() as u64;
        self.output_results(&resolved_packages, duration_ms)?;
        
        Ok(())
    }

    fn validate_arguments(&self) -> Result<()> {
        if self.packages.is_empty() {
            return Err(PpmError::ValidationError(
                "No packages specified (usage: ppm add <packages...>)".to_string()
            ));
        }

        if self.javascript && self.python {
            return Err(PpmError::ValidationError(
                "Cannot specify both --javascript and --python flags".to_string()
            ));
        }

        if let Some(version) = &self.version {
            if self.packages.len() > 1 {
                return Err(PpmError::ValidationError(
                    "Cannot specify --version with multiple packages".to_string()
                ));
            }
        }

        Ok(())
    }

    fn load_project(&self) -> Result<Project> {
        let project_path = PathBuf::from("project.toml");
        if !project_path.exists() {
            return Err(PpmError::ConfigError(
                "No project.toml found (run 'ppm init' first)".to_string()
            ));
        }

        let content = fs::read_to_string("project.toml")
            .map_err(|e| PpmError::IoError(e))?;
        
        let project_toml: ProjectToml = toml::from_str(&content)
            .map_err(|e| PpmError::ConfigError(format!("Invalid project.toml: {}", e)))?;
        
        let project = Project::from(project_toml);
        Ok(project)
    }

    fn parse_packages(&self) -> Result<Vec<(String, String)>> {
        self.packages.iter()
            .map(|spec| self.parse_package_spec(spec))
            .collect()
    }

    fn parse_package_spec(&self, spec: &str) -> Result<(String, String)> {
        if let Some(at_pos) = spec.rfind('@') {
            // Handle scoped packages like @types/node@^18.0.0
            if spec.starts_with('@') && at_pos > 0 {
                let name = spec[..at_pos].to_string();
                let version = spec[at_pos + 1..].to_string();
                return Ok((name, version));
            }
            // Handle regular packages like express@^4.18.0
            else if !spec.starts_with('@') {
                let name = spec[..at_pos].to_string();
                let version = spec[at_pos + 1..].to_string();
                return Ok((name, version));
            }
        }

        // No version specified, use default or command-line version
        let version = if let Some(cmd_version) = &self.version {
            cmd_version.clone()
        } else {
            "latest".to_string()
        };

        Ok((spec.to_string(), version))
    }

    async fn resolve_package_ecosystems(&self, packages: &[(String, String)]) -> Result<Vec<(String, String, Ecosystem)>> {
        let mut resolved = Vec::new();
        for (name, version) in packages {
            let ecosystem = self.detect_ecosystem(name).await?;
            resolved.push((name.clone(), version.clone(), ecosystem));
        }
        Ok(resolved)
    }

    async fn detect_ecosystem(&self, package_name: &str) -> Result<Ecosystem> {
        if self.javascript {
            return Ok(Ecosystem::JavaScript);
        }
        if self.python {
            return Ok(Ecosystem::Python);
        }

        // Simple heuristics for ecosystem detection
        match package_name {
            "react" | "lodash" | "express" | "jest" => Ok(Ecosystem::JavaScript),
            "flask" | "django" | "requests" | "pytest" => Ok(Ecosystem::Python),
            _ => Err(PpmError::ValidationError(
                format!("Could not detect ecosystem for package '{}'. Use --javascript or --python to specify.", package_name)
            )),
        }
    }

    fn check_existing_packages(&self, project: &Project, packages: &[(String, String, Ecosystem)]) -> Result<()> {
        for (name, _version, ecosystem) in packages {
            if let Some(deps) = project.dependencies.get(ecosystem) {
                if deps.contains_key(name) {
                    return Err(PpmError::ValidationError(
                        format!("Package '{}' already exists in dependencies", name)
                    ));
                }
            }

            if let Some(dev_deps) = project.dev_dependencies.get(ecosystem) {
                if dev_deps.contains_key(name) {
                    return Err(PpmError::ValidationError(
                        format!("Package '{}' already exists in dev-dependencies", name)
                    ));
                }
            }
        }
        Ok(())
    }

    fn update_project_config(&self, project: &mut Project, packages: &[(String, String, Ecosystem)]) -> Result<()> {
        for (name, version, ecosystem) in packages {
            let deps_map = if self.save_dev {
                project.dev_dependencies.entry(*ecosystem).or_insert_with(HashMap::new)
            } else {
                project.dependencies.entry(*ecosystem).or_insert_with(HashMap::new)
            };

            deps_map.insert(name.clone(), version.clone());
        }
        Ok(())
    }

    fn save_project(&self, project: &Project) -> Result<()> {
        let project_toml = ProjectToml::from(project.clone());
        let toml_content = toml::to_string_pretty(&project_toml)
            .map_err(|e| PpmError::ConfigError(format!("Failed to serialize project.toml: {}", e)))?;

        fs::write("project.toml", toml_content)
            .map_err(|e| PpmError::IoError(e))?;

        Ok(())
    }

    async fn install_packages(&self, _project: &Project, packages: &[(String, String, Ecosystem)]) -> Result<()> {
        // Simulate package installation
        for (name, version, ecosystem) in packages {
            println!("Installing {} {} ({:?})...", name, version, ecosystem);
        }
        Ok(())
    }

    fn output_results(&self, packages: &[(String, String, Ecosystem)], duration_ms: u64) -> Result<()> {
        if self.json {
            self.output_json_response(packages, duration_ms)?;
        } else {
            self.output_text_response(packages, duration_ms)?;
        }
        Ok(())
    }

    fn output_json_response(&self, packages: &[(String, String, Ecosystem)], duration_ms: u64) -> Result<()> {
        let response = json!({
            "success": true,
            "packages": packages.iter().map(|(name, version, ecosystem)| {
                json!({
                    "name": name,
                    "version": version,
                    "ecosystem": format!("{:?}", ecosystem).to_lowercase(),
                    "target": if self.save_dev { "dev-dependencies" } else { "dependencies" }
                })
            }).collect::<Vec<_>>(),
            "duration_ms": duration_ms
        });

        println!("{}", serde_json::to_string_pretty(&response)
            .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
        Ok(())
    }

    fn output_text_response(&self, packages: &[(String, String, Ecosystem)], duration_ms: u64) -> Result<()> {
        let target = if self.save_dev { "dev-dependencies" } else { "dependencies" };
        
        for (name, version, ecosystem) in packages {
            println!("Added {} {} to {} ({:?})", name, version, target, ecosystem);
        }
        
        println!("\nCompleted in {}ms", duration_ms);
        Ok(())
    }
}
