use crate::{Parser, ErrorInfo};
use regex::Regex;
use std::collections::HashMap;
use lazy_static::lazy_static;

pub struct PythonParser;

lazy_static! {
    static ref EXCEPTION_PATTERNS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Import/Module errors
        m.insert(r"ImportError", "Module import failed");
        m.insert(r"ModuleNotFoundError: No module named '([^']+)'", "Module not installed or not found");
        m.insert(r"ImportError: cannot import name '([^']+)'", "Import name not found in module");
        
        // Syntax errors
        m.insert(r"SyntaxError", "Invalid Python syntax");
        m.insert(r"IndentationError", "Incorrect indentation");
        m.insert(r"TabError", "Inconsistent tab/space usage");
        
        // Runtime errors
        m.insert(r"NameError: name '([^']+)' is not defined", "Undefined variable or function");
        m.insert(r"AttributeError: '([^']+)' object has no attribute '([^']+)'", "Attribute not found");
        m.insert(r"TypeError: '([^']+)' object is not callable", "Attempting to call non-callable object");
        m.insert(r"TypeError: .*takes \d+ positional argument", "Wrong number of function arguments");
        m.insert(r"KeyError: '([^']+)'", "Dictionary key not found");
        m.insert(r"IndexError: list index out of range", "List/array index out of bounds");
        m.insert(r"ValueError", "Invalid value for operation");
        m.insert(r"ZeroDivisionError", "Division by zero");
        m.insert(r"FileNotFoundError", "File or directory not found");
        m.insert(r"PermissionError", "Insufficient permissions");
        
        // Network/HTTP errors
        m.insert(r"ConnectionError", "Network connection failed");
        m.insert(r"TimeoutError", "Operation timed out");
        m.insert(r"requests\.exceptions\.ConnectionError", "HTTP connection error");
        m.insert(r"urllib\.error\.URLError", "URL access error");
        
        // Type/conversion errors
        m.insert(r"TypeError: unsupported operand type", "Invalid operation between types");
        m.insert(r"TypeError: can't convert", "Type conversion failed");
        m.insert(r"UnicodeDecodeError", "Text encoding/decoding error");
        m.insert(r"UnicodeEncodeError", "Text encoding error");
        
        // Memory/resource errors
        m.insert(r"MemoryError", "Out of memory");
        m.insert(r"RecursionError", "Maximum recursion depth exceeded");
        m.insert(r"OSError", "Operating system error");
        
        // Framework-specific
        m.insert(r"django\.", "Django framework error");
        m.insert(r"flask\.", "Flask framework error");
        m.insert(r"pandas\.", "Pandas data processing error");
        m.insert(r"numpy\.", "NumPy array processing error");
        
        m
    };

    static ref TRACEBACK_RE: Regex = Regex::new(
        r#"File "([^"]+)", line (\d+)(?:, in ([^\n]+))?"#
    ).unwrap();
    
    static ref ERROR_LINE_RE: Regex = Regex::new(
        r"^([A-Z][a-zA-Z]*Error|Exception):"
    ).unwrap();
    
    static ref LOCATION_RE: Regex = Regex::new(
        r#"File "([^"]+)", line (\d+)"#
    ).unwrap();
}

impl Parser for PythonParser {
    fn parse(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];
        
        // Check for Python exceptions
        if let Some(exception_error) = self.parse_exception(input) {
            errors.push(exception_error);
        }
        
        // Check for syntax errors (special handling)
        errors.extend(self.parse_syntax_errors(input));
        
        // Check for pip/package manager errors
        errors.extend(self.parse_pip_errors(input));
        
        // Check for linting errors (flake8, pylint, mypy)
        errors.extend(self.parse_linting_errors(input));
        
        errors
    }
}

impl PythonParser {
    fn parse_exception(&self, input: &str) -> Option<ErrorInfo> {
        // Look for specific exception patterns
        for (pattern, description) in EXCEPTION_PATTERNS.iter() {
            let re = Regex::new(pattern).ok()?;
            if let Some(caps) = re.captures(input) {
                let location = self.extract_location(input);
                let mut message = description.to_string();
                
                // Add captured groups for more specific messages
                if caps.len() > 1 {
                    if let Some(detail) = caps.get(1) {
                        message = format!("{}: {}", description, detail.as_str());
                    }
                }
                
                return Some(ErrorInfo {
                    message,
                    location,
                    language: "python".to_string(),
                });
            }
        }
        
        // Generic exception detection
        if ERROR_LINE_RE.is_match(input) {
            let location = self.extract_location(input);
            return Some(ErrorInfo {
                message: "Python exception occurred".to_string(),
                location,
                language: "python".to_string(),
            });
        }
        
        None
    }
    
    fn parse_syntax_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];
        
        if input.contains("SyntaxError") {
            let location = self.extract_location(input);
            let mut message = "Syntax error".to_string();
            
            // Try to extract more specific syntax error info
            if input.contains("invalid syntax") {
                message = "Invalid Python syntax".to_string();
            } else if input.contains("unexpected EOF") {
                message = "Unexpected end of file - missing closing bracket/quote".to_string();
            } else if input.contains("unmatched") {
                message = "Unmatched brackets or quotes".to_string();
            }
            
            errors.push(ErrorInfo {
                message,
                location,
                language: "python".to_string(),
            });
        }
        
        if input.contains("IndentationError") {
            let location = self.extract_location(input);
            errors.push(ErrorInfo {
                message: "Indentation error - check spaces and tabs".to_string(),
                location,
                language: "python".to_string(),
            });
        }
        
        errors
    }
    
    fn parse_pip_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];
        
        if input.contains("ERROR: Could not find a version") {
            errors.push(ErrorInfo {
                message: "Package not found in PyPI".to_string(),
                location: None,
                language: "python".to_string(),
            });
        }
        
        if input.contains("ERROR: No matching distribution found") {
            errors.push(ErrorInfo {
                message: "Package version not available".to_string(),
                location: None,
                language: "python".to_string(),
            });
        }
        
        if input.contains("pip install") && input.contains("permission denied") {
            errors.push(ErrorInfo {
                message: "Pip install permission denied - try with --user or virtual environment".to_string(),
                location: None,
                language: "python".to_string(),
            });
        }
        
        if input.contains("Microsoft Visual C++") && input.contains("required") {
            errors.push(ErrorInfo {
                message: "Missing Visual C++ Build Tools on Windows".to_string(),
                location: None,
                language: "python".to_string(),
            });
        }
        
        errors
    }
    
    fn parse_linting_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];
        
        // Flake8 errors
        let flake8_re = Regex::new(r"([^:]+):(\d+):(\d+): ([A-Z]\d+) (.+)").unwrap();
        for caps in flake8_re.captures_iter(input) {
            if let (Some(file), Some(line), Some(code), Some(msg)) = 
                (caps.get(1), caps.get(2), caps.get(4), caps.get(5)) {
                errors.push(ErrorInfo {
                    message: format!("Linting error {}: {}", code.as_str(), msg.as_str()),
                    location: Some(format!("{}:{}", file.as_str(), line.as_str())),
                    language: "python".to_string(),
                });
            }
        }
        
        // MyPy type errors
        if input.contains("mypy") && input.contains("error:") {
            let location = self.extract_location(input);
            errors.push(ErrorInfo {
                message: "Type checking error".to_string(),
                location,
                language: "python".to_string(),
            });
        }
        
        errors
    }
    
    fn extract_location(&self, input: &str) -> Option<String> {
        // Look for Python traceback format: File "filename", line number
        if let Some(caps) = LOCATION_RE.captures(input) {
            let file = caps.get(1)?.as_str();
            let line = caps.get(2)?.as_str();
            return Some(format!("{}:{}", file, line));
        }
        
        None
    }
    
    pub fn extract_traceback(&self, input: &str) -> Vec<String> {
        let mut frames = vec![];
        
        for caps in TRACEBACK_RE.captures_iter(input) {
            if let (Some(file), Some(line)) = (caps.get(1), caps.get(2)) {
                let function = caps.get(3).map(|m| m.as_str()).unwrap_or("unknown");
                frames.push(format!("{} in {} ({}:{})", 
                    function, file.as_str(), file.as_str(), line.as_str()));
            }
        }
        
        frames
    }
    
    pub fn suggest_fixes(&self, error_message: &str) -> Vec<String> {
        let mut fixes = vec![];
        
        if error_message.contains("Module not installed") {
            fixes.push("pip install <module_name>".to_string());
            fixes.push("pip install --user <module_name>".to_string());
            fixes.push("Create and activate a virtual environment".to_string());
        }
        
        if error_message.contains("Undefined variable") {
            fixes.push("Check variable spelling and scope".to_string());
            fixes.push("Initialize variable before use".to_string());
            fixes.push("Import required modules".to_string());
        }
        
        if error_message.contains("Indentation error") {
            fixes.push("Use consistent indentation (4 spaces recommended)".to_string());
            fixes.push("Check for mixed tabs and spaces".to_string());
            fixes.push("Use IDE auto-formatting".to_string());
        }
        
        if error_message.contains("Division by zero") {
            fixes.push("Add check: if denominator != 0:".to_string());
            fixes.push("Use try/except ZeroDivisionError".to_string());
        }
        
        if error_message.contains("Index out of bounds") {
            fixes.push("Check list length before accessing".to_string());
            fixes.push("Use try/except IndexError".to_string());
            fixes.push("Use list.get() for dictionaries".to_string());
        }
        
        fixes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_import_error() {
        let parser = PythonParser;
        let input = r#"
Traceback (most recent call last):
  File "main.py", line 5, in <module>
    import nonexistent_module
ModuleNotFoundError: No module named 'nonexistent_module'
        "#;
        
        let errors = parser.parse(input);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Module not installed"));
        assert_eq!(errors[0].location, Some("main.py:5".to_string()));
    }
    
    #[test]
    fn test_syntax_error() {
        let parser = PythonParser;
        let input = r#"
  File "test.py", line 10
    if x = 5:
         ^
SyntaxError: invalid syntax
        "#;
        
        let errors = parser.parse(input);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("syntax"));
    }
    
    #[test]
    fn test_traceback_extraction() {
        let parser = PythonParser;
        let input = r#"
Traceback (most recent call last):
  File "main.py", line 10, in main
    result = divide(a, b)
  File "utils.py", line 5, in divide
    return x / y
ZeroDivisionError: division by zero
        "#;
        
        let frames = parser.extract_traceback(input);
        assert_eq!(frames.len(), 2);
        assert!(frames[0].contains("main.py"));
        assert!(frames[1].contains("utils.py"));
    }
    
    #[test]
    fn test_fix_suggestions() {
        let parser = PythonParser;
        let fixes = parser.suggest_fixes("Module not installed or not found: requests");
        assert!(fixes.iter().any(|f| f.contains("pip install")));
    }
}
