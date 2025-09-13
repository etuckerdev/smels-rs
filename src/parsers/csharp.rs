use crate::{ErrorInfo, Parser};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

pub struct CsharpParser;

lazy_static! {
    static ref CSHARP_ERROR_PATTERNS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();

        // Compilation errors
        m.insert(r"CS0103:\s*The name '([^']+)' does not exist", "Undefined identifier");
        m.insert(r"CS0246:\s*The type or namespace name '([^']+)' could not be found", "Type not found");
        m.insert(r"CS0029:\s*Cannot implicitly convert type '([^']+)' to '([^']+)'", "Implicit conversion error");
        m.insert(r"CS0117:\s*'([^']+)' does not contain a definition for '([^']+)'", "Member not found");
        m.insert(r"CS1061:\s*'([^']+)' does not contain a definition for '([^']+)'", "Method/property not found");
        m.insert(r"CS0019:\s*Operator '([^']+)' cannot be applied to operands", "Invalid operator usage");
        m.insert(r"CS0120:\s*An object reference is required", "Static context error");
        m.insert(r"CS0161:\s*'([^']+)': not all code paths return a value", "Missing return statement");
        m.insert(r"CS0162:\s*Unreachable code detected", "Dead code warning");
        m.insert(r"CS0165:\s*Use of unassigned local variable '([^']+)'", "Uninitialized variable");
        m.insert(r"CS0266:\s*Cannot implicitly convert type.*explicit conversion exists", "Explicit conversion required");
        m.insert(r"CS1729:\s*'([^']+)' does not contain a constructor that takes \\d+ arguments", "Constructor mismatch");

        // Access modifier errors
        m.insert(r"CS0122:\s*'([^']+)' is inaccessible due to its protection level", "Access level error");
        m.insert(r"CS0101:\s*The namespace '([^']+)' already contains a definition for '([^']+)'", "Duplicate definition");
        m.insert(r"CS0102:\s*The type '([^']+)' already contains a definition for '([^']+)'", "Member already defined");

        // Generic/Template errors
        m.insert(r"CS0305:\s*Using the generic type '([^']+)' requires \\d+ type arguments", "Generic argument count mismatch");
        m.insert(r"CS0309:\s*The type '([^']+)' must be convertible to '([^']+)'", "Generic constraint violation");
        m.insert(r"CS0452:\s*The type '([^']+)' must be a reference type", "Reference type constraint violation");

        // Nullable reference errors (.NET 6+)
        m.insert(r"CS8600:\s*Converting null literal.*to non-nullable reference type", "Null assignment to non-nullable");
        m.insert(r"CS8602:\s*Dereference of a possibly null reference", "Possible null reference");
        m.insert(r"CS8604:\s*Possible null reference argument", "Null reference argument");

        // Interface/Abstract errors
        m.insert(r"CS0144:\s*Cannot create an instance of the abstract class or interface '([^']+)'", "Abstract instantiation");
        m.insert(r"CS0535:\s*'([^']+)' does not implement interface member '([^']+)'", "Interface not implemented");
        m.insert(r"CS0534:\s*'([^']+)' does not implement inherited abstract member '([^']+)'", "Abstract member not implemented");

        // Async/await errors
        m.insert(r"CS4032:\s*The 'await' operator can only be used within an async method", "Await outside async method");
        m.insert(r"CS1983:\s*The return type of an async method must be void, Task.*", "Invalid async return type");

        m
    };

    static ref CSHARP_RUNTIME_PATTERNS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();

        // Common runtime exceptions
        m.insert(r"System\.NullReferenceException", "Null reference exception");
        m.insert(r"System\\.ArgumentNullException", "Null argument exception");
        m.insert(r"System\\.ArgumentException", "Invalid argument exception");
        m.insert(r"System\\.ArgumentOutOfRangeException", "Argument out of range");
        m.insert(r"System\\.IndexOutOfRangeException", "Array index out of range");
        m.insert(r"System\\.InvalidOperationException", "Invalid operation");
        m.insert(r"System\\.NotImplementedException", "Method not implemented");
        m.insert(r"System\\.NotSupportedException", "Operation not supported");
        m.insert(r"System\\.FormatException", "String format error");
        m.insert(r"System\\.OverflowException", "Arithmetic overflow");
        m.insert(r"System\\.DivideByZeroException", "Division by zero");
        m.insert(r"System\\.OutOfMemoryException", "Out of memory");
        m.insert(r"System\\.StackOverflowException", "Stack overflow");
        m.insert(r"System\\.AccessViolationException", "Memory access violation");

        // IO exceptions
        m.insert(r"System\\.IO\\.FileNotFoundException", "File not found");
        m.insert(r"System\\.IO\\.DirectoryNotFoundException", "Directory not found");
        m.insert(r"System\\.IO\\.IOException", "Input/output error");
        m.insert(r"System\\.UnauthorizedAccessException", "Access denied");

        // Network exceptions
        m.insert(r"System\\.Net\\.WebException", "Web request exception");
        m.insert(r"System\\.Net\\.Sockets\\.SocketException", "Socket error");
        m.insert(r"System\\.Net\\.HttpRequestException", "HTTP request failed");

        // Threading exceptions
        m.insert(r"System\\.Threading\\.ThreadAbortException", "Thread aborted");
        m.insert(r"System\\.Threading\\.SynchronizationLockException", "Lock synchronization error");
        m.insert(r"System\\.Threading\\.Tasks\\.TaskCanceledException", "Task was cancelled");

        // Serialization exceptions
        m.insert(r"System\\.Text\\.Json\\.JsonException", "JSON parsing error");
        m.insert(r"Newtonsoft\\.Json\\.JsonException", "JSON.NET parsing error");
        m.insert(r"System\\.Xml\\.XmlException", "XML parsing error");

        m
    };

    static ref CSHARP_BUILD_PATTERNS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert(r"Build FAILED", "Build failed");
        m.insert(r"error MSB\\d+", "MSBuild error");
        m.insert(r"The command .* exited with code \\d+", "Build command failed");
        m.insert(r"Could not load file or assembly", "Assembly load error");
        m.insert(r"Package '([^']+)' is not found", "NuGet package not found");
        m.insert(r"Version conflict detected", "Package version conflict");
        m
    };

    static ref CSHARP_LOCATION_RE: Regex = Regex::new(
        r"([^(\s]+\.cs)\((\d+),(\d+)\)"
    ).unwrap();

    static ref CSHARP_SIMPLE_LOCATION_RE: Regex = Regex::new(
        r"([^:\\s]+\\.cs):(\\d+):?(\\d+)?"
    ).unwrap();

    static ref CSHARP_STACK_FRAME_RE: Regex = Regex::new(
        r"at\s+([^\s(]+(?:\([^)]*\))?)\s+in\s+(.+?):(line\s+)?(\d+)"
    ).unwrap();

    static ref DOTNET_CLI_ERROR_RE: Regex = Regex::new(
        r"error\\s+(\\w+):\\s*(.+)"
    ).unwrap();
}

impl Parser for CsharpParser {
    fn parse(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        // Parse compilation errors
        errors.extend(self.parse_compilation_errors(input));

        // Parse runtime exceptions
        errors.extend(self.parse_runtime_errors(input));

        // Parse build system errors
        errors.extend(self.parse_build_errors(input));

        // Parse .NET CLI errors
        errors.extend(self.parse_dotnet_cli_errors(input));

        // Generic C# error detection as fallback
        if errors.is_empty() {
            if let Some(generic_error) = self.parse_generic_csharp_error(input) {
                errors.push(generic_error);
            }
        }

        errors
    }
}

impl CsharpParser {
    fn parse_compilation_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        for (pattern, description) in CSHARP_ERROR_PATTERNS.iter() {
            let re = Regex::new(pattern).unwrap();
            for caps in re.captures_iter(input) {
                let location =
                    self.extract_location_from_context(input, caps.get(0).unwrap().start());
                let message = if caps.len() > 1 {
                    let mut msg = description.to_string();
                    if let Some(detail) = caps.get(1) {
                        msg = format!("{}: {}", msg, detail.as_str());
                    }
                    msg
                } else {
                    description.to_string()
                };

                errors.push(ErrorInfo {
                    message,
                    location,
                    language: "csharp".to_string(),
                });
            }
        }

        errors
    }

    fn parse_runtime_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        for (pattern, description) in CSHARP_RUNTIME_PATTERNS.iter() {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(input) {
                let location = self.extract_location(input);
                errors.push(ErrorInfo {
                    message: description.to_string(),
                    location,
                    language: "csharp".to_string(),
                });
            }
        }

        // Unhandled exception detection
        if input.contains("Unhandled exception") || input.contains("Unhandled Exception") {
            let location = self.extract_location(input);
            errors.push(ErrorInfo {
                message: "Unhandled exception occurred".to_string(),
                location,
                language: "csharp".to_string(),
            });
        }

        errors
    }

    fn parse_build_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        for (pattern, description) in CSHARP_BUILD_PATTERNS.iter() {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(input) {
                errors.push(ErrorInfo {
                    message: description.to_string(),
                    location: None,
                    language: "csharp".to_string(),
                });
            }
        }

        // NuGet restore errors
        if input.contains("NU") && (input.contains("error") || input.contains("warning")) {
            let nu_error_re = Regex::new(r"(NU\\d+):\\s*(.+)").unwrap();
            for caps in nu_error_re.captures_iter(input) {
                if let (Some(code), Some(message)) = (caps.get(1), caps.get(2)) {
                    errors.push(ErrorInfo {
                        message: format!("NuGet {}: {}", code.as_str(), message.as_str()),
                        location: None,
                        language: "csharp".to_string(),
                    });
                }
            }
        }

        errors
    }

    fn parse_dotnet_cli_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        // .NET CLI specific errors
        if input.contains("dotnet") && input.contains("error") {
            for caps in DOTNET_CLI_ERROR_RE.captures_iter(input) {
                if let (Some(code), Some(message)) = (caps.get(1), caps.get(2)) {
                    errors.push(ErrorInfo {
                        message: format!(".NET CLI {}: {}", code.as_str(), message.as_str()),
                        location: None,
                        language: "csharp".to_string(),
                    });
                }
            }
        }

        // Project file errors
        if input.contains(".csproj") && input.contains("error") {
            errors.push(ErrorInfo {
                message: "Project file error".to_string(),
                location: None,
                language: "csharp".to_string(),
            });
        }

        errors
    }

    fn parse_generic_csharp_error(&self, input: &str) -> Option<ErrorInfo> {
        let csharp_indicators = [
            ".cs:",
            ".csproj",
            "dotnet",
            "msbuild",
            "System.",
            "namespace",
            "using System",
        ];
        let has_csharp_content = csharp_indicators
            .iter()
            .any(|indicator| input.contains(indicator));

        if has_csharp_content
            && (input.contains("error") || input.contains("exception") || input.contains("failed"))
        {
            let location = self.extract_location(input);
            return Some(ErrorInfo {
                message: "C# compilation or runtime error".to_string(),
                location,
                language: "csharp".to_string(),
            });
        }

        None
    }

    fn extract_location(&self, input: &str) -> Option<String> {
        // Try Visual Studio format: File.cs(line,column)
        if let Some(caps) = CSHARP_LOCATION_RE.captures(input) {
            let file = caps.get(1)?.as_str();
            let line = caps.get(2)?.as_str();
            let col = caps.get(3)?.as_str();
            return Some(format!("{file}:{line}:{col}"));
        }

        // Try simpler format: File.cs:line:column
        if let Some(caps) = CSHARP_SIMPLE_LOCATION_RE.captures(input) {
            let file = caps.get(1)?.as_str();
            let line = caps.get(2)?.as_str();

            if let Some(col) = caps.get(3) {
                return Some(format!("{}:{}:{}", file, line, col.as_str()));
            } else {
                return Some(format!("{file}:{line}"));
            }
        }

        None
    }

    fn extract_location_from_context(&self, input: &str, error_pos: usize) -> Option<String> {
        let context_start = error_pos.saturating_sub(200);
        let context_end = std::cmp::min(error_pos + 200, input.len());
        let context = &input[context_start..context_end];

        self.extract_location(context)
            .or_else(|| self.extract_location(input))
    }

    pub fn extract_stack_trace(&self, input: &str) -> Vec<String> {
        let mut frames = vec![];

        for caps in CSHARP_STACK_FRAME_RE.captures_iter(input) {
            if let (Some(method), Some(file), Some(line)) = (caps.get(1), caps.get(2), caps.get(4))
            {
                frames.push(format!(
                    "{} in {}:{}",
                    method.as_str(),
                    file.as_str(),
                    line.as_str()
                ));
            }
        }

        frames
    }

    pub fn suggest_fixes(&self, error_message: &str) -> Vec<String> {
        let mut fixes = vec![];

        if error_message.contains("Undefined identifier") {
            fixes.push("Add using directive for required namespace".to_string());
            fixes.push("Check spelling and capitalization".to_string());
            fixes.push("Ensure variable is declared in scope".to_string());
        }

        if error_message.contains("Type not found") {
            fixes.push("Add reference to required assembly".to_string());
            fixes.push("Install NuGet package: dotnet add package <PackageName>".to_string());
            fixes.push("Add using directive for namespace".to_string());
        }

        if error_message.contains("Null reference exception") {
            fixes.push("Add null check: if (obj != null)".to_string());
            fixes.push("Use null-conditional operator: obj?.Property".to_string());
            fixes.push("Initialize object before use".to_string());
        }

        if error_message.contains("Member not found") {
            fixes.push("Check method/property name spelling".to_string());
            fixes.push("Verify object type has the member".to_string());
            fixes.push("Add required using statements".to_string());
        }

        if error_message.contains("conversion error") {
            fixes.push("Use explicit cast: (TargetType)value".to_string());
            fixes.push("Use Convert class: Convert.ToInt32()".to_string());
            fixes.push("Use TryParse for safe conversion".to_string());
        }

        if error_message.contains("NuGet") {
            fixes.push("dotnet restore".to_string());
            fixes.push("Clear NuGet cache: dotnet nuget locals all --clear".to_string());
            fixes.push("Update package references in .csproj".to_string());
        }

        if error_message.contains("Build failed") {
            fixes.push("dotnet clean && dotnet build".to_string());
            fixes.push("Check for compilation errors above".to_string());
            fixes.push("Verify all references and dependencies".to_string());
        }

        if error_message.contains("Assembly load error") {
            fixes.push("Check target framework compatibility".to_string());
            fixes.push("Verify assembly is in output directory".to_string());
            fixes.push("Update binding redirects if needed".to_string());
        }

        if error_message.contains("Uninitialized variable") {
            fixes.push("Initialize variable before use".to_string());
            fixes.push("Assign value in all code paths".to_string());
            fixes.push("Use default value or nullable type".to_string());
        }

        if error_message.contains("Possible null reference") {
            fixes.push("Enable nullable reference types".to_string());
            fixes.push("Add null checks or null-forgiving operator (!)".to_string());
            fixes.push("Initialize non-nullable properties".to_string());
        }

        fixes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undefined_identifier() {
        let parser = CsharpParser;
        let input = r#"
Program.cs(15,13): error CS0103: The name 'undefinedVar' does not exist in the current context
        "#;

        let errors = parser.parse(input);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Undefined identifier"));
        assert_eq!(errors[0].location, Some("Program.cs:15:13".to_string()));
    }

    #[test]
    fn test_null_reference_exception() {
        let parser = CsharpParser;
        let input = r#"
Unhandled exception. System.NullReferenceException: Object reference not set to an instance of an object.
   at Program.Main() in C:\project\Program.cs:line 20
        "#;

        let errors = parser.parse(input);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Null reference exception"));
    }

    #[test]
    fn test_build_error() {
        let parser = CsharpParser;
        let input = r#"
Build FAILED.
error MSB3073: The command "echo test" exited with code 1.
        "#;

        let errors = parser.parse(input);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("Build failed")));
    }

    #[test]
    fn test_stack_trace_extraction() {
        let parser = CsharpParser;
        let input = r#"
   at System.String.GetHashCode() in System.String.cs:line 350
   at Program.ProcessData() in C:\project\Program.cs:line 25
   at Program.Main() in C:\project\Program.cs:line 10
        "#;

        let frames = parser.extract_stack_trace(input);
        assert_eq!(frames.len(), 3);
        assert!(frames[1].contains("Program.ProcessData"));
        assert!(frames[1].contains("Program.cs:25"));
    }

    #[test]
    fn test_fix_suggestions() {
        let parser = CsharpParser;
        let fixes = parser.suggest_fixes("Type not found: SomeClass");
        assert!(fixes.iter().any(|f| f.contains("Add reference")));
    }
}
