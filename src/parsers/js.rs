use crate::{ErrorInfo, Parser};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

pub struct JsParser;

lazy_static! {
    static ref ERROR_PATTERNS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();

        // Module/Import errors
        m.insert(r"(?:Error:\s*)?Cannot find module '([^']+)'", "Module not found");
        m.insert(r"Module not found: Error: Can't resolve '([^']+)'", "Webpack module resolution failed");
        m.insert(r"ENOENT: no such file or directory", "File or directory not found");
        m.insert(r"Cannot resolve dependency '([^']+)'", "Dependency resolution failed");
        m.insert(r"Module parse failed", "Module parsing error");

        // Network/Server errors
        m.insert(r"EADDRINUSE.*:(\d+)", "Port already in use");
        m.insert(r"EACCES.*permission denied", "Permission denied");
        m.insert(r"ECONNREFUSED", "Connection refused");
        m.insert(r"ETIMEDOUT", "Connection timed out");
        m.insert(r"ENOTFOUND", "Host not found");
        m.insert(r"getaddrinfo ENOTFOUND", "DNS resolution failed");

        // Runtime errors
        m.insert(r"ReferenceError: ([^\s]+) is not defined", "Undefined variable or function");
        m.insert(r"TypeError: Cannot read propert(?:y|ies) of (null|undefined)", "Null/undefined property access");
        m.insert(r"TypeError: ([^\s]+) is not a function", "Attempting to call non-function");
        m.insert(r"TypeError: Cannot set propert(?:y|ies) of (null|undefined)", "Cannot set property on null/undefined");
        m.insert(r"RangeError: Maximum call stack size exceeded", "Stack overflow/infinite recursion");
        m.insert(r"SyntaxError: Unexpected token", "Invalid JavaScript syntax");
        m.insert(r"SyntaxError: Unexpected end of JSON input", "Malformed JSON");
        m.insert(r"URIError", "Invalid URI/URL format");

        // Promise/Async errors
        m.insert(r"UnhandledPromiseRejectionWarning", "Unhandled promise rejection");
        m.insert(r"DeprecationWarning.*Unhandled promise rejections", "Unhandled async error");

        // Framework-specific errors
        // m.insert(r"Error: Cannot find module.*express", "Express.js not installed"); // Removed - handled by generic pattern
        m.insert(r"Error: Cannot find module.*react", "React not installed");
        m.insert(r"Error: Cannot find module.*vue", "Vue.js not installed");
        m.insert(r"Error: Cannot find module.*angular", "Angular not installed");
        m.insert(r"Error: Cannot find module.*next", "Next.js not installed");

        // Build tool errors
        m.insert(r"webpack.*Error", "Webpack build error");
        m.insert(r"Vite.*Error", "Vite build error");
        m.insert(r"Rollup.*Error", "Rollup build error");
        m.insert(r"Babel.*Error", "Babel transpilation error");
        m.insert(r"TypeScript.*error", "TypeScript compilation error");

        // Package manager errors
        m.insert(r"npm ERR!", "npm error");
        m.insert(r"yarn error", "Yarn error");
        m.insert(r"pnpm ERR", "pnpm error");
        m.insert(r"ERESOLVE unable to resolve dependency tree", "Dependency conflict");
        m.insert(r"peer dep missing", "Missing peer dependency");

        // Memory/Performance errors
        m.insert(r"JavaScript heap out of memory", "Out of memory");
        m.insert(r"FATAL ERROR: Ineffective mark-compacts", "Memory allocation failed");

        m
    };

    static ref STACK_TRACE_RE: Regex = Regex::new(
        r"at (?:([^(]+) \()?([^:]+):(\d+):(\d+)\)?"
    ).unwrap();

    static ref LOCATION_RE: Regex = Regex::new(
        r"(?:at|in) ([^:]+):(\d+):?(\d+)?"
    ).unwrap();

    static ref CONSOLE_ERROR_RE: Regex = Regex::new(
        r"console\.(?:error|warn|log)"
    ).unwrap();
}

impl Parser for JsParser {
    fn parse(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        // Check for JavaScript runtime errors
        if let Some(runtime_error) = self.parse_runtime_error(input) {
            // Check if it's a port/network error before pushing
            let is_network_error = runtime_error.message.contains("Port")
                || runtime_error.message.contains("Connection")
                || runtime_error.message.contains("Host");

            errors.push(runtime_error);

            // If it's a port/network error, skip network parsing to avoid duplicates
            if is_network_error {
                return errors;
            }
        }

        // Check for build/compile errors
        errors.extend(self.parse_build_errors(input));

        // Check for package manager errors
        errors.extend(self.parse_package_errors(input));

        // Check for network/server errors (only if not already found)
        errors.extend(self.parse_network_errors(input));

        // Check for framework-specific errors
        errors.extend(self.parse_framework_errors(input));

        errors
    }
}

impl JsParser {
    fn parse_runtime_error(&self, input: &str) -> Option<ErrorInfo> {
        // Look for specific error patterns
        for (pattern, description) in ERROR_PATTERNS.iter() {
            let re = Regex::new(pattern).ok()?;
            if let Some(caps) = re.captures(input) {
                let location = self.extract_location(input);
                let mut message = description.to_string();

                // Add captured details for more context
                if caps.len() > 1 {
                    if let Some(detail) = caps.get(1) {
                        message = format!("{}: {}", description, detail.as_str());
                    }
                }

                return Some(ErrorInfo {
                    message,
                    location,
                    language: "javascript".to_string(),
                });
            }
        }

        // Generic error detection
        if input.contains("Error:")
            || input.contains("TypeError:")
            || input.contains("ReferenceError:")
        {
            let location = self.extract_location(input);
            return Some(ErrorInfo {
                message: "JavaScript runtime error".to_string(),
                location,
                language: "javascript".to_string(),
            });
        }

        None
    }

    fn parse_build_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        if input.contains("webpack") && input.contains("ERROR") {
            let location = self.extract_location(input);
            errors.push(ErrorInfo {
                message: "Webpack build failed".to_string(),
                location,
                language: "javascript".to_string(),
            });
        }

        if input.contains("Module build failed") {
            let location = self.extract_location(input);
            errors.push(ErrorInfo {
                message: "Module compilation failed".to_string(),
                location,
                language: "javascript".to_string(),
            });
        }

        if input.contains("TypeScript error") || input.contains("TS") && input.contains("error") {
            let location = self.extract_location(input);
            errors.push(ErrorInfo {
                message: "TypeScript compilation error".to_string(),
                location,
                language: "typescript".to_string(),
            });
        }

        if input.contains("Babel") && input.contains("error") {
            let location = self.extract_location(input);
            errors.push(ErrorInfo {
                message: "Babel transpilation failed".to_string(),
                location,
                language: "javascript".to_string(),
            });
        }

        errors
    }

    fn parse_package_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        if input.contains("npm ERR!") {
            if input.contains("ERESOLVE") {
                errors.push(ErrorInfo {
                    message: "npm dependency resolution conflict".to_string(),
                    location: None,
                    language: "javascript".to_string(),
                });
            } else if input.contains("EACCES") {
                errors.push(ErrorInfo {
                    message: "npm permission error".to_string(),
                    location: None,
                    language: "javascript".to_string(),
                });
            } else {
                errors.push(ErrorInfo {
                    message: "npm installation error".to_string(),
                    location: None,
                    language: "javascript".to_string(),
                });
            }
        }

        if input.contains("peer dep missing") {
            errors.push(ErrorInfo {
                message: "Missing peer dependency".to_string(),
                location: None,
                language: "javascript".to_string(),
            });
        }

        if input.contains("gyp ERR!") || input.contains("node-gyp") {
            errors.push(ErrorInfo {
                message: "Native module compilation failed".to_string(),
                location: None,
                language: "javascript".to_string(),
            });
        }

        errors
    }

    fn parse_network_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        if input.contains("EADDRINUSE") {
            let port = self.extract_port(input);
            let message = if let Some(p) = port {
                format!("Port {} already in use", p)
            } else {
                "Port already in use".to_string()
            };

            errors.push(ErrorInfo {
                message,
                location: None,
                language: "javascript".to_string(),
            });
        }

        if input.contains("ECONNREFUSED") {
            errors.push(ErrorInfo {
                message: "Connection refused - server not running".to_string(),
                location: None,
                language: "javascript".to_string(),
            });
        }

        if input.contains("ENOTFOUND") {
            errors.push(ErrorInfo {
                message: "Host not found - check URL or network".to_string(),
                location: None,
                language: "javascript".to_string(),
            });
        }

        errors
    }

    fn parse_framework_errors(&self, input: &str) -> Vec<ErrorInfo> {
        let mut errors = vec![];

        // React errors
        if input.contains("React") && (input.contains("Error") || input.contains("Warning")) {
            let location = self.extract_location(input);
            errors.push(ErrorInfo {
                message: "React component error".to_string(),
                location,
                language: "javascript".to_string(),
            });
        }

        // Vue errors
        if input.contains("Vue warn") || input.contains("[Vue") {
            let location = self.extract_location(input);
            errors.push(ErrorInfo {
                message: "Vue.js error or warning".to_string(),
                location,
                language: "javascript".to_string(),
            });
        }

        // Express errors
        if input.contains("Express") && input.contains("Error") {
            errors.push(ErrorInfo {
                message: "Express.js server error".to_string(),
                location: None,
                language: "javascript".to_string(),
            });
        }

        errors
    }

    fn extract_location(&self, input: &str) -> Option<String> {
        // Try to find file:line:column patterns in stack traces
        if let Some(caps) = LOCATION_RE.captures(input) {
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

    fn extract_port(&self, input: &str) -> Option<String> {
        let port_re = Regex::new(r"EADDRINUSE.*:(\d+)").unwrap();
        if let Some(caps) = port_re.captures(input) {
            return caps.get(1).map(|m| m.as_str().to_string());
        }
        None
    }

    pub fn extract_stack_trace(&self, input: &str) -> Vec<String> {
        let mut frames = vec![];

        for line in input.lines() {
            if let Some(caps) = STACK_TRACE_RE.captures(line) {
                if let (Some(file), Some(line_num)) = (caps.get(2), caps.get(3)) {
                    let function = caps.get(1).map(|m| m.as_str()).unwrap_or("anonymous");
                    let col = caps
                        .get(4)
                        .map(|m| format!(":{}", m.as_str()))
                        .unwrap_or_default();

                    frames.push(format!(
                        "{} ({}:{}{})",
                        function,
                        file.as_str(),
                        line_num.as_str(),
                        col
                    ));
                }
            }
        }

        frames
    }

    pub fn suggest_fixes(&self, error_message: &str) -> Vec<String> {
        let mut fixes = vec![];

        if error_message.contains("Module not found") {
            fixes.push("npm install <module_name>".to_string());
            fixes.push("Check import path and spelling".to_string());
            fixes.push("Verify package.json dependencies".to_string());
        }

        if error_message.contains("Port already in use") {
            fixes.push("Kill process using port: lsof -ti:PORT | xargs kill -9".to_string());
            fixes.push("Use a different port: PORT=3001 npm start".to_string());
            fixes.push("Check for running dev servers".to_string());
        }

        if error_message.contains("Permission denied") {
            fixes.push("Run with sudo (not recommended for npm)".to_string());
            fixes.push("Fix npm permissions: npm config set prefix ~/.npm-global".to_string());
            fixes.push("Use a Node version manager (nvm, n)".to_string());
        }

        if error_message.contains("Undefined variable") {
            fixes.push("Check variable spelling and scope".to_string());
            fixes.push("Import required modules".to_string());
            fixes.push("Declare variable before use".to_string());
        }

        if error_message.contains("Cannot read property") {
            fixes.push("Add null/undefined check: if (obj && obj.prop)".to_string());
            fixes.push("Use optional chaining: obj?.prop".to_string());
            fixes.push("Initialize object before use".to_string());
        }

        if error_message.contains("dependency resolution conflict") {
            fixes.push("npm install --legacy-peer-deps".to_string());
            fixes.push("npm install --force (use with caution)".to_string());
            fixes.push("Update conflicting dependencies".to_string());
            fixes.push("Use npm ls to inspect dependency tree".to_string());
        }

        fixes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_not_found() {
        let parser = JsParser;
        let input = r#"
Error: Cannot find module 'express'
Require stack:
- /app/server.js
    at Function.Module._resolveFilename (internal/modules/cjs/loader.js:889:15)
        "#;

        let errors = parser.parse(input);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Module not found"));
    }

    #[test]
    fn test_port_conflict() {
        let parser = JsParser;
        let input = r#"
Error: listen EADDRINUSE: address already in use :::3000
    at Server.setupListenHandle [as _listen2] (net.js:1318:16)
        "#;

        let errors = parser.parse(input);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Port"));
        assert!(errors[0].message.contains("3000"));
    }

    #[test]
    fn test_stack_trace_extraction() {
        let parser = JsParser;
        let input = r#"
TypeError: Cannot read property 'name' of undefined
    at getUserName (/app/utils.js:15:23)
    at main (/app/index.js:8:12)
    at Object.<anonymous> (/app/index.js:20:1)
        "#;

        let frames = parser.extract_stack_trace(input);
        assert_eq!(frames.len(), 3);
        assert!(frames[0].contains("getUserName"));
        assert!(frames[0].contains("/app/utils.js:15:23"));
    }

    #[test]
    fn test_fix_suggestions() {
        let parser = JsParser;
        let fixes = parser.suggest_fixes("Module not found: express");
        assert!(fixes.iter().any(|f| f.contains("npm install")));
    }
}
