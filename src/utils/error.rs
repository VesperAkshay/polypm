// Common error types for PPM

use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum PpmError {
    /// File system operations failed
    IoError(std::io::Error),
    /// Configuration parsing or validation failed
    ConfigError(String),
    /// Network operations failed
    NetworkError(String),
    /// Input validation failed
    ValidationError(String),
    /// Command execution failed
    ExecutionError(String),
    /// Symlink operations failed
    SymlinkError(String),
    /// Dependency resolution failed
    DependencyError(String),
    /// Package installation failed
    InstallationError(String),
    /// Registry operation failed
    RegistryError(String),
    /// Environment setup failed
    EnvironmentError(String),
}

impl fmt::Display for PpmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PpmError::IoError(err) => write!(f, "File system error: {}", err),
            PpmError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            PpmError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            PpmError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            PpmError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            PpmError::SymlinkError(msg) => write!(f, "Symlink error: {}", msg),
            PpmError::DependencyError(msg) => write!(f, "Dependency error: {}", msg),
            PpmError::InstallationError(msg) => write!(f, "Installation error: {}", msg),
            PpmError::RegistryError(msg) => write!(f, "Registry error: {}", msg),
            PpmError::EnvironmentError(msg) => write!(f, "Environment error: {}", msg),
        }
    }
}

impl Error for PpmError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PpmError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for PpmError {
    fn from(err: std::io::Error) -> Self {
        PpmError::IoError(err)
    }
}

impl From<serde_json::Error> for PpmError {
    fn from(err: serde_json::Error) -> Self {
        PpmError::ConfigError(format!("JSON parsing error: {}", err))
    }
}

impl From<toml::de::Error> for PpmError {
    fn from(err: toml::de::Error) -> Self {
        PpmError::ConfigError(format!("TOML parsing error: {}", err))
    }
}

pub type Result<T> = std::result::Result<T, PpmError>;

/// User-friendly error messages with context and suggestions
pub struct UserError {
    /// The main error message
    pub message: String,
    /// Additional context about what was happening
    pub context: Option<String>,
    /// Suggestions for how to fix the error
    pub suggestions: Vec<String>,
    /// Whether this error should exit with non-zero code
    pub exit_code: i32,
}

impl UserError {
    pub fn new(message: String) -> Self {
        Self {
            message,
            context: None,
            suggestions: Vec::new(),
            exit_code: 1,
        }
    }
    
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
    
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }
    
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }
    
    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = code;
        self
    }
    
    /// Convert a PpmError to a user-friendly error with suggestions
    pub fn from_ppm_error(err: &PpmError) -> Self {
        match err {
            PpmError::ConfigError(msg) if msg.contains("project.toml") => {
                UserError::new(format!("Configuration file error: {}", msg))
                    .with_context("Failed to read or parse project.toml".to_string())
                    .with_suggestions(vec![
                        "Check that project.toml exists in the current directory".to_string(),
                        "Verify the TOML syntax is correct".to_string(),
                        "Run 'ppm init' to create a new project.toml file".to_string(),
                    ])
            },
            PpmError::NetworkError(msg) => {
                UserError::new(format!("Network error: {}", msg))
                    .with_context("Failed to connect to package registry".to_string())
                    .with_suggestions(vec![
                        "Check your internet connection".to_string(),
                        "Verify that the registry is accessible".to_string(),
                        "Try again later if the registry is temporarily unavailable".to_string(),
                    ])
            },
            PpmError::DependencyError(msg) if msg.contains("not found") => {
                UserError::new(format!("Package not found: {}", msg))
                    .with_context("Could not resolve one or more dependencies".to_string())
                    .with_suggestions(vec![
                        "Check the package name spelling".to_string(),
                        "Verify the package exists in the registry".to_string(),
                        "Try using a different version specification".to_string(),
                    ])
            },
            PpmError::InstallationError(msg) => {
                UserError::new(format!("Installation failed: {}", msg))
                    .with_context("Could not install packages".to_string())
                    .with_suggestions(vec![
                        "Check that you have write permissions".to_string(),
                        "Verify you have enough disk space".to_string(),
                        "Try clearing the package cache".to_string(),
                    ])
            },
            PpmError::EnvironmentError(msg) => {
                UserError::new(format!("Environment error: {}", msg))
                    .with_context("Failed to set up development environment".to_string())
                    .with_suggestions(vec![
                        "Check that required tools are installed (Node.js, Python, etc.)".to_string(),
                        "Verify your PATH environment variable".to_string(),
                        "Try running with elevated permissions if needed".to_string(),
                    ])
            },
            PpmError::ValidationError(msg) => {
                UserError::new(format!("Invalid input: {}", msg))
                    .with_suggestions(vec![
                        "Check the command syntax and arguments".to_string(),
                        "Run 'ppm --help' for usage information".to_string(),
                    ])
            },
            _ => {
                UserError::new(format!("{}", err))
                    .with_context("An unexpected error occurred".to_string())
                    .with_suggestions(vec![
                        "Please report this issue if it persists".to_string(),
                    ])
            }
        }
    }
    
    /// Print the error in a user-friendly format
    pub fn print(&self) {
        eprintln!("❌ {}", self.message);
        
        if let Some(context) = &self.context {
            eprintln!("   {}", context);
        }
        
        if !self.suggestions.is_empty() {
            eprintln!();
            eprintln!("💡 Suggestions:");
            for suggestion in &self.suggestions {
                eprintln!("   • {}", suggestion);
            }
        }
    }
    
    /// Convert IO errors to more specific error types based on context
    pub fn from_io_error_with_context(err: std::io::Error, context: &str) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                if context.contains("project.toml") {
                    UserError::new("Project configuration not found".to_string())
                        .with_context("No project.toml file found in current directory".to_string())
                        .with_suggestions(vec![
                            "Run 'ppm init' to create a new project".to_string(),
                            "Navigate to an existing PPM project directory".to_string(),
                            "Check that you're in the correct directory".to_string(),
                        ])
                } else if context.contains("node_modules") || context.contains("package") {
                    UserError::new("Package files not found".to_string())
                        .with_context(format!("Cannot access package files: {}", context))
                        .with_suggestions(vec![
                            "Run 'ppm install' to install dependencies".to_string(),
                            "Check that the package is correctly installed".to_string(),
                        ])
                } else {
                    UserError::new(format!("File not found: {}", err))
                        .with_context(context.to_string())
                }
            },
            std::io::ErrorKind::PermissionDenied => {
                UserError::new("Permission denied".to_string())
                    .with_context(format!("Cannot access: {}", context))
                    .with_suggestions(vec![
                        "Check file/directory permissions".to_string(),
                        "Try running with elevated permissions".to_string(),
                        "Ensure you have write access to the project directory".to_string(),
                    ])
            },
            std::io::ErrorKind::AlreadyExists => {
                UserError::new("File already exists".to_string())
                    .with_context(format!("Cannot create: {}", context))
                    .with_suggestions(vec![
                        "Use --force flag to overwrite existing files".to_string(),
                        "Choose a different name or location".to_string(),
                    ])
            },
            _ => {
                UserError::new(format!("File system error: {}", err))
                    .with_context(context.to_string())
                    .with_suggestions(vec![
                        "Check file system permissions".to_string(),
                        "Verify disk space is available".to_string(),
                    ])
            }
        }
    }
    
    /// Detect common network errors and provide helpful suggestions
    pub fn from_network_error(msg: &str, operation: &str) -> Self {
        if msg.contains("timeout") || msg.contains("timed out") {
            UserError::new("Network request timed out".to_string())
                .with_context(format!("Failed to {}", operation))
                .with_suggestions(vec![
                    "Check your internet connection".to_string(),
                    "Try again later - the server may be overloaded".to_string(),
                    "Configure a longer timeout if on a slow connection".to_string(),
                ])
        } else if msg.contains("DNS") || msg.contains("name resolution") {
            UserError::new("Cannot resolve server address".to_string())
                .with_context(format!("DNS lookup failed while trying to {}", operation))
                .with_suggestions(vec![
                    "Check your internet connection".to_string(),
                    "Verify DNS settings".to_string(),
                    "Try using a different DNS server".to_string(),
                ])
        } else if msg.contains("refused") || msg.contains("unreachable") {
            UserError::new("Cannot connect to server".to_string())
                .with_context(format!("Connection refused while trying to {}", operation))
                .with_suggestions(vec![
                    "Check if the server is running and accessible".to_string(),
                    "Verify firewall settings".to_string(),
                    "Try using a VPN if the server is blocked".to_string(),
                ])
        } else if msg.contains("certificate") || msg.contains("SSL") || msg.contains("TLS") {
            UserError::new("SSL/TLS connection error".to_string())
                .with_context(format!("Certificate verification failed while trying to {}", operation))
                .with_suggestions(vec![
                    "Check your system clock is correct".to_string(),
                    "Update your system certificates".to_string(),
                    "Contact your network administrator if on corporate network".to_string(),
                ])
        } else {
            UserError::new(format!("Network error: {}", msg))
                .with_context(format!("Failed to {}", operation))
                .with_suggestions(vec![
                    "Check your internet connection".to_string(),
                    "Try again later".to_string(),
                ])
        }
    }
}
