# Changelog

All notable changes to PPM (Polyglot Package Manager) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive error handling with user-friendly messages and suggestions
- Input validation for package names, versions, and script names
- Network error detection with specific guidance
- CLI help text improvements with examples and detailed descriptions
- Complete documentation with README.md, examples, and contributing guide
- Example projects demonstrating different use cases

### Changed
- Enhanced error messages across all CLI commands
- Improved validation with ecosystem-specific rules (npm vs PyPI)
- Better context and suggestions for common error scenarios

### Fixed
- Error handling edge cases in network operations
- Input validation for malformed package specifications

## [0.1.0] - 2025-09-13

### Added
- Initial release of PPM (Polyglot Package Manager)
- Support for JavaScript (npm) and Python (PyPI) package management
- Unified `project.toml` configuration file
- CLI commands: `init`, `install`, `add`, `run`, `venv`
- Python virtual environment management
- JavaScript package installation with symlink optimization
- Dependency resolution across ecosystems
- Lock file generation for reproducible builds
- Cross-platform support (Windows, macOS, Linux)
- Registry integration with npm and PyPI
- Script execution with proper environment setup
- Global package store with content-addressable storage
- Comprehensive test suite (unit, integration, contract tests)
- Performance optimizations with parallel downloads and caching

### Core Features
- **Project Initialization**: Create polyglot projects with `ppm init`
- **Dependency Management**: Add packages with `ppm add`, install with `ppm install`
- **Virtual Environments**: Automatic Python venv creation and management
- **Script Execution**: Run project scripts with `ppm run` and proper environment
- **Registry Integration**: Direct integration with npm and PyPI registries
- **Lock File Support**: Generate and use lock files for consistent builds
- **Error Handling**: User-friendly error messages with actionable suggestions

### Supported Ecosystems
- **JavaScript**: npm registry, Node.js packages, TypeScript support
- **Python**: PyPI registry, pip packages, virtual environment isolation

### CLI Commands
- `ppm init`: Initialize new polyglot projects
- `ppm add`: Add dependencies to project
- `ppm install`: Install dependencies from project.toml
- `ppm run`: Execute project scripts with environment setup
- `ppm venv`: Manage Python virtual environments

### Architecture
- **Models**: Project, Package, Dependency, LockFile, VirtualEnvironment
- **Services**: NPM/PyPI clients, dependency resolver, package installer
- **CLI**: Command-line interface with clap framework
- **Storage**: Global store with SHA-256 content addressing
- **Environment**: Cross-platform symlink and venv management

### Technical Stack
- **Language**: Rust 1.75+
- **CLI Framework**: clap 4.x
- **Serialization**: serde with TOML and JSON support
- **HTTP Client**: reqwest with async/await
- **Crypto**: SHA-256 hashing for integrity verification
- **Cross-platform**: symlink and process management

### Development
- Test-driven development with comprehensive test coverage
- Contract tests ensuring CLI backward compatibility
- Integration tests for end-to-end workflows
- Unit tests for core business logic
- Performance benchmarks and optimization
- Code quality with clippy and rustfmt

[Unreleased]: https://github.com/VesperAkshay/polypm/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/VesperAkshay/polypm/releases/tag/v0.1.0
