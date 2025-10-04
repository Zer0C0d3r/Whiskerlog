# Contributing to Whiskerlog

Thank you for your interest in contributing to Whiskerlog! 🎉

This document provides guidelines and information for contributors to help make the development process smooth and effective.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contributing Process](#contributing-process)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)
- [Performance Considerations](#performance-considerations)
- [Security Guidelines](#security-guidelines)
- [Release Process](#release-process)

## Code of Conduct

This project adheres to a code of conduct that we expect all contributors to follow. Please be respectful, inclusive, and constructive in all interactions.

### Our Standards

- **Be respectful**: Treat everyone with respect and kindness
- **Be inclusive**: Welcome newcomers and help them get started
- **Be constructive**: Provide helpful feedback and suggestions
- **Be patient**: Remember that everyone has different experience levels
- **Be collaborative**: Work together towards common goals

## Getting Started

### Prerequisites

- **Rust**: Install the latest stable Rust toolchain from [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **Make**: For using the provided Makefile (optional but recommended)

### Quick Setup

1. **Fork and clone the repository**:
   ```bash
   git clone https://github.com/your-username/whiskerlog.git
   cd whiskerlog
   ```

2. **Set up the development environment**:
   ```bash
   make setup
   ```

3. **Build and test**:
   ```bash
   make build
   make test
   ```

4. **Run the application**:
   ```bash
   make dev
   ```

## Development Setup

### Development Tools

The project uses several development tools that can be installed with:

```bash
make setup
```

This installs:
- `clippy` - Rust linter
- `rustfmt` - Code formatter
- `cargo-watch` - File watcher for development
- `cargo-audit` - Security vulnerability scanner
- `cargo-outdated` - Dependency update checker
- `cargo-tarpaulin` - Code coverage tool

### Cross-Compilation Setup

For building on multiple platforms:

```bash
make setup-cross
```

### IDE Configuration

#### Visual Studio Code
Recommended extensions:
- `rust-analyzer` - Rust language server
- `CodeLLDB` - Debugger
- `Better TOML` - TOML syntax highlighting
- `Error Lens` - Inline error display

#### Other IDEs
The project works well with any IDE that supports Rust through the Language Server Protocol (LSP).

## Contributing Process

### 1. Issue First

Before starting work on a significant change:
1. Check existing issues to avoid duplication
2. Create a new issue describing the problem or feature
3. Discuss the approach with maintainers
4. Wait for approval before starting implementation

### 2. Branch Strategy

- Create feature branches from `main`
- Use descriptive branch names: `feature/package-analysis`, `fix/memory-leak`, `docs/api-reference`
- Keep branches focused on a single feature or fix

### 3. Development Workflow

1. **Create a branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**:
   - Write code following our [coding standards](#coding-standards)
   - Add tests for new functionality
   - Update documentation as needed

3. **Test your changes**:
   ```bash
   make pre-commit  # Runs formatting, linting, and tests
   ```

4. **Commit your changes**:
   ```bash
   git add .
   git commit -m "feat: add package analysis feature"
   ```

5. **Push and create a pull request**:
   ```bash
   git push origin feature/your-feature-name
   ```

### 4. Pull Request Guidelines

- Use the provided PR template
- Write clear, descriptive titles and descriptions
- Reference related issues using `Fixes #123` or `Closes #123`
- Ensure all CI checks pass
- Request review from maintainers
- Be responsive to feedback and suggestions

## Coding Standards

### Rust Style

- Follow the official [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `cargo fmt` to format code automatically
- Run `cargo clippy` to catch common issues
- Prefer explicit types when it improves readability
- Use meaningful variable and function names

### Code Organization

```
src/
├── main.rs          # Application entry point
├── lib.rs           # Library exports
├── app.rs           # Main application state
├── config/          # Configuration management
├── db/              # Database operations
├── history/         # Command history parsing
├── analysis/        # Command analysis modules
└── ui/              # User interface components
```

### Error Handling

- Use `anyhow::Result` for application errors
- Use `thiserror` for custom error types
- Provide meaningful error messages
- Handle errors gracefully in the UI

### Documentation

- Document all public APIs with rustdoc comments
- Include examples in documentation
- Keep README.md up to date
- Document complex algorithms and business logic

### Performance

- Profile performance-critical code
- Use appropriate data structures
- Minimize allocations in hot paths
- Consider memory usage for large datasets

## Testing

### Test Categories

1. **Unit Tests**: Test individual functions and modules
2. **Integration Tests**: Test component interactions
3. **End-to-End Tests**: Test complete workflows

### Running Tests

```bash
# Run all tests
make test

# Run tests with coverage
make test-coverage

# Run specific test categories
make test-unit
make test-integration
```

### Writing Tests

- Write tests for all new functionality
- Test both success and error cases
- Use descriptive test names
- Keep tests focused and independent
- Mock external dependencies

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name_should_behavior() {
        // Arrange
        let input = create_test_input();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
}
```

## Documentation

### Types of Documentation

1. **Code Documentation**: Rustdoc comments in source code
2. **User Documentation**: README.md and user guides
3. **Developer Documentation**: This file and technical docs
4. **API Documentation**: Generated from rustdoc comments

### Documentation Standards

- Use clear, concise language
- Include examples where helpful
- Keep documentation up to date with code changes
- Use proper markdown formatting

### Building Documentation

```bash
make doc        # Build documentation
make doc-open   # Build and open documentation
```

## Performance Considerations

### Profiling

- Use `cargo bench` for benchmarking
- Profile with `perf` on Linux or Instruments on macOS
- Monitor memory usage with `valgrind` or similar tools

### Optimization Guidelines

- Measure before optimizing
- Focus on algorithmic improvements first
- Use appropriate data structures
- Consider memory vs. CPU trade-offs
- Profile in release mode

### Benchmarking

```bash
make bench  # Run benchmarks
```

## Security Guidelines

### Security Practices

- Never commit secrets or credentials
- Validate all user input
- Use secure defaults
- Follow the principle of least privilege
- Keep dependencies up to date

### Security Tools

```bash
make audit      # Run security audit
make outdated   # Check for outdated dependencies
```

### Reporting Security Issues

Please report security vulnerabilities privately to the maintainers rather than creating public issues.

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):
- `MAJOR.MINOR.PATCH`
- Major: Breaking changes
- Minor: New features (backward compatible)
- Patch: Bug fixes (backward compatible)

### Release Checklist

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run full test suite
4. Create and push git tag
5. GitHub Actions handles the rest automatically

## Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and discussions
- **Pull Request Reviews**: Code-specific discussions

### Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Ratatui Documentation](https://ratatui.rs/)
- [Project Documentation](https://docs.rs/whiskerlog)

## Recognition

Contributors are recognized in several ways:
- Listed in `CONTRIBUTORS.md`
- Mentioned in release notes
- GitHub contributor statistics
- Special recognition for significant contributions

Thank you for contributing to Whiskerlog! Your efforts help make command line analysis better for everyone. 🚀