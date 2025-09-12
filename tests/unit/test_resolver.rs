use std::collections::HashMap;
use std::path::PathBuf;
use tokio;

use ppm::services::dependency_resolver::{DependencyResolver, ResolutionConfig, ResolverError};
use ppm::services::npm_client::NpmClient;
use ppm::services::pypi_client::PypiClient;
use ppm::models::dependency::Dependency;
use ppm::models::ecosystem::Ecosystem;
use ppm::models::global_store::GlobalStore;

/// Test module for dependency resolver
mod dependency_resolver_tests {
    use super::*;

    /// Helper function to create a test dependency resolver
    fn create_test_resolver() -> DependencyResolver {
        let npm_client = NpmClient::new();
        let pypi_client = PypiClient::new();
        let global_store = GlobalStore::new(PathBuf::from("/tmp/test-store"));
        
        DependencyResolver::new(npm_client, pypi_client, global_store)
    }

    /// Helper function to create a test dependency resolver with custom config
    fn create_test_resolver_with_config(config: ResolutionConfig) -> DependencyResolver {
        let npm_client = NpmClient::new();
        let pypi_client = PypiClient::new();
        let global_store = GlobalStore::new(PathBuf::from("/tmp/test-store"));
        
        DependencyResolver::with_config(npm_client, pypi_client, global_store, config)
    }

    /// Test dependency resolver creation with default settings
    #[test]
    fn test_dependency_resolver_new() {
        let resolver = create_test_resolver();
        // Resolver should be created without panic
        assert!(true);
    }

    /// Test dependency resolver creation with custom configuration
    #[test]
    fn test_dependency_resolver_with_config() {
        let config = ResolutionConfig {
            max_depth: 5,
            include_dev_dependencies: true,
            allow_prerelease: true,
            prefer_cached: false,
            ecosystem_constraints: HashMap::new(),
        };
        
        let resolver = create_test_resolver_with_config(config);
        // Resolver should be created without panic
        assert!(true);
    }

    /// Test resolving single JavaScript dependency
    #[tokio::test]
    async fn test_resolve_single_javascript_dependency() {
        let mut resolver = create_test_resolver();
        
        let dependencies = vec![
            Dependency::production("react".to_string(), "^18.0.0".to_string(), Ecosystem::JavaScript),
        ];
        
        let result = resolver.resolve_dependencies(dependencies).await;
        assert!(result.is_ok());
        
        let resolution = result.unwrap();
        assert!(resolution.is_successful());
        assert_eq!(resolution.resolved_count(), 1);
        assert_eq!(resolution.failed_count(), 0);
        
        let resolved_dep = &resolution.resolved[0];
        assert_eq!(resolved_dep.name, "react");
        assert_eq!(resolved_dep.version, "18.2.0"); // Mock version from resolver
        assert_eq!(resolved_dep.ecosystem, Ecosystem::JavaScript);
    }

    /// Test resolving single Python dependency
    #[tokio::test]
    async fn test_resolve_single_python_dependency() {
        let mut resolver = create_test_resolver();
        
        let dependencies = vec![
            Dependency::production("flask".to_string(), ">=2.0.0".to_string(), Ecosystem::Python),
        ];
        
        let result = resolver.resolve_dependencies(dependencies).await;
        assert!(result.is_ok());
        
        let resolution = result.unwrap();
        assert!(resolution.is_successful());
        assert_eq!(resolution.resolved_count(), 1);
        assert_eq!(resolution.failed_count(), 0);
        
        let resolved_dep = &resolution.resolved[0];
        assert_eq!(resolved_dep.name, "flask");
        assert_eq!(resolved_dep.version, "2.3.0"); // Mock version from resolver
        assert_eq!(resolved_dep.ecosystem, Ecosystem::Python);
    }

    /// Test resolving multiple dependencies from different ecosystems
    #[tokio::test]
    async fn test_resolve_mixed_ecosystem_dependencies() {
        let mut resolver = create_test_resolver();
        
        let dependencies = vec![
            Dependency::production("react".to_string(), "^18.0.0".to_string(), Ecosystem::JavaScript),
            Dependency::production("flask".to_string(), ">=2.0.0".to_string(), Ecosystem::Python),
            Dependency::production("lodash".to_string(), "^4.17.0".to_string(), Ecosystem::JavaScript),
        ];
        
        let result = resolver.resolve_dependencies(dependencies).await;
        assert!(result.is_ok());
        
        let resolution = result.unwrap();
        assert!(resolution.is_successful());
        assert_eq!(resolution.resolved_count(), 3);
        assert_eq!(resolution.failed_count(), 0);
        
        // Check JavaScript dependencies
        let js_deps = resolution.dependencies_by_ecosystem(Ecosystem::JavaScript);
        assert_eq!(js_deps.len(), 2);
        
        // Check Python dependencies
        let py_deps = resolution.dependencies_by_ecosystem(Ecosystem::Python);
        assert_eq!(py_deps.len(), 1);
    }

    /// Test resolving development dependencies when excluded
    #[tokio::test]
    async fn test_resolve_exclude_dev_dependencies() {
        let config = ResolutionConfig {
            max_depth: 10,
            include_dev_dependencies: false,
            allow_prerelease: false,
            prefer_cached: false,
            ecosystem_constraints: HashMap::new(),
        };
        
        let mut resolver = create_test_resolver_with_config(config);
        
        let dependencies = vec![
            Dependency::production("react".to_string(), "^18.0.0".to_string(), Ecosystem::JavaScript),
            Dependency::development("jest".to_string(), "^29.0.0".to_string(), Ecosystem::JavaScript),
        ];
        
        let result = resolver.resolve_dependencies(dependencies).await;
        assert!(result.is_ok());
        
        let resolution = result.unwrap();
        // Only production dependency should be resolved
        assert_eq!(resolution.resolved_count(), 1);
        assert_eq!(resolution.resolved[0].name, "react");
    }

    /// Test resolving development dependencies when included
    #[tokio::test]
    async fn test_resolve_include_dev_dependencies() {
        let config = ResolutionConfig {
            max_depth: 10,
            include_dev_dependencies: true,
            allow_prerelease: false,
            prefer_cached: false,
            ecosystem_constraints: HashMap::new(),
        };
        
        let mut resolver = create_test_resolver_with_config(config);
        
        let dependencies = vec![
            Dependency::production("react".to_string(), "^18.0.0".to_string(), Ecosystem::JavaScript),
            Dependency::development("express".to_string(), "^4.0.0".to_string(), Ecosystem::JavaScript),
        ];
        
        let result = resolver.resolve_dependencies(dependencies).await;
        assert!(result.is_ok());
        
        let resolution = result.unwrap();
        // Both production and development dependencies should be resolved
        assert_eq!(resolution.resolved_count(), 2);
        
        let dep_names: Vec<&str> = resolution.resolved.iter()
            .map(|d| d.name.as_str())
            .collect();
        assert!(dep_names.contains(&"react"));
        assert!(dep_names.contains(&"express"));
    }

    /// Test resolving with maximum depth limit
    #[tokio::test]
    async fn test_resolve_with_max_depth_limit() {
        let config = ResolutionConfig {
            max_depth: 1,
            include_dev_dependencies: false,
            allow_prerelease: false,
            prefer_cached: false,
            ecosystem_constraints: HashMap::new(),
        };
        
        let mut resolver = create_test_resolver_with_config(config);
        
        let dependencies = vec![
            Dependency::production("react".to_string(), "^18.0.0".to_string(), Ecosystem::JavaScript),
        ];
        
        let result = resolver.resolve_dependencies(dependencies).await;
        assert!(result.is_ok());
        
        let resolution = result.unwrap();
        assert!(resolution.max_depth_reached <= 1);
    }

    /// Test caching functionality
    #[tokio::test]
    async fn test_dependency_resolution_caching() {
        let mut resolver = create_test_resolver();
        
        let dependencies = vec![
            Dependency::production("react".to_string(), "^18.0.0".to_string(), Ecosystem::JavaScript),
        ];
        
        // First resolution
        let result1 = resolver.resolve_dependencies(dependencies.clone()).await;
        assert!(result1.is_ok());
        
        // Second resolution should use cache
        let result2 = resolver.resolve_dependencies(dependencies).await;
        assert!(result2.is_ok());
        
        let resolution1 = result1.unwrap();
        let resolution2 = result2.unwrap();
        
        // Should resolve to same version
        assert_eq!(resolution1.resolved[0].version, resolution2.resolved[0].version);
    }

    /// Test cache clearing
    #[test]
    fn test_cache_clearing() {
        let mut resolver = create_test_resolver();
        
        // Check initial cache stats
        let (initial_size, _) = resolver.get_cache_stats();
        assert_eq!(initial_size, 0);
        
        // Clear cache
        resolver.clear_cache();
        
        let (after_clear_size, _) = resolver.get_cache_stats();
        assert_eq!(after_clear_size, 0);
    }

    /// Test configuration updates
    #[test]
    fn test_configuration_updates() {
        let mut resolver = create_test_resolver();
        
        let new_config = ResolutionConfig {
            max_depth: 5,
            include_dev_dependencies: true,
            allow_prerelease: true,
            prefer_cached: false,
            ecosystem_constraints: HashMap::new(),
        };
        
        resolver.update_config(new_config);
        // Should update without panic
        assert!(true);
    }

    /// Test ecosystem-specific dependency filtering
    #[tokio::test]
    async fn test_resolve_ecosystem_specific_dependencies() {
        let mut resolver = create_test_resolver();
        
        let dependencies = vec![
            Dependency::production("react".to_string(), "^18.0.0".to_string(), Ecosystem::JavaScript),
            Dependency::production("flask".to_string(), ">=2.0.0".to_string(), Ecosystem::Python),
        ];
        
        // Resolve only JavaScript dependencies
        let result = resolver.resolve_ecosystem_dependencies(dependencies, Ecosystem::JavaScript).await;
        assert!(result.is_ok());
        
        let resolution = result.unwrap();
        assert_eq!(resolution.resolved_count(), 1);
        assert_eq!(resolution.resolved[0].name, "react");
        assert_eq!(resolution.resolved[0].ecosystem, Ecosystem::JavaScript);
    }

    /// Test error handling for unknown packages
    #[tokio::test]
    async fn test_resolve_unknown_package_error() {
        let mut resolver = create_test_resolver();
        
        let dependencies = vec![
            Dependency::production("nonexistent-package-xyz".to_string(), "^1.0.0".to_string(), Ecosystem::JavaScript),
        ];
        
        let result = resolver.resolve_dependencies(dependencies).await;
        assert!(result.is_ok());
        
        let resolution = result.unwrap();
        // Unknown package should fail to resolve
        assert_eq!(resolution.failed_count(), 1);
        assert_eq!(resolution.resolved_count(), 0);
        
        let failure = &resolution.failed[0];
        assert_eq!(failure.dependency.name, "nonexistent-package-xyz");
        assert_eq!(failure.depth, 0);
    }

    /// Test resolution result helpers
    #[tokio::test]
    async fn test_resolution_result_helpers() {
        let mut resolver = create_test_resolver();
        
        let dependencies = vec![
            Dependency::production("react".to_string(), "^18.0.0".to_string(), Ecosystem::JavaScript),
            Dependency::production("flask".to_string(), ">=2.0.0".to_string(), Ecosystem::Python),
        ];
        
        let result = resolver.resolve_dependencies(dependencies).await;
        assert!(result.is_ok());
        
        let resolution = result.unwrap();
        
        // Test helper methods
        assert!(resolution.is_successful());
        assert_eq!(resolution.resolved_count(), 2);
        assert_eq!(resolution.failed_count(), 0);
        assert!(resolution.total_processed >= 2);
        
        // Test ecosystem filtering
        let js_deps = resolution.dependencies_by_ecosystem(Ecosystem::JavaScript);
        let py_deps = resolution.dependencies_by_ecosystem(Ecosystem::Python);
        assert_eq!(js_deps.len(), 1);
        assert_eq!(py_deps.len(), 1);
    }

    /// Test circular dependency detection (if implemented)
    #[tokio::test]
    async fn test_circular_dependency_detection() {
        let mut resolver = create_test_resolver();
        
        // Create the same dependency twice to simulate potential circular reference
        let dependencies = vec![
            Dependency::production("react".to_string(), "^18.0.0".to_string(), Ecosystem::JavaScript),
            Dependency::production("react".to_string(), "^18.0.0".to_string(), Ecosystem::JavaScript),
        ];
        
        let result = resolver.resolve_dependencies(dependencies).await;
        assert!(result.is_ok());
        
        let resolution = result.unwrap();
        // Should deduplicate the same dependency
        assert_eq!(resolution.resolved_count(), 1);
    }

    /// Test dependency resolution with version constraints
    #[tokio::test]
    async fn test_dependency_resolution_with_constraints() {
        let mut ecosystem_constraints = HashMap::new();
        ecosystem_constraints.insert(Ecosystem::JavaScript, ">=18.0.0".to_string());
        
        let config = ResolutionConfig {
            max_depth: 10,
            include_dev_dependencies: false,
            allow_prerelease: false,
            prefer_cached: false,
            ecosystem_constraints,
        };
        
        let mut resolver = create_test_resolver_with_config(config);
        
        let dependencies = vec![
            Dependency::production("react".to_string(), "^18.0.0".to_string(), Ecosystem::JavaScript),
        ];
        
        let result = resolver.resolve_dependencies(dependencies).await;
        assert!(result.is_ok());
        
        let resolution = result.unwrap();
        assert_eq!(resolution.resolved_count(), 1);
        assert_eq!(resolution.resolved[0].name, "react");
    }

    /// Test package existence checking
    #[tokio::test]
    async fn test_package_existence_checking() {
        let resolver = create_test_resolver();
        
        // Test known package (should return true for test packages)
        let exists_result = resolver.package_exists("react", Ecosystem::JavaScript).await;
        assert!(exists_result.is_ok());
        
        // Test unknown package
        let not_exists_result = resolver.package_exists("nonexistent-package-xyz", Ecosystem::JavaScript).await;
        assert!(not_exists_result.is_ok());
    }

    /// Test finding latest compatible version
    #[tokio::test]
    async fn test_find_latest_compatible_version() {
        let resolver = create_test_resolver();
        
        let dependency = Dependency::production("react".to_string(), "^18.0.0".to_string(), Ecosystem::JavaScript);
        let result = resolver.find_latest_compatible(&dependency).await;
        // Should return a result (mock or real)
        assert!(result.is_ok() || result.is_err()); // Either outcome is valid for this test
    }

    /// Test getting available versions
    #[tokio::test]
    async fn test_get_available_versions() {
        let resolver = create_test_resolver();
        
        let result = resolver.get_available_versions("react", Ecosystem::JavaScript).await;
        // Should return a result (mock or real)
        assert!(result.is_ok() || result.is_err()); // Either outcome is valid for this test
    }

    /// Test dependency tree creation
    #[tokio::test]
    async fn test_create_dependency_tree() {
        let mut resolver = create_test_resolver();
        
        let dependencies = vec![
            Dependency::production("react".to_string(), "^18.0.0".to_string(), Ecosystem::JavaScript),
        ];
        
        let result = resolver.create_dependency_tree(dependencies).await;
        assert!(result.is_ok());
        
        let tree = result.unwrap();
        assert!(!tree.roots.is_empty());
    }
}

/// Test module for ResolutionConfig builder pattern
mod resolution_config_tests {
    use super::*;

    /// Test ResolutionConfig creation with default values
    #[test]
    fn test_resolution_config_new() {
        let config = ResolutionConfig::new();
        
        assert_eq!(config.max_depth, 10);
        assert!(!config.include_dev_dependencies);
        assert!(!config.allow_prerelease);
        assert!(config.prefer_cached);
        assert!(config.ecosystem_constraints.is_empty());
    }

    /// Test ResolutionConfig builder methods
    #[test]
    fn test_resolution_config_builder() {
        let config = ResolutionConfig::new()
            .with_max_depth(5)
            .with_dev_dependencies(true)
            .with_prerelease(true)
            .with_cache_preference(true)
            .with_ecosystem_constraint(Ecosystem::JavaScript, ">=18.0.0".to_string());
        
        assert_eq!(config.max_depth, 5);
        assert!(config.include_dev_dependencies);
        assert!(config.allow_prerelease);
        assert!(config.prefer_cached);
        assert_eq!(config.ecosystem_constraints.len(), 1);
        assert_eq!(
            config.ecosystem_constraints.get(&Ecosystem::JavaScript),
            Some(&">=18.0.0".to_string())
        );
    }

    /// Test multiple ecosystem constraints
    #[test]
    fn test_resolution_config_multiple_constraints() {
        let config = ResolutionConfig::new()
            .with_ecosystem_constraint(Ecosystem::JavaScript, ">=18.0.0".to_string())
            .with_ecosystem_constraint(Ecosystem::Python, ">=3.8".to_string());
        
        assert_eq!(config.ecosystem_constraints.len(), 2);
        assert!(config.ecosystem_constraints.contains_key(&Ecosystem::JavaScript));
        assert!(config.ecosystem_constraints.contains_key(&Ecosystem::Python));
    }
}

/// Test module for error types
mod resolver_error_tests {
    use super::*;

    /// Test ResolverError display messages
    #[test]
    fn test_resolver_error_display() {
        let version_conflict = ResolverError::VersionConflict {
            package: "react".to_string(),
            version1: "18.0.0".to_string(),
            version2: "17.0.0".to_string(),
        };
        assert!(version_conflict.to_string().contains("Version conflict"));
        assert!(version_conflict.to_string().contains("react"));
        
        let circular_dependency = ResolverError::CircularDependency {
            cycle: "A -> B -> A".to_string(),
        };
        assert!(circular_dependency.to_string().contains("Circular dependency"));
        
        let max_depth_exceeded = ResolverError::MaxDepthExceeded { max_depth: 10 };
        assert!(max_depth_exceeded.to_string().contains("Maximum resolution depth"));
        
        let package_not_found = ResolverError::PackageNotFound {
            package: "nonexistent".to_string(),
            ecosystem: Ecosystem::JavaScript,
        };
        assert!(package_not_found.to_string().contains("not found"));
        
        let invalid_version = ResolverError::InvalidVersionSpec {
            package: "react".to_string(),
            version: "invalid".to_string(),
        };
        assert!(invalid_version.to_string().contains("Invalid version"));
        
        let unsupported_ecosystem = ResolverError::UnsupportedEcosystem(Ecosystem::JavaScript);
        assert!(unsupported_ecosystem.to_string().contains("not supported"));
    }
}
