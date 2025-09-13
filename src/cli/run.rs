use clap::Args;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::env;
use serde_json::json;

use crate::models::project::Project;
use crate::models::ecosystem::Ecosystem;
use crate::utils::error::{PpmError, Result};
use crate::utils::config::ConfigParser;

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
            PpmError::ValidationError("Script name required.\n\nUsage:\n  ppm run <script-name>    # Run a script\n  ppm run --list          # List all available scripts\n\nExample: ppm run build".to_string())
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
        ConfigParser::load_project_config("project.toml")
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
                println!("\nTo add scripts, edit your project.toml file:");
                println!("[scripts]");
                println!("build = \"npm run build\"");
                println!("test = \"npm test\"");
                println!("start = \"node index.js\"");
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
            let available_scripts: Vec<_> = project.scripts.keys().cloned().collect();
            let suggestion = if available_scripts.is_empty() {
                "\n\nNo scripts are currently defined. Add some to your project.toml:\n[scripts]\nbuild = \"npm run build\"\ntest = \"npm test\"".to_string()
            } else {
                format!("\n\nAvailable scripts: {}\n\nUse 'ppm run --list' to see all scripts with their commands.", available_scripts.join(", "))
            };
            
            return Err(PpmError::ValidationError(
                format!("Script '{}' not found.{}", script_name, suggestion)
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
                "No scripts defined in project.toml.\n\nTo add scripts, edit your project.toml file:\n[scripts]\nbuild = \"npm run build\"\ntest = \"npm test\"\nstart = \"node index.js\"\n\nThen run: ppm run <script-name>".to_string()
            ));
        }

        // Find the script
        let script_command = project.scripts.get(script_name).ok_or_else(|| {
            let available: Vec<_> = project.scripts.keys().cloned().collect();
            let suggestion = if available.is_empty() {
                "\n\nNo scripts available. Add some to your project.toml:\n[scripts]\nbuild = \"your-build-command\"".to_string()
            } else {
                format!("\n\nDid you mean one of these?\n  {}\n\nUse 'ppm run --list' to see all available scripts.", available.join("\n  "))
            };
            
            PpmError::ValidationError(
                format!("Script '{}' not found.{}", script_name, suggestion)
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
                format!("Script '{}' failed with exit code {}.\n\nCommand executed: {}\nArguments: {}\n\nCheck the output above for error details.", 
                    script_name, 
                    exit_code, 
                    script_command,
                    if self.args.is_empty() { "none".to_string() } else { self.args.join(" ") })
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
            
            let venv_path = current_dir.join(".venv");
            if venv_path.exists() {
                env_vars.insert(
                    "VIRTUAL_ENV".to_string(),
                    venv_path.to_string_lossy().to_string()
                );
                
                // Set PYTHONHOME to empty to avoid conflicts
                env_vars.insert("PYTHONHOME".to_string(), String::new());
                
                // Add virtual environment to PATH
                let venv_scripts = if cfg!(target_os = "windows") {
                    venv_path.join("Scripts")
                } else {
                    venv_path.join("bin")
                };
                
                if venv_scripts.exists() {
                    // Get current PATH and prepend venv scripts
                    let current_path = env::var("PATH").unwrap_or_default();
                    let new_path = format!("{}{}{}", 
                        venv_scripts.to_string_lossy(),
                        if cfg!(target_os = "windows") { ";" } else { ":" },
                        current_path
                    );
                    env_vars.insert("PATH".to_string(), new_path);
                }
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
