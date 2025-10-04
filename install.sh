#!/bin/bash

# Whiskerlog Simple Installer
# Supports: install, uninstall

set -e

# Configuration
BINARY_NAME="whiskerlog"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="$HOME/.config/whiskerlog"
DATA_DIR="$HOME/.local/share/whiskerlog"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect system using regex
detect_system() {
    local os arch
    
    # Detect OS
    os=$(uname -s)
    if [[ "$os" =~ ^Linux ]]; then
        OS="linux"
    elif [[ "$os" =~ ^Darwin ]]; then
        OS="macos"
    else
        log_error "Unsupported OS: $os"
        exit 1
    fi
    
    # Detect architecture
    arch=$(uname -m)
    if [[ "$arch" =~ ^(x86_64|amd64)$ ]]; then
        ARCH="x86_64"
    elif [[ "$arch" =~ ^(aarch64|arm64)$ ]]; then
        ARCH="aarch64"
    else
        log_error "Unsupported architecture: $arch"
        exit 1
    fi
    
    log_info "Detected: $OS-$ARCH"
}

# Check if Rust is installed
check_rust() {
    if ! command -v cargo >/dev/null 2>&1; then
        log_error "Rust/Cargo not found. Please install Rust first:"
        log_info "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    local rust_version
    rust_version=$(rustc --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
    log_info "Rust version: $rust_version"
}

# Check permissions
check_permissions() {
    if [[ ! -w "$INSTALL_DIR" ]]; then
        log_error "No write permission to $INSTALL_DIR"
        log_info "Please run with sudo: sudo $0 $1"
        exit 1
    fi
}

# Build and install
install() {
    log_info "Installing Whiskerlog..."
    
    detect_system
    check_permissions
    
    # Check if binary already exists (pre-built)
    if [[ -f "target/release/$BINARY_NAME" ]]; then
        log_info "Using existing binary"
    else
        # Need to build, so check Rust
        check_rust
        log_info "Building release binary..."
        if ! cargo build --release; then
            log_error "Build failed"
            exit 1
        fi
    fi
    
    # Check if binary exists
    if [[ ! -f "target/release/$BINARY_NAME" ]]; then
        log_error "Binary not found. Please build first with: cargo build --release"
        exit 1
    fi
    
    # Install binary
    log_info "Installing to $INSTALL_DIR..."
    cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    
    # Create directories
    mkdir -p "$CONFIG_DIR" "$DATA_DIR"
    
    # Verify installation
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        log_success "Installation complete!"
        log_info "Binary installed at: $INSTALL_DIR/$BINARY_NAME"
        log_info "Run '$BINARY_NAME' to start"
    else
        log_error "Installation verification failed"
        exit 1
    fi
}

# Uninstall
uninstall() {
    log_info "Uninstalling Whiskerlog..."
    
    check_permissions
    
    # Remove binary
    if [[ -f "$INSTALL_DIR/$BINARY_NAME" ]]; then
        rm "$INSTALL_DIR/$BINARY_NAME"
        log_success "Binary removed"
    else
        log_warning "Binary not found"
    fi
    
    # Ask about config/data removal
    echo
    read -p "Remove configuration and data files? [y/N]: " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$CONFIG_DIR" "$DATA_DIR"
        log_success "Configuration and data removed"
    else
        log_info "Configuration preserved at: $CONFIG_DIR"
        log_info "Data preserved at: $DATA_DIR"
    fi
    
    log_success "Uninstallation complete!"
}

# Show help
show_help() {
    cat << EOF
Whiskerlog Simple Installer

USAGE:
    $0 [COMMAND]

COMMANDS:
    install     Build and install Whiskerlog globally
    uninstall   Remove Whiskerlog from system
    help        Show this help

EXAMPLES:
    sudo $0 install      # Install Whiskerlog
    sudo $0 uninstall    # Uninstall Whiskerlog

REQUIREMENTS:
    - Rust/Cargo installed
    - sudo access for global installation
    - Linux or macOS (x86_64 or aarch64)

EOF
}

# Main function
main() {
    case "${1:-help}" in
        install)
            install
            ;;
        uninstall)
            uninstall
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            log_error "Unknown command: $1"
            echo
            show_help
            exit 1
            ;;
    esac
}

# Run main with all arguments
main "$@"