# PPM Installation Guide

PPM (Polyglot Package Manager) can be installed in multiple ways depending on your system and preferences.

## Quick Install

### Method 1: Install from Crates.io (Recommended)

```bash
# Install PPM globally using Cargo
cargo install ppm

# Verify installation
ppm --version
```

### Method 2: Install from GitHub

```bash
# Install directly from the GitHub repository
cargo install --git https://github.com/VesperAkshay/polypm

# Verify installation
ppm --version
```

### Method 3: Build from Source

```bash
# Clone the repository
git clone https://github.com/VesperAkshay/polypm.git
cd polypm

# Build and install
cargo build --release
cargo install --path .

# Verify installation
ppm --version
```

## System Requirements

- **Rust**: 1.75 or later
- **Python**: 3.8 or later (for Python package management)
- **Node.js**: 14 or later (for JavaScript package management)

## Platform Support

- ✅ **Windows** (Windows 10+)
- ✅ **macOS** (macOS 10.15+)
- ✅ **Linux** (Ubuntu 18.04+, CentOS 7+, Arch Linux)

## Post-Installation Setup

1. **Verify Installation**:
   ```bash
   ppm --version
   ppm --help
   ```

2. **Initialize Your First Project**:
   ```bash
   mkdir my-project
   cd my-project
   ppm init --name my-project
   ```

3. **Add Dependencies**:
   ```bash
   # Add JavaScript packages
   ppm add react express

   # Add Python packages  
   ppm add flask requests

   # Install all dependencies
   ppm install
   ```

## Troubleshooting

### Common Issues

**1. `cargo: command not found`**
- Install Rust and Cargo from [rustup.rs](https://rustup.rs/)

**2. `ppm: command not found` after installation**
- Ensure `~/.cargo/bin` is in your PATH
- On Windows: Add `%USERPROFILE%\.cargo\bin` to your PATH

**3. Python/Node.js not found**
- Install Python from [python.org](https://python.org)
- Install Node.js from [nodejs.org](https://nodejs.org)

### Getting Help

- 📖 **Documentation**: [GitHub README](https://github.com/VesperAkshay/polypm#readme)
- 🐛 **Issues**: [GitHub Issues](https://github.com/VesperAkshay/polypm/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/VesperAkshay/polypm/discussions)

## Uninstallation

```bash
# Remove PPM
cargo uninstall ppm

# Remove PPM data (optional)
rm -rf ~/.ppm-store  # Linux/macOS
rmdir /s %USERPROFILE%\.ppm-store  # Windows
```

---

**Next Steps**: Check out the [Quick Start Guide](README.md#quick-start) to start using PPM!
