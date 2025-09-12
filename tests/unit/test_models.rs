use ppm::models::{
    dependency::Dependency,
    ecosystem::{Ecosystem, PackageFormat},
    global_store::GlobalStore,
    lock_file::LockFile,
    package::{Package, PackageMetadata},
    project::Project,
    resolved_dependency::ResolvedDependency,
    symlink_structure::SymlinkStructure,
    virtual_environment::{VirtualEnvironment, VenvConfig},
};
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(test)]
mod ecosystem_tests {
    use super::*;

    #[test]
    fn test_ecosystem_registry_urls() {
        assert_eq!(
            Ecosystem::JavaScript.registry_url(),
            "https://registry.npmjs.org"
        );
        assert_eq!(
            Ecosystem::Python.registry_url(),
            "https://pypi.org/simple"
        );
    }

    #[test]
    fn test_ecosystem_package_formats() {
        assert_eq!(
            Ecosystem::JavaScript.package_format(),
            PackageFormat::Tarball
        );
        assert_eq!(
            Ecosystem::Python.package_format(),
            PackageFormat::Wheel
        );
    }

    #[test]
    fn test_ecosystem_package_extensions() {
        assert_eq!(Ecosystem::JavaScript.package_extension(), ".tgz");
        assert_eq!(Ecosystem::Python.package_extension(), ".whl");
    }

    #[test]
    fn test_ecosystem_package_managers() {
        assert_eq!(Ecosystem::JavaScript.package_manager(), "npm");
        assert_eq!(Ecosystem::Python.package_manager(), "pip");
    }

    #[test]
    fn test_ecosystem_validate_package_name_valid() {
        // Valid JavaScript package names
        assert!(Ecosystem::JavaScript.validate_package_name("lodash").is_ok());
        assert!(Ecosystem::JavaScript.validate_package_name("@types/node").is_ok());
        assert!(Ecosystem::JavaScript.validate_package_name("react-dom").is_ok());

        // Valid Python package names
        assert!(Ecosystem::Python.validate_package_name("requests").is_ok());
        assert!(Ecosystem::Python.validate_package_name("numpy").is_ok());
        assert!(Ecosystem::Python.validate_package_name("django-rest-framework").is_ok());
    }

    #[test]
    fn test_ecosystem_validate_package_name_invalid() {
        // Empty names should be invalid
        assert!(Ecosystem::JavaScript.validate_package_name("").is_err());
        assert!(Ecosystem::Python.validate_package_name("").is_err());
    }

    #[test]
    fn test_ecosystem_serialization() {
        let js = Ecosystem::JavaScript;
        let py = Ecosystem::Python;

        let js_json = serde_json::to_string(&js).unwrap();
        let py_json = serde_json::to_string(&py).unwrap();

        assert_eq!(js_json, "\"javascript\"");
        assert_eq!(py_json, "\"python\"");

        let js_deserialized: Ecosystem = serde_json::from_str(&js_json).unwrap();
        let py_deserialized: Ecosystem = serde_json::from_str(&py_json).unwrap();

        assert_eq!(js, js_deserialized);
        assert_eq!(py, py_deserialized);
    }
}

#[cfg(test)]
mod package_tests {
    use super::*;

    fn create_test_package() -> Package {
        // Use a valid SHA-256 hash (64 hex characters)
        let valid_hash = "a".repeat(64);
        Package::new(
            "test-package".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            valid_hash,
            PathBuf::from("/test/store/package"),
        )
    }

    #[test]
    fn test_package_creation() {
        let package = create_test_package();
        
        assert_eq!(package.name, "test-package");
        assert_eq!(package.version, "1.0.0");
        assert_eq!(package.ecosystem, Ecosystem::JavaScript);
        assert_eq!(package.hash, "a".repeat(64));
        assert_eq!(package.store_path, PathBuf::from("/test/store/package"));
    }

    #[test]
    fn test_package_metadata_default() {
        let metadata = PackageMetadata::default();
        
        assert!(metadata.description.is_none());
        assert!(metadata.author.is_none());
        assert!(metadata.license.is_none());
        assert!(metadata.homepage.is_none());
        assert!(metadata.repository.is_none());
        assert!(metadata.keywords.is_empty());
        assert!(metadata.extra.is_empty());
    }

    #[test]
    fn test_package_with_metadata() {
        let mut metadata = PackageMetadata::default();
        metadata.description = Some("A test package".to_string());
        metadata.author = Some("Test Author".to_string());
        metadata.keywords = vec!["test".to_string(), "package".to_string()];

        let package = Package::with_metadata(
            "test-package".to_string(),
            "1.0.0".to_string(),
            Ecosystem::JavaScript,
            "b".repeat(64),
            metadata.clone(),
            PathBuf::from("/test/store/package"),
        );

        assert_eq!(package.metadata.description, Some("A test package".to_string()));
        assert_eq!(package.metadata.author, Some("Test Author".to_string()));
        assert_eq!(package.metadata.keywords, vec!["test", "package"]);
    }

    #[test]
    fn test_package_serialization() {
        let package = create_test_package();
        let json = serde_json::to_string(&package).unwrap();
        let deserialized: Package = serde_json::from_str(&json).unwrap();
        
        assert_eq!(package, deserialized);
    }

    #[test]
    fn test_package_equality() {
        let package1 = create_test_package();
        let package2 = create_test_package();
        let package3 = Package::new(
            "test-package".to_string(),
            "2.0.0".to_string(),
            Ecosystem::JavaScript,
            "b".repeat(64),
            PathBuf::from("/test/store/package"),
        );

        assert_eq!(package1, package2);
        assert_ne!(package1, package3);
    }
}

#[cfg(test)]
mod dependency_tests {
    use super::*;

    #[test]
    fn test_dependency_new() {
        let dep = Dependency::new(
            "lodash".to_string(),
            "^4.17.0".to_string(),
            Ecosystem::JavaScript,
            false,
        );

        assert_eq!(dep.name, "lodash");
        assert_eq!(dep.version_spec, "^4.17.0");
        assert_eq!(dep.ecosystem, Ecosystem::JavaScript);
        assert!(!dep.dev_only);
        assert!(dep.resolved_version.is_none());
    }

    #[test]
    fn test_dependency_production() {
        let dep = Dependency::production(
            "react".to_string(),
            "^18.0.0".to_string(),
            Ecosystem::JavaScript,
        );

        assert_eq!(dep.name, "react");
        assert_eq!(dep.version_spec, "^18.0.0");
        assert_eq!(dep.ecosystem, Ecosystem::JavaScript);
        assert!(!dep.dev_only);
    }

    #[test]
    fn test_dependency_development() {
        let dep = Dependency::development(
            "jest".to_string(),
            "^29.0.0".to_string(),
            Ecosystem::JavaScript,
        );

        assert_eq!(dep.name, "jest");
        assert_eq!(dep.version_spec, "^29.0.0");
        assert_eq!(dep.ecosystem, Ecosystem::JavaScript);
        assert!(dep.dev_only);
    }

    #[test]
    fn test_dependency_with_resolved_version() {
        let dep = Dependency::with_resolved_version(
            "axios".to_string(),
            "^1.0.0".to_string(),
            "1.6.2".to_string(),
            Ecosystem::JavaScript,
            false,
        );

        assert_eq!(dep.name, "axios");
        assert_eq!(dep.version_spec, "^1.0.0");
        assert_eq!(dep.resolved_version, Some("1.6.2".to_string()));
        assert_eq!(dep.ecosystem, Ecosystem::JavaScript);
        assert!(!dep.dev_only);
    }

    #[test]
    fn test_dependency_equality() {
        let dep1 = Dependency::production(
            "lodash".to_string(),
            "^4.17.0".to_string(),
            Ecosystem::JavaScript,
        );
        let dep2 = Dependency::production(
            "lodash".to_string(),
            "^4.17.0".to_string(),
            Ecosystem::JavaScript,
        );
        let dep3 = Dependency::development(
            "lodash".to_string(),
            "^4.17.0".to_string(),
            Ecosystem::JavaScript,
        );

        assert_eq!(dep1, dep2);
        assert_ne!(dep1, dep3);
    }

    #[test]
    fn test_dependency_serialization() {
        let dep = Dependency::production(
            "react".to_string(),
            "^18.0.0".to_string(),
            Ecosystem::JavaScript,
        );

        let json = serde_json::to_string(&dep).unwrap();
        let deserialized: Dependency = serde_json::from_str(&json).unwrap();
        
        assert_eq!(dep, deserialized);
    }
}

#[cfg(test)]
mod resolved_dependency_tests {
    use super::*;

    #[test]
    fn test_resolved_dependency_creation() {
        let valid_hash = "a".repeat(64);
        let resolved = ResolvedDependency::new(
            "lodash".to_string(),
            "4.17.21".to_string(),
            Ecosystem::JavaScript,
            valid_hash.clone(),
            "integrity123".to_string(),
            "/store/path".to_string(),
        );

        assert_eq!(resolved.name, "lodash");
        assert_eq!(resolved.version, "4.17.21");
        assert_eq!(resolved.ecosystem, Ecosystem::JavaScript);
        assert_eq!(resolved.hash, valid_hash);
        assert_eq!(resolved.integrity, "integrity123");
        assert_eq!(resolved.store_path, "/store/path");
    }

    #[test]
    fn test_resolved_dependency_with_hash_integrity() {
        let valid_hash = "b".repeat(64);
        let resolved = ResolvedDependency::with_hash_integrity(
            "react".to_string(),
            "18.2.0".to_string(),
            Ecosystem::JavaScript,
            valid_hash.clone(),
            "/store/path".to_string(),
        );

        assert_eq!(resolved.name, "react");
        assert_eq!(resolved.version, "18.2.0");
        assert_eq!(resolved.ecosystem, Ecosystem::JavaScript);
        assert_eq!(resolved.hash, valid_hash.clone());
        assert_eq!(resolved.integrity, valid_hash); // Should be same as hash
        assert_eq!(resolved.store_path, "/store/path");
    }

    #[test]
    fn test_resolved_dependency_serialization() {
        let valid_hash = "c".repeat(64);
        let resolved = ResolvedDependency::new(
            "axios".to_string(),
            "1.6.2".to_string(),
            Ecosystem::JavaScript,
            valid_hash,
            "integrity789".to_string(),
            "/store/path".to_string(),
        );

        let json = serde_json::to_string(&resolved).unwrap();
        let deserialized: ResolvedDependency = serde_json::from_str(&json).unwrap();
        
        assert_eq!(resolved, deserialized);
    }
}

#[cfg(test)]
mod project_tests {
    use super::*;

    fn create_test_project() -> Project {
        let mut project = Project::new("test-project".to_string(), "1.0.0".to_string());
        project.ecosystems = vec![Ecosystem::JavaScript, Ecosystem::Python];
        project
    }

    #[test]
    fn test_project_creation() {
        let project = create_test_project();
        
        assert_eq!(project.name, "test-project");
        assert_eq!(project.version, "1.0.0");
        assert_eq!(project.ecosystems, vec![Ecosystem::JavaScript, Ecosystem::Python]);
    }

    #[test]
    fn test_project_with_both_ecosystems() {
        let mut project = Project::with_both_ecosystems("test-project".to_string(), "1.0.0".to_string());
        
        // Add dependencies to make the ecosystems show up in get_ecosystems()
        project.add_dependency(Ecosystem::JavaScript, "lodash".to_string(), "^4.17.0".to_string());
        project.add_dependency(Ecosystem::Python, "requests".to_string(), "^2.28.0".to_string());
        
        assert_eq!(project.name, "test-project");
        assert_eq!(project.version, "1.0.0");
        let ecosystems = project.get_ecosystems();
        assert!(ecosystems.contains(&Ecosystem::JavaScript));
        assert!(ecosystems.contains(&Ecosystem::Python));
        assert!(project.venv_config.is_some());
    }

    #[test]
    fn test_project_add_dependency() {
        let mut project = create_test_project();
        
        project.add_dependency(
            Ecosystem::JavaScript,
            "lodash".to_string(),
            "^4.17.0".to_string(),
        );

        assert!(project.dependencies.contains_key(&Ecosystem::JavaScript));
        assert!(project.dependencies[&Ecosystem::JavaScript].contains_key("lodash"));
    }

    #[test]
    fn test_project_add_dev_dependency() {
        let mut project = create_test_project();
        
        project.add_dev_dependency(
            Ecosystem::JavaScript,
            "jest".to_string(),
            "^29.0.0".to_string(),
        );

        assert!(project.dev_dependencies.contains_key(&Ecosystem::JavaScript));
        assert!(project.dev_dependencies[&Ecosystem::JavaScript].contains_key("jest"));
    }

    #[test]
    fn test_project_get_ecosystems() {
        let mut project = create_test_project();
        
        // Initially no ecosystems since no dependencies added
        assert_eq!(project.get_ecosystems().len(), 0);
        
        // Add some dependencies
        project.add_dependency(Ecosystem::JavaScript, "lodash".to_string(), "^4.17.0".to_string());
        project.add_dependency(Ecosystem::Python, "requests".to_string(), "^2.28.0".to_string());
        
        let ecosystems = project.get_ecosystems();
        assert!(ecosystems.contains(&Ecosystem::JavaScript));
        assert!(ecosystems.contains(&Ecosystem::Python));
        assert_eq!(ecosystems.len(), 2);
    }
}

#[cfg(test)]
mod lock_file_tests {
    use super::*;

    #[test]
    fn test_lock_file_creation() {
        let lock_file = LockFile::new("project_hash123".to_string(), "0.1.0".to_string());
        
        assert_eq!(lock_file.version, LockFile::CURRENT_VERSION);
        assert_eq!(lock_file.project_hash, "project_hash123");
        assert_eq!(lock_file.ppm_version, "0.1.0");
        assert!(lock_file.resolved_dependencies.is_empty());
        assert!(!lock_file.generation_timestamp.is_empty());
    }

    #[test]
    fn test_lock_file_with_dependencies() {
        let mut resolved_deps = HashMap::new();
        let valid_hash = "d".repeat(64);
        let js_deps = vec![
            ResolvedDependency::new(
                "lodash".to_string(),
                "4.17.21".to_string(),
                Ecosystem::JavaScript,
                valid_hash,
                "integrity123".to_string(),
                "/store/path".to_string(),
            )
        ];
        resolved_deps.insert(Ecosystem::JavaScript, js_deps);

        let lock_file = LockFile::with_dependencies(
            "project_hash123".to_string(),
            "0.1.0".to_string(),
            resolved_deps,
        );

        assert_eq!(lock_file.project_hash, "project_hash123");
        assert!(lock_file.resolved_dependencies.contains_key(&Ecosystem::JavaScript));
        assert_eq!(lock_file.resolved_dependencies[&Ecosystem::JavaScript].len(), 1);
    }

    #[test]
    fn test_lock_file_serialization() {
        let lock_file = LockFile::new("project_hash123".to_string(), "0.1.0".to_string());

        let json = serde_json::to_string_pretty(&lock_file).unwrap();
        let deserialized: LockFile = serde_json::from_str(&json).unwrap();
        
        assert_eq!(lock_file.project_hash, deserialized.project_hash);
        assert_eq!(lock_file.version, deserialized.version);
        assert_eq!(lock_file.ppm_version, deserialized.ppm_version);
    }
}

#[cfg(test)]
mod global_store_tests {
    use super::*;

    #[test]
    fn test_global_store_creation() {
        let store = GlobalStore::new(PathBuf::from("/test/store"));
        
        assert_eq!(store.root_path, PathBuf::from("/test/store"));
    }

    #[test]
    fn test_global_store_store_package() {
        let mut store = GlobalStore::new(PathBuf::from("/test/store"));
        
        // Use a valid SHA-256 hash (64 hex characters)
        let valid_hash = "a".repeat(64);
        let package = Package::new(
            "lodash".to_string(),
            "4.17.21".to_string(),
            Ecosystem::JavaScript,
            valid_hash.clone(),
            PathBuf::from("/test/store/package"),
        );

        let hash = store.store_package(&package).unwrap();
        assert_eq!(hash, valid_hash);
        
        let retrieved = store.get_package(&valid_hash);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "lodash");
    }
}

#[cfg(test)]
mod virtual_environment_tests {
    use super::*;

    #[test]
    fn test_virtual_environment_creation() {
        let venv = VirtualEnvironment::new(
            "test-venv".to_string(),
            PathBuf::from("/test/venv"),
            Ecosystem::Python,
            VenvConfig::default(),
        );
        
        assert_eq!(venv.name, "test-venv");
        assert_eq!(venv.path, PathBuf::from("/test/venv"));
        assert_eq!(venv.ecosystem, Ecosystem::Python);
    }

    #[test]
    fn test_virtual_environment_serialization() {
        let venv = VirtualEnvironment::new(
            "test-venv".to_string(),
            PathBuf::from("/test/venv"),
            Ecosystem::Python,
            VenvConfig::default(),
        );

        let json = serde_json::to_string(&venv).unwrap();
        let deserialized: VirtualEnvironment = serde_json::from_str(&json).unwrap();
        
        assert_eq!(venv.name, deserialized.name);
        assert_eq!(venv.path, deserialized.path);
        assert_eq!(venv.ecosystem, deserialized.ecosystem);
    }
}

#[cfg(test)]
mod symlink_structure_tests {
    use super::*;

    #[test]
    fn test_symlink_structure_creation() {
        let structure = SymlinkStructure::new(
            PathBuf::from("/test/project"),
            Ecosystem::JavaScript,
        );
        
        assert_eq!(structure.root_path, PathBuf::from("/test/project"));
        assert_eq!(structure.ecosystem, Ecosystem::JavaScript);
        assert!(structure.links.is_empty());
    }

    #[test]
    fn test_symlink_structure_serialization() {
        let structure = SymlinkStructure::new(
            PathBuf::from("/test/project"),
            Ecosystem::JavaScript,
        );

        let json = serde_json::to_string(&structure).unwrap();
        let deserialized: SymlinkStructure = serde_json::from_str(&json).unwrap();
        
        assert_eq!(structure.root_path, deserialized.root_path);
        assert_eq!(structure.ecosystem, deserialized.ecosystem);
        assert_eq!(structure.links.len(), deserialized.links.len());
    }
}
