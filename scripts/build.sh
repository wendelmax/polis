#!/bin/bash

# Polis Build Script
# This script builds the Polis project for different platforms and architectures

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="polis"
VERSION=${VERSION:-"0.1.0"}
BUILD_DIR="target"
RELEASE_DIR="release"
PLATFORMS=("linux-x86_64" "linux-aarch64" "macos-x86_64" "macos-aarch64" "windows-x86_64")
RUST_TARGETS=("x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu" "x86_64-apple-darwin" "aarch64-apple-darwin" "x86_64-pc-windows-gnu")

# Functions
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

# Check if required tools are installed
check_dependencies() {
    log_info "Checking dependencies..."
    
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo is not installed. Please install Rust first."
        exit 1
    fi
    
    if ! command -v rustup &> /dev/null; then
        log_error "Rustup is not installed. Please install Rust first."
        exit 1
    fi
    
    # Check for cross-compilation targets
    for target in "${RUST_TARGETS[@]}"; do
        if ! rustup target list --installed | grep -q "$target"; then
            log_warning "Target $target is not installed. Installing..."
            rustup target add "$target"
        fi
    done
    
    log_success "All dependencies are available"
}

# Clean previous builds
clean_build() {
    log_info "Cleaning previous builds..."
    cargo clean
    rm -rf "$RELEASE_DIR"
    mkdir -p "$RELEASE_DIR"
    log_success "Clean completed"
}

# Build for a specific target
build_target() {
    local target=$1
    local platform=$2
    
    log_info "Building for $platform ($target)..."
    
    # Set environment variables for cross-compilation
    case $target in
        *-pc-windows-gnu)
            export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
            export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++
            ;;
        *-apple-darwin)
            # macOS cross-compilation requires additional setup
            log_warning "macOS cross-compilation may require additional setup"
            ;;
    esac
    
    # Build the project
    cargo build --release --target "$target" --bin polis-cli
    
    # Create platform-specific directory
    local platform_dir="$RELEASE_DIR/$platform"
    mkdir -p "$platform_dir"
    
    # Copy binary
    local binary_name="polis-cli"
    if [[ $target == *-pc-windows-gnu ]]; then
        binary_name="polis-cli.exe"
    fi
    
    cp "$BUILD_DIR/$target/release/$binary_name" "$platform_dir/"
    
    # Copy additional files
    cp README.md "$platform_dir/"
    cp LICENSE "$platform_dir/"
    cp -r docs "$platform_dir/" 2>/dev/null || true
    cp -r examples "$platform_dir/" 2>/dev/null || true
    
    # Create platform-specific README
    cat > "$platform_dir/README.md" << EOF
# Polis $VERSION - $platform

This is a pre-built binary of Polis for $platform.

## Installation

### Linux
\`\`\`bash
# Make executable
chmod +x polis-cli

# Move to PATH
sudo mv polis-cli /usr/local/bin/

# Verify installation
polis-cli --version
\`\`\`

### macOS
\`\`\`bash
# Make executable
chmod +x polis-cli

# Move to PATH
sudo mv polis-cli /usr/local/bin/

# Verify installation
polis-cli --version
\`\`\`

### Windows
1. Add the directory containing \`polis-cli.exe\` to your PATH
2. Open a new command prompt
3. Run \`polis-cli --version\`

## Quick Start

\`\`\`bash
# Start Polis server
polis-cli server start

# Create a container
polis-cli container create --name my-container --image alpine:latest

# List containers
polis-cli container list
\`\`\`

## Documentation

See the \`docs/\` directory for complete documentation.

## License

This software is licensed under the MIT License. See LICENSE for details.
EOF

    log_success "Build completed for $platform"
}

# Build all targets
build_all() {
    log_info "Building for all platforms..."
    
    for i in "${!PLATFORMS[@]}"; do
        local platform="${PLATFORMS[$i]}"
        local target="${RUST_TARGETS[$i]}"
        
        build_target "$target" "$platform"
    done
    
    log_success "All builds completed"
}

# Create release packages
create_packages() {
    log_info "Creating release packages..."
    
    for platform in "${PLATFORMS[@]}"; do
        local platform_dir="$RELEASE_DIR/$platform"
        local package_name="${PROJECT_NAME}-${VERSION}-${platform}"
        
        log_info "Creating package for $platform..."
        
        if [[ $platform == *"windows"* ]]; then
            # Create ZIP for Windows
            cd "$RELEASE_DIR"
            zip -r "${package_name}.zip" "$platform"
            cd ..
        else
            # Create TAR.GZ for Unix-like systems
            cd "$RELEASE_DIR"
            tar -czf "${package_name}.tar.gz" "$platform"
            cd ..
        fi
        
        log_success "Package created: ${package_name}.tar.gz"
    done
    
    # Create checksums
    log_info "Creating checksums..."
    cd "$RELEASE_DIR"
    sha256sum *.tar.gz *.zip > checksums.txt
    cd ..
    
    log_success "Checksums created"
}

# Create Docker images
create_docker_images() {
    log_info "Creating Docker images..."
    
    # Create Dockerfile for multi-arch build
    cat > Dockerfile << EOF
FROM rust:1.70 as builder

WORKDIR /app
COPY . .

# Install dependencies
RUN apt-get update && apt-get install -y \\
    libseccomp-dev \\
    libssl-dev \\
    pkg-config \\
    && rm -rf /var/lib/apt/lists/*

# Build the project
RUN cargo build --release --bin polis-cli

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \\
    libseccomp2 \\
    libssl3 \\
    ca-certificates \\
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/polis-cli /usr/local/bin/

# Create polis user
RUN useradd -r -s /bin/false polis

# Set up directories
RUN mkdir -p /var/lib/polis /var/log/polis /etc/polis \\
    && chown -R polis:polis /var/lib/polis /var/log/polis /etc/polis

# Expose ports
EXPOSE 8080 50051

# Switch to polis user
USER polis

# Set working directory
WORKDIR /var/lib/polis

# Default command
CMD ["polis-cli", "server", "start"]
EOF

    # Build Docker image
    docker build -t polis:latest .
    docker build -t polis:$VERSION .
    
    log_success "Docker images created"
}

# Create installation script
create_install_script() {
    log_info "Creating installation script..."
    
    cat > install.sh << 'EOF'
#!/bin/bash

# Polis Installation Script
# This script installs Polis on various Linux distributions

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
POLIS_VERSION="0.1.0"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/polis"
DATA_DIR="/var/lib/polis"
LOG_DIR="/var/log/polis"

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if [ -f /etc/os-release ]; then
            . /etc/os-release
            OS=$NAME
            VER=$VERSION_ID
        else
            log_error "Cannot detect OS"
            exit 1
        fi
    else
        log_error "Unsupported OS: $OSTYPE"
        exit 1
    fi
}

# Install dependencies
install_dependencies() {
    log_info "Installing dependencies for $OS..."
    
    case $OS in
        "Ubuntu"|"Debian GNU/Linux")
            sudo apt-get update
            sudo apt-get install -y curl wget libseccomp2 libssl3 ca-certificates
            ;;
        "CentOS Linux"|"Red Hat Enterprise Linux")
            sudo yum update -y
            sudo yum install -y curl wget libseccomp openssl ca-certificates
            ;;
        "Fedora")
            sudo dnf update -y
            sudo dnf install -y curl wget libseccomp openssl ca-certificates
            ;;
        "Arch Linux")
            sudo pacman -Syu --noconfirm
            sudo pacman -S --noconfirm curl wget libseccomp openssl ca-certificates
            ;;
        *)
            log_warning "Unknown OS: $OS. Please install dependencies manually."
            ;;
    esac
    
    log_success "Dependencies installed"
}

# Download and install Polis
install_polis() {
    log_info "Installing Polis $POLIS_VERSION..."
    
    # Detect architecture
    ARCH=$(uname -m)
    case $ARCH in
        x86_64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            log_error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac
    
    # Download binary
    DOWNLOAD_URL="https://github.com/polis-project/polis/releases/download/v$POLIS_VERSION/polis-$POLIS_VERSION-linux-$ARCH.tar.gz"
    
    log_info "Downloading from $DOWNLOAD_URL..."
    wget -O polis.tar.gz "$DOWNLOAD_URL"
    
    # Extract and install
    tar -xzf polis.tar.gz
    sudo mv polis-linux-$ARCH/polis-cli $INSTALL_DIR/
    sudo chmod +x $INSTALL_DIR/polis-cli
    
    # Create directories
    sudo mkdir -p $CONFIG_DIR $DATA_DIR $LOG_DIR
    sudo chown -R root:root $CONFIG_DIR
    sudo chown -R polis:polis $DATA_DIR $LOG_DIR 2>/dev/null || true
    
    # Create systemd service
    sudo tee /etc/systemd/system/polis.service > /dev/null << EOF
[Unit]
Description=Polis Container Runtime
After=network.target

[Service]
Type=simple
User=polis
Group=polis
ExecStart=$INSTALL_DIR/polis-cli server start
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

    # Enable service
    sudo systemctl daemon-reload
    sudo systemctl enable polis
    
    # Cleanup
    rm -rf polis.tar.gz polis-linux-$ARCH
    
    log_success "Polis installed successfully"
}

# Main installation
main() {
    log_info "Starting Polis installation..."
    
    detect_os
    install_dependencies
    install_polis
    
    log_success "Installation completed!"
    log_info "To start Polis: sudo systemctl start polis"
    log_info "To check status: sudo systemctl status polis"
    log_info "To view logs: sudo journalctl -u polis -f"
}

main "$@"
EOF

    chmod +x install.sh
    log_success "Installation script created"
}

# Create uninstall script
create_uninstall_script() {
    log_info "Creating uninstall script..."
    
    cat > uninstall.sh << 'EOF'
#!/bin/bash

# Polis Uninstall Script

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Stop and disable service
log_info "Stopping Polis service..."
sudo systemctl stop polis 2>/dev/null || true
sudo systemctl disable polis 2>/dev/null || true

# Remove binary
log_info "Removing Polis binary..."
sudo rm -f /usr/local/bin/polis-cli

# Remove systemd service
log_info "Removing systemd service..."
sudo rm -f /etc/systemd/system/polis.service
sudo systemctl daemon-reload

# Remove directories (with confirmation)
log_warning "This will remove all Polis data and configuration."
read -p "Do you want to remove data directories? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    log_info "Removing data directories..."
    sudo rm -rf /etc/polis /var/lib/polis /var/log/polis
fi

log_success "Polis uninstalled successfully"
EOF

    chmod +x uninstall.sh
    log_success "Uninstall script created"
}

# Create GitHub Actions workflow for releases
create_release_workflow() {
    log_info "Creating GitHub Actions release workflow..."
    
    mkdir -p .github/workflows
    
    cat > .github/workflows/release.yml << 'EOF'
name: Release

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
    
    - name: Install cross-compilation targets
      run: |
        rustup target add x86_64-unknown-linux-gnu
        rustup target add aarch64-unknown-linux-gnu
        rustup target add x86_64-apple-darwin
        rustup target add aarch64-apple-darwin
        rustup target add x86_64-pc-windows-gnu
    
    - name: Install cross-compilation dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y gcc-mingw-w64-x86-64
    
    - name: Build for Linux x86_64
      run: cargo build --release --target x86_64-unknown-linux-gnu --bin polis-cli
    
    - name: Build for Linux aarch64
      run: cargo build --release --target aarch64-unknown-linux-gnu --bin polis-cli
    
    - name: Build for macOS x86_64
      run: cargo build --release --target x86_64-apple-darwin --bin polis-cli
    
    - name: Build for macOS aarch64
      run: cargo build --release --target aarch64-apple-darwin --bin polis-cli
    
    - name: Build for Windows x86_64
      run: cargo build --release --target x86_64-pc-windows-gnu --bin polis-cli
    
    - name: Create release packages
      run: |
        mkdir -p release
        cp target/x86_64-unknown-linux-gnu/release/polis-cli release/polis-linux-x86_64
        cp target/aarch64-unknown-linux-gnu/release/polis-cli release/polis-linux-aarch64
        cp target/x86_64-apple-darwin/release/polis-cli release/polis-macos-x86_64
        cp target/aarch64-apple-darwin/release/polis-cli release/polis-macos-aarch64
        cp target/x86_64-pc-windows-gnu/release/polis-cli.exe release/polis-windows-x86_64.exe
        
        # Create archives
        cd release
        tar -czf polis-linux-x86_64.tar.gz polis-linux-x86_64
        tar -czf polis-linux-aarch64.tar.gz polis-linux-aarch64
        tar -czf polis-macos-x86_64.tar.gz polis-macos-x86_64
        tar -czf polis-macos-aarch64.tar.gz polis-macos-aarch64
        zip polis-windows-x86_64.zip polis-windows-x86_64.exe
        
        # Create checksums
        sha256sum *.tar.gz *.zip > checksums.txt
    
    - name: Create Release
      uses: softprops/action-gh-release@v1
      with:
        files: |
          release/*.tar.gz
          release/*.zip
          release/checksums.txt
        draft: false
        prerelease: false
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
EOF

    log_success "Release workflow created"
}

# Main function
main() {
    log_info "Starting Polis build process..."
    log_info "Version: $VERSION"
    log_info "Build directory: $BUILD_DIR"
    log_info "Release directory: $RELEASE_DIR"
    
    check_dependencies
    clean_build
    build_all
    create_packages
    create_docker_images
    create_install_script
    create_uninstall_script
    create_release_workflow
    
    log_success "Build process completed!"
    log_info "Release packages are available in: $RELEASE_DIR/"
    log_info "Docker images: polis:latest, polis:$VERSION"
    log_info "Installation script: install.sh"
    log_info "Uninstall script: uninstall.sh"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --clean-only)
            clean_build
            exit 0
            ;;
        --docker-only)
            create_docker_images
            exit 0
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --version VERSION    Set version (default: 0.1.0)"
            echo "  --clean-only         Only clean build directory"
            echo "  --docker-only        Only create Docker images"
            echo "  --help               Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

main "$@"
