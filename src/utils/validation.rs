// Common validation utilities for PPM CLI commands

use crate::utils::error::{PpmError, Result};
use std::path::Path;

/// Validate package name format for different ecosystems
pub fn validate_package_name(name: &str, ecosystem: Option<&str>) -> Result<()> {
    if name.is_empty() {
        return Err(PpmError::ValidationError(
            "Package name cannot be empty.\n\nProvide a valid package name:\n  ppm add express\n  ppm add @types/node".to_string()
        ));
    }

    // Check for common invalid characters
    if name.contains(' ') {
        return Err(PpmError::ValidationError(
            format!("Invalid package name '{}' - cannot contain spaces.\n\nValid package names:\n  ✓ express\n  ✓ @types/node\n  ✗ my package", name)
        ));
    }

    // Ecosystem-specific validation
    match ecosystem {
        Some("javascript") | Some("npm") => validate_npm_package_name(name),
        Some("python") | Some("pypi") => validate_python_package_name(name),
        _ => Ok(()), // Generic validation passed
    }
}

/// Validate npm package name according to npm naming rules
fn validate_npm_package_name(name: &str) -> Result<()> {
    // Check length
    if name.len() > 214 {
        return Err(PpmError::ValidationError(
            format!("NPM package name '{}' is too long (max 214 characters).", name)
        ));
    }

    // Check for uppercase letters (npm names are lowercase)
    if name != name.to_lowercase() {
        return Err(PpmError::ValidationError(
            format!("NPM package name '{}' contains uppercase letters.\n\nNPM package names must be lowercase:\n  ✓ express\n  ✓ @types/node\n  ✗ Express", name)
        ));
    }

    // Check for leading dots or underscores
    if name.starts_with('.') || name.starts_with('_') {
        return Err(PpmError::ValidationError(
            format!("NPM package name '{}' cannot start with '.' or '_'.", name)
        ));
    }

    Ok(())
}

/// Validate Python package name according to PEP 508
fn validate_python_package_name(name: &str) -> Result<()> {
    // Python package names are more flexible than npm
    // Check for basic validity
    if name.contains(' ') || name.contains('\t') {
        return Err(PpmError::ValidationError(
            format!("Python package name '{}' cannot contain whitespace.\n\nValid Python package names:\n  ✓ requests\n  ✓ Django\n  ✓ beautifulsoup4", name)
        ));
    }

    Ok(())
}

/// Validate version specifier
pub fn validate_version_spec(version: &str) -> Result<()> {
    if version.is_empty() {
        return Err(PpmError::ValidationError(
            "Version specifier cannot be empty.\n\nValid version formats:\n  ✓ 1.0.0\n  ✓ ^1.0.0\n  ✓ ~1.0.0\n  ✓ >=1.0.0\n  ✓ latest".to_string()
        ));
    }

    // Check for obviously invalid characters
    if version.contains(' ') && version != "latest" {
        return Err(PpmError::ValidationError(
            format!("Invalid version specifier '{}' - cannot contain spaces.\n\nValid version formats:\n  ✓ 1.0.0\n  ✓ ^1.0.0\n  ✓ >=1.0.0", version)
        ));
    }

    Ok(())
}

/// Validate script name for ppm run command
pub fn validate_script_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(PpmError::ValidationError(
            "Script name cannot be empty.\n\nUsage: ppm run <script-name>\nExample: ppm run build".to_string()
        ));
    }

    // Check for path traversal attempts
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(PpmError::ValidationError(
            format!("Invalid script name '{}' - cannot contain path separators or '..'.\n\nScript names should be simple identifiers:\n  ✓ build\n  ✓ test\n  ✓ start", name)
        ));
    }

    Ok(())
}

/// Validate project path exists and is readable
pub fn validate_project_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(PpmError::ConfigError(
            format!("Project file '{}' not found.\n\nTo initialize a new PPM project:\n  ppm init\n\nOr navigate to an existing PPM project directory.", path.display())
        ));
    }

    if !path.is_file() {
        return Err(PpmError::ConfigError(
            format!("'{}' is not a file.\n\nExpected a project.toml file in the current directory.", path.display())
        ));
    }

    Ok(())
}

/// Validate network connectivity for package operations
pub fn validate_network_available() -> Result<()> {
    // This is a placeholder for network validation
    // In a real implementation, you might ping registries or check connectivity
    // For now, we'll assume network is available
    Ok(())
}

/// Validate disk space for installation operations
pub fn validate_disk_space_available(required_mb: u64) -> Result<()> {
    // This is a placeholder for disk space validation
    // In a real implementation, you would check available disk space
    // For now, we'll assume sufficient space is available
    if required_mb > 10000 {
        return Err(PpmError::EnvironmentError(
            format!("Operation requires {} MB of disk space. Please ensure sufficient space is available.", required_mb)
        ));
    }
    Ok(())
}

/// Validate that required tools are available (python, node, etc.)
pub fn validate_ecosystem_tools(ecosystem: &str) -> Result<()> {
    match ecosystem {
        "javascript" | "npm" => {
            // In a real implementation, check if node/npm is available
            // For now, assume tools are available
            Ok(())
        }
        "python" | "pypi" => {
            // In a real implementation, check if python/pip is available
            // For now, assume tools are available
            Ok(())
        }
        _ => Err(PpmError::ValidationError(
            format!("Unknown ecosystem '{}'. Supported ecosystems: javascript, python", ecosystem)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_package_name() {
        // Valid names
        assert!(validate_package_name("express", None).is_ok());
        assert!(validate_package_name("@types/node", Some("javascript")).is_ok());
        
        // Invalid names
        assert!(validate_package_name("", None).is_err());
        assert!(validate_package_name("my package", None).is_err());
        assert!(validate_package_name("Express", Some("javascript")).is_err());
    }

    #[test]
    fn test_validate_version_spec() {
        // Valid versions
        assert!(validate_version_spec("1.0.0").is_ok());
        assert!(validate_version_spec("^1.0.0").is_ok());
        assert!(validate_version_spec("latest").is_ok());
        
        // Invalid versions
        assert!(validate_version_spec("").is_err());
        assert!(validate_version_spec("1 0 0").is_err());
    }

    #[test]
    fn test_validate_script_name() {
        // Valid script names
        assert!(validate_script_name("build").is_ok());
        assert!(validate_script_name("test").is_ok());
        
        // Invalid script names
        assert!(validate_script_name("").is_err());
        assert!(validate_script_name("../malicious").is_err());
        assert!(validate_script_name("script/name").is_err());
    }
}
