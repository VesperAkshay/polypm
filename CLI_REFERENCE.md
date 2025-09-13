# PPM CLI Reference

Quick reference guide for PPM (Polyglot Package Manager) commands.

## Global Options

All commands support these global options:
- `--help, -h`: Show help information
- `--version, -V`: Show version information

## Commands Overview

| Command | Purpose | Example |
|---------|---------|---------|
| [`init`](#ppm-init) | Initialize new project | `ppm init --name my-app` |
| [`install`](#ppm-install) | Install dependencies | `ppm install --dev` |
| [`add`](#ppm-add) | Add new dependencies | `ppm add express requests` |
| [`run`](#ppm-run) | Execute project scripts | `ppm run build` |
| [`venv`](#ppm-venv) | Manage virtual environments | `ppm venv create` |

## `ppm init`

Initialize a new polyglot project with unified configuration.

### Usage
```bash
ppm init [OPTIONS]
```

### Options
- `--name <NAME>`: Project name (default: current directory name)
- `--version <VERSION>`: Initial version (default: "1.0.0")
- `--javascript`: Include JavaScript dependencies section only
- `--python`: Include Python dependencies section only
- `--force`: Overwrite existing project.toml
- `--json`: Output results in JSON format

### Examples
```bash
# Basic initialization
ppm init

# Custom name and version
ppm init --name my-awesome-app --version 0.1.0

# JavaScript-only project
ppm init --javascript

# Python-only project
ppm init --python

# Force overwrite existing config
ppm init --force
```

## `ppm install`

Install dependencies from project.toml or add new packages.

### Usage
```bash
ppm install [PACKAGES]... [OPTIONS]
```

### Arguments
- `[PACKAGES]...`: Optional packages to install (if empty, installs from project.toml)

### Options
- `--save`: Add packages to dependencies (default for new packages)
- `--save-dev`: Add packages to dev-dependencies
- `--javascript`: Force JavaScript ecosystem
- `--python`: Force Python ecosystem
- `--no-symlinks`: Skip symlink creation (install to global store only)
- `--offline`: Use only cached packages (fail if not available)
- `--frozen`: Use exact versions from lock file (CI mode)
- `--json`: Output results in JSON format

### Examples
```bash
# Install all dependencies from project.toml
ppm install

# Install including dev dependencies
ppm install --dev

# Install specific packages
ppm install express@4.18.0 requests

# Python packages only
ppm install --python

# Offline mode (cache only)
ppm install --offline

# CI mode (exact versions)
ppm install --frozen
```

## `ppm add`

Add new dependencies to the project and install them.

### Usage
```bash
ppm add <PACKAGES>... [OPTIONS]
```

### Arguments
- `<PACKAGES>...`: List of packages to add (required)

### Options
- `--save-dev`: Add to dev-dependencies instead of dependencies
- `--javascript`: Force JavaScript ecosystem detection
- `--python`: Force Python ecosystem detection
- `--version <VERSION>`: Specify version constraint for single package
- `--json`: Output results in JSON format

### Examples
```bash
# Add packages (auto-detect ecosystem)
ppm add express lodash

# Add with specific versions
ppm add express@^4.18.0 requests@2.28.0

# Force ecosystem
ppm add @types/node --javascript
ppm add pytest --python

# Add dev dependencies
ppm add jest eslint --save-dev

# Single package with version flag
ppm add express --version "^4.18.0"
```

## `ppm run`

Execute project scripts with proper environment setup.

### Usage
```bash
ppm run [SCRIPT] [OPTIONS] [-- <ARGS>...]
```

### Arguments
- `[SCRIPT]`: Script name from project.toml [scripts] section
- `[ARGS]...`: Additional arguments to pass to the script

### Options
- `--list`: Show available scripts instead of running
- `--env`: Show environment variables that would be set
- `--json`: Output results in JSON format

### Examples
```bash
# Run a script
ppm run build

# Run script with arguments
ppm run test -- --verbose --coverage

# List all available scripts
ppm run --list

# Show environment for script
ppm run start --env

# JSON output
ppm run build --json
```

## `ppm venv`

Manage Python virtual environments for the project.

### Usage
```bash
ppm venv [COMMAND]
```

### Subcommands
- `create`: Create new virtual environment (default)
- `remove`: Remove existing virtual environment
- `info`: Show virtual environment information
- `shell`: Print activation command for current shell (Unix only)

### `ppm venv create`

```bash
ppm venv create [OPTIONS]

Options:
  --python <PYTHON>    Python version to use
  --path <PATH>        Custom path for venv
  --force             Remove existing venv before creating
  --json              Output as JSON
```

### `ppm venv info`

```bash
ppm venv info [OPTIONS]

Options:
  --json              Output as JSON
```

### Examples
```bash
# Create default virtual environment
ppm venv create

# Create with specific Python version
ppm venv create --python python3.11

# Create with custom path
ppm venv create --path ./my-venv

# Force recreate
ppm venv create --force

# Show virtual environment info
ppm venv info

# Remove virtual environment
ppm venv remove

# Get shell activation command (Unix)
ppm venv shell
```

## Environment Variables

PPM sets these environment variables during script execution:

### JavaScript Environment
- `NODE_PATH`: Path to node_modules directory
- `PPM_PROJECT_NAME`: Project name from project.toml
- `PPM_PROJECT_VERSION`: Project version from project.toml
- `PPM_PROJECT_ROOT`: Absolute path to project root

### Python Environment
- `VIRTUAL_ENV`: Path to Python virtual environment
- `PYTHONHOME`: Set to empty to avoid conflicts
- `PATH`: Virtual environment bin/Scripts prepended
- `PPM_PROJECT_NAME`: Project name from project.toml
- `PPM_PROJECT_VERSION`: Project version from project.toml
- `PPM_PROJECT_ROOT`: Absolute path to project root

## Exit Codes

PPM uses these exit codes:

- `0`: Success
- `1`: General error (validation, configuration, etc.)
- `2`: Network error
- `3`: File system error
- `4`: Dependency resolution error
- `5`: Installation error

## Configuration File

PPM uses a `project.toml` file for configuration:

```toml
[project]
name = "my-project"
version = "1.0.0"

[dependencies.javascript]
express = "^4.18.0"
lodash = "^4.17.21"

[dependencies.python]
requests = "^2.28.0"
flask = "^2.2.0"

[dev-dependencies.javascript]
jest = "^29.0.0"
"@types/node" = "^18.0.0"

[dev-dependencies.python]
pytest = "^7.0.0"
black = "^22.0.0"

[scripts]
build = "npm run build"
test = "npm test && python -m pytest"
start = "node server.js"

[venv]
python = "python3.11"
path = ".venv"
```

## Common Patterns

### Initialize JavaScript Project
```bash
ppm init --name my-app --javascript
ppm add express cors helmet
ppm add @types/node jest --save-dev
```

### Initialize Python Project
```bash
ppm init --name my-app --python
ppm add flask sqlalchemy requests
ppm add pytest black mypy --save-dev
ppm venv create
```

### Initialize Polyglot Project
```bash
ppm init --name my-app
ppm add express react --javascript
ppm add flask pandas --python
ppm add jest @types/react --javascript --save-dev
ppm add pytest black --python --save-dev
```

### Development Workflow
```bash
# Install dependencies
ppm install --dev

# Create virtual environment
ppm venv create

# Run development
ppm run dev

# Run tests
ppm run test

# Build for production
ppm run build
```

## Troubleshooting

### Common Issues

1. **No project.toml found**
   ```bash
   ppm init  # Initialize project first
   ```

2. **Package not found**
   ```bash
   # Check package name spelling
   # Verify package exists in registry
   # Try different version specification
   ```

3. **Permission errors**
   ```bash
   # Check file/directory permissions
   # Try running with elevated permissions
   # Ensure write access to project directory
   ```

4. **Network errors**
   ```bash
   # Check internet connection
   # Try offline mode: ppm install --offline
   # Configure proxy if needed
   ```

### Getting Help

- `ppm --help`: General help
- `ppm <command> --help`: Command-specific help
- Check the [full documentation](README.md)
- Visit [GitHub repository](https://github.com/VesperAkshay/polypm)
