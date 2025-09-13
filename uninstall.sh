#!/bin/bash
# PPM Uninstall Script for Linux/macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/VesperAkshay/polypm/main/uninstall.sh | bash
# Or: bash uninstall.sh

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

cat << 'EOF'

╔═══════════════════════════════════════╗
║      PPM Uninstaller for Unix/Linux   ║
║    Polyglot Package Manager v1.0.0    ║
╚═══════════════════════════════════════╝

EOF

info "Starting PPM uninstallation..."

# Check if PPM is installed
if ! command_exists ppm; then
    warning "PPM is not found in your PATH"
    info "It may already be uninstalled or installed in a custom location"
else
    PPM_PATH=$(which ppm)
    info "Found PPM at: $PPM_PATH"
    
    # Remove the executable
    if rm "$PPM_PATH" 2>/dev/null; then
        success "Removed PPM executable"
    else
        error "Failed to remove PPM executable"
        info "You may need to run this script with sudo"
        exit 1
    fi
fi

# Check for cargo bin in PATH
if echo "$PATH" | grep -q ".cargo/bin"; then
    info "Note: .cargo/bin is still in your PATH (may be used by other Rust tools)"
    info "If you don't use Rust, you can remove it from your shell profile"
fi

# Check for PPM project files in current directory
if [ -f "project.toml" ]; then
    warning "Found project.toml in current directory"
    echo -n "Would you like to keep PPM project files? (Y/n): "
    read -r response
    case "$response" in
        [nN][oO]|[nN])
            if [ -d "node_modules" ]; then
                info "Removing node_modules..."
                rm -rf node_modules
            fi
            if [ -d ".venv" ]; then
                info "Removing Python virtual environment..."
                rm -rf .venv
            fi
            if [ -f "ppm.lock" ]; then
                info "Removing lock file..."
                rm -f ppm.lock
            fi
            success "Removed PPM project files"
            ;;
        *)
            info "Keeping PPM project files"
            ;;
    esac
fi

cat << 'EOF'

🎉 PPM Uninstallation Complete!

What was removed:
  ✅ PPM executable
  ✅ Project files (if requested)

Manual cleanup (if needed):
  • Remove .cargo/bin from PATH if you don't use Rust
  • Delete any remaining PPM project directories
  • Remove global packages cache: ~/.ppm/

Thank you for using PPM!

EOF
