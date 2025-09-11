use serde::{Deserialize, Serialize};
use std::fmt;

/// Enumeration of supported package ecosystems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ecosystem {
    /// JavaScript ecosystem (npm registry)
    JavaScript,
    /// Python ecosystem (PyPI registry)
    Python,
}

impl Ecosystem {
    /// Returns the registry URL for this ecosystem
    pub fn registry_url(&self) -> &'static str {
        match self {
            Ecosystem::JavaScript => "https://registry.npmjs.org",
            Ecosystem::Python => "https://pypi.org/simple",
        }
    }

    /// Returns the package format for this ecosystem
    pub fn package_format(&self) -> PackageFormat {
        match self {
            Ecosystem::JavaScript => PackageFormat::Tarball,
            Ecosystem::Python => PackageFormat::Wheel,
        }
    }

    /// Returns the file extension for packages in this ecosystem
    pub fn package_extension(&self) -> &'static str {
        match self {
            Ecosystem::JavaScript => ".tgz",
            Ecosystem::Python => ".whl",
        }
    }

    /// Returns the package manager command for this ecosystem
    pub fn package_manager(&self) -> &'static str {
        match self {
            Ecosystem::JavaScript => "npm",
            Ecosystem::Python => "pip",
        }
    }

    /// Validates a package name for this ecosystem
    pub fn validate_package_name(&self, name: &str) -> Result<(), EcosystemError> {
        if name.is_empty() {
            return Err(EcosystemError::InvalidPackageName(
                "Package name cannot be empty".to_string(),
            ));
        }

        match self {
            Ecosystem::JavaScript => {
                // npm package name validation
                if name.starts_with('.') || name.starts_with('_') {
                    return Err(EcosystemError::InvalidPackageName(
                        "npm package names cannot start with . or _".to_string(),
                    ));
                }
                if name.len() > 214 {
                    return Err(EcosystemError::InvalidPackageName(
                        "npm package names must be 214 characters or less".to_string(),
                    ));
                }
                if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '/' || c == '@') {
                    return Err(EcosystemError::InvalidPackageName(
                        "npm package names can only contain lowercase letters, digits, hyphens, slashes, and @".to_string(),
                    ));
                }
            }
            Ecosystem::Python => {
                // PyPI package name validation (PEP 508)
                if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') {
                    return Err(EcosystemError::InvalidPackageName(
                        "Python package names can only contain letters, digits, hyphens, underscores, and periods".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validates a version specification for this ecosystem
    pub fn validate_version_spec(&self, version_spec: &str) -> Result<(), EcosystemError> {
        if version_spec.is_empty() {
            return Err(EcosystemError::InvalidVersionSpec(
                "Version specification cannot be empty".to_string(),
            ));
        }

        match self {
            Ecosystem::JavaScript => {
                // Basic npm semver validation (simplified)
                if !version_spec.chars().any(|c| c.is_ascii_digit()) {
                    return Err(EcosystemError::InvalidVersionSpec(
                        "npm version spec must contain at least one digit".to_string(),
                    ));
                }
            }
            Ecosystem::Python => {
                // Basic PEP 440 validation (simplified)
                if !version_spec.chars().any(|c| c.is_ascii_digit()) {
                    return Err(EcosystemError::InvalidVersionSpec(
                        "Python version spec must contain at least one digit".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Returns all supported ecosystems
    pub fn all() -> &'static [Ecosystem] {
        &[Ecosystem::JavaScript, Ecosystem::Python]
    }
}

impl fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ecosystem::JavaScript => write!(f, "javascript"),
            Ecosystem::Python => write!(f, "python"),
        }
    }
}

impl std::str::FromStr for Ecosystem {
    type Err = EcosystemError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "javascript" | "js" | "npm" | "node" => Ok(Ecosystem::JavaScript),
            "python" | "py" | "pypi" | "pip" => Ok(Ecosystem::Python),
            _ => Err(EcosystemError::UnknownEcosystem(s.to_string())),
        }
    }
}

/// Package distribution format per ecosystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageFormat {
    /// JavaScript .tgz tarball files
    Tarball,
    /// Python .whl wheel files
    Wheel,
    /// Python source distributions
    Source,
}

impl PackageFormat {
    /// Returns the file extension for this package format
    pub fn extension(&self) -> &'static str {
        match self {
            PackageFormat::Tarball => ".tgz",
            PackageFormat::Wheel => ".whl",
            PackageFormat::Source => ".tar.gz",
        }
    }

    /// Returns the MIME type for this package format
    pub fn mime_type(&self) -> &'static str {
        match self {
            PackageFormat::Tarball => "application/gzip",
            PackageFormat::Wheel => "application/zip",
            PackageFormat::Source => "application/gzip",
        }
    }
}

impl fmt::Display for PackageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageFormat::Tarball => write!(f, "tarball"),
            PackageFormat::Wheel => write!(f, "wheel"),
            PackageFormat::Source => write!(f, "source"),
        }
    }
}

/// Trait for ecosystem-specific version parsing and validation
pub trait VersionParser {
    /// Parse a version string into components
    fn parse_version(&self, version: &str) -> Result<ParsedVersion, EcosystemError>;
    
    /// Check if a version satisfies a version specification
    fn satisfies(&self, version: &str, spec: &str) -> Result<bool, EcosystemError>;
    
    /// Compare two versions (-1: v1 < v2, 0: v1 == v2, 1: v1 > v2)
    fn compare_versions(&self, v1: &str, v2: &str) -> Result<i8, EcosystemError>;
}

/// Parsed version components
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
    pub build: Option<String>,
}

/// JavaScript/npm version parser implementing semver
pub struct JavaScriptVersionParser;

impl VersionParser for JavaScriptVersionParser {
    fn parse_version(&self, version: &str) -> Result<ParsedVersion, EcosystemError> {
        // Simplified semver parsing - would use a proper semver crate in production
        let clean_version = version.trim_start_matches('v');
        let parts: Vec<&str> = clean_version.split('.').collect();
        
        if parts.len() < 3 {
            return Err(EcosystemError::InvalidVersion(format!(
                "Invalid semver format: {}", version
            )));
        }

        let major = parts[0].parse().map_err(|_| {
            EcosystemError::InvalidVersion(format!("Invalid major version: {}", parts[0]))
        })?;
        
        let minor = parts[1].parse().map_err(|_| {
            EcosystemError::InvalidVersion(format!("Invalid minor version: {}", parts[1]))
        })?;
        
        let patch = parts[2].parse().map_err(|_| {
            EcosystemError::InvalidVersion(format!("Invalid patch version: {}", parts[2]))
        })?;

        Ok(ParsedVersion {
            major,
            minor,
            patch,
            pre_release: None,
            build: None,
        })
    }

    fn satisfies(&self, version: &str, spec: &str) -> Result<bool, EcosystemError> {
        // Simplified - would implement full semver range checking
        Ok(version == spec || spec == "*" || spec == "latest")
    }

    fn compare_versions(&self, v1: &str, v2: &str) -> Result<i8, EcosystemError> {
        let parsed_v1 = self.parse_version(v1)?;
        let parsed_v2 = self.parse_version(v2)?;

        if parsed_v1.major != parsed_v2.major {
            return Ok(if parsed_v1.major > parsed_v2.major { 1 } else { -1 });
        }
        if parsed_v1.minor != parsed_v2.minor {
            return Ok(if parsed_v1.minor > parsed_v2.minor { 1 } else { -1 });
        }
        if parsed_v1.patch != parsed_v2.patch {
            return Ok(if parsed_v1.patch > parsed_v2.patch { 1 } else { -1 });
        }

        Ok(0)
    }
}

/// Python version parser implementing PEP 440
pub struct PythonVersionParser;

impl VersionParser for PythonVersionParser {
    fn parse_version(&self, version: &str) -> Result<ParsedVersion, EcosystemError> {
        // Simplified PEP 440 parsing - would use a proper PEP 440 crate in production
        let parts: Vec<&str> = version.split('.').collect();
        
        if parts.is_empty() {
            return Err(EcosystemError::InvalidVersion(format!(
                "Invalid PEP 440 format: {}", version
            )));
        }

        let major = parts[0].parse().map_err(|_| {
            EcosystemError::InvalidVersion(format!("Invalid major version: {}", parts[0]))
        })?;
        
        let minor = if parts.len() > 1 {
            parts[1].parse().map_err(|_| {
                EcosystemError::InvalidVersion(format!("Invalid minor version: {}", parts[1]))
            })?
        } else {
            0
        };
        
        let patch = if parts.len() > 2 {
            parts[2].parse().map_err(|_| {
                EcosystemError::InvalidVersion(format!("Invalid patch version: {}", parts[2]))
            })?
        } else {
            0
        };

        Ok(ParsedVersion {
            major,
            minor,
            patch,
            pre_release: None,
            build: None,
        })
    }

    fn satisfies(&self, version: &str, spec: &str) -> Result<bool, EcosystemError> {
        // Simplified - would implement full PEP 440 version matching
        Ok(version == spec || spec == "*")
    }

    fn compare_versions(&self, v1: &str, v2: &str) -> Result<i8, EcosystemError> {
        let parsed_v1 = self.parse_version(v1)?;
        let parsed_v2 = self.parse_version(v2)?;

        if parsed_v1.major != parsed_v2.major {
            return Ok(if parsed_v1.major > parsed_v2.major { 1 } else { -1 });
        }
        if parsed_v1.minor != parsed_v2.minor {
            return Ok(if parsed_v1.minor > parsed_v2.minor { 1 } else { -1 });
        }
        if parsed_v1.patch != parsed_v2.patch {
            return Ok(if parsed_v1.patch > parsed_v2.patch { 1 } else { -1 });
        }

        Ok(0)
    }
}

impl Ecosystem {
    /// Returns a version parser for this ecosystem
    pub fn version_parser(&self) -> Box<dyn VersionParser> {
        match self {
            Ecosystem::JavaScript => Box::new(JavaScriptVersionParser),
            Ecosystem::Python => Box::new(PythonVersionParser),
        }
    }
}

/// Errors that can occur when working with ecosystems
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EcosystemError {
    /// Unknown ecosystem name
    UnknownEcosystem(String),
    /// Invalid package name for ecosystem
    InvalidPackageName(String),
    /// Invalid version specification
    InvalidVersionSpec(String),
    /// Invalid version format
    InvalidVersion(String),
    /// Version parsing error
    VersionParseError(String),
}

impl fmt::Display for EcosystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EcosystemError::UnknownEcosystem(name) => {
                write!(f, "Unknown ecosystem: {}", name)
            }
            EcosystemError::InvalidPackageName(msg) => {
                write!(f, "Invalid package name: {}", msg)
            }
            EcosystemError::InvalidVersionSpec(msg) => {
                write!(f, "Invalid version specification: {}", msg)
            }
            EcosystemError::InvalidVersion(msg) => {
                write!(f, "Invalid version: {}", msg)
            }
            EcosystemError::VersionParseError(msg) => {
                write!(f, "Version parse error: {}", msg)
            }
        }
    }
}

impl std::error::Error for EcosystemError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecosystem_display() {
        assert_eq!(Ecosystem::JavaScript.to_string(), "javascript");
        assert_eq!(Ecosystem::Python.to_string(), "python");
    }

    #[test]
    fn test_ecosystem_from_str() {
        assert_eq!("javascript".parse::<Ecosystem>().unwrap(), Ecosystem::JavaScript);
        assert_eq!("js".parse::<Ecosystem>().unwrap(), Ecosystem::JavaScript);
        assert_eq!("npm".parse::<Ecosystem>().unwrap(), Ecosystem::JavaScript);
        assert_eq!("node".parse::<Ecosystem>().unwrap(), Ecosystem::JavaScript);
        
        assert_eq!("python".parse::<Ecosystem>().unwrap(), Ecosystem::Python);
        assert_eq!("py".parse::<Ecosystem>().unwrap(), Ecosystem::Python);
        assert_eq!("pypi".parse::<Ecosystem>().unwrap(), Ecosystem::Python);
        assert_eq!("pip".parse::<Ecosystem>().unwrap(), Ecosystem::Python);

        assert!("unknown".parse::<Ecosystem>().is_err());
    }

    #[test]
    fn test_registry_urls() {
        assert_eq!(Ecosystem::JavaScript.registry_url(), "https://registry.npmjs.org");
        assert_eq!(Ecosystem::Python.registry_url(), "https://pypi.org/simple");
    }

    #[test]
    fn test_package_formats() {
        assert_eq!(Ecosystem::JavaScript.package_format(), PackageFormat::Tarball);
        assert_eq!(Ecosystem::Python.package_format(), PackageFormat::Wheel);
    }

    #[test]
    fn test_package_name_validation() {
        // JavaScript validation
        assert!(Ecosystem::JavaScript.validate_package_name("react").is_ok());
        assert!(Ecosystem::JavaScript.validate_package_name("@types/node").is_ok());
        assert!(Ecosystem::JavaScript.validate_package_name(".private").is_err());
        assert!(Ecosystem::JavaScript.validate_package_name("_internal").is_err());

        // Python validation
        assert!(Ecosystem::Python.validate_package_name("flask").is_ok());
        assert!(Ecosystem::Python.validate_package_name("django-rest-framework").is_ok());
        assert!(Ecosystem::Python.validate_package_name("invalid@name").is_err());
    }

    #[test]
    fn test_version_parsing() {
        let js_parser = JavaScriptVersionParser;
        let version = js_parser.parse_version("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);

        let py_parser = PythonVersionParser;
        let version = py_parser.parse_version("2.1.0").unwrap();
        assert_eq!(version.major, 2);
        assert_eq!(version.minor, 1);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_version_comparison() {
        let js_parser = JavaScriptVersionParser;
        assert_eq!(js_parser.compare_versions("1.0.0", "1.0.0").unwrap(), 0);
        assert_eq!(js_parser.compare_versions("1.0.1", "1.0.0").unwrap(), 1);
        assert_eq!(js_parser.compare_versions("1.0.0", "1.0.1").unwrap(), -1);
    }

    #[test]
    fn test_all_ecosystems() {
        let all = Ecosystem::all();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&Ecosystem::JavaScript));
        assert!(all.contains(&Ecosystem::Python));
    }
}
