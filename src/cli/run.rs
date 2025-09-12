use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::env;
use std::fs;
use serde_json::json;

use crate::models::project::{Project, ProjectToml};
use crate::models::ecosystem::Ecosystem;
use crate::utils::error::{PpmError, Result};

/// Run project scripts
#[derive(Debug, Args)]
pub struct RunCommand {
    /// Script name to run
    pub script: Option<String>,

    /// List all available scripts
    #[arg(long)]
    pub list: bool,

    /// Show environment variables for script execution
    #[arg(long)]
    pub env: bool,

    /// Output results in JSON format
    #[arg(long)]
    pub json: bool,

    /// Additional arguments to pass to the script
    #[arg(last = true)]
    pub args: Vec<String>,
}

impl RunCommand {
    /// Execute the run command
    pub async fn execute(&self) -> Result<()> {
        // Load project configuration
        let project = self.load_project()?;

        // Handle --list flag
        if self.list {
            return self.list_scripts(&project);
        }

        // Require script name if not listing
        let script_name = self.script.as_ref().ok_or_else(|| {
            PpmError::ValidationError("Script name required (use --list to see available scripts)".to_string())
        })?;

        // Handle --env flag
        if self.env {
            return self.show_environment(&project, script_name);
        }

        // Execute the script
        self.execute_script(&project, script_name).await
    }

    /// Load project configuration
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

    /// List all available scripts
    fn list_scripts(&self, project: &Project) -> Result<()> {
        if project.scripts.is_empty() {
            if self.json {
                let response = json!({
                    "scripts": [],
                    "message": "No scripts defined in project.toml"
                });
                println!("{}", serde_json::to_string_pretty(&response)
                    .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
            } else {
                println!("No scripts defined in project.toml");
            }
            return Ok(());
        }

        if self.json {
            let scripts: Vec<_> = project.scripts.iter()
                .map(|(name, command)| json!({
                    "name": name,
                    "command": command
                }))
                .collect();
            
            let response = json!({
                "scripts": scripts
            });
            
            println!("{}", serde_json::to_string_pretty(&response)
                .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
        } else {
            println!("Available scripts:");
            for (name, command) in &project.scripts {
                println!("  {}: {}", name, command);
            }
        }

        Ok(())
    }

    /// Show environment variables for script execution
    fn show_environment(&self, project: &Project, script_name: &str) -> Result<()> {
        // Check if script exists
        if !project.scripts.contains_key(script_name) {
            return Err(PpmError::ValidationError(
                format!("Script '{}' not found. Available scripts: {}", 
                    script_name, 
                    project.scripts.keys().cloned().collect::<Vec<_>>().join(", "))
            ));
        }

        let env_vars = self.setup_environment(&project)?;

        if self.json {
            let response = json!({
                "script": script_name,
                "environment": env_vars
            });
            println!("{}", serde_json::to_string_pretty(&response)
                .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
        } else {
            println!("Environment for script '{}':", script_name);
            for (key, value) in &env_vars {
                println!("  {}={}", key, value);
            }
        }

        Ok(())
    }

    /// Execute a script
    async fn execute_script(&self, project: &Project, script_name: &str) -> Result<()> {
        // Check if any scripts are defined
        if project.scripts.is_empty() {
            return Err(PpmError::ValidationError(
                "No scripts defined in project.toml".to_string()
            ));
        }

        // Find the script
        let script_command = project.scripts.get(script_name).ok_or_else(|| {
            let available: Vec<_> = project.scripts.keys().cloned().collect();
            PpmError::ValidationError(
                format!("Script '{}' not found. Available scripts: {}", 
                    script_name, 
                    available.join(", "))
            )
        })?;

        // Set up environment
        let env_vars = self.setup_environment(&project)?;

        // Prepare command with arguments
        let mut full_command = script_command.clone();
        if !self.args.is_empty() {
            full_command.push(' ');
            full_command.push_str(&self.args.join(" "));
        }

        // Execute the script
        let mut command = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.args(&["/C", &full_command]);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.args(&["-c", &full_command]);
            cmd
        };

        // Add environment variables
        for (key, value) in &env_vars {
            command.env(key, value);
        }

        // Execute and capture result
        let output = command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| PpmError::IoError(e))?;

        let exit_code = output.status.code().unwrap_or(-1);

        // Handle JSON output
        if self.json {
            let response = json!({
                "status": if exit_code == 0 { "success" } else { "failure" },
                "script": script_name,
                "command": script_command,
                "exit_code": exit_code,
                "args": self.args
            });
            
            println!("{}", serde_json::to_string_pretty(&response)
                .map_err(|e| PpmError::ConfigError(format!("JSON serialization error: {}", e)))?);
        }

        // Return error if script failed
        if exit_code != 0 {
            return Err(PpmError::ExecutionError(
                format!("Script '{}' failed with exit code {}", script_name, exit_code)
            ));
        }

        Ok(())
    }

    /// Set up environment variables for script execution
    fn setup_environment(&self, project: &Project) -> Result<HashMap<String, String>> {
        let mut env_vars = HashMap::new();

        // Get current working directory
        let current_dir = env::current_dir()
            .map_err(|e| PpmError::IoError(e))?;

        // Set up JavaScript environment if applicable
        if project.dependencies.contains_key(&Ecosystem::JavaScript) || 
           project.dev_dependencies.contains_key(&Ecosystem::JavaScript) {
            
            let node_modules = current_dir.join("node_modules");
            if node_modules.exists() {
                env_vars.insert(
                    "NODE_PATH".to_string(),
                    node_modules.to_string_lossy().to_string()
                );
            } else {
                // Set a default path in .ppm
                env_vars.insert(
                    "NODE_PATH".to_string(),
                    current_dir.join(".ppm").join("node_modules").to_string_lossy().to_string()
                );
            }
        }

        // Set up Python environment if applicable
        if project.dependencies.contains_key(&Ecosystem::Python) || 
           project.dev_dependencies.contains_key(&Ecosystem::Python) {
            
            let venv_path = current_dir.join(".ppm").join("venv");
            if venv_path.exists() {
                env_vars.insert(
                    "VIRTUAL_ENV".to_string(),
                    venv_path.to_string_lossy().to_string()
                );
                
                // Set PYTHONPATH to include site-packages
                let site_packages = if cfg!(target_os = "windows") {
                    venv_path.join("Lib").join("site-packages")
                } else {
                    venv_path.join("lib").join("python3").join("site-packages")
                };
                
                if site_packages.exists() {
                    env_vars.insert(
                        "PYTHONPATH".to_string(),
                        site_packages.to_string_lossy().to_string()
                    );
                }
            } else {
                // Set a default path in .ppm
                env_vars.insert(
                    "PYTHONPATH".to_string(),
                    current_dir.join(".ppm").join("python").to_string_lossy().to_string()
                );
            }
        }

        // Add project-specific environment variables
        env_vars.insert(
            "PPM_PROJECT_NAME".to_string(),
            project.name.clone()
        );
        
        env_vars.insert(
            "PPM_PROJECT_VERSION".to_string(),
            project.version.clone()
        );

        env_vars.insert(
            "PPM_PROJECT_ROOT".to_string(),
            current_dir.to_string_lossy().to_string()
        );

        Ok(env_vars)
    }
}
