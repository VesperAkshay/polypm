# Contributing to PPM

Thank you for your interest in contributing to PPM (Polyglot Package Manager)! This guide will help you get started with contributing to the project.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contributing Guidelines](#contributing-guidelines)
- [Pull Request Process](#pull-request-process)
- [Testing](#testing)
- [Code Style](#code-style)
- [Documentation](#documentation)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

This project adheres to a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to [conduct@ppm.dev](mailto:conduct@ppm.dev).

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Git
- Node.js 18+ (for JavaScript ecosystem testing)
- Python 3.8+ (for Python ecosystem testing)

### Quick Start

1. **Fork and Clone**
   ```bash
   git clone https://github.com/your-username/polypm.git
   cd polypm
   ```

2. **Build and Test**
   ```bash
   cargo build
   cargo test
   cargo clippy
   ```

3. **Run PPM**
   ```bash
   cargo run --bin ppm -- --help
   ```

## Development Setup

### Repository Structure

```
polypm/
├── src/                    # Core source code
│   ├── cli/               # Command-line interface
│   ├── models/            # Data structures
│   ├── services/          # Business logic
│   └── utils/             # Shared utilities
├── tests/                 # Test suites
│   ├── contract/          # CLI contract tests
│   ├── integration/       # End-to-end tests
│   └── unit/              # Unit tests
├── examples/              # Example projects
├── docs/                  # Documentation
└── specs/                 # Design specifications
```

### Development Commands

```bash
# Build
cargo build                 # Debug build
cargo build --release       # Release build

# Testing
cargo test                  # Run all tests
cargo test --test unit_tests      # Run unit tests only
cargo test --test integration_tests # Run integration tests only

# Code Quality
cargo clippy               # Linting
cargo fmt                  # Code formatting
cargo test && cargo clippy # Full check

# Documentation
cargo doc --open           # Generate and open docs
```

### Environment Setup

For testing ecosystem integration:

```bash
# JavaScript ecosystem
npm install -g npm@latest

# Python ecosystem  
python -m pip install --upgrade pip virtualenv

# Test both ecosystems work
node --version
python --version
```

## Contributing Guidelines

### Types of Contributions

We welcome several types of contributions:

- **Bug Fixes**: Fix issues in existing functionality
- **Features**: Implement new features from the roadmap
- **Documentation**: Improve docs, examples, and guides
- **Tests**: Add test coverage for existing code
- **Performance**: Optimize existing implementations
- **Refactoring**: Improve code structure and maintainability

### Choosing What to Work On

1. **Check Issues**: Look for issues labeled `good first issue` or `help wanted`
2. **Feature Roadmap**: Check the [project roadmap](ROADMAP.md) for planned features
3. **Ask First**: For large changes, open an issue to discuss the approach

### Development Workflow

1. **Create Branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-description
   ```

2. **Make Changes**
   - Write code following our [style guide](#code-style)
   - Add tests for new functionality
   - Update documentation as needed

3. **Test Changes**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

4. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat: add support for yarn workspaces"
   ```

5. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```

## Pull Request Process

### Before Submitting

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code follows style guidelines
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (if applicable)

### PR Template

When creating a PR, please include:

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Refactoring

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
```

### Review Process

1. **Automated Checks**: CI will run tests and linting
2. **Code Review**: Maintainers will review your code
3. **Address Feedback**: Make requested changes
4. **Final Approval**: Maintainer approval required for merge

## Testing

### Test Categories

1. **Unit Tests** (`tests/unit/`)
   - Test individual functions and modules
   - Fast execution, no external dependencies
   - Should cover edge cases and error conditions

2. **Integration Tests** (`tests/integration/`)
   - Test component interactions
   - May use file system or network
   - Test realistic workflows

3. **Contract Tests** (`tests/contract/`)
   - Test CLI command contracts
   - Ensure backward compatibility
   - Validate command outputs and exit codes

### Writing Tests

#### Unit Test Example
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_name_validation() {
        assert!(validate_package_name("express").is_ok());
        assert!(validate_package_name("@types/node").is_ok());
        assert!(validate_package_name("").is_err());
    }
}
```

#### Integration Test Example
```rust
#[tokio::test]
async fn test_install_express() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().join("project.toml");
    
    create_test_project(&project_path).await;
    
    let result = install_packages(&["express"]).await;
    assert!(result.is_ok());
    
    assert!(temp_dir.path().join("node_modules/express").exists());
}
```

### Running Specific Tests

```bash
# Run by pattern
cargo test package_installer

# Run specific test file
cargo test --test integration_tests

# Run with output
cargo test -- --nocapture

# Run single test
cargo test test_package_validation -- --exact
```

## Code Style

### Rust Style Guide

We follow the standard Rust style with some additions:

```rust
// Use explicit error types
fn parse_version(version: &str) -> Result<Version, ParseError> {
    // Implementation
}

// Prefer descriptive variable names
let resolved_dependencies = resolver.resolve(&dependencies)?;

// Use structured error handling
match result {
    Ok(value) => handle_success(value),
    Err(ParseError::InvalidFormat(msg)) => handle_parse_error(msg),
    Err(ParseError::UnsupportedVersion(version)) => handle_version_error(version),
}
```

### Documentation Comments

```rust
/// Resolves package dependencies using the specified constraints.
/// 
/// # Arguments
/// 
/// * `dependencies` - List of package dependencies to resolve
/// * `constraints` - Version constraints to apply during resolution
/// 
/// # Returns
/// 
/// Returns `Ok(Vec<ResolvedDependency>)` if all dependencies can be resolved,
/// or `Err(ResolverError)` if resolution fails.
/// 
/// # Examples
/// 
/// ```rust
/// let deps = vec![Dependency::new("express", "^4.18.0")];
/// let resolved = resolver.resolve(&deps, &constraints)?;
/// ```
pub async fn resolve_dependencies(
    &self,
    dependencies: &[Dependency],
    constraints: &VersionConstraints,
) -> Result<Vec<ResolvedDependency>, ResolverError> {
    // Implementation
}
```

### Configuration

The project uses:
- **rustfmt**: Code formatting (configuration in `rustfmt.toml`)
- **clippy**: Linting (configuration in `.clippy.toml`)

## Documentation

### Types of Documentation

1. **API Documentation**: Rust doc comments (`cargo doc`)
2. **User Guide**: README.md and examples
3. **Developer Guide**: This contributing guide
4. **Design Docs**: Architecture and design decisions

### Documentation Standards

- **Code Comments**: Explain *why*, not *what*
- **API Docs**: Include examples and error conditions
- **Examples**: Working code that users can copy-paste
- **Guides**: Step-by-step instructions with context

### Updating Documentation

When making changes:
- Update relevant doc comments
- Add examples for new features
- Update README.md if CLI changes
- Add example projects for new use cases

## Reporting Issues

We use GitHub Issues to track all bugs and feature requests. We've set up several issue templates to help you submit well-structured reports. When creating a new issue, you'll be prompted to choose the most appropriate template.

### Issue Types

1. **🐛 Bug Report**
   - For reporting bugs and unexpected behavior
   - Please include steps to reproduce, expected vs actual behavior, and environment details
   - [Create a Bug Report](https://github.com/VesperAkshay/polypm/issues/new?template=bug_report.md)

2. **🚀 Feature Request**
   - For suggesting new features or improvements
   - Please describe the use case and why it would be valuable
   - [Request a Feature](https://github.com/VesperAkshay/polypm/issues/new?template=feature_request.md)

3. **🔒 Security Vulnerability**
   - For reporting security issues
   - Please do not disclose security issues publicly
   - [Report a Security Issue](https://github.com/VesperAkshay/polypm/security/advisories/new)

4. **✨ Enhancement**
   - For proposing specific improvements to existing features
   - Please include the benefit and potential impact
   - [Suggest an Enhancement](https://github.com/VesperAkshay/polypm/issues/new?template=enhancement.md)

### Before You Submit

- Search existing issues to avoid duplicates
- Check if the issue has been fixed in the latest version
- Provide as much detail as possible
- Include code samples or test cases where applicable

### Bug Reports

Use the bug report template:

```markdown
**Bug Description**
Clear description of the bug

**Steps to Reproduce**
1. Run `ppm init`
2. Add dependency `ppm add express`
3. See error

**Expected Behavior**
What should happen

**Actual Behavior** 
What actually happens

**Environment**
- OS: Windows 11
- Rust version: 1.75.0
- PPM version: 0.1.0
- Node.js version: 18.17.0
- Python version: 3.11.0

**Additional Context**
Any other relevant information
```

### Feature Requests

For feature requests:
- Check if it's already planned in the roadmap
- Describe the use case and problem it solves
- Suggest a possible implementation approach
- Consider if it fits PPM's scope and philosophy

### Getting Help

- **Documentation**: Check README.md and examples first
- **Discussions**: Use GitHub Discussions for questions
- **Issues**: Use GitHub Issues for bugs and feature requests
- **Chat**: Join our Discord community

## Release Process

### Versioning

We use [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

1. Update CHANGELOG.md
2. Bump version in Cargo.toml
3. Run full test suite
4. Create release tag
5. Publish to crates.io
6. Update documentation

## Recognition

Contributors are recognized in:
- CONTRIBUTORS.md file
- Release notes
- GitHub contributor stats

Thank you for contributing to PPM! 🚀
