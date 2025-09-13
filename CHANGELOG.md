# Changelog

All notable changes to SMELS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of SMELS - Small Model Error Log Summarizer
- Multi-language support (Rust, JavaScript, Python, Java, Go, C++, C#)
- Small model analysis using Ollama
- Web interface with modern UI
- CLI tool with multiple output formats (text, JSON, HTML, Markdown)
- Caching system for performance optimization
- Comprehensive test suite
- Documentation and examples

### Features
- Automatic language detection
- Curated documentation links
- Port conflict detection and resolution
- Stack trace analysis
- Error pattern recognition
- Multiple output formats with templates

## [0.1.0] - 2025-09-12

### Added
- Core analyzer functionality
- Parser trait system
- Basic rule engine
- Generic parser with language detection
- Rust parser implementation
- JavaScript/TypeScript parser
- Python parser
- Java parser
- Go parser
- C++ and C# parser stubs
- Common rules for error patterns
- AI integration with Ollama
- Web server with REST API
- Modern web interface
- CLI with comprehensive options
- Multiple output formatters
- Caching system
- Comprehensive documentation
- Test suite with unit and integration tests
- Apache 2.0 license

### Technical Details
- Built with Rust 2021 edition
- Async runtime with Tokio
- Web framework with Axum
- Serialization with Serde
- Regex parsing
- Feature flags for optional dependencies
- Cross-platform support
