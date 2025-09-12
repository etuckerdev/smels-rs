use crate::{ErrorInfo, Parser};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

pub struct CppParser;

lazy_static! {
    static ref CPP_ERROR_PATTERNS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();

        // Compilation errors
        m.insert(r"error:\s*'([^']+)' was not declared in this scope", "Undeclared identifier");
        m.insert(r"error:\s*'([^']+)' does not name a type", "Unknown type");
        m.insert(r"error:\s*no matching function for call to '([^']+)<([^>]*)>'", "Template function mismatch");
        m.insert(r"error:\s*no matching function for call to '([^<']+)'", "Function call mismatch");
        m.insert(r"error:\s*no matching constructor for initialization", "Constructor mismatch");
        m.insert(r"error:\s*cannot convert '([^']+)' to '([^']+)'", "Type conversion error");
        m.insert(r"error:\s*invalid conversion from '([^']+)' to '([^']+)'", "Invalid type conversion");
        m.insert(r"error:\s*'([^']+)' has no member named '([^']+)'", "Member not found");
        m.insert(r"error:\s*expected '([^']+)' before '([^']+)'", "Syntax error");
        m.insert(r"error:\s*expected ';' before", "Missing semicolon");
        m.insert(r"error:\s*expected '\\}' at end of input", "Missing closing brace");
        m.insert(r"error:\s*redefinition of '([^']+)'", "Multiple definition");
        m.insert(r"error:\s*conflicting declaration '([^']+)'", "Declaration conflict");
        m.insert(r"error:\s*incomplete type '([^']+)'", "Incomplete type");
        m.insert(r"error:\s*abstract class type '([^']+)'", "Abstract class instantiation");
        m.insert(r"error:\s*pure virtual function called", "Pure virtual call");

        // Template errors
        m.insert(r"error:\s*template argument deduction/substitution failed", "Template deduction failed");
        m.insert(r"error:\s*no matching function for call to '([^']+)<([^>]*)>'", "Template function mismatch");
        m.insert(r"error:\s*invalid use of template-name '([^']+)' without an argument list", "Template argument missing");
        m.insert(r"error:\s*template parameter '([^']+)' is not used", "Unused template parameter");

        // Linker errors
        m.insert(r"undefined reference to [`']([^`']+)[`']", "Undefined symbol");
        m.insert(r"multiple definition of '([^']+)'", "Symbol multiply defined");
        m.insert(r"cannot find -l([a-zA-Z0-9_]+)", "Library not found");
        m.insert(r"No such file or directory.*\\.so", "Shared library missing");

        // Standard library errors
        m.insert(r"error:\s*no member named '([^']+)' in namespace 'std'", "Standard library member missing");
        m.insert(r"error:\s*'([^']+)' is not a member of 'std'", "Standard library error");
        m.insert(r"terminate called after throwing an instance of '([^']+)'", "Unhandled exception");
        m.insert(r"what\\(\\):\\s*(.+)", "Exception message");

        // CMake/Build system errors
        m.insert(r"CMake Error", "CMake configuration error");
        m.insert(r"Could NOT find ([a-zA-Z0-9_]+)", "CMake dependency not found");
        m.insert(r"No rule to make target", "Make target missing");
        m.insert(r"recipe for target .* failed", "Build recipe failed");

        // Preprocessor errors
        m.insert(r"fatal error:\s*([^:]+):\s*No such file or directory", "Header file not found");
        m.insert(r"error:\s*#error (.+)", "Preprocessor error directive");
        m.insert(r"warning:\s*#warning (.+)", "Preprocessor warning");

        m
    };

    static ref CPP_WARNING_PATTERNS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert(r"warning:\s*unused variable '([^']+)'", "Unused variable");
        m.insert(r"warning:\s*unused parameter '([^']+)'", "Unused parameter");
        m.insert(r"warning:\s*comparison between signed and unsigned", "Signed/unsigned comparison");
        m.insert(r"warning:\s*suggest parentheses around", "Precedence warning");
        m.insert(r"warning:\s*deprecated", "Deprecated feature used");
        m.insert(r"warning:\s*ignoring return value", "Return value ignored");
        m.insert(r"warning:\s*may be used uninitialized", "Uninitialized variable");
        m
    };

    static ref CPP_LOCATION_RE: Regex = Regex::new(
        r"([^:\s]+\.(?:cpp|cc|cxx|c\+\+|c|h|hpp|hxx)):(\d+):?(\d+)?"
    ).unwrap();

    static ref GDB_BACKTRACE_RE: Regex = Regex::new(
        r"#(\d+)\s+(?:0x[0-9a-f]+\s+in\s+)?([^(\s]+(?:\s*\([^)]*\))?)(?:\s+at\s+([^:]+):(\d+))?(?:\s+from\s+(.+))?"
    ).unwrap();

    static ref VALGRIND_ERROR_RE: Regex = Regex::new(
        r"==\d+==\s*(.+?)\s+at\s+0x[0-9A-F]+:\s*(.+?)\s*\(([^:)]+):(\d+)\)"
    ).unwrap();

    static ref COMPILER_RE: Regex = Regex::new(
        r"(g\+\+|gcc|clang\+\+|clang|cl\.exe)"
    ).unwrap();
}

impl Parser for CppParser {
    fn parse(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        // Parse compilation errors
        errors.extend(self.parse_compilation_errors(input));

        // Parse linker errors
        errors.extend(self.parse_linker_errors(input));

        // Parse runtime errors
        errors.extend(self.parse_runtime_errors(input));

        // Parse build system errors
        errors.extend(self.parse_build_errors(input));

        // Parse warnings (if configured to treat as errors)
        errors.extend(self.parse_warnings(input));

        // Generic C++ error detection as fallback
        if errors.is_empty() {
            if let Some(generic_error) = self.parse_generic_cpp_error(input) {
                errors.push(generic_error);
            }
        }

        errors
    }
}

impl CppParser {
    fn parse_compilation_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        for (pattern, description) in CPP_ERROR_PATTERNS.iter() {
            let re = Regex::new(pattern).unwrap();
            for caps in re.captures_iter(input) {
                let location =
                    self.extract_location_from_context(input, caps.get(0).unwrap().start());
                let message = if caps.len() > 1 {
                    let mut msg = description.to_string();
                    for i in 1..caps.len() {
                        if let Some(capture) = caps.get(i) {
                            msg = format!("{}: {}", msg, capture.as_str());
                            break; // Use first capture for main detail
                        }
                    }
                    msg
                } else {
                    description.to_string()
                };

                errors.push(ErrorInfo {
                    message,
                    location,
                    language: "cpp".to_string(),
                });
            }
        }

        errors
    }

    fn parse_linker_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        let linker_patterns = [
            r"undefined reference to [`']([^`']+)[`']",
            r"multiple definition of '([^']+)'",
            r"cannot find -l([a-zA-Z0-9_]+)",
            r"No such file or directory.*\.so",
        ];

        for pattern in &linker_patterns {
            let re = Regex::new(pattern).unwrap();
            for caps in re.captures_iter(input) {
                let message = match pattern {
                    p if p.contains("undefined reference") => {
                        format!("Undefined symbol: {}", caps.get(1).unwrap().as_str())
                    }
                    p if p.contains("multiple definition") => {
                        format!("Symbol multiply defined: {}", caps.get(1).unwrap().as_str())
                    }
                    p if p.contains("cannot find -l") => {
                        format!("Library not found: lib{}", caps.get(1).unwrap().as_str())
                    }
                    _ => "Linker error".to_string(),
                };

                errors.push(ErrorInfo {
                    message,
                    location: None,
                    language: "cpp".to_string(),
                });
            }
        }

        errors
    }

    fn parse_runtime_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        // Sanitizer errors
        let sanitizer_patterns = [
            (r"AddressSanitizer:\s*(.+)", "Address sanitizer"),
            (r"LeakSanitizer:\s*(.+)", "Memory leak"),
            (r"ThreadSanitizer:\s*(.+)", "Thread error"),
            (r"UndefinedBehaviorSanitizer:\s*(.+)", "Undefined behavior"),
        ];

        for (pattern, error_type) in &sanitizer_patterns {
            let re = Regex::new(pattern).unwrap();
            for caps in re.captures_iter(input) {
                let location = self.extract_location(input);
                errors.push(ErrorInfo {
                    message: format!("{}: {}", error_type, caps.get(1).unwrap().as_str()),
                    location,
                    language: "cpp".to_string(),
                });
            }
        }

        // Segmentation faults
        if input.contains("Segmentation fault") || input.contains("SIGSEGV") {
            errors.push(ErrorInfo {
                message: "Segmentation fault - invalid memory access".to_string(),
                location: self.extract_location(input),
                language: "cpp".to_string(),
            });
        }

        // Exception handling
        let exception_re =
            Regex::new(r"terminate called after throwing an instance of '([^']+)'").unwrap();
        for caps in exception_re.captures_iter(input) {
            errors.push(ErrorInfo {
                message: format!("Unhandled exception: {}", caps.get(1).unwrap().as_str()),
                location: self.extract_location(input),
                language: "cpp".to_string(),
            });
        }

        errors
    }

    fn parse_build_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        // CMake errors
        if input.contains("CMake Error") {
            if input.contains("Could NOT find") {
                let find_re = Regex::new(r"Could NOT find ([a-zA-Z0-9_]+)").unwrap();
                if let Some(caps) = find_re.captures(input) {
                    errors.push(ErrorInfo {
                        message: format!(
                            "CMake dependency not found: {}",
                            caps.get(1).unwrap().as_str()
                        ),
                        location: None,
                        language: "cpp".to_string(),
                    });
                }
            } else {
                errors.push(ErrorInfo {
                    message: "CMake configuration error".to_string(),
                    location: None,
                    language: "cpp".to_string(),
                });
            }
        }

        // Make errors
        if input.contains("No rule to make target") {
            errors.push(ErrorInfo {
                message: "Make target not found".to_string(),
                location: None,
                language: "cpp".to_string(),
            });
        }

        if input.contains("recipe for target") && input.contains("failed") {
            errors.push(ErrorInfo {
                message: "Build recipe failed".to_string(),
                location: None,
                language: "cpp".to_string(),
            });
        }

        errors
    }

    fn parse_warnings(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        // Only parse warnings if they're configured to be treated as errors
        // or if there are no actual errors
        if !input.contains("error:") {
            for (pattern, description) in CPP_WARNING_PATTERNS.iter() {
                let re = Regex::new(pattern).unwrap();
                for caps in re.captures_iter(input) {
                    let location =
                        self.extract_location_from_context(input, caps.get(0).unwrap().start());
                    let message = if caps.len() > 1 {
                        format!(
                            "Warning - {}: {}",
                            description,
                            caps.get(1).unwrap().as_str()
                        )
                    } else {
                        format!("Warning - {}", description)
                    };

                    errors.push(ErrorInfo {
                        message,
                        location,
                        language: "cpp".to_string(),
                    });
                }
            }
        }

        errors
    }

    fn parse_generic_cpp_error(&self, input: &str) -> Option<ErrorInfo> {
        // Check if this looks like C++ content
        let cpp_indicators = [
            ".cpp:", ".cc:", ".cxx:", ".hpp:", ".h:", "g++", "gcc", "clang++", "clang", "#include",
        ];
        let has_cpp_content = cpp_indicators
            .iter()
            .any(|indicator| input.contains(indicator));

        if has_cpp_content
            && (input.contains("error") || input.contains("failed") || input.contains("fatal"))
        {
            let location = self.extract_location(input);
            return Some(ErrorInfo {
                message: "C++ compilation or runtime error".to_string(),
                location,
                language: "cpp".to_string(),
            });
        }

        None
    }

    fn extract_location(&self, input: &str) -> Option<String> {
        if let Some(caps) = CPP_LOCATION_RE.captures(input) {
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

    fn extract_location_from_context(&self, input: &str, error_pos: usize) -> Option<String> {
        // Look for location information near the error
        let context_start = error_pos.saturating_sub(200);
        let context_end = std::cmp::min(error_pos + 200, input.len());
        let context = &input[context_start..context_end];

        self.extract_location(context)
            .or_else(|| self.extract_location(input))
    }

    pub fn extract_gdb_backtrace(&self, input: &str) -> Vec<String> {
        let mut frames = vec![];

        for caps in GDB_BACKTRACE_RE.captures_iter(input) {
            if let Some(function) = caps.get(2) {
                if let Some(file) = caps.get(3) {
                    if let Some(line) = caps.get(4) {
                        frames.push(format!(
                            "{} at {}:{}",
                            function.as_str(),
                            file.as_str(),
                            line.as_str()
                        ));
                    }
                } else if let Some(library) = caps.get(5) {
                    frames.push(format!("{} from {}", function.as_str(), library.as_str()));
                } else {
                    // Frame without location info
                    frames.push(function.as_str().to_string());
                }
            }
        }

        frames
    }

    pub fn extract_valgrind_errors(&self, input: &str) -> Vec<String> {
        let mut errors = vec![];

        for caps in VALGRIND_ERROR_RE.captures_iter(input) {
            if let (Some(error_type), Some(function), Some(file), Some(line)) =
                (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
            {
                errors.push(format!(
                    "{} in {} ({}:{})",
                    error_type.as_str(),
                    function.as_str(),
                    file.as_str(),
                    line.as_str()
                ));
            }
        }

        errors
    }

    pub fn suggest_fixes(&self, error_message: &str) -> Vec<String> {
        let mut fixes = vec![];

        if error_message.contains("Undeclared identifier") {
            fixes.push("Include required header file".to_string());
            fixes.push("Check spelling and scope".to_string());
            fixes.push("Declare variable or function before use".to_string());
        }

        if error_message.contains("Unknown type") {
            fixes.push("Include header defining the type".to_string());
            fixes.push("Check namespace qualifications".to_string());
            fixes.push("Verify forward declarations".to_string());
        }

        if error_message.contains("Function call mismatch") {
            fixes.push("Check function signature and parameters".to_string());
            fixes.push("Include function declaration header".to_string());
            fixes.push("Verify argument types match parameters".to_string());
        }

        if error_message.contains("Undefined symbol") {
            fixes.push("Link required libraries: -l<library>".to_string());
            fixes.push("Add library path: -L<path>".to_string());
            fixes.push("Implement missing function definition".to_string());
        }

        if error_message.contains("Header file not found") {
            fixes.push("Install development packages".to_string());
            fixes.push("Add include path: -I<path>".to_string());
            fixes.push("Check header file name spelling".to_string());
        }

        if error_message.contains("Segmentation fault") {
            fixes.push("Check for null pointer dereference".to_string());
            fixes.push("Verify array bounds access".to_string());
            fixes.push("Use debugger: gdb ./program".to_string());
            fixes.push("Run with AddressSanitizer: -fsanitize=address".to_string());
        }

        if error_message.contains("Memory leak") {
            fixes.push("Match every new with delete".to_string());
            fixes.push("Use smart pointers: std::unique_ptr, std::shared_ptr".to_string());
            fixes.push("Check for exception safety".to_string());
        }

        if error_message.contains("Template") {
            fixes.push("Check template argument types".to_string());
            fixes.push("Verify template constraints are met".to_string());
            fixes.push("Include template definition headers".to_string());
        }

        if error_message.contains("CMake") {
            fixes.push("Install required development packages".to_string());
            fixes.push("Set CMAKE_PREFIX_PATH if needed".to_string());
            fixes.push("Check CMakeLists.txt syntax".to_string());
        }

        fixes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undeclared_identifier() {
        let parser = CppParser;
        let input = r#"
main.cpp:15:5: error: 'undeclaredVar' was not declared in this scope
   15 |     undeclaredVar = 42;
      |     ^~~~~~~~~~~~~
        "#;

        let errors = parser.parse(input);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Undeclared identifier"));
        assert_eq!(errors[0].location, Some("main.cpp:15:5".to_string()));
    }

    #[test]
    fn test_linker_error() {
        let parser = CppParser;
        let input = r#"
/usr/bin/ld: main.o: in function `main':
main.cpp:(.text+0x1e): undefined reference to `missingFunction()'
collect2: error: ld returned 1 exit status
        "#;

        let errors = parser.parse(input);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Undefined symbol"));
    }

    #[test]
    fn test_template_error() {
        let parser = CppParser;
        let input = r#"
template.cpp:25: error: template argument deduction/substitution failed:
template.cpp:25: error: no matching function for call to 'process<int>'
        "#;

        let errors = parser.parse(input);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Template"));
    }

    #[test]
    fn test_sanitizer_error() {
        let parser = CppParser;
        let input = r#"
AddressSanitizer: heap-buffer-overflow on address 0x60200000eff4
READ of size 4 at 0x60200000eff4 thread T0
    #0 0x401234 in main main.cpp:15:10
        "#;

        let errors = parser.parse(input);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Address sanitizer"));
    }

    #[test]
    fn test_gdb_backtrace_extraction() {
        let parser = CppParser;
        let input = r#"
#0  0x0040123f in main () at main.cpp:15
#1  0x7ffff7a05b97 in __libc_start_main () from /lib64/libc.so.6
#2  0x00401179 in _start ()
        "#;

        let frames = parser.extract_gdb_backtrace(input);
        assert_eq!(frames.len(), 3);
        assert!(frames[0].contains("main () at main.cpp:15"));
    }

    #[test]
    fn test_fix_suggestions() {
        let parser = CppParser;
        let fixes = parser.suggest_fixes("Undeclared identifier: someFunction");
        assert!(fixes.iter().any(|f| f.contains("Include required header")));
    }
}
