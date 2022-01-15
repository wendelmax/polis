# Polis Build Script for Windows
# This script builds the Polis project for Windows platforms

param(
    [string]$Version = "0.1.0",
    [switch]$CleanOnly,
    [switch]$DockerOnly,
    [switch]$Help
)

# Configuration
$ProjectName = "polis"
$BuildDir = "target"
$ReleaseDir = "release"
$Platforms = @("windows-x86_64", "windows-aarch64")
$RustTargets = @("x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc")

# Functions
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Check if required tools are installed
function Test-Dependencies {
    Write-Info "Checking dependencies..."
    
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Error "Cargo is not installed. Please install Rust first."
        exit 1
    }
    
    if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
        Write-Error "Rustup is not installed. Please install Rust first."
        exit 1
    }
    
    # Check for cross-compilation targets
    foreach ($target in $RustTargets) {
        $installed = rustup target list --installed | Select-String $target
        if (-not $installed) {
            Write-Warning "Target $target is not installed. Installing..."
            rustup target add $target
        }
    }
    
    Write-Success "All dependencies are available"
}

# Clean previous builds
function Clear-Build {
    Write-Info "Cleaning previous builds..."
    cargo clean
    if (Test-Path $ReleaseDir) {
        Remove-Item -Recurse -Force $ReleaseDir
    }
    New-Item -ItemType Directory -Path $ReleaseDir | Out-Null
    Write-Success "Clean completed"
}

# Build for a specific target
function Build-Target {
    param(
        [string]$Target,
        [string]$Platform
    )
    
    Write-Info "Building for $Platform ($Target)..."
    
    # Build the project
    cargo build --release --target $Target --bin polis-cli
    
    # Create platform-specific directory
    $platformDir = Join-Path $ReleaseDir $Platform
    New-Item -ItemType Directory -Path $platformDir -Force | Out-Null
    
    # Copy binary
    $binaryName = "polis-cli.exe"
    $sourcePath = Join-Path $BuildDir $Target "release" $binaryName
    $destPath = Join-Path $platformDir $binaryName
    Copy-Item $sourcePath $destPath
    
    # Copy additional files
    Copy-Item "README.md" $platformDir
    Copy-Item "LICENSE" $platformDir
    if (Test-Path "docs") {
        Copy-Item -Recurse "docs" $platformDir
    }
    if (Test-Path "examples") {
        Copy-Item -Recurse "examples" $platformDir
    }
    
    # Create platform-specific README
    $readmeContent = @"
# Polis $Version - $Platform

This is a pre-built binary of Polis for $Platform.

## Installation

1. Add the directory containing `polis-cli.exe` to your PATH
2. Open a new command prompt
3. Run `polis-cli --version`

## Quick Start

``````cmd
# Start Polis server
polis-cli server start

# Create a container
polis-cli container create --name my-container --image alpine:latest

# List containers
polis-cli container list
``````

## Documentation

See the `docs\` directory for complete documentation.

## License

This software is licensed under the MIT License. See LICENSE for details.
"@
    
    $readmePath = Join-Path $platformDir "README.md"
    Set-Content -Path $readmePath -Value $readmeContent
    
    Write-Success "Build completed for $Platform"
}

# Build all targets
function Build-All {
    Write-Info "Building for all platforms..."
    
    for ($i = 0; $i -lt $Platforms.Count; $i++) {
        $platform = $Platforms[$i]
        $target = $RustTargets[$i]
        
        Build-Target -Target $target -Platform $platform
    }
    
    Write-Success "All builds completed"
}

# Create release packages
function New-Packages {
    Write-Info "Creating release packages..."
    
    foreach ($platform in $Platforms) {
        $platformDir = Join-Path $ReleaseDir $platform
        $packageName = "$ProjectName-$Version-$platform"
        
        Write-Info "Creating package for $platform..."
        
        # Create ZIP for Windows
        $zipPath = Join-Path $ReleaseDir "$packageName.zip"
        Compress-Archive -Path $platformDir -DestinationPath $zipPath -Force
        
        Write-Success "Package created: $packageName.zip"
    }
    
    # Create checksums
    Write-Info "Creating checksums..."
    $checksumPath = Join-Path $ReleaseDir "checksums.txt"
    Get-ChildItem -Path $ReleaseDir -Filter "*.zip" | ForEach-Object {
        $hash = Get-FileHash $_.FullName -Algorithm SHA256
        "$($hash.Hash)  $($_.Name)" | Add-Content $checksumPath
    }
    
    Write-Success "Checksums created"
}

# Create Docker images
function New-DockerImages {
    Write-Info "Creating Docker images..."
    
    # Create Dockerfile for Windows
    $dockerfileContent = @"
FROM mcr.microsoft.com/windows/servercore:ltsc2022

# Install Rust
RUN powershell -Command \
    Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile 'rustup-init.exe'; \
    .\rustup-init.exe -y; \
    Remove-Item rustup-init.exe

# Set up environment
ENV PATH="C:\Users\ContainerUser\.cargo\bin;${PATH}"

# Install dependencies
RUN powershell -Command \
    Invoke-WebRequest -Uri 'https://github.com/microsoft/vcpkg/releases/download/2023.04.15/vcpkg.exe' -OutFile 'vcpkg.exe'; \
    .\vcpkg.exe integrate install

WORKDIR /app
COPY . .

# Build the project
RUN cargo build --release --bin polis-cli

# Copy binary to final location
RUN copy target\release\polis-cli.exe C:\polis-cli.exe

# Expose ports
EXPOSE 8080 50051

# Default command
CMD ["C:\\polis-cli.exe", "server", "start"]
"@
    
    Set-Content -Path "Dockerfile.windows" -Value $dockerfileContent
    
    # Build Docker image
    docker build -f Dockerfile.windows -t polis:latest-windows .
    docker build -f Dockerfile.windows -t polis:$Version-windows .
    
    Write-Success "Docker images created"
}

# Create installation script for Windows
function New-InstallScript {
    Write-Info "Creating Windows installation script..."
    
    $installScript = @"
@echo off
REM Polis Installation Script for Windows
REM This script installs Polis on Windows

setlocal enabledelayedexpansion

set POLIS_VERSION=0.1.0
set INSTALL_DIR=%ProgramFiles%\Polis
set CONFIG_DIR=%ProgramData%\Polis
set DATA_DIR=%ProgramData%\Polis\data
set LOG_DIR=%ProgramData%\Polis\logs

echo [INFO] Starting Polis installation...

REM Create directories
echo [INFO] Creating directories...
mkdir "%INSTALL_DIR%" 2>nul
mkdir "%CONFIG_DIR%" 2>nul
mkdir "%DATA_DIR%" 2>nul
mkdir "%LOG_DIR%" 2>nul

REM Download and install Polis
echo [INFO] Installing Polis %POLIS_VERSION%...

REM Detect architecture
if "%PROCESSOR_ARCHITECTURE%"=="AMD64" (
    set ARCH=x86_64
) else if "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
    set ARCH=aarch64
) else (
    echo [ERROR] Unsupported architecture: %PROCESSOR_ARCHITECTURE%
    exit /b 1
)

REM Download binary
set DOWNLOAD_URL=https://github.com/polis-project/polis/releases/download/v%POLIS_VERSION%/polis-%POLIS_VERSION%-windows-%ARCH%.zip
echo [INFO] Downloading from %DOWNLOAD_URL%...

powershell -Command "Invoke-WebRequest -Uri '%DOWNLOAD_URL%' -OutFile 'polis.zip'"

REM Extract and install
powershell -Command "Expand-Archive -Path 'polis.zip' -DestinationPath 'polis-temp' -Force"
copy "polis-temp\polis-windows-%ARCH%\polis-cli.exe" "%INSTALL_DIR%\"
rmdir /s /q "polis-temp"
del "polis.zip"

REM Add to PATH
echo [INFO] Adding to PATH...
setx PATH "%PATH%;%INSTALL_DIR%" /M

REM Create Windows service
echo [INFO] Creating Windows service...
sc create "Polis" binPath="%INSTALL_DIR%\polis-cli.exe server start" start=auto

REM Start service
echo [INFO] Starting Polis service...
sc start "Polis"

echo [SUCCESS] Polis installed successfully!
echo [INFO] To check status: sc query Polis
echo [INFO] To start service: sc start Polis
echo [INFO] To stop service: sc stop Polis

pause
"@
    
    Set-Content -Path "install.bat" -Value $installScript
    Write-Success "Windows installation script created"
}

# Create uninstall script for Windows
function New-UninstallScript {
    Write-Info "Creating Windows uninstall script..."
    
    $uninstallScript = @"
@echo off
REM Polis Uninstall Script for Windows

echo [INFO] Stopping Polis service...
sc stop "Polis" 2>nul
sc delete "Polis" 2>nul

echo [INFO] Removing Polis binary...
del "%ProgramFiles%\Polis\polis-cli.exe" 2>nul
rmdir "%ProgramFiles%\Polis" 2>nul

echo [WARNING] This will remove all Polis data and configuration.
set /p CONFIRM="Do you want to remove data directories? (y/N): "
if /i "%CONFIRM%"=="y" (
    echo [INFO] Removing data directories...
    rmdir /s /q "%ProgramData%\Polis" 2>nul
)

echo [SUCCESS] Polis uninstalled successfully!
pause
"@
    
    Set-Content -Path "uninstall.bat" -Value $uninstallScript
    Write-Success "Windows uninstall script created"
}

# Create GitHub Actions workflow for Windows releases
function New-ReleaseWorkflow {
    Write-Info "Creating GitHub Actions release workflow..."
    
    $workflowDir = ".github\workflows"
    if (-not (Test-Path $workflowDir)) {
        New-Item -ItemType Directory -Path $workflowDir | Out-Null
    }
    
    $workflowContent = @"
name: Release

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build-windows:
    runs-on: windows-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
    
    - name: Install cross-compilation targets
      run: |
        rustup target add x86_64-pc-windows-msvc
        rustup target add aarch64-pc-windows-msvc
    
    - name: Build for Windows x86_64
      run: cargo build --release --target x86_64-pc-windows-msvc --bin polis-cli
    
    - name: Build for Windows aarch64
      run: cargo build --release --target aarch64-pc-windows-msvc --bin polis-cli
    
    - name: Create release packages
      run: |
        mkdir release
        copy target\x86_64-pc-windows-msvc\release\polis-cli.exe release\polis-windows-x86_64.exe
        copy target\aarch64-pc-windows-msvc\release\polis-cli.exe release\polis-windows-aarch64.exe
        
        # Create archives
        cd release
        powershell -Command "Compress-Archive -Path polis-windows-x86_64.exe -DestinationPath polis-windows-x86_64.zip"
        powershell -Command "Compress-Archive -Path polis-windows-aarch64.exe -DestinationPath polis-windows-aarch64.zip"
        
        # Create checksums
        powershell -Command "Get-ChildItem -Filter '*.zip' | ForEach-Object { Get-FileHash `$_.FullName -Algorithm SHA256 | ForEach-Object { \"`$(`$_.Hash)  `$(`$_.Name)\" } } | Out-File -FilePath checksums.txt -Encoding ASCII"
    
    - name: Create Release
      uses: softprops/action-gh-release@v1
      with:
        files: |
          release/*.zip
          release/checksums.txt
        draft: false
        prerelease: false
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
"@
    
    $workflowPath = Join-Path $workflowDir "release-windows.yml"
    Set-Content -Path $workflowPath -Value $workflowContent
    Write-Success "Windows release workflow created"
}

# Main function
function Main {
    Write-Info "Starting Polis build process..."
    Write-Info "Version: $Version"
    Write-Info "Build directory: $BuildDir"
    Write-Info "Release directory: $ReleaseDir"
    
    Test-Dependencies
    Clear-Build
    Build-All
    New-Packages
    New-DockerImages
    New-InstallScript
    New-UninstallScript
    New-ReleaseWorkflow
    
    Write-Success "Build process completed!"
    Write-Info "Release packages are available in: $ReleaseDir\"
    Write-Info "Docker images: polis:latest-windows, polis:$Version-windows"
    Write-Info "Installation script: install.bat"
    Write-Info "Uninstall script: uninstall.bat"
}

# Parse command line arguments
if ($Help) {
    Write-Host "Usage: .\build.ps1 [OPTIONS]"
    Write-Host "Options:"
    Write-Host "  -Version VERSION    Set version (default: 0.1.0)"
    Write-Host "  -CleanOnly         Only clean build directory"
    Write-Host "  -DockerOnly        Only create Docker images"
    Write-Host "  -Help              Show this help message"
    exit 0
}

if ($CleanOnly) {
    Clear-Build
    exit 0
}

if ($DockerOnly) {
    New-DockerImages
    exit 0
}

Main
