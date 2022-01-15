#!/bin/bash

# Polis CI Check Script
# This script runs all the same checks as the CI pipeline locally

set -e

echo "🔍 Polis CI Check Script"
echo "========================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the root of the Polis project"
    exit 1
fi

# Install dependencies if needed
print_status "Installing dependencies..."
if command -v apt-get &> /dev/null; then
    sudo apt-get update
    sudo apt-get install -y libseccomp-dev
elif command -v brew &> /dev/null; then
    brew install libseccomp
else
    print_warning "Could not install system dependencies. Please install libseccomp manually."
fi

# Check Rust version
print_status "Checking Rust version..."
rustc --version
cargo --version

# Format check
print_status "Checking code formatting..."
if cargo fmt --all -- --check; then
    print_success "Code formatting is correct"
else
    print_error "Code formatting issues found. Run 'cargo fmt --all' to fix."
    exit 1
fi

# Clippy check
print_status "Running clippy..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    print_success "Clippy checks passed"
else
    print_error "Clippy found issues. Please fix them."
    exit 1
fi

# Build check
print_status "Building project..."
if cargo build --all; then
    print_success "Build successful"
else
    print_error "Build failed"
    exit 1
fi

# Test check
print_status "Running tests..."
if cargo test --all; then
    print_success "All tests passed"
else
    print_error "Some tests failed"
    exit 1
fi

# Security audit (if cargo-audit is installed)
if command -v cargo-audit &> /dev/null; then
    print_status "Running security audit..."
    if cargo audit; then
        print_success "Security audit passed"
    else
        print_warning "Security audit found issues"
    fi
else
    print_warning "cargo-audit not installed. Install with: cargo install cargo-audit"
fi

# Code coverage (if cargo-llvm-cov is installed)
if command -v cargo-llvm-cov &> /dev/null; then
    print_status "Generating code coverage..."
    if cargo llvm-cov --all --workspace --lcov --output-path lcov.info; then
        print_success "Code coverage generated: lcov.info"
    else
        print_warning "Code coverage generation failed"
    fi
else
    print_warning "cargo-llvm-cov not installed. Install with: cargo install cargo-llvm-cov"
fi

print_success "All checks completed successfully! 🎉"
