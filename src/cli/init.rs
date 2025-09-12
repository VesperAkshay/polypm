use clap::Args;
use std::fs;
use serde::{Deserialize, Serialize};
use crate::utils::error::{PpmError, Result};
use crate::utils::config::ConfigParser;

/// Initialize a new polyglot project with unified configuration
#[derive(Debug, Args)]
pub struct InitCommand {
    /// Project name (default: current directory name)
    #[arg(long)]
    pub name: Option<String>,
    
    /// Initial version (default: "1.0.0")
    #[arg(long)]
    pub version: Option<String>,
    
    /// Include JavaScript dependencies section only
    #[arg(long, conflicts_with = "python")]
    pub javascript: bool,
    
    /// Include Python dependencies section only
    #[arg(long, conflicts_with = "javascript")]
    pub python: bool,
    
    /// Overwrite existing project.toml
    #[arg(long)]
    pub force: bool,
    
    /// Output JSON instead of human-readable text
    #[arg(long)]
    pub json: bool,
}

/// JSON response format for init command
#[derive(Debug, Serialize, Deserialize)]
pub struct InitResponse {
    pub status: String,
    pub project_name: String,
    pub project_version: String,
    pub config_path: String,
    pub ecosystems: Vec<String>,
}

impl InitCommand {
    /// Execute the init command
    pub async fn run(&self) -> Result<()> {
        let current_dir = std::env::current_dir()
            .map_err(|e| PpmError::IoError(e))?;
        
        let config_path = current_dir.join("project.toml");
        
        // Check if project.toml already exists
        if config_path.exists() && !self.force {
            return Err(PpmError::ValidationError(
                "project.toml already exists (use --force to overwrite)".to_string()
            ));
        }
        
        // Determine project name
        let project_name = match &self.name {
            Some(name) => {
                validate_project_name(name)?;
                name.clone()
            }
            None => {
                let dir_name = current_dir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unnamed-project")
                    .to_string();
                
                // If directory name is not valid, use a default name
                if validate_project_name(&dir_name).is_ok() {
                    dir_name
                } else {
                    "unnamed-project".to_string()
                }
            }
        };
        
        // Determine project version
        let project_version = match &self.version {
            Some(version) => {
                validate_version(version)?;
                version.clone()
            }
            None => "1.0.0".to_string(),
        };
        
        // Determine which ecosystems to include
        let ecosystems = if self.javascript {
            vec!["javascript".to_string()]
        } else if self.python {
            vec!["python".to_string()]
        } else {
            vec!["javascript".to_string(), "python".to_string()]
        };
        
        // Generate project.toml content
        let toml_content = generate_project_toml(&project_name, &project_version, &ecosystems)?;
        
        // Write project.toml file
        fs::write(&config_path, toml_content)
            .map_err(|e| PpmError::IoError(e))?;
        
        // Output response
        if self.json {
            let response = InitResponse {
                status: "success".to_string(),
                project_name: project_name.clone(),
                project_version: project_version.clone(),
                config_path: "./project.toml".to_string(),
                ecosystems: ecosystems.clone(),
            };
            
            let json_output = serde_json::to_string_pretty(&response)
                .map_err(|e| PpmError::ValidationError(format!("Failed to serialize JSON response: {}", e)))?;
            
            println!("{}", json_output);
        } else {
            println!("Created project.toml for {} v{}", project_name, project_version);
        }
        
        Ok(())
    }
}

/// Validate project name according to our rules
fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(PpmError::ValidationError(
            "Invalid project name '' (must be valid identifier)".to_string()
        ));
    }
    
    // Check if name contains only valid characters (alphanumeric, hyphens, underscores)
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(PpmError::ValidationError(
            format!("Invalid project name '{}' (must be valid identifier)", name)
        ));
    }
    
    // Name should not start or end with hyphen/underscore
    if name.starts_with('-') || name.starts_with('_') || name.ends_with('-') || name.ends_with('_') {
        return Err(PpmError::ValidationError(
            format!("Invalid project name '{}' (must be valid identifier)", name)
        ));
    }
    
    Ok(())
}

/// Validate version according to semver rules (simplified)
fn validate_version(version: &str) -> Result<()> {
    // Basic semver validation: MAJOR.MINOR.PATCH
    let parts: Vec<&str> = version.split('.').collect();
    
    if parts.len() != 3 {
        return Err(PpmError::ValidationError(
            format!("Invalid version '{}' (must be valid semver)", version)
        ));
    }
    
    // Check that each part is a valid number
    for part in parts {
        if part.parse::<u32>().is_err() {
            return Err(PpmError::ValidationError(
                format!("Invalid version '{}' (must be valid semver)", version)
            ));
        }
    }
    
    Ok(())
}

/// Generate project.toml content
fn generate_project_toml(name: &str, version: &str, ecosystems: &[String]) -> Result<String> {
    let mut content = String::new();
    
    // Project section
    content.push_str("[project]\n");
    content.push_str(&format!("name = \"{}\"\n", name));
    content.push_str(&format!("version = \"{}\"\n", version));
    content.push_str("\n");
    
    // Dependencies section - only add if ecosystems are specified
    if !ecosystems.is_empty() {
        for ecosystem in ecosystems {
            match ecosystem.as_str() {
                "javascript" => {
                    content.push_str("[dependencies.javascript]\n");
                    content.push_str("# Add JavaScript dependencies here\n");
                    content.push_str("\n");
                }
                "python" => {
                    content.push_str("[dependencies.python]\n");
                    content.push_str("# Add Python dependencies here\n");
                    content.push_str("\n");
                }
                _ => {
                    return Err(PpmError::ValidationError(
                        format!("Unknown ecosystem: {}", ecosystem)
                    ));
                }
            }
        }
    }
    
    // Scripts section
    content.push_str("[scripts]\n");
    content.push_str("# Add project scripts here\n");
    content.push_str("# Example:\n");
    content.push_str("# dev = \"npm run dev && python app.py\"\n");
    content.push_str("# test = \"npm test && pytest\"\n");
    
    // Validate that generated TOML is parseable
    toml::from_str::<toml::Value>(&content)
        .map_err(|e| PpmError::ValidationError(format!("Generated invalid TOML: {}", e)))?;
    
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_validate_project_name_valid() {
        assert!(validate_project_name("my-app").is_ok());
        assert!(validate_project_name("my_app").is_ok());
        assert!(validate_project_name("myapp123").is_ok());
        assert!(validate_project_name("app-123_test").is_ok());
    }
    
    #[test]
    fn test_validate_project_name_invalid() {
        assert!(validate_project_name("").is_err());
        assert!(validate_project_name("my app").is_err());
        assert!(validate_project_name("my-app!").is_err());
        assert!(validate_project_name("my@app").is_err());
        assert!(validate_project_name("-myapp").is_err());
        assert!(validate_project_name("myapp-").is_err());
        assert!(validate_project_name("_myapp").is_err());
        assert!(validate_project_name("myapp_").is_err());
    }
    
    #[test]
    fn test_validate_version_valid() {
        assert!(validate_version("1.0.0").is_ok());
        assert!(validate_version("0.1.0").is_ok());
        assert!(validate_version("10.20.30").is_ok());
    }
    
    #[test]
    fn test_validate_version_invalid() {
        assert!(validate_version("1.0").is_err());
        assert!(validate_version("1.0.0.1").is_err());
        assert!(validate_version("1.x.0").is_err());
        assert!(validate_version("1.0.x").is_err());
        assert!(validate_version("v1.0.0").is_err());
        assert!(validate_version("1.0.0-alpha").is_err());
    }
    
    #[test]
    fn test_generate_project_toml_both_ecosystems() {
        let ecosystems = vec!["javascript".to_string(), "python".to_string()];
        let content = generate_project_toml("test-app", "1.0.0", &ecosystems).unwrap();
        
        assert!(content.contains("[project]"));
        assert!(content.contains("name = \"test-app\""));
        assert!(content.contains("version = \"1.0.0\""));
        assert!(content.contains("[dependencies.javascript]"));
        assert!(content.contains("[dependencies.python]"));
        assert!(content.contains("[scripts]"));
        
        // Verify it's valid TOML
        toml::from_str::<toml::Value>(&content).unwrap();
    }
    
    #[test]
    fn test_generate_project_toml_javascript_only() {
        let ecosystems = vec!["javascript".to_string()];
        let content = generate_project_toml("js-app", "0.1.0", &ecosystems).unwrap();
        
        assert!(content.contains("[dependencies.javascript]"));
        assert!(!content.contains("[dependencies.python]"));
        
        // Verify it's valid TOML
        toml::from_str::<toml::Value>(&content).unwrap();
    }
    
    #[test]
    fn test_generate_project_toml_python_only() {
        let ecosystems = vec!["python".to_string()];
        let content = generate_project_toml("py-app", "2.0.0", &ecosystems).unwrap();
        
        assert!(content.contains("[dependencies.python]"));
        assert!(!content.contains("[dependencies.javascript]"));
        
        // Verify it's valid TOML
        toml::from_str::<toml::Value>(&content).unwrap();
    }
    
    #[tokio::test]
    async fn test_init_command_basic() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        
        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let cmd = InitCommand {
            name: Some("test-app".to_string()),
            version: Some("1.0.0".to_string()),
            javascript: false,
            python: false,
            force: false,
            json: false,
        };
        
        let result = cmd.run().await;
        assert!(result.is_ok());
        
        // Verify file was created
        let config_path = temp_dir.path().join("project.toml");
        assert!(config_path.exists());
        
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("name = \"test-app\""));
        assert!(content.contains("version = \"1.0.0\""));
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
    
    #[tokio::test]
    async fn test_init_command_file_exists_without_force() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        
        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        // Create existing project.toml in the current working directory
        let config_path = std::env::current_dir().unwrap().join("project.toml");
        fs::write(&config_path, "existing content").unwrap();
        
        // Verify the file exists before running the command
        assert!(config_path.exists(), "project.toml should exist before test");
        
        let cmd = InitCommand {
            name: Some("test-app".to_string()),
            version: Some("1.0.0".to_string()),
            javascript: false,
            python: false,
            force: false,
            json: false,
        };
        
        let result = cmd.run().await;
        assert!(result.is_err(), "Expected error when project.toml already exists");
        
        if let Err(PpmError::ValidationError(msg)) = result {
            assert!(msg.contains("already exists"));
        } else {
            panic!("Expected ValidationError");
        }
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
    
    #[tokio::test]
    async fn test_init_command_json_output() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        
        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let cmd = InitCommand {
            name: Some("test-app".to_string()),
            version: Some("1.0.0".to_string()),
            javascript: false,
            python: false,
            force: false,
            json: true,
        };
        
        let result = cmd.run().await;
        assert!(result.is_ok());
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}
