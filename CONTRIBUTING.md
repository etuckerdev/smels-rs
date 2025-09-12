# Contributing to SMELS

Thank you for your interest in contributing to SMELS! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git
- (Optional) Ollama for AI features

### Setup

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/smels-rs.git
   cd smels-rs
   ```

3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/etuckerdev/smels-rs.git
   ```

4. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

5. Build the project:
   ```bash
   cargo build
   ```

6. Run tests:
   ```bash
   cargo test
   ```

## Development Workflow

### 1. Choose an Issue
- Check the [Issues](https://github.com/etuckerdev/smels-rs/issues) page
- Look for issues labeled `good first issue` or `help wanted`
- Comment on the issue to indicate you're working on it

### 2. Make Changes
- Write clear, concise commit messages
- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed

### 3. Testing
- Run the full test suite: `cargo test --all-features`
- Check code formatting: `cargo fmt --all -- --check`
- Run clippy: `cargo clippy -- -D warnings`
- Test manually with sample error logs

### 4. Submit a Pull Request
- Push your changes to your fork
- Create a Pull Request with a clear description
- Reference any related issues
- Ensure CI checks pass

## Code Guidelines

### Rust Style
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for consistent formatting
- Use `cargo clippy` for linting
- Write comprehensive documentation comments

### Error Handling
- Use `Result` and `Option` appropriately
- Provide meaningful error messages
- Avoid `unwrap()` in production code

### Testing
- Write unit tests for all public functions
- Include integration tests for CLI functionality
- Test edge cases and error conditions
- Aim for good test coverage

## Adding New Language Support

To add support for a new programming language:

1. Create a new parser in `src/parsers/`
2. Implement the `Parser` trait
3. Add language detection patterns
4. Update the analyzer to use the new parser
5. Add tests for the new parser
6. Update documentation

Example:
```rust
pub struct NewLangParser;

impl Parser for NewLangParser {
    fn parse(&self, input: &str) -> Vec<ErrorInfo> {
        // Implementation here
    }
}
```

## Feature Requests

- Open an issue with the `enhancement` label
- Describe the problem you're trying to solve
- Explain why the feature would be valuable
- Consider implementation complexity

## Bug Reports

- Use the bug report template
- Include steps to reproduce
- Provide sample error logs
- Include your environment details

## Documentation

- Update README.md for user-facing changes
- Add doc comments for public APIs
- Update examples and usage instructions

## Release Process

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Create a git tag: `git tag v1.2.3`
4. Push the tag: `git push origin v1.2.3`
5. GitHub Actions will handle the release

## Communication

- Use GitHub Issues for bugs and features
- Join discussions in Pull Request comments
- Be respectful and constructive in all interactions

## License

By contributing to SMELS, you agree that your contributions will be licensed under the Apache License 2.0.

Thank you for contributing to SMELS! 🎉
