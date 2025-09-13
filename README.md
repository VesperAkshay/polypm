# PPM - Polyglot Package Manager

[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A unified package manager for JavaScript and Python projects that enables seamless dependency management across multiple ecosystems in a single project.

## 🚀 Features

- **Polyglot Support**: Manage JavaScript (npm) and Python (PyPI) dependencies in one project
- **Unified Configuration**: Single `project.toml` file for all dependencies
- **Virtual Environments**: Automatic Python virtual environment management
- **Symlink Management**: Efficient JavaScript package installation with symlinks
- **Script Execution**: Run project scripts with proper environment setup
- **Lock File Support**: Reproducible builds with dependency locking
- **Cross-Platform**: Works on Windows, macOS, and Linux

## 📋 Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
- [Configuration](#configuration)
- [Examples](#examples)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [License](#license)

## 🔧 Installation

### Quick Install (Recommended)

**Windows (PowerShell):**
```powershell
Invoke-WebRequest https://raw.githubusercontent.com/VesperAkshay/polypm/main/install.ps1 -UseBasicParsing | Invoke-Expression
```

**Linux/macOS (Bash):**
```bash
curl -fsSL https://raw.githubusercontent.com/VesperAkshay/polypm/main/install.sh | bash
```

### Alternative Installation Methods

#### From Pre-built Binaries

Download the latest release for your platform:
- [Windows x64](https://github.com/VesperAkshay/polypm/releases/latest/download/ppm-windows-x86_64.exe)
- [Linux x64](https://github.com/VesperAkshay/polypm/releases/latest/download/ppm-linux-x86_64)
- [macOS x64](https://github.com/VesperAkshay/polypm/releases/latest/download/ppm-macos-x86_64)
- [macOS ARM64](https://github.com/VesperAkshay/polypm/releases/latest/download/ppm-macos-aarch64)

#### From Cargo (Rust Package Manager)

```bash
cargo install ppm
```

#### From Source

```bash
git clone https://github.com/VesperAkshay/polypm.git
cd polypm
cargo build --release
```

The binary will be available at `target/release/ppm`.

## 🗑️ Uninstallation

### Quick Uninstall

**Windows (PowerShell):**
```powershell
Invoke-WebRequest https://raw.githubusercontent.com/VesperAkshay/polypm/main/uninstall.ps1 -UseBasicParsing | Invoke-Expression
```

**Linux/macOS (Bash):**
```bash
curl -fsSL https://raw.githubusercontent.com/VesperAkshay/polypm/main/uninstall.sh | bash
```

### Manual Uninstallation

Since PPM is installed as a standalone executable, you can manually remove it:

1. **Find PPM location:**
   ```bash
   which ppm        # Linux/macOS
   where ppm        # Windows
   ```

2. **Remove the executable:**
   ```bash
   rm $(which ppm)         # Linux/macOS
   del "$(where ppm)"      # Windows
   ```

3. **Clean up project files (optional):**
   - Remove `node_modules/` directories
   - Remove `.venv/` directories  
   - Remove `ppm.lock` files
   - Remove global cache: `~/.ppm/` (Linux/macOS) or `%USERPROFILE%\.ppm\` (Windows)

### Prerequisites

- Rust 1.75 or later
- Node.js (for JavaScript package management)
- Python 3.8+ (for Python package management)

## 🚀 Quick Start

### 1. Initialize a New Project

```bash
# Create a new polyglot project
ppm init --name my-awesome-project

# Or initialize with specific ecosystem
ppm init --javascript  # JavaScript only
ppm init --python      # Python only
```

### 2. Add Dependencies

```bash
# Add JavaScript dependencies
ppm add express lodash --javascript

# Add Python dependencies  
ppm add requests flask --python

# Add with specific versions
ppm add express@4.18.0 requests@2.28.0
```

### 3. Install Dependencies

```bash
# Install all dependencies
ppm install

# This will:
# - Install JavaScript packages to node_modules/
# - Create Python virtual environment in .venv/
# - Install Python packages to the virtual environment
```

### 4. Run Scripts

```bash
# Run project scripts
ppm run build
ppm run test
ppm run start

# List available scripts
ppm run --list
```

### 5. Manage Virtual Environments

```bash
# Create Python virtual environment
ppm venv create

# Get virtual environment info
ppm venv info

# Remove virtual environment
ppm venv remove
```

## 📚 Commands

### `ppm init`

Initialize a new polyglot project.

```bash
ppm init [OPTIONS]

Options:
  --name <NAME>           Project name (default: current directory name)
  --version <VERSION>     Initial version (default: "1.0.0")
  --javascript           Include JavaScript dependencies section only
  --python               Include Python dependencies section only
  --force                Overwrite existing project.toml
  --json                 Output results in JSON format
  -h, --help             Print help
```

**Examples:**
```bash
ppm init                                    # Full polyglot project
ppm init --name my-app --version 0.1.0    # Custom name and version
ppm init --python --force                  # Python-only project, overwrite existing
```

### `ppm add`

Add new dependencies to the project.

```bash
ppm add <PACKAGES>... [OPTIONS]

Arguments:
  <PACKAGES>...    List of packages to add

Options:
  --save-dev              Add to dev-dependencies instead of dependencies
  --javascript           Force JavaScript ecosystem detection
  --python               Force Python ecosystem detection
  --version <VERSION>     Specify version constraint for single package
  --json                 Output results in JSON format
  -h, --help             Print help
```

**Examples:**
```bash
ppm add express lodash                      # Auto-detect ecosystem
ppm add express@^4.18.0 --javascript      # Specific version and ecosystem
ppm add pytest black --python --save-dev   # Python dev dependencies
ppm add @types/node --version "^18.0.0"   # Scoped package with version
```

### `ppm install`

Install all project dependencies.

```bash
ppm install [OPTIONS]

Options:
  --dev                  Install dev dependencies as well
  --javascript          Install JavaScript dependencies only
  --python              Install Python dependencies only
  --offline             Use cached packages only (no network)
  --json                Output results in JSON format
  -h, --help            Print help
```

**Examples:**
```bash
ppm install                    # Install all dependencies
ppm install --dev             # Include dev dependencies
ppm install --python          # Python packages only
ppm install --offline         # Use cache only
```

### `ppm run`

Execute project scripts with proper environment setup.

```bash
ppm run [SCRIPT] [OPTIONS] [-- <ARGS>...]

Arguments:
  [SCRIPT]      Script name to run
  [ARGS]...     Additional arguments to pass to the script

Options:
  --list        List all available scripts
  --env         Show environment variables for script execution
  --json        Output results in JSON format
  -h, --help    Print help
```

**Examples:**
```bash
ppm run build                      # Run build script
ppm run test -- --verbose         # Run test script with arguments
ppm run --list                     # Show all available scripts
ppm run start --env               # Show environment for start script
```

### `ppm venv`

Manage Python virtual environments.

```bash
ppm venv [COMMAND]

Commands:
  create    Create new virtual environment
  remove    Remove existing virtual environment
  info      Show virtual environment information
  shell     Activate venv in current shell (Unix only)
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help    Print help
```

**Subcommand Options:**

#### `ppm venv create`
```bash
ppm venv create [OPTIONS]

Options:
  --python <PYTHON>    Python version to use
  --path <PATH>        Custom path for venv
  --force             Remove existing venv before creating
  --json              Output as JSON
```

#### `ppm venv info`
```bash
ppm venv info [OPTIONS]

Options:
  --json              Output as JSON
```

**Examples:**
```bash
ppm venv create                           # Create default virtual environment
ppm venv create --python python3.11     # Use specific Python version
ppm venv create --path ./my-venv --force # Custom path, overwrite existing
ppm venv info                            # Show current venv information
ppm venv remove                          # Remove virtual environment
```

## ⚙️ Configuration

PPM uses a single `project.toml` file for configuration:

```toml
[project]
name = "my-awesome-project"
version = "1.0.0"

# JavaScript dependencies
[dependencies.javascript]
express = "^4.18.0"
lodash = "^4.17.21"

# Python dependencies  
[dependencies.python]
requests = "^2.28.0"
flask = "^2.2.0"

# Development dependencies
[dev-dependencies.javascript]
"@types/node" = "^18.0.0"
jest = "^29.0.0"

[dev-dependencies.python]
pytest = "^7.0.0"
black = "^22.0.0"

# Project scripts
[scripts]
build = "npm run build"
test = "npm test && python -m pytest"
start = "node server.js"
lint = "npm run lint && black ."
dev = "npm run dev"

# Virtual environment configuration
[venv]
python = "python3.11"
path = ".venv"
```

### Configuration Sections

- **`[project]`**: Basic project metadata
- **`[dependencies.{ecosystem}]`**: Production dependencies by ecosystem
- **`[dev-dependencies.{ecosystem}]`**: Development dependencies by ecosystem  
- **`[scripts]`**: Custom commands for `ppm run`
- **`[venv]`**: Python virtual environment settings

## 💡 Examples

### Example 1: Full-Stack Web Application

```bash
# Initialize project
ppm init --name fullstack-app

# Add frontend dependencies
ppm add react typescript webpack --javascript

# Add backend dependencies  
ppm add flask sqlalchemy --python

# Add development tools
ppm add @types/react jest --javascript --save-dev
ppm add pytest black --python --save-dev

# Install everything
ppm install --dev
```

**Resulting `project.toml`:**
```toml
[project]
name = "fullstack-app"
version = "1.0.0"

[dependencies.javascript]
react = "latest"
typescript = "latest"
webpack = "latest"

[dependencies.python]
flask = "latest"
sqlalchemy = "latest"

[dev-dependencies.javascript]
"@types/react" = "latest"
jest = "latest"

[dev-dependencies.python]
pytest = "latest"
black = "latest"

[scripts]
build = "webpack --mode production"
dev = "webpack serve --mode development"
test = "jest && python -m pytest"
format = "black . && npm run prettier"
```

### Example 2: Data Science Project

```bash
# Initialize Python-focused project
ppm init --name data-analysis --python

# Add data science dependencies
ppm add pandas numpy matplotlib jupyter --python

# Add development tools
ppm add pytest mypy --python --save-dev

# Install and create environment
ppm install --dev
ppm venv create --python python3.11
```

### Example 3: Node.js with Python Scripts

```bash
# Initialize JavaScript-focused project
ppm init --name node-with-python --javascript

# Add Node.js dependencies
ppm add express cors helmet --javascript

# Add Python for data processing
ppm add pandas requests --python

# Install everything
ppm install
```

## 🏗️ Architecture

PPM is built with a modular architecture:

```
src/
├── cli/           # Command-line interface
├── models/        # Data structures (Project, Package, etc.)
├── services/      # Core business logic
│   ├── npm_client.rs        # NPM registry integration
│   ├── pypi_client.rs       # PyPI registry integration
│   ├── dependency_resolver.rs # Dependency resolution
│   ├── package_installer.rs  # Package installation
│   └── virtual_environment_manager.rs # Python venv management
└── utils/         # Shared utilities
```

### Key Components

- **Registry Clients**: Interface with npm and PyPI registries
- **Dependency Resolver**: Resolves version constraints and conflicts
- **Package Installer**: Downloads and installs packages with integrity verification
- **Virtual Environment Manager**: Creates and manages Python virtual environments
- **Symlink Manager**: Creates efficient symlink structures for JavaScript packages

## 🛠️ Development

### Building from Source

```bash
git clone https://github.com/VesperAkshay/polypm.git
cd polypm
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test unit_tests
cargo test --test integration_tests
cargo test --test contract_tests
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Both together
cargo test && cargo clippy
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Run the test suite (`cargo test && cargo clippy`)
6. Commit your changes (`git commit -am 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- [Repository](https://github.com/VesperAkshay/polypm)
- [Issues](https://github.com/VesperAkshay/polypm/issues)
- [Discussions](https://github.com/VesperAkshay/polypm/discussions)

## 📞 Support

- 📧 Email: support@ppm.dev
- 💬 Discord: [PPM Community](https://discord.gg/ppm)
- 🐛 Bug Reports: [GitHub Issues](https://github.com/VesperAkshay/polypm/issues)

---

**Made with ❤️ for the polyglot development community**
