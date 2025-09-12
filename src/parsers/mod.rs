pub mod cpp;
pub mod csharp;
pub mod go;
pub mod java;
pub mod js;
pub mod python;
pub mod rust;

use crate::{ErrorInfo, Parser};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

// Re-export all parsers for easy access
pub use cpp::CppParser;
pub use csharp::CsharpParser;
pub use go::GoParser;
pub use java::JavaParser;
pub use js::JsParser;
pub use python::PythonParser;
pub use rust::RustParser;

lazy_static! {
    static ref GENERIC_PATTERNS: HashMap<&'static str, (&'static str, &'static str)> = {
        let mut m = HashMap::new();

        // Network/System errors (cross-platform)
        m.insert(r"Connection refused", ("Network error", "generic"));
        m.insert(r"Permission denied", ("Permission error", "generic"));
        m.insert(r"No such file or directory", ("File not found", "generic"));
        m.insert(r"Address already in use", ("Port conflict", "generic"));
        m.insert(r"Timeout", ("Operation timed out", "generic"));
        m.insert(r"Out of memory", ("Memory allocation failed", "generic"));

        // Build tool errors
        m.insert(r"fatal: not a git repository", ("Git repository error", "git"));
        m.insert(r"Docker.*error", ("Docker operation failed", "docker"));
        m.insert(r"kubernetes.*error", ("Kubernetes error", "k8s"));
        m.insert(r"terraform.*error", ("Terraform infrastructure error", "terraform"));

        // Database errors
        m.insert(r"connection.*database.*failed", ("Database connection failed", "database"));
        m.insert(r"syntax error.*SQL", ("SQL syntax error", "database"));
        m.insert(r"table.*does not exist", ("Database table missing", "database"));

        // SSL/TLS errors
        m.insert(r"certificate.*invalid", ("SSL certificate error", "tls"));
        m.insert(r"SSL.*handshake.*failed", ("SSL handshake failed", "tls"));
        m.insert(r"TLS.*error", ("TLS connection error", "tls"));

        // Authentication errors
        m.insert(r"authentication.*failed", ("Authentication failed", "auth"));
        m.insert(r"unauthorized", ("Authorization denied", "auth"));
        m.insert(r"invalid.*token", ("Invalid authentication token", "auth"));

        // Configuration errors
        m.insert(r"configuration.*error", ("Configuration error", "config"));
        m.insert(r"missing.*environment.*variable", ("Missing environment variable", "config"));
        m.insert(r"invalid.*format.*JSON", ("Invalid JSON format", "config"));
        m.insert(r"invalid.*format.*YAML", ("Invalid YAML format", "config"));

        // API/HTTP errors
        m.insert(r"HTTP.*404", ("Resource not found (404)", "http"));
        m.insert(r"HTTP.*500", ("Internal server error (500)", "http"));
        m.insert(r"HTTP.*403", ("Forbidden access (403)", "http"));
        m.insert(r"HTTP.*401", ("Unauthorized (401)", "http"));
        m.insert(r"rate.*limit.*exceeded", ("API rate limit exceeded", "http"));

        m
    };

    static ref LANGUAGE_INDICATORS: Vec<(&'static str, &'static str)> = vec![
        // File extensions
        (r"\.rs:", "rust"),
        (r"\.js:", "javascript"),
        (r"\.ts:", "typescript"),
        (r"\.py:", "python"),
        (r"\.java:", "java"),
        (r"\.cpp:", "cpp"),
        (r"\.c:", "c"),
        (r"\.go:", "go"),
        (r"\.rb:", "ruby"),
        (r"\.php:", "php"),

        // Language-specific keywords
        (r"cargo", "rust"),
        (r"rustc", "rust"),
        (r"npm", "javascript"),
        (r"node", "javascript"),
        (r"yarn", "javascript"),
        (r"pip", "python"),
        (r"python", "python"),
        (r"javac", "java"),
        (r"maven", "java"),
        (r"gradle", "java"),
        (r"gcc", "c"),
        (r"g\+\+", "cpp"),
        (r"go build", "go"),

        // Framework indicators
        (r"react", "javascript"),
        (r"vue", "javascript"),
        (r"angular", "javascript"),
        (r"django", "python"),
        (r"flask", "python"),
        (r"spring", "java"),
        (r"rails", "ruby"),
    ];

    static ref GENERIC_LOCATION_RE: Regex = Regex::new(
        r"([^:\s]+\.[a-zA-Z]+):(\d+):?(\d+)?"
    ).unwrap();

    static ref ERROR_LEVEL_RE: Regex = Regex::new(
        r"(?i)(error|warning|fatal|critical):"
    ).unwrap();
}

pub struct GenericParser;

impl Parser for GenericParser {
    fn parse(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        // Try to detect language and delegate to specific parser
        if let Some(detected_lang) = self.detect_language(input) {
            match detected_lang {
                "rust" => return RustParser.parse(input),
                "javascript" | "typescript" => return JsParser.parse(input),
                "python" => return PythonParser.parse(input),
                "java" => return JavaParser.parse(input),
                "go" => return GoParser.parse(input),
                _ => {} // Continue with generic parsing
            }
        }

        // Parse generic patterns
        errors.extend(self.parse_generic_errors(input));

        // If no specific patterns found, try generic error detection
        if errors.is_empty() {
            if let Some(generic_error) = self.parse_fallback_error(input) {
                errors.push(generic_error);
            }
        }

        errors
    }
}

impl GenericParser {
    fn detect_language(&self, input: &str) -> Option<&'static str> {
        for (pattern, lang) in LANGUAGE_INDICATORS.iter() {
            let re = Regex::new(pattern).ok()?;
            if re.is_match(input) {
                return Some(lang);
            }
        }
        None
    }

    fn parse_generic_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        for (pattern, (description, category)) in GENERIC_PATTERNS.iter() {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(input) {
                let location = self.extract_generic_location(input);
                errors.push(ErrorInfo {
                    message: description.to_string(),
                    location,
                    language: category.to_string(),
                });
            }
        }

        errors
    }

    fn parse_fallback_error(&self, input: &str) -> Option<ErrorInfo> {
        // Look for generic error indicators
        if ERROR_LEVEL_RE.is_match(input) {
            let location = self.extract_generic_location(input);
            let error_type = self.classify_error_type(input);

            return Some(ErrorInfo {
                message: format!("Generic {} detected", error_type),
                location,
                language: "unknown".to_string(),
            });
        }

        // Check for common error words
        let error_words = ["failed", "exception", "error", "fatal", "abort", "crash"];
        for word in &error_words {
            if input.to_lowercase().contains(word) {
                let location = self.extract_generic_location(input);
                return Some(ErrorInfo {
                    message: format!("Possible error detected: contains '{}'", word),
                    location,
                    language: "unknown".to_string(),
                });
            }
        }

        None
    }

    fn extract_generic_location(&self, input: &str) -> Option<String> {
        // Try to find file:line:column patterns
        if let Some(caps) = GENERIC_LOCATION_RE.captures(input) {
            let file = caps.get(1)?.as_str();
            let line = caps.get(2)?.as_str();

            if let Some(col) = caps.get(3) {
                return Some(format!("{}:{}:{}", file, line, col.as_str()));
            } else {
                return Some(format!("{}:{}", file, line));
            }
        }

        None
    }

    fn classify_error_type(&self, input: &str) -> &'static str {
        let lower_input = input.to_lowercase();

        if lower_input.contains("network") || lower_input.contains("connection") {
            "network error"
        } else if lower_input.contains("permission") || lower_input.contains("access") {
            "permission error"
        } else if lower_input.contains("memory") || lower_input.contains("allocation") {
            "memory error"
        } else if lower_input.contains("timeout") || lower_input.contains("time out") {
            "timeout error"
        } else if lower_input.contains("syntax") || lower_input.contains("parse") {
            "syntax error"
        } else if lower_input.contains("build") || lower_input.contains("compile") {
            "build error"
        } else if lower_input.contains("test") || lower_input.contains("assertion") {
            "test failure"
        } else {
            "error"
        }
    }

    pub fn suggest_generic_fixes(&self, _error_message: &str, category: &str) -> Vec<String> {
        let mut fixes = vec![];

        match category {
            "network" => {
                fixes.push("Check network connectivity".to_string());
                fixes.push("Verify server is running".to_string());
                fixes.push("Check firewall settings".to_string());
            }
            "permission" => {
                fixes.push("Check file/directory permissions".to_string());
                fixes.push("Run with appropriate privileges".to_string());
                fixes.push("Verify user has required access".to_string());
            }
            "config" => {
                fixes.push("Validate configuration syntax".to_string());
                fixes.push("Check environment variables".to_string());
                fixes.push("Review configuration file paths".to_string());
            }
            "database" => {
                fixes.push("Check database connection string".to_string());
                fixes.push("Verify database server is running".to_string());
                fixes.push("Check database credentials".to_string());
            }
            "auth" => {
                fixes.push("Verify credentials are correct".to_string());
                fixes.push("Check token expiration".to_string());
                fixes.push("Review authentication configuration".to_string());
            }
            "docker" => {
                fixes.push("Check Docker daemon is running".to_string());
                fixes.push("Verify Dockerfile syntax".to_string());
                fixes.push("Check image availability".to_string());
            }
            "git" => {
                fixes.push("Initialize git repository: git init".to_string());
                fixes.push("Check if in correct directory".to_string());
                fixes.push("Verify git installation".to_string());
            }
            _ => {
                fixes.push("Check error message for specific details".to_string());
                fixes.push("Review recent changes".to_string());
                fixes.push("Search documentation or forums".to_string());
            }
        }

        fixes
    }

    pub fn get_language_parser(&self, language: &str) -> Box<dyn Parser> {
        match language.to_lowercase().as_str() {
            "rust" => Box::new(RustParser),
            "javascript" | "js" | "node" | "typescript" | "ts" => Box::new(JsParser),
            "python" | "py" => Box::new(PythonParser),
            "java" => Box::new(JavaParser),
            "go" => Box::new(GoParser),
            _ => Box::new(GenericParser),
        }
    }
}

// Factory function for getting the right parser
pub fn get_parser_for_input(input: &str) -> Box<dyn Parser> {
    let generic = GenericParser;

    // Try to detect language from input
    if let Some(detected_lang) = generic.detect_language(input) {
        return generic.get_language_parser(detected_lang);
    }

    // Default to generic parser
    Box::new(generic)
}

// Convenience function to parse any input with automatic language detection
pub fn parse_any_error(input: &str) -> Vec<ErrorInfo> {
    let parser = get_parser_for_input(input);
    parser.parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        let parser = GenericParser;

        assert_eq!(parser.detect_language("cargo build failed"), Some("rust"));
        assert_eq!(
            parser.detect_language("npm install error"),
            Some("javascript")
        );
        assert_eq!(parser.detect_language("pip install failed"), Some("python"));
        assert_eq!(parser.detect_language("javac Main.java"), Some("java"));
    }

    #[test]
    fn test_generic_error_parsing() {
        let parser = GenericParser;
        let input = "Connection refused on port 3000";

        let errors = parser.parse(input);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Network error"));
        assert_eq!(errors[0].language, "generic");
    }

    #[test]
    fn test_parser_factory() {
        let rust_input = "cargo run failed with panic";
        let parser = get_parser_for_input(rust_input);
        let errors = parser.parse(rust_input);

        // Should delegate to RustParser
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_parse_any_error() {
        let errors = parse_any_error("docker build failed");
        assert!(!errors.is_empty());
    }
}
