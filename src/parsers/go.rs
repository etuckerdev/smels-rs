use crate::{ErrorInfo, Parser};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref GO_ERROR_PATTERNS: Vec<(&'static str, &'static str)> = vec![
        (r"panic:\s*(.+)", "Runtime panic"),
        (r"runtime error:\s*(.+)", "Runtime error"),
        (r"cannot find package", "Package not found"),
        (r"undefined:\s*([^\s]+)", "Undefined symbol"),
        (r"cannot assign to (.+)", "Assignment error"),
        (r"invalid operation:\s*(.+)", "Invalid operation"),
        (r"type mismatch|cannot convert", "Type conversion error"),
        (r"import cycle", "Import cycle detected"),
        (r"build failed", "Build failure"),
    ];
    static ref GO_LOCATION_RE: Regex = Regex::new(r"([^:\s]+\.go):(\d+):?(\d+)?").unwrap();
    static ref GO_STACK_FRAME_RE: Regex = Regex::new(r"([^:]+)\.go:(\d+)\s+\+0x[0-9a-f]+").unwrap();
}

pub struct GoParser;

impl Parser for GoParser {
    fn parse(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        // Check for Go-specific error patterns
        for (pattern, description) in GO_ERROR_PATTERNS.iter() {
            let re = Regex::new(pattern).unwrap();
            for caps in re.captures_iter(input) {
                let location = self.extract_location(input);
                let message = if caps.len() > 1 {
                    format!("{}: {}", description, &caps[1])
                } else {
                    description.to_string()
                };

                errors.push(ErrorInfo {
                    message,
                    location,
                    language: "go".to_string(),
                });
            }
        }

        // Generic Go error detection
        if input.contains("go:") || input.contains("go.mod") || input.contains("go.sum") {
            if errors.is_empty() && (input.contains("error") || input.contains("failed")) {
                let location = self.extract_location(input);
                errors.push(ErrorInfo {
                    message: "Go build or runtime error".to_string(),
                    location,
                    language: "go".to_string(),
                });
            }
        }

        errors
    }
}

impl GoParser {
    fn extract_location(&self, input: &str) -> Option<String> {
        // Try Go-specific patterns first
        if let Some(caps) = GO_LOCATION_RE.captures(input) {
            let file = caps.get(1)?.as_str();
            let line = caps.get(2)?.as_str();

            if let Some(col) = caps.get(3) {
                return Some(format!("{}:{}:{}", file, line, col.as_str()));
            } else {
                return Some(format!("{}:{}", file, line));
            }
        }

        // Try stack frame pattern
        if let Some(caps) = GO_STACK_FRAME_RE.captures(input) {
            let file = caps.get(1)?.as_str();
            let line = caps.get(2)?.as_str();
            return Some(format!("{}:{}", file, line));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_panic_detection() {
        let parser = GoParser;
        let input = "panic: runtime error: invalid memory address or nil pointer dereference\n\tmain.go:15 +0x123";

        let errors = parser.parse(input);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("panic"));
        assert_eq!(errors[0].language, "go");
    }

    #[test]
    fn test_go_import_error() {
        let parser = GoParser;
        let input = "cannot find package \"github.com/example/lib\" in any of:\n\t/path/to/vendor";

        let errors = parser.parse(input);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Package not found"));
    }

    #[test]
    fn test_go_location_extraction() {
        let parser = GoParser;
        let input = "main.go:42: syntax error: unexpected semicolon";

        let location = parser.extract_location(input);
        assert_eq!(location, Some("main.go:42".to_string()));
    }
}
