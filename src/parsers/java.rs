use crate::{Parser, ErrorInfo};
use regex::Regex;
use std::collections::HashMap;
use lazy_static::lazy_static;

pub struct JavaParser;

lazy_static! {
    static ref EXCEPTION_PATTERNS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        
        // Runtime exceptions
        m.insert(r"java\.lang\.NullPointerException", "Null pointer exception - accessing null object");
        m.insert(r"java\.lang\.ArrayIndexOutOfBoundsException", "Array index out of bounds");
        m.insert(r"java\.lang\.StringIndexOutOfBoundsException", "String index out of bounds");
        m.insert(r"java\.lang\.ClassCastException", "Invalid type casting");
        m.insert(r"java\.lang\.IllegalArgumentException", "Invalid method argument");
        m.insert(r"java\.lang\.IllegalStateException", "Object in illegal state");
        m.insert(r"java\.lang\.NumberFormatException", "Invalid number format conversion");
        m.insert(r"java\.lang\.ArithmeticException", "Arithmetic error (division by zero)");
        m.insert(r"java\.lang\.OutOfMemoryError", "Out of memory (heap space)");
        m.insert(r"java\.lang\.StackOverflowError", "Stack overflow (infinite recursion)");
        
        // IO exceptions
        m.insert(r"java\.io\.FileNotFoundException", "File not found");
        m.insert(r"java\.io\.IOException", "Input/output operation failed");
        m.insert(r"java\.io\.EOFException", "Unexpected end of file");
        m.insert(r"java\.nio\.file\.NoSuchFileException", "File or directory does not exist");
        m.insert(r"java\.security\.AccessControlException", "Security access denied");
        
        // Network exceptions
        m.insert(r"java\.net\.ConnectException", "Network connection failed");
        m.insert(r"java\.net\.SocketTimeoutException", "Network operation timed out");
        m.insert(r"java\.net\.UnknownHostException", "Host name resolution failed");
        m.insert(r"java\.net\.BindException", "Port already in use");
        
        // Reflection exceptions
        m.insert(r"java\.lang\.ClassNotFoundException", "Class not found in classpath");
        m.insert(r"java\.lang\.NoSuchMethodException", "Method not found");
        m.insert(r"java\.lang\.NoSuchFieldException", "Field not found");
        m.insert(r"java\.lang\.InstantiationException", "Cannot instantiate abstract class/interface");
        
        // Concurrency exceptions
        m.insert(r"java\.util\.concurrent\.TimeoutException", "Operation timed out");
        m.insert(r"java\.lang\.InterruptedException", "Thread was interrupted");
        m.insert(r"java\.util\.concurrent\.ExecutionException", "Async execution failed");
        
        // Database exceptions
        m.insert(r"java\.sql\.SQLException", "Database operation failed");
        m.insert(r"java\.sql\.SQLSyntaxErrorException", "Invalid SQL syntax");
        m.insert(r"java\.sql\.SQLIntegrityConstraintViolationException", "Database constraint violation");
        
        // Serialization exceptions
        m.insert(r"java\.io\.NotSerializableException", "Object not serializable");
        m.insert(r"java\.io\.InvalidClassException", "Serialization class mismatch");
        
        // Spring Framework exceptions
        m.insert(r"org\.springframework", "Spring Framework error");
        m.insert(r"NoSuchBeanDefinitionException", "Spring bean not found");
        m.insert(r"BeanCreationException", "Spring bean creation failed");
        
        m
    };

    static ref COMPILATION_PATTERNS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        
        m.insert(r"cannot find symbol", "Undefined variable, method, or class");
        m.insert(r"package .* does not exist", "Package not found or not imported");
        m.insert(r"incompatible types", "Type mismatch");
        m.insert(r"method .* cannot be applied", "Incorrect method parameters");
        m.insert(r"constructor .* cannot be applied", "Incorrect constructor parameters");
        m.insert(r"abstract method .* cannot be accessed directly", "Abstract method access error");
        m.insert(r"non-static .* cannot be referenced from a static context", "Static context violation");
        m.insert(r"variable .* might not have been initialized", "Uninitialized variable");
        m.insert(r"unreachable statement", "Dead code detected");
        m.insert(r"missing return statement", "Method missing return statement");
        m.insert(r"duplicate method", "Method already defined");
        m.insert(r"class .* is public, should be declared in a file named", "File name mismatch");
        
        m
    };

    static ref STACK_TRACE_RE: Regex = Regex::new(
        r"at ([^(]+)\(([^:)]+):(\d+)\)"
    ).unwrap();
    
    static ref LOCATION_RE: Regex = Regex::new(
        r"([^:]+\.java):(\d+):"
    ).unwrap();
    
    static ref CAUSED_BY_RE: Regex = Regex::new(
        r"Caused by: ([^\n\r]+)"
    ).unwrap();
}

impl Parser for JavaParser {
    fn parse(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];
        
        // Check for runtime exceptions
        if let Some(runtime_error) = self.parse_runtime_exception(input) {
            errors.push(runtime_error);
        }
        
        // Check for compilation errors
        errors.extend(self.parse_compilation_errors(input));
        
        // Check for build tool errors
        errors.extend(self.parse_build_errors(input));
        
        // Check for framework-specific errors
        errors.extend(self.parse_framework_errors(input));
        
        errors
    }
}

impl JavaParser {
    fn parse_runtime_exception(&self, input: &str) -> Option<ErrorInfo> {
        // Look for specific exception patterns
        for (pattern, description) in EXCEPTION_PATTERNS.iter() {
            let re = Regex::new(pattern).ok()?;
            if re.is_match(input) {
                let location = self.extract_location(input);
                let root_cause = self.extract_root_cause(input);
                
                let message = if let Some(cause) = root_cause {
                    format!("{} ({})", description, cause)
                } else {
                    description.to_string()
                };
                
                return Some(ErrorInfo {
                    message,
                    location,
                    language: "java".to_string(),
                });
            }
        }
        
        // Generic exception detection
        if input.contains("Exception") || input.contains("Error") {
            let location = self.extract_location(input);
            return Some(ErrorInfo {
                message: "Java runtime exception".to_string(),
                location,
                language: "java".to_string(),
            });
        }
        
        None
    }
    
    fn parse_compilation_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];
        
        for (pattern, description) in COMPILATION_PATTERNS.iter() {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(input) {
                let location = self.extract_location(input);
                errors.push(ErrorInfo {
                    message: format!("Compilation error: {}", description),
                    location,
                    language: "java".to_string(),
                });
            }
        }
        
        // Generic compilation error patterns
        if input.contains("javac") && input.contains("error:") {
            if errors.is_empty() {
                let location = self.extract_location(input);
                errors.push(ErrorInfo {
                    message: "Java compilation failed".to_string(),
                    location,
                    language: "java".to_string(),
                });
            }
        }
        
        errors
    }
    
    fn parse_build_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];
        
        // Maven errors
        if input.contains("BUILD FAILURE") || input.contains("[ERROR]") {
            if input.contains("dependency") {
                errors.push(ErrorInfo {
                    message: "Maven dependency resolution failed".to_string(),
                    location: None,
                    language: "java".to_string(),
                });
            } else if input.contains("compilation failure") {
                errors.push(ErrorInfo {
                    message: "Maven compilation failed".to_string(),
                    location: None,
                    language: "java".to_string(),
                });
            } else {
                errors.push(ErrorInfo {
                    message: "Maven build failed".to_string(),
                    location: None,
                    language: "java".to_string(),
                });
            }
        }
        
        // Gradle errors
        if input.contains("FAILURE: Build failed") || input.contains("Execution failed") {
            if input.contains("Could not resolve") {
                errors.push(ErrorInfo {
                    message: "Gradle dependency resolution failed".to_string(),
                    location: None,
                    language: "java".to_string(),
                });
            } else {
                errors.push(ErrorInfo {
                    message: "Gradle build failed".to_string(),
                    location: None,
                    language: "java".to_string(),
                });
            }
        }
        
        // Classpath issues
        if input.contains("ClassNotFoundException") || input.contains("NoClassDefFoundError") {
            errors.push(ErrorInfo {
                message: "Classpath issue - missing JAR or class file".to_string(),
                location: None,
                language: "java".to_string(),
            });
        }
        
        errors
    }
    
    fn parse_framework_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];
        
        // Spring Framework errors
        if input.contains("org.springframework") {
            if input.contains("NoSuchBeanDefinitionException") {
                errors.push(ErrorInfo {
                    message: "Spring: Bean not found or not properly configured".to_string(),
                    location: None,
                    language: "java".to_string(),
                });
            } else if input.contains("BeanCreationException") {
                errors.push(ErrorInfo {
                    message: "Spring: Bean creation failed".to_string(),
                    location: None,
                    language: "java".to_string(),
                });
            } else {
                errors.push(ErrorInfo {
                    message: "Spring Framework error".to_string(),
                    location: None,
                    language: "java".to_string(),
                });
            }
        }
        
        // Hibernate/JPA errors
        if input.contains("org.hibernate") || input.contains("javax.persistence") {
            errors.push(ErrorInfo {
                message: "Hibernate/JPA database mapping error".to_string(),
                location: None,
                language: "java".to_string(),
            });
        }
        
        // JUnit test errors
        if input.contains("org.junit") || input.contains("AssertionError") {
            errors.push(ErrorInfo {
                message: "JUnit test failure".to_string(),
                location: self.extract_location(input),
                language: "java".to_string(),
            });
        }
        
        errors
    }
    
    fn extract_location(&self, input: &str) -> Option<String> {
        // Look for Java file:line patterns
        if let Some(caps) = LOCATION_RE.captures(input) {
            let file = caps.get(1)?.as_str();
            let line = caps.get(2)?.as_str();
            return Some(format!("{}:{}", file, line));
        }
        
        // Look in stack traces
        for caps in STACK_TRACE_RE.captures_iter(input) {
            if let (Some(file), Some(line)) = (caps.get(2), caps.get(3)) {
                let filename = file.as_str();
                if filename.ends_with(".java") {
                    return Some(format!("{}:{}", filename, line.as_str()));
                }
            }
        }
        
        None
    }
    
    fn extract_root_cause(&self, input: &str) -> Option<String> {
        if let Some(caps) = CAUSED_BY_RE.captures(input) {
            return Some(caps.get(1)?.as_str().trim().to_string());
        }
        None
    }
    
    pub fn extract_stack_trace(&self, input: &str) -> Vec<String> {
        let mut frames = vec![];
        
        for caps in STACK_TRACE_RE.captures_iter(input) {
            if let (Some(method), Some(file), Some(line)) = 
                (caps.get(1), caps.get(2), caps.get(3)) {
                frames.push(format!("{} ({}:{})", 
                    method.as_str(), file.as_str(), line.as_str()));
            }
        }
        
        frames
    }
    
    pub fn suggest_fixes(&self, error_message: &str) -> Vec<String> {
        let mut fixes = vec![];
        
        if error_message.contains("NullPointerException") {
            fixes.push("Add null check: if (obj != null)".to_string());
            fixes.push("Initialize object before use".to_string());
            fixes.push("Use Optional<T> for nullable values".to_string());
        }
        
        if error_message.contains("ClassNotFoundException") {
            fixes.push("Add missing JAR to classpath".to_string());
            fixes.push("Check Maven/Gradle dependencies".to_string());
            fixes.push("Verify class name and package".to_string());
        }
        
        if error_message.contains("ArrayIndexOutOfBoundsException") {
            fixes.push("Check array bounds: if (i < array.length)".to_string());
            fixes.push("Use enhanced for loop: for (Type item : array)".to_string());
            fixes.push("Validate array size before access".to_string());
        }
        
        if error_message.contains("cannot find symbol") {
            fixes.push("Check variable/method spelling".to_string());
            fixes.push("Import required classes".to_string());
            fixes.push("Declare variable before use".to_string());
        }
        
        if error_message.contains("incompatible types") {
            fixes.push("Cast to correct type: (TargetType) object".to_string());
            fixes.push("Check method return types".to_string());
            fixes.push("Use proper generic types".to_string());
        }
        
        if error_message.contains("Maven") && error_message.contains("dependency") {
            fixes.push("mvn clean install".to_string());
            fixes.push("Update Maven dependencies in pom.xml".to_string());
            fixes.push("Check Maven repository connectivity".to_string());
        }
        
        if error_message.contains("Gradle") {
            fixes.push("./gradlew clean build".to_string());
            fixes.push("Update Gradle dependencies".to_string());
            fixes.push("Check Gradle wrapper version".to_string());
        }
        
        if error_message.contains("Spring") {
            fixes.push("Check Spring configuration files".to_string());
            fixes.push("Verify component scanning packages".to_string());
            fixes.push("Add missing @Component, @Service, or @Repository annotations".to_string());
        }
        
        fixes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_null_pointer_exception() {
        let parser = JavaParser;
        let input = r#"
Exception in thread "main" java.lang.NullPointerException
    at com.example.Main.processData(Main.java:25)
    at com.example.Main.main(Main.java:10)
        "#;
        
        let errors = parser.parse(input);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Null pointer exception"));
        assert_eq!(errors[0].location, Some("Main.java:25".to_string()));
    }
    
    #[test]
    fn test_compilation_error() {
        let parser = JavaParser;
        let input = r#"
Main.java:15: error: cannot find symbol
        System.out.println(undefinedVariable);
                           ^
  symbol:   variable undefinedVariable
  location: class Main
        "#;
        
        let errors = parser.parse(input);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Compilation error"));
    }
    
    #[test]
    fn test_stack_trace_extraction() {
        let parser = JavaParser;
        let input = r#"
java.lang.ArrayIndexOutOfBoundsException: Index 5 out of bounds for length 3
    at com.example.ArrayProcessor.process(ArrayProcessor.java:42)
    at com.example.Main.main(Main.java:15)
        "#;
        
        let frames = parser.extract_stack_trace(input);
        assert_eq!(frames.len(), 2);
        assert!(frames[0].contains("ArrayProcessor.process"));
        assert!(frames[0].contains("ArrayProcessor.java:42"));
    }
    
    #[test]
    fn test_maven_build_error() {
        let parser = JavaParser;
        let input = r#"
[ERROR] Failed to execute goal org.apache.maven.plugins:maven-compiler-plugin:3.8.1:compile
[ERROR] BUILD FAILURE
        "#;
        
        let errors = parser.parse(input);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Maven"));
    }
}
