# Contributing to Polis

Thank you for your interest in contributing to Polis! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contributing Process](#contributing-process)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)
- [Release Process](#release-process)
- [Community Guidelines](#community-guidelines)

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold this code.

## Getting Started

### Prerequisites

- **Rust**: 1.70 or later
- **Git**: 2.30 or later
- **Docker**: 20.10 or later (optional)
- **Operating System**: Linux, macOS, or Windows

### Development Setup

1. **Fork the repository**
   ```bash
   # Fork on GitHub, then clone your fork
   git clone https://github.com/your-username/polis.git
   cd polis
   ```

2. **Set up the development environment**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   
   # Install development dependencies
   cargo install cargo-audit cargo-deny cargo-llvm-cov
   ```

3. **Build the project**
   ```bash
   # Build all components
   cargo build
   
   # Run tests
   cargo test
   
   # Run benchmarks
   cargo bench
   ```

4. **Set up pre-commit hooks**
   ```bash
   # Install pre-commit
   pip install pre-commit
   
   # Install hooks
   pre-commit install
   ```

## Contributing Process

### 1. Choose an Issue

- Look for issues labeled `good first issue` or `help wanted`
- Comment on the issue to express interest
- Wait for maintainer approval before starting work

### 2. Create a Branch

```bash
# Create a new branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/your-bug-fix
```

### 3. Make Changes

- Write clean, well-documented code
- Follow the coding standards
- Add tests for new functionality
- Update documentation as needed

### 4. Test Your Changes

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --package polis-core

# Run with coverage
cargo llvm-cov --html

# Run benchmarks
cargo bench
```

### 5. Commit Your Changes

```bash
# Stage changes
git add .

# Commit with descriptive message
git commit -m "feat: add new feature X

- Implemented feature X
- Added tests for feature X
- Updated documentation

Fixes #123"
```

### 6. Push and Create Pull Request

```bash
# Push to your fork
git push origin feature/your-feature-name

# Create pull request on GitHub
```

## Coding Standards

### Rust Code Style

- Use `rustfmt` for formatting
- Use `clippy` for linting
- Follow Rust naming conventions
- Write comprehensive documentation

### Code Organization

- Keep functions small and focused
- Use meaningful variable and function names
- Add comments for complex logic
- Group related functionality in modules

### Error Handling

- Use `Result<T, E>` for fallible operations
- Use `anyhow` for application errors
- Use `thiserror` for library errors
- Provide helpful error messages

### Performance

- Profile before optimizing
- Use appropriate data structures
- Avoid unnecessary allocations
- Consider async/await for I/O operations

## Testing

### Unit Tests

- Write tests for all public functions
- Test edge cases and error conditions
- Use descriptive test names
- Keep tests independent and isolated

### Integration Tests

- Test component interactions
- Test API endpoints
- Test CLI commands
- Test error scenarios

### Performance Tests

- Write benchmarks for critical paths
- Monitor performance regressions
- Use `criterion` for benchmarking
- Document performance characteristics

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

## Documentation

### Code Documentation

- Document all public APIs
- Use rustdoc format
- Provide examples in documentation
- Keep documentation up to date

### User Documentation

- Update user guides for new features
- Add examples and tutorials
- Keep installation instructions current
- Document breaking changes

### API Documentation

- Document all REST endpoints
- Document all gRPC services
- Provide request/response examples
- Include error codes and messages

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

- [ ] All tests pass
- [ ] Documentation is updated
- [ ] Changelog is updated
- [ ] Version numbers are updated
- [ ] Release notes are written
- [ ] Security audit is complete

### Creating a Release

1. **Update version numbers**
   ```bash
   # Update Cargo.toml files
   # Update CHANGELOG.md
   # Update RELEASE_NOTES.md
   ```

2. **Create release branch**
   ```bash
   git checkout -b release/v0.1.1
   git push origin release/v0.1.1
   ```

3. **Create pull request**
   - Review changes
   - Get approval
   - Merge to main

4. **Tag release**
   ```bash
   git tag v0.1.1
   git push origin v0.1.1
   ```

5. **Create GitHub release**
   - Use release notes
   - Attach binaries
   - Announce to community

## Community Guidelines

### Communication

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Discord**: Real-time chat and support
- **Email**: Security issues and private matters

### Getting Help

- Search existing issues and discussions
- Ask questions in Discord
- Create an issue if needed
- Be patient and respectful

### Code Review

- Be constructive and helpful
- Focus on the code, not the person
- Suggest improvements
- Ask questions if unclear

### Recognition

- Contributors are recognized in release notes
- Significant contributors may become maintainers
- Community members are valued and appreciated

## Development Workflow

### Daily Workflow

1. **Sync with upstream**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Create feature branch**
   ```bash
   git checkout -b feature/new-feature
   ```

3. **Make changes and test**
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```

4. **Commit and push**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   git push origin feature/new-feature
   ```

5. **Create pull request**
   - Fill out template
   - Request review
   - Address feedback

### Weekly Workflow

1. **Review open issues**
2. **Plan next week's work**
3. **Update project status**
4. **Participate in community discussions**

### Monthly Workflow

1. **Review project metrics**
2. **Plan major features**
3. **Update documentation**
4. **Release new version**

## Project Structure

```
polis/
├── polis-core/          # Core library
├── polis-runtime/       # Container runtime
├── polis-api/           # REST and gRPC APIs
├── polis-cli/           # Command-line interface
├── polis-image/         # Image management
├── polis-network/       # Networking
├── polis-security/      # Security features
├── polis-storage/       # Storage management
├── polis-orchestrator/  # Orchestration
├── polis-monitor/       # Monitoring
├── polis-auth/          # Authentication
├── polis-benchmarks/    # Performance benchmarks
├── polis-optimization/  # Performance optimization
├── polis-sdk/           # Software development kit
├── polis-tests/         # Integration tests
├── docs/                # Documentation
├── examples/            # Example code
├── scripts/             # Build and utility scripts
└── tests/               # Test suites
```

## Getting Help

### Documentation

- [User Guide](docs/USER_GUIDE.md)
- [API Reference](docs/API_REFERENCE.md)
- [Developer Guide](docs/DEVELOPER_GUIDE.md)
- [Installation Guide](docs/INSTALLATION.md)

### Community

- [GitHub Issues](https://github.com/polis-project/polis/issues)
- [Discord](https://discord.gg/polis)
- [GitHub Discussions](https://github.com/polis-project/polis/discussions)

### Contact

- **General Questions**: [Discord](https://discord.gg/polis)
- **Bug Reports**: [GitHub Issues](https://github.com/polis-project/polis/issues)
- **Security Issues**: [security@polis.dev](mailto:security@polis.dev)
- **Legal Questions**: [legal@polis.dev](mailto:legal@polis.dev)

## Thank You

Thank you for contributing to Polis! Your contributions help make Polis better for everyone.

---

**Happy Contributing!** 