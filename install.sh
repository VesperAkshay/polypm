#!/bin/bash
# PPM Installation Script for Linux/macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/VesperAkshay/polypm/main/install.sh | bash

set -e

# Configuration
VERSION="${PPM_VERSION:-latest}"
INSTALL_DIR="${PPM_INSTALL_DIR:-$HOME/.cargo/bin}"
FORCE="${PPM_FORCE:-false}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
info() {
    echo -e "${BLUE}ℹ️ $1${NC}"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

# Utility functions
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

detect_os() {
    case "$(uname -s)" in
        Darwin*)
            echo "macos"
            ;;
        Linux*)
            echo "linux"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

install_rust() {
    info "Rust not found. Installing Rust..."
    
    if command_exists curl; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    elif command_exists wget; then
        wget -qO- https://sh.rustup.rs | sh -s -- -y
    else
        error "Neither curl nor wget found. Please install one of them first."
        return 1
    fi
    
    # Source the cargo environment
    source "$HOME/.cargo/env" 2>/dev/null || true
    export PATH="$HOME/.cargo/bin:$PATH"
    
    if command_exists cargo; then
        success "Rust installed successfully!"
        return 0
    else
        error "Rust installation failed"
        return 1
    fi
}

install_ppm_binary() {
    info "Attempting to install pre-compiled binary..."
    
    local os_type
    local arch
    local binary_name="ppm"
    
    os_type=$(detect_os)
    arch=$(uname -m)
    
    case "$arch" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        *)
            warning "Unsupported architecture: $arch. Only x86_64 is supported."
            return 1
            ;;
    esac
    
    if [ "$os_type" = "unknown" ]; then
        warning "Unsupported OS"
        return 1
    fi
    
    local asset_name
    case "$os_type" in
        linux)
            asset_name="ppm-linux-${arch}"
            ;;
        macos)
            asset_name="ppm-macos-${arch}"
            ;;
        *)
            warning "Unsupported OS for binary installation: $os_type"
            return 1
            ;;
    esac
    
    local download_url="https://github.com/VesperAkshay/polypm/releases/latest/download/${asset_name}"
    local install_path="$INSTALL_DIR/ppm"
    
    info "Downloading from: $download_url"
    
    # Create install directory
    mkdir -p "$(dirname "$install_path")"
    
    # Download binary
    if command_exists curl; then
        if curl -fsSL "$download_url" -o "$install_path"; then
            chmod +x "$install_path"
            success "PPM binary installed successfully!"
            return 0
        fi
    elif command_exists wget; then
        if wget -q "$download_url" -O "$install_path"; then
            chmod +x "$install_path"
            success "PPM binary installed successfully!"
            return 0
        fi
    fi
    
    warning "Failed to download pre-compiled binary"
    return 1
}

install_ppm() {
    # Try binary installation first (no Rust required)
    if install_ppm_binary; then
        return 0
    fi
    
    # Fallback to source compilation
    if ! command_exists cargo; then
        error "Pre-compiled binary not available and Rust/Cargo not found"
        error "Please install Rust from https://rustup.rs/ or wait for binary support"
        return 1
    fi
    
    info "Installing PPM from source..."
    
    if [ "$VERSION" = "latest" ]; then
        info "Installing latest version from crates.io..."
        if cargo install ppm; then
            success "PPM installed successfully!"
            return 0
        else
            warning "Crates.io installation failed. Trying GitHub..."
            if cargo install --git https://github.com/VesperAkshay/polypm; then
                success "PPM installed successfully from GitHub!"
                return 0
            else
                error "Failed to install PPM from both crates.io and GitHub"
                return 1
            fi
        fi
    else
        info "Installing version $VERSION from crates.io..."
        if cargo install ppm --version "$VERSION"; then
            success "PPM $VERSION installed successfully!"
            return 0
        else
            error "Failed to install PPM version $VERSION"
            return 1
        fi
    fi
}

verify_installation() {
    info "Verifying PPM installation..."
    
    if command_exists ppm; then
        local version
        version=$(ppm --version 2>/dev/null || echo "unknown")
        success "PPM $version is installed and working!"
        return 0
    else
        error "PPM installation verification failed"
        return 1
    fi
}

main() {
    cat << 'EOF'

╔═══════════════════════════════════════╗
║       PPM Installer for Unix/Linux    ║
║   Polyglot Package Manager v1.0.0     ║
╚═══════════════════════════════════════╝

EOF

    info "Starting PPM installation..."
    info "Detected OS: $(detect_os)"

    # Check if already installed
    if command_exists ppm && [ "$FORCE" != "true" ]; then
        local existing_version
        existing_version=$(ppm --version 2>/dev/null || echo "unknown")
        warning "PPM is already installed: $existing_version"
        info "Set PPM_FORCE=true to reinstall"
        exit 0
    fi

    # Check for Rust/Cargo (only if binary installation might fail)
    info "Checking installation options..."
    
    # Try to install without Rust first
    if install_ppm; then
        if verify_installation; then
            cat << 'EOF'

🎉 Installation Complete!

PPM has been successfully installed to your system.

Quick Start:
  ppm --help                    # Show help
  ppm init --name my-project    # Create new project
  ppm add react flask           # Add dependencies
  ppm install                   # Install dependencies

Examples:
  • Fullstack webapp: https://github.com/VesperAkshay/polypm/tree/main/examples/fullstack-webapp
  • Data science: https://github.com/VesperAkshay/polypm/tree/main/examples/data-science

Documentation:
  https://github.com/VesperAkshay/polypm#readme

Support:
  https://github.com/VesperAkshay/polypm/issues

EOF

            # Add to shell profile if needed
            if ! command_exists ppm; then
                warning "PPM may not be in your PATH. You might need to restart your shell or run:"
                info "  export PATH=\"$INSTALL_DIR:\$PATH\""
                info "  # Or add the above line to your ~/.bashrc or ~/.zshrc"
            fi
        else
            error "Installation completed but verification failed"
            exit 1
        fi
    else
        error "Failed to install PPM"
        exit 1
    fi
}

# Run main function
main "$@"
