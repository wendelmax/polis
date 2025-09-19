# Polis CI Check Script for Windows
# This script runs all the same checks as the CI pipeline locally

param(
    [switch]$SkipDependencies
)

Write-Host " Polis CI Check Script (Windows)" -ForegroundColor Blue
Write-Host "====================================" -ForegroundColor Blue

# Function to print colored output
function Write-Status {
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

# Check if we're in the right directory
if (-not (Test-Path "Cargo.toml")) {
    Write-Error "Please run this script from the root of the Polis project"
    exit 1
}

# Install dependencies if needed
if (-not $SkipDependencies) {
    Write-Status "Installing dependencies..."
    Write-Warning "Please install libseccomp manually on Windows or use WSL for full functionality"
}

# Check Rust version
Write-Status "Checking Rust version..."
rustc --version
cargo --version

# Format check
Write-Status "Checking code formatting..."
try {
    cargo fmt --all -- --check
    Write-Success "Code formatting is correct"
} catch {
    Write-Error "Code formatting issues found. Run 'cargo fmt --all' to fix."
    exit 1
}

# Clippy check
Write-Status "Running clippy..."
try {
    cargo clippy --all-targets --all-features -- -D warnings
    Write-Success "Clippy checks passed"
} catch {
    Write-Error "Clippy found issues. Please fix them."
    exit 1
}

# Build check
Write-Status "Building project..."
try {
    cargo build --all
    Write-Success "Build successful"
} catch {
    Write-Error "Build failed"
    exit 1
}

# Test check
Write-Status "Running tests..."
try {
    cargo test --all
    Write-Success "All tests passed"
} catch {
    Write-Error "Some tests failed"
    exit 1
}

# Security audit (if cargo-audit is installed)
if (Get-Command cargo-audit -ErrorAction SilentlyContinue) {
    Write-Status "Running security audit..."
    try {
        cargo audit
        Write-Success "Security audit passed"
    } catch {
        Write-Warning "Security audit found issues"
    }
} else {
    Write-Warning "cargo-audit not installed. Install with: cargo install cargo-audit"
}

# Code coverage (if cargo-llvm-cov is installed)
if (Get-Command cargo-llvm-cov -ErrorAction SilentlyContinue) {
    Write-Status "Generating code coverage..."
    try {
        cargo llvm-cov --all --workspace --lcov --output-path lcov.info
        Write-Success "Code coverage generated: lcov.info"
    } catch {
        Write-Warning "Code coverage generation failed"
    }
} else {
    Write-Warning "cargo-llvm-cov not installed. Install with: cargo install cargo-llvm-cov"
}

Write-Success "All checks completed successfully!"
