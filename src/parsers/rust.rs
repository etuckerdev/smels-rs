use crate::{ErrorInfo, Parser};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

pub struct RustParser;

lazy_static! {
    static ref PANIC_PATTERNS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert(
            r"called `Option::unwrap\(\)` on a `None` value",
            "Option::unwrap on None",
        );
        m.insert(
            r"called `Result::unwrap\(\)` on an `Err` value",
            "Result::unwrap on Err",
        );
        m.insert(r"index out of bounds", "Array index out of bounds");
        m.insert(r"attempt to divide by zero", "Division by zero");
        m.insert(
            r"attempt to subtract with overflow",
            "Integer overflow/underflow",
        );
        m.insert(r"attempt to add with overflow", "Integer overflow");
        m.insert(r"assertion failed", "Assertion failure");
        m.insert(r"not yet implemented", "Unimplemented code path");
        m.insert(r"unreachable", "Unreachable code executed");
        m
    };
    static ref COMPILE_PATTERNS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert(
            r"cannot find (?:function|struct|enum|trait|type|macro) `([^`]+)`",
            "Undefined identifier",
        );
        m.insert(r"mismatched types", "Type mismatch");
        m.insert(
            r"borrowed value does not live long enough",
            "Lifetime/borrow checker error",
        );
        m.insert(r"cannot borrow .* as mutable", "Borrow checker violation");
        m.insert(r"use of moved value", "Use after move");
        m.insert(
            r"this function takes \d+ argument",
            "Incorrect function arguments",
        );
        m.insert(
            r"field `([^`]+)` of struct .* is private",
            "Private field access",
        );
        m.insert(r"no method named `([^`]+)` found", "Method not found");
        m
    };
    static ref LOCATION_RE: Regex =
        Regex::new(r"(?:at\s+|-->)?\s*([^:\s]+):(\d+):?(\d+)?").unwrap();
    static ref STACK_FRAME_RE: Regex =
        Regex::new(r"\s*\d+:\s+([^:]+)::\w+.*?at\s+([^:]+):(\d+):?(\d+)?").unwrap();
    static ref FRAME_NUMBER_RE: Regex = Regex::new(r"^\s*(\d+):\s+(.+)").unwrap();
    static ref FUNC_AT_LOC_RE: Regex = Regex::new(r"(.+?)\s+at\s+(.+)").unwrap();
}

impl Parser for RustParser {
    fn parse(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        // Check for panics first
        if let Some(panic_error) = self.parse_panic(input) {
            errors.push(panic_error);
        }

        // Check for compilation errors
        errors.extend(self.parse_compile_errors(input));

        // Check for cargo/build errors
        errors.extend(self.parse_cargo_errors(input));

        errors
    }
}

impl RustParser {
    fn parse_panic(&self, input: &str) -> Option<ErrorInfo> {
        // Look for panic patterns
        for (pattern, description) in PANIC_PATTERNS.iter() {
            let re = Regex::new(pattern).ok()?;
            if re.is_match(input) {
                let location = self.extract_location(input);
                return Some(ErrorInfo {
                    message: format!("Panic: {}", description),
                    location,
                    language: "rust".to_string(),
                });
            }
        }

        // Generic panic detection
        if input.contains("panic") || input.contains("thread panicked") {
            let location = self.extract_location(input);
            return Some(ErrorInfo {
                message: "Runtime panic detected".to_string(),
                location,
                language: "rust".to_string(),
            });
        }

        None
    }

    fn parse_compile_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        for (pattern, description) in COMPILE_PATTERNS.iter() {
            let re = Regex::new(pattern).unwrap();
            for caps in re.captures_iter(input) {
                let location = self.extract_location(&caps.get(0).unwrap().as_str());
                errors.push(ErrorInfo {
                    message: format!("Compile error: {}", description),
                    location,
                    language: "rust".to_string(),
                });
            }
        }

        // Generic error/warning detection
        if input.contains("error[E") || input.contains("error:") {
            if errors.is_empty() {
                let location = self.extract_location(input);
                errors.push(ErrorInfo {
                    message: "Compilation error".to_string(),
                    location,
                    language: "rust".to_string(),
                });
            }
        }

        errors
    }

    fn parse_cargo_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        // Cargo-specific errors
        if input.contains("could not compile") {
            errors.push(ErrorInfo {
                message: "Cargo build failed".to_string(),
                location: None,
                language: "rust".to_string(),
            });
        }

        if input.contains("failed to resolve dependencies") {
            errors.push(ErrorInfo {
                message: "Dependency resolution failed".to_string(),
                location: None,
                language: "rust".to_string(),
            });
        }

        if input.contains("linker") && input.contains("not found") {
            errors.push(ErrorInfo {
                message: "Linker error - missing system dependencies".to_string(),
                location: None,
                language: "rust".to_string(),
            });
        }

        errors
    }

    fn extract_location(&self, input: &str) -> Option<String> {
        // Try to find file:line:col patterns
        if let Some(caps) = LOCATION_RE.captures(input) {
            let file = caps.get(1)?.as_str();
            let line = caps.get(2)?.as_str();

            if let Some(col) = caps.get(3) {
                return Some(format!("{}:{}:{}", file, line, col.as_str()));
            } else {
                return Some(format!("{}:{}", file, line));
            }
        }

        // Try to extract from stack frames
        if let Some(caps) = STACK_FRAME_RE.captures(input) {
            let file = caps.get(2)?.as_str();
            let line = caps.get(3)?.as_str();
            return Some(format!("{}:{}", file, line));
        }

        None
    }

    pub fn extract_stack_trace(&self, input: &str) -> Vec<String> {
        let mut frames = vec![];
        let lines: Vec<&str> = input.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();

            // Look for frame number pattern
            if let Some(caps) = FRAME_NUMBER_RE.captures(line) {
                let func_part = caps.get(2).unwrap().as_str();

                // Check if next line has the location
                let location = if i + 1 < lines.len() {
                    let next_line = lines[i + 1].trim();
                    if next_line.starts_with("at ") {
                        let loc_part = &next_line[3..]; // Remove "at "
                        self.extract_location(&format!("at {}", loc_part))
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(loc) = location {
                    frames.push(format!("{} ({})", func_part, loc));
                } else {
                    // Try to extract location from current line
                    if let Some(caps) = FUNC_AT_LOC_RE.captures(line) {
                        let func = caps.get(1).unwrap().as_str();
                        let loc = caps.get(2).unwrap().as_str();
                        frames.push(format!("{} ({})", func, loc));
                    }
                }

                // Skip the next line if we used it for location
                if i + 1 < lines.len() && lines[i + 1].trim().starts_with("at ") {
                    i += 1;
                }
            }
            i += 1;
        }

        frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panic_detection() {
        let parser = RustParser;
        let input = r#"
thread 'main' panicked at 'called `Option::unwrap()` on a `None` value', src/main.rs:42:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
        "#;

        let errors = parser.parse(input);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Panic: Option::unwrap on None");
        assert_eq!(errors[0].location, Some("src/main.rs:42:5".to_string()));
    }

    #[test]
    fn test_compile_error_detection() {
        let parser = RustParser;
        let input = r#"
error[E0425]: cannot find function `unknown_function` in this scope
  --> src/main.rs:10:5
   |
10 |     unknown_function();
   |     ^^^^^^^^^^^^^^^^ not found in this scope
        "#;

        let errors = parser.parse(input);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Compile error"));
    }

    #[test]
    fn test_stack_trace_extraction() {
        let parser = RustParser;
        let input = r#"
stack backtrace:
   0: rust_begin_unwind
             at /rustc/hash/library/std/src/panicking.rs:595:5
   1: core::panicking::panic_fmt
             at /rustc/hash/library/core/src/panicking.rs:67:14
   2: hello_world::main::h123abc4567890def
             at src/main.rs:42:5
        "#;

        let frames = parser.extract_stack_trace(input);
        assert!(!frames.is_empty());
        assert!(frames.iter().any(|f| f.contains("hello_world::main")));
    }
}
