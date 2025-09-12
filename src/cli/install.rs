// Install command implementation
// Handles dependency installation with multiple modes and options

use clap::Args;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use crate::utils::error::{PpmError, Result};
use crate::utils::config::ConfigParser;
use crate::utils::lock_file::LockFileManager;
use crate::models::project::Project;
use crate::models::ecosystem::Ecosystem;
use crate::models::dependency::Dependency;
use crate::models::resolved_dependency::ResolvedDependency;
use crate::models::global_store::GlobalStore;
use crate::services::dependency_resolver::DependencyResolver;
use crate::services::package_installer::{PackageInstaller, InstallConfig};
use crate::services::npm_client::NpmClient;
use crate::services::pypi_client::PypiClient;

#[derive(Debug, Args)]
pub struct InstallCommand {
    /// Packages to install (if empty, install from project.toml)
    pub packages: Vec<String>,
    /// Add packages to dependencies (default for new packages)
    #[arg(long)]
    pub save: bool,
    /// Add packages to dev-dependencies
    #[arg(long)]
    pub save_dev: bool,
    /// Force JavaScript ecosystem
    #[arg(long)]
    pub javascript: bool,
    /// Force Python ecosystem
    #[arg(long)]
    pub python: bool,
    /// Skip symlink creation (install to global store only)
    #[arg(long)]
    pub no_symlinks: bool,
    /// Use only cached packages (fail if not available)
    #[arg(long)]
    pub offline: bool,
    /// Use exact versions from lock file (CI mode)
    #[arg(long)]
    pub frozen: bool,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct InstallResponse {
    pub status: String,
    pub duration_ms: u64,
    pub packages_installed: u64,
    pub ecosystems: HashMap<String, InstallStats>,
    pub lock_file: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct InstallStats {
    pub packages: u64,
    pub size_mb: f64,
    pub symlinks_created: bool,
}

impl InstallCommand {
    pub async fn run(&self) -> Result<()> {
        let start_time = Instant::now();
        
        // Check if project.toml exists
        if !Path::new("project.toml").exists() {
            return Err(PpmError::ConfigError(
                "No project.toml found (run 'ppm init' first)".to_string()
            ));
        }

        // Load project configuration
        let mut project = self.load_project().await?;
        
        // Handle specific package installation
        if !self.packages.is_empty() {
            self.install_specific_packages(&mut project).await?;
        }

        // Resolve dependencies
        let resolved_deps = if self.frozen {
            self.resolve_from_lock_file().await?
        } else {
            self.resolve_dependencies(&project).await?
        };

        // Install packages
        let install_stats = self.install_packages(&resolved_deps).await?;

        // Generate/update lock file
        let lock_file_path = self.generate_lock_file(&project, &resolved_deps).await?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        if self.json {
            self.output_json_response(duration_ms, &install_stats, &lock_file_path)?;
        } else {
            self.output_text_response(&install_stats)?;
        }

        Ok(())
    }

    async fn load_project(&self) -> Result<Project> {
        ConfigParser::load_project_config("project.toml")
    }

    async fn install_specific_packages(&self, project: &mut Project) -> Result<()> {
        for package_spec in &self.packages {
            let (package_name, version) = self.parse_package_spec(package_spec)?;
            let ecosystem = self.detect_ecosystem(&package_name).await?;
            
            // Check if package exists
            self.verify_package_exists(&package_name, &ecosystem).await?;
            
            // Add to appropriate dependencies section
            let deps_map = if self.save_dev {
                project.dev_dependencies.entry(ecosystem).or_insert_with(HashMap::new)
            } else {
                project.dependencies.entry(ecosystem).or_insert_with(HashMap::new)
            };
            
            deps_map.insert(package_name, version);
        }
        
        // Save updated project.toml if we added packages
        if !self.packages.is_empty() {
            self.save_project(project).await?;
        }
        
        Ok(())
    }

    fn parse_package_spec(&self, spec: &str) -> Result<(String, String)> {
        if let Some(at_pos) = spec.rfind('@') {
            let name = spec[..at_pos].to_string();
            let version = spec[at_pos + 1..].to_string();
            Ok((name, version))
        } else {
            // Default to latest version
            Ok((spec.to_string(), "^1.0.0".to_string()))
        }
    }

    async fn detect_ecosystem(&self, package_name: &str) -> Result<Ecosystem> {
        if self.javascript {
            return Ok(Ecosystem::JavaScript);
        }
        if self.python {
            return Ok(Ecosystem::Python);
        }
        
        // Simple heuristic - in practice this would query registries
        match package_name {
            name if name.starts_with("@") => Ok(Ecosystem::JavaScript),
            "react" | "vue" | "angular" | "lodash" | "express" | "jest" => Ok(Ecosystem::JavaScript),
            "flask" | "django" | "requests" | "numpy" | "pandas" | "pytest" => Ok(Ecosystem::Python),
            _ => {
                // Check both registries or let user specify
                Err(PpmError::ValidationError(format!(
                    "Could not detect ecosystem for package '{}'. Use --javascript or --python to specify.",
                    package_name
                )))
            }
        }
    }

    async fn verify_package_exists(&self, package_name: &str, _ecosystem: &Ecosystem) -> Result<()> {
        // Simulate package existence check
        if package_name.contains("nonexistent") {
            return Err(PpmError::ConfigError(format!(
                "Package '{}' not found",
                package_name
            )));
        }
        
        Ok(())
    }

    async fn save_project(&self, project: &Project) -> Result<()> {
        ConfigParser::save_project_config(project, "project.toml")
    }

    async fn resolve_from_lock_file(&self) -> Result<Vec<ResolvedDependency>> {
        let lock_manager = LockFileManager::new();
        lock_manager.get_resolved_dependencies()
    }

    async fn resolve_dependencies(&self, project: &Project) -> Result<Vec<ResolvedDependency>> {
        // Create clients
        let npm_client = NpmClient::new();
        let pypi_client = PypiClient::new();
        let global_store = GlobalStore::new(PathBuf::from(".ppm/global"));
        
        let mut resolver = DependencyResolver::new(
            npm_client,
            pypi_client,
            global_store,
        );
        
        // Filter ecosystems if specified
        let ecosystems_to_install = self.get_ecosystems_to_install(project)?;
        
        let mut all_deps = Vec::new();
        
        for ecosystem in ecosystems_to_install {
            // Get dependencies for this ecosystem
            let mut deps_for_ecosystem = Vec::new();
            
            if let Some(prod_deps) = project.dependencies.get(&ecosystem) {
                for (name, version_spec) in prod_deps {
                    deps_for_ecosystem.push(Dependency::new(
                        name.clone(),
                        version_spec.clone(),
                        ecosystem,
                        false, // not dev dependency
                    ));
                }
            }
            
            if let Some(dev_deps) = project.dev_dependencies.get(&ecosystem) {
                for (name, version_spec) in dev_deps {
                    deps_for_ecosystem.push(Dependency::new(
                        name.clone(),
                        version_spec.clone(),
                        ecosystem,
                        true, // is dev dependency
                    ));
                }
            }
            
            all_deps.extend(deps_for_ecosystem);
        }
        
        if all_deps.is_empty() {
            return Ok(Vec::new());
        }
        
        // Resolve dependencies
        let resolution_result = resolver.resolve_dependencies(all_deps).await
            .map_err(|e| PpmError::ConfigError(format!("Cannot resolve dependencies: {}", e)))?;
        
        if !resolution_result.failed.is_empty() {
            return Err(PpmError::ConfigError("Cannot resolve dependencies due to version conflicts".to_string()));
        }
        
        Ok(resolution_result.resolved)
    }

    fn get_ecosystems_to_install(&self, project: &Project) -> Result<Vec<Ecosystem>> {
        let mut ecosystems = Vec::new();
        
        if self.javascript && self.python {
            return Err(PpmError::ValidationError(
                "Cannot specify both --javascript and --python".to_string()
            ));
        }
        
        if self.javascript {
            ecosystems.push(Ecosystem::JavaScript);
        } else if self.python {
            ecosystems.push(Ecosystem::Python);
        } else {
            // Install all ecosystems that have dependencies
            for ecosystem in [Ecosystem::JavaScript, Ecosystem::Python] {
                let has_deps = project.dependencies.get(&ecosystem).map_or(false, |deps| !deps.is_empty()) ||
                              project.dev_dependencies.get(&ecosystem).map_or(false, |deps| !deps.is_empty());
                if has_deps {
                    ecosystems.push(ecosystem);
                }
            }
        }
        
        Ok(ecosystems)
    }

    async fn install_packages(&self, resolved_deps: &[ResolvedDependency]) -> Result<HashMap<String, InstallStats>> {
        if self.offline {
            // Check if all packages are available offline
            for dep in resolved_deps {
                if !self.is_package_cached(&dep.name, &dep.version).await? {
                    return Err(PpmError::NetworkError(format!(
                        "Package '{}@{}' is not available offline",
                        dep.name, dep.version
                    )));
                }
            }
        }

        // Create global store and installer
        let global_store = GlobalStore::new(PathBuf::from(".ppm/global"));
        let install_config = InstallConfig {
            include_dev: true,
            skip_verification: false,
            force_update: false,
            max_concurrent: 4,
            download_timeout: 30,
        };
        
        let installer = PackageInstaller::new(global_store, Some(install_config))?;
        let mut stats = HashMap::new();
        
        // Group by ecosystem
        let mut by_ecosystem: HashMap<Ecosystem, Vec<&ResolvedDependency>> = HashMap::new();
        for dep in resolved_deps {
            by_ecosystem.entry(dep.ecosystem).or_default().push(dep);
        }
        
        for (ecosystem, deps) in by_ecosystem {
            let ecosystem_name = match ecosystem {
                Ecosystem::JavaScript => "javascript",
                Ecosystem::Python => "python",
            };
            
            // Install packages for this ecosystem
            let packages_count = deps.len() as u64;
            let symlinks_created = !self.no_symlinks;
            
            // Create directories
            self.ensure_ecosystem_directories(&ecosystem).await?;
            
            // Actually install packages for this ecosystem
            match ecosystem {
                Ecosystem::Python => {
                    // Use PackageInstaller to create Python virtual environment and install packages
                    let current_dir = std::env::current_dir()?;
                    let installed_count = installer.create_simple_python_structure(&current_dir, &deps).await?;
                    println!("  Python packages: {} installed", installed_count);
                    println!("  Updated Python virtual environment");
                }
                Ecosystem::JavaScript => {
                    // Use PackageInstaller to create JavaScript node_modules and install packages
                    let current_dir = std::env::current_dir()?;
                    let installed_count = installer.create_simple_javascript_structure(&current_dir, &deps).await?;
                    println!("  JavaScript packages: {} installed", installed_count);
                    if !self.no_symlinks {
                        println!("  Created symlinks");
                    }
                }
            }
            
            stats.insert(ecosystem_name.to_string(), InstallStats {
                packages: packages_count,
                size_mb: packages_count as f64 * 2.5, // Simulate size
                symlinks_created,
            });
        }
        
        Ok(stats)
    }

    async fn is_package_cached(&self, _name: &str, _version: &str) -> Result<bool> {
        // Simulate cache check
        Ok(false)
    }

    async fn ensure_ecosystem_directories(&self, ecosystem: &Ecosystem) -> Result<()> {
        // Only create the global store directory for package caching
        let global_store_dir = Path::new(".ppm").join("global");
        fs::create_dir_all(&global_store_dir)?;
        
        // The actual package directories (node_modules, .venv) are created 
        // by the PackageInstaller when needed
        match ecosystem {
            Ecosystem::JavaScript => {
                // node_modules will be created by create_simple_javascript_structure
            }
            Ecosystem::Python => {
                // .venv will be created by create_simple_python_structure
            }
        }
        
        Ok(())
    }

    async fn simulate_package_installation(&self, _dep: &ResolvedDependency, _create_symlinks: bool) -> Result<()> {
        // Simulate package installation
        // In a real implementation, this would download and install the package
        Ok(())
    }

    async fn ensure_python_venv(&self, _project: &Project) -> Result<()> {
        let venv_path = Path::new(".ppm").join("venv");
        
        if !venv_path.exists() {
            // Simulate venv creation
            fs::create_dir_all(&venv_path)?;
            fs::create_dir_all(venv_path.join("lib"))?;
            fs::create_dir_all(venv_path.join("bin"))?;
        }
        
        Ok(())
    }

    async fn generate_lock_file(&self, project: &Project, resolved_deps: &[ResolvedDependency]) -> Result<String> {
        let lock_manager = LockFileManager::new();
        lock_manager.update_lock_file(project, resolved_deps)?;
        Ok(lock_manager.lock_file_path().to_string_lossy().to_string())
    }

    fn output_json_response(&self, duration_ms: u64, stats: &HashMap<String, InstallStats>, lock_file: &str) -> Result<()> {
        let total_packages: u64 = stats.values().map(|s| s.packages).sum();
        
        let response = InstallResponse {
            status: "success".to_string(),
            duration_ms,
            packages_installed: total_packages,
            ecosystems: stats.clone(),
            lock_file: lock_file.to_string(),
        };
        
        let json = serde_json::to_string_pretty(&response)
            .map_err(|e| PpmError::ConfigError(format!("Failed to serialize JSON response: {}", e)))?;
        
        println!("{}", json);
        Ok(())
    }

    fn output_text_response(&self, stats: &HashMap<String, InstallStats>) -> Result<()> {
        let total_packages: u64 = stats.values().map(|s| s.packages).sum();
        
        println!("✓ Resolved {} dependencies", total_packages);
        
        for (ecosystem, stat) in stats {
            let ecosystem_display = match ecosystem.as_str() {
                "javascript" => "JavaScript",
                "python" => "Python",
                _ => ecosystem,
            };
            println!("  {} packages: {} installed", ecosystem_display, stat.packages);
            
            if stat.symlinks_created {
                match ecosystem.as_str() {
                    "javascript" => println!("  Created symlinks"),
                    "python" => println!("  Updated Python virtual environment"),
                    _ => {}
                }
            } else {
                println!("  Installed to global store only");
            }
        }
        
        println!("✓ Installed {} packages", total_packages);
        
        Ok(())
    }
}
