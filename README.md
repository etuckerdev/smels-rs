# SMELS - AI-Powered Error Log Analyzer

[![CI](https://github.com/etuckerdev/smels-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/etuckerdev/smels-rs/actions/workflows/ci.yml)
[![Security Audit](https://github.com/etuckerdev/smels-rs/actions/workflows/security.yml/badge.svg)](https://github.com/etuckerdev/smels-rs/actions/workflows/security.yml)
[![Crates.io](https://img.shields.io/crates/v/smels.svg)](https://crates.io/crates/smels)
[![Documentation](https://docs.rs/smels/badge.svg)](https://docs.rs/smels)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

SMELS is an intelligent error log analyzer that uses AI to understand and provide actionable insights for errors across multiple programming languages including Rust, JavaScript/TypeScript, Python, and Java.

## ✨ Features

- **Multi-language Support**: Automatically detects and analyzes errors from Rust, JavaScript, Python, Java, and more
- **AI-Powered Analysis**: Uses local AI models (via Ollama) to provide intelligent summaries and root cause analysis
- **Actionable Fixes**: Provides specific, actionable solutions for common error patterns
- **Curated Documentation Links**: Includes relevant documentation links for each error type
- **Multiple Output Formats**: Text, JSON, and interactive web interface
- **Caching System**: Remembers previous analyses for improved performance
- **Privacy-Focused**: All AI processing happens locally - no data sent to external services

## 🚀 Quick Start

### Installation

```bash
cargo install smels
```

### Basic Usage

Analyze an error from stdin:
```bash
echo "thread 'main' panicked at 'called Option::unwrap() on a None value'" | smels analyze
```

Analyze an error from a file:
```bash
smels analyze -f error.log
```

### Examples

**Rust Panic Analysis:**
```bash
echo "thread 'main' panicked at 'called Option::unwrap() on a None value', src/main.rs:10:5" | smels analyze
```

**JavaScript Module Error:**
```bash
echo "Error: Cannot find module 'express'" | smels analyze
```

**Port Conflict:**
```bash
echo "Error: listen EADDRINUSE: address already in use :::3000" | smels analyze
```

## 📖 Usage

### Command Line Options

```bash
smels analyze [OPTIONS] [INPUT]

OPTIONS:
    --ai                    Enable AI-powered analysis (requires Ollama)
    --no-ai                 Disable AI analysis (use deterministic rules only)
    --json                  Output results in JSON format
    --web                   Start web interface on localhost:3001
    --lang <LANGUAGE>       Force specific language detection
    -f <FILE>               Read input from file instead of stdin
    --ai-debug              Enable AI debugging output
```

### Environment Variables

- `SMELS_NO_AI=1` - Disable AI analysis globally
- `SMELS_AI_DEBUG=1` - Enable detailed AI debugging output

### Web Interface

For a more interactive experience, use the web interface:

```bash
echo "your error message here" | smels analyze --web
```

Then open http://localhost:3001 in your browser.

## 🔧 Configuration

SMELS works out of the box with sensible defaults. For advanced usage:

### AI Setup (Optional)

To enable AI-powered analysis:

1. Install [Ollama](https://ollama.ai/)
2. Pull a code-focused model:
   ```bash
   ollama pull qwen2.5-coder:0.5b
   ```
3. Run SMELS with `--ai` flag

### Language Detection

SMELS automatically detects the programming language from error messages. You can override this with `--lang`:

```bash
smels analyze --lang rust < error.log
```

Supported languages: `rust`, `javascript`, `python`, `java`, `generic`

## 🏗️ Architecture

SMELS consists of several key components:

- **Parsers**: Language-specific error parsers that extract structured information
- **Rules Engine**: Deterministic rules for common error patterns
- **AI Analyzer**: Uses local AI models for intelligent analysis
- **Templates**: Output formatting for different interfaces
- **Cache System**: Performance optimization through result caching

## 🧪 Testing & Quality

SMELS maintains high code quality through:

- **Comprehensive Test Suite**: Unit tests, integration tests, and CLI tests
- **Continuous Integration**: Automated testing on multiple Rust versions
- **Security Audits**: Regular dependency vulnerability scanning
- **Code Quality**: Clippy linting and formatting checks
- **Documentation**: Auto-generated docs.rs documentation

## 📊 Sample Output

```
Summary: Rust panic detected - Option::unwrap() called on None value

Root causes:
- Unchecked None value access in Rust code

Fixes:
- Use expect() with descriptive message: some_option.expect("Expected value to be present")
- Use pattern matching: if let Some(value) = some_option { ... } else { handle_none_case() }
- Use unwrap_or() with default: some_option.unwrap_or(default_value)
- Use unwrap_or_else() with lazy default: some_option.unwrap_or_else(|| compute_default())

Related:
- https://doc.rust-lang.org/std/option/struct.Option.html
- https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
```

## 🤝 Contributing

Contributions are welcome! See our [Contributing Guide](CONTRIBUTING.md) for details.

## 🛠️ Development

### Pre-commit Checks

This project includes automated quality checks that run before each commit:

- **Clippy**: Catches common mistakes and suggests improvements
- **Tests**: Ensures all functionality works correctly  
- **Formatting**: Ensures consistent code style

Run checks manually with:
```bash
./check.sh
```

See [PRE_COMMIT_CHECKS.md](PRE_COMMIT_CHECKS.md) for more details.

### Development Setup

```bash
git clone https://github.com/etuckerdev/smels-rs.git
cd smels-rs
cargo build
cargo test
```

### Repository Setup

For first-time setup, see [GitHub Setup Guide](GITHUB_SETUP.md).

### Adding New Language Support

1. Create a new parser in `src/parsers/`
2. Implement the `Parser` trait
3. Add language detection patterns
4. Update the analyzer to use the new parser

## 📄 License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- Built with [Ollama](https://ollama.ai/) for local AI processing
- Inspired by the need for better error analysis tools in development workflows

## 🔍 Troubleshooting

### Common Issues

**"AI unavailable" message:**
- Ensure Ollama is installed and running
- Check that the required model is downloaded
- Verify the model name in the code matches your downloaded model

**Poor language detection:**
- Use `--lang` flag to force specific language
- Check that error message contains language-specific keywords

**Performance issues:**
- Enable caching (enabled by default)
- Use `--no-ai` for faster deterministic analysis

### Getting Help

- Check the [GitHub Issues](https://github.com/etuckerdev/smels-rs/issues) for similar problems
- Create a new issue with your error message and SMELS output

---

**Made with ❤️ for developers who deserve better error messages**
