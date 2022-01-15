# Polis Installation Guide

This guide provides detailed instructions for installing Polis on various operating systems and platforms.

## Table of Contents

- [System Requirements](#system-requirements)
- [Quick Start](#quick-start)
- [Linux Installation](#linux-installation)
- [macOS Installation](#macOS-installation)
- [Windows Installation](#windows-installation)
- [Docker Installation](#docker-installation)
- [Building from Source](#building-from-source)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)
- [Uninstallation](#uninstallation)

## System Requirements

### Minimum Requirements
- **CPU**: 2 cores, 2.0 GHz
- **RAM**: 4 GB
- **Storage**: 10 GB free space
- **OS**: Linux (x86_64, ARM64), macOS (x86_64, ARM64), Windows 10/11 (x86_64, ARM64)

### Recommended Requirements
- **CPU**: 4+ cores, 3.0+ GHz
- **RAM**: 8+ GB
- **Storage**: 50+ GB free space
- **OS**: Latest LTS versions

### Dependencies
- **Rust**: 1.70+ (for building from source)
- **Docker**: 20.10+ (optional, for container support)
- **Git**: 2.30+ (for building from source)

## Quick Start

### Using Package Managers

#### Linux (Ubuntu/Debian)
```bash
# Add Polis repository
curl -fsSL https://apt.polis.dev/gpg | sudo gpg --dearmor -o /usr/share/keyrings/polis-archive-keyring.gpg
echo "deb [arch=amd64 signed-by=/usr/share/keyrings/polis-archive-keyring.gpg] https://apt.polis.dev stable main" | sudo tee /etc/apt/sources.list.d/polis.list

# Install Polis
sudo apt update
sudo apt install polis
```

#### macOS (Homebrew)
```bash
# Add Polis tap
brew tap polis-project/polis

# Install Polis
brew install polis
```

#### Windows (Chocolatey)
```powershell
# Install Polis
choco install polis
```

### Using Pre-built Binaries

#### Linux
```bash
# Download and install
curl -L https://github.com/polis-project/polis/releases/download/v0.1.0/polis-0.1.0-linux-x86_64.tar.gz | tar -xz
sudo mv polis-linux-x86_64/polis-cli /usr/local/bin/
sudo chmod +x /usr/local/bin/polis-cli
```

#### macOS
```bash
# Download and install
curl -L https://github.com/polis-project/polis/releases/download/v0.1.0/polis-0.1.0-macos-x86_64.tar.gz | tar -xz
sudo mv polis-macos-x86_64/polis-cli /usr/local/bin/
sudo chmod +x /usr/local/bin/polis-cli
```

#### Windows
```powershell
# Download and extract
Invoke-WebRequest -Uri "https://github.com/polis-project/polis/releases/download/v0.1.0/polis-0.1.0-windows-x86_64.zip" -OutFile "polis.zip"
Expand-Archive -Path "polis.zip" -DestinationPath "polis"
Move-Item "polis\polis-windows-x86_64\polis-cli.exe" "C:\Program Files\Polis\"
```

## Linux Installation

### Ubuntu/Debian

#### Using APT Repository
```bash
# Add GPG key
curl -fsSL https://apt.polis.dev/gpg | sudo gpg --dearmor -o /usr/share/keyrings/polis-archive-keyring.gpg

# Add repository
echo "deb [arch=amd64 signed-by=/usr/share/keyrings/polis-archive-keyring.gpg] https://apt.polis.dev stable main" | sudo tee /etc/apt/sources.list.d/polis.list

# Update package list
sudo apt update

# Install Polis
sudo apt install polis

# Verify installation
polis-cli --version
```

#### Using Snap
```bash
# Install Polis
sudo snap install polis

# Verify installation
polis-cli --version
```

#### Using Flatpak
```bash
# Add Flathub repository
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

# Install Polis
flatpak install flathub dev.polis.Polis

# Run Polis
flatpak run dev.polis.Polis --version
```

### CentOS/RHEL/Fedora

#### Using YUM/DNF
```bash
# Add Polis repository
sudo tee /etc/yum.repos.d/polis.repo <<EOF
[polis]
name=Polis Repository
baseurl=https://yum.polis.dev/stable/\$basearch
enabled=1
gpgcheck=1
gpgkey=https://yum.polis.dev/gpg
EOF

# Install Polis
sudo yum install polis  # CentOS/RHEL
# or
sudo dnf install polis  # Fedora

# Verify installation
polis-cli --version
```

### Arch Linux

#### Using AUR
```bash
# Install Polis from AUR
yay -S polis
# or
paru -S polis

# Verify installation
polis-cli --version
```

### Alpine Linux

#### Using APK
```bash
# Add Polis repository
echo "https://apk.polis.dev/stable" >> /etc/apk/repositories

# Add GPG key
wget -O - https://apk.polis.dev/gpg | sudo tee /etc/apk/keys/polis.rsa.pub

# Update package index
sudo apk update

# Install Polis
sudo apk add polis

# Verify installation
polis-cli --version
```

## macOS Installation

### Using Homebrew

#### Intel Macs
```bash
# Add Polis tap
brew tap polis-project/polis

# Install Polis
brew install polis

# Verify installation
polis-cli --version
```

#### Apple Silicon Macs
```bash
# Add Polis tap
brew tap polis-project/polis

# Install Polis
brew install polis

# Verify installation
polis-cli --version
```

### Using MacPorts
```bash
# Update MacPorts
sudo port selfupdate

# Install Polis
sudo port install polis

# Verify installation
polis-cli --version
```

### Manual Installation
```bash
# Download for Intel Macs
curl -L https://github.com/polis-project/polis/releases/download/v0.1.0/polis-0.1.0-macos-x86_64.tar.gz | tar -xz

# Download for Apple Silicon Macs
curl -L https://github.com/polis-project/polis/releases/download/v0.1.0/polis-0.1.0-macos-aarch64.tar.gz | tar -xz

# Install
sudo mv polis-macos-*/polis-cli /usr/local/bin/
sudo chmod +x /usr/local/bin/polis-cli

# Verify installation
polis-cli --version
```

## Windows Installation

### Using Chocolatey
```powershell
# Install Polis
choco install polis

# Verify installation
polis-cli --version
```

### Using Scoop
```powershell
# Add Polis bucket
scoop bucket add polis https://github.com/polis-project/polis-scoop

# Install Polis
scoop install polis

# Verify installation
polis-cli --version
```

### Using Winget
```powershell
# Install Polis
winget install PolisProject.Polis

# Verify installation
polis-cli --version
```

### Manual Installation
```powershell
# Download Polis
Invoke-WebRequest -Uri "https://github.com/polis-project/polis/releases/download/v0.1.0/polis-0.1.0-windows-x86_64.zip" -OutFile "polis.zip"

# Extract
Expand-Archive -Path "polis.zip" -DestinationPath "polis"

# Add to PATH
$env:PATH += ";C:\Program Files\Polis"
[Environment]::SetEnvironmentVariable("PATH", $env:PATH, [EnvironmentVariableTarget]::Machine)

# Verify installation
polis-cli --version
```

## Docker Installation

### Using Docker Hub
```bash
# Pull Polis image
docker pull polis/polis:latest

# Run Polis
docker run -d --name polis \
  -p 8080:8080 \
  -p 50051:50051 \
  -v /var/lib/polis:/var/lib/polis \
  polis/polis:latest
```

### Using Docker Compose
```yaml
# docker-compose.yml
version: '3.8'
services:
  polis:
    image: polis/polis:latest
    container_name: polis
    ports:
      - "8080:8080"
      - "50051:50051"
    volumes:
      - polis-data:/var/lib/polis
      - polis-logs:/var/log/polis
    restart: unless-stopped

volumes:
  polis-data:
  polis-logs:
```

```bash
# Start Polis
docker-compose up -d

# Check status
docker-compose ps
```

## Building from Source

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install build dependencies (Linux)
sudo apt update
sudo apt install -y libseccomp-dev libssl-dev pkg-config

# Install build dependencies (macOS)
brew install libseccomp openssl pkg-config

# Install build dependencies (Windows)
# Install Visual Studio Build Tools
# Install vcpkg
```

### Clone and Build
```bash
# Clone repository
git clone https://github.com/polis-project/polis.git
cd polis

# Build Polis
cargo build --release

# Install Polis
sudo cargo install --path .

# Verify installation
polis-cli --version
```

### Cross-compilation
```bash
# Install cross-compilation targets
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup target add x86_64-pc-windows-gnu

# Build for different targets
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-pc-windows-gnu
```

## Verification

### Check Installation
```bash
# Check version
polis-cli --version

# Check help
polis-cli --help

# Check server status
polis-cli server status
```

### Test Basic Functionality
```bash
# Start Polis server
polis-cli server start

# Create a test container
polis-cli container create --name test-container --image alpine:latest

# List containers
polis-cli container list

# Stop container
polis-cli container stop test-container

# Remove container
polis-cli container remove test-container
```

### Check System Integration
```bash
# Check systemd service (Linux)
sudo systemctl status polis

# Check launchd service (macOS)
launchctl list | grep polis

# Check Windows service
sc query polis
```

## Troubleshooting

### Common Issues

#### Permission Denied
```bash
# Linux/macOS
sudo chmod +x /usr/local/bin/polis-cli

# Windows
# Run as Administrator
```

#### Port Already in Use
```bash
# Check what's using the port
sudo netstat -tulpn | grep :8080
sudo netstat -tulpn | grep :50051

# Kill the process
sudo kill -9 <PID>

# Or change the port
polis-cli server start --rest-port 8081 --grpc-port 50052
```

#### Missing Dependencies
```bash
# Linux
sudo apt install libseccomp2 libssl3 ca-certificates

# macOS
brew install libseccomp openssl

# Windows
# Install Visual C++ Redistributable
```

#### Configuration Issues
```bash
# Check configuration
polis-cli config show

# Reset configuration
polis-cli config reset

# Edit configuration
polis-cli config edit
```

### Logs and Debugging
```bash
# View logs
polis-cli logs

# Enable debug logging
RUST_LOG=debug polis-cli server start

# Check system logs
sudo journalctl -u polis -f  # Linux
log show --predicate 'process == "polis"' --last 1h  # macOS
Get-WinEvent -LogName Application | Where-Object {$_.ProviderName -eq "polis"}  # Windows
```

### Getting Help
- [GitHub Issues](https://github.com/polis-project/polis/issues)
- [Discord Community](https://discord.gg/polis)
- [Documentation](https://docs.polis.dev)
- [FAQ](https://docs.polis.dev/faq)

## Uninstallation

### Package Managers

#### Linux (APT)
```bash
sudo apt remove polis
sudo apt autoremove
```

#### Linux (YUM/DNF)
```bash
sudo yum remove polis  # CentOS/RHEL
# or
sudo dnf remove polis  # Fedora
```

#### macOS (Homebrew)
```bash
brew uninstall polis
```

#### Windows (Chocolatey)
```powershell
choco uninstall polis
```

### Manual Uninstallation

#### Linux/macOS
```bash
# Remove binary
sudo rm -f /usr/local/bin/polis-cli

# Remove configuration
rm -rf ~/.config/polis
rm -rf /etc/polis

# Remove data
sudo rm -rf /var/lib/polis
sudo rm -rf /var/log/polis
```

#### Windows
```powershell
# Remove binary
Remove-Item "C:\Program Files\Polis\polis-cli.exe"

# Remove configuration
Remove-Item -Recurse -Force "$env:APPDATA\Polis"

# Remove data
Remove-Item -Recurse -Force "C:\ProgramData\Polis"
```

### Docker
```bash
# Stop and remove container
docker stop polis
docker rm polis

# Remove image
docker rmi polis/polis:latest

# Remove volumes
docker volume rm polis-data polis-logs
```

## Next Steps

After installing Polis, you can:

1. **Read the [User Guide](USER_GUIDE.md)** to learn how to use Polis
2. **Check the [API Reference](API_REFERENCE.md)** for detailed API documentation
3. **Follow the [Tutorials](TUTORIALS.md)** for step-by-step guides
4. **Join the [Community](COMMUNITY.md)** to get help and contribute

## Support

If you encounter any issues during installation:

1. Check the [Troubleshooting](#troubleshooting) section
2. Search [GitHub Issues](https://github.com/polis-project/polis/issues)
3. Ask for help on [Discord](https://discord.gg/polis)
4. Create a new issue with detailed information about your problem

---

**Note**: This installation guide is regularly updated. For the latest information, always refer to the [official documentation](https://docs.polis.dev).
