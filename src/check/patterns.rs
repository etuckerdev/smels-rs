use crate::check::{CheckError, CheckIssue, Severity};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

pub struct CheckPattern {
    name: String,
    regex: Regex,
    message: String,
}

impl CheckPattern {
    pub fn new(name: &str, pattern: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            regex: Regex::new(pattern).unwrap(),
            message: message.to_string(),
        }
    }

    pub fn check(&self, content: &str, file: &Path) -> Result<Option<CheckIssue>, CheckError> {
        if let Some(mat) = self.regex.find(content) {
            let line_num = content[..mat.start()].lines().count();

            Ok(Some(CheckIssue {
                file: file.to_path_buf(),
                line: Some(line_num),
                pattern: self.name.clone(),
                message: self.message.clone(),
                severity: Severity::Warning, // Could be configurable
                language: String::new(),     // Will be set by caller
            }))
        } else {
            Ok(None)
        }
    }
}

pub struct PatternRegistry {
    patterns: HashMap<&'static str, Vec<CheckPattern>>,
}

impl PatternRegistry {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(
            "rust",
            vec![
                CheckPattern::new(
                    "unwrap_chain",
                    r"\.unwrap\(\)",
                    "Potential panic: .unwrap() call",
                ),
                CheckPattern::new(
                    "expect_call",
                    r"\.expect\(",
                    "Potential panic: .expect() call",
                ),
                CheckPattern::new("panic_macro", r"panic!\(", "Explicit panic found"),
                CheckPattern::new("todo_macro", r"todo!\(", "Unfinished code: todo!() macro"),
                CheckPattern::new(
                    "unimplemented",
                    r"unimplemented!\(",
                    "Unimplemented code found",
                ),
                CheckPattern::new("unsafe_block", r"unsafe\s*\{", "Unsafe block usage"),
            ],
        );

        // JavaScript/TypeScript patterns
        patterns.insert(
            "javascript",
            vec![
                CheckPattern::new(
                    "console_log",
                    r"console\.log\(",
                    "Debug code: console.log found",
                ),
                CheckPattern::new("eval_usage", r"\beval\(", "Security risk: eval() usage"),
                CheckPattern::new(
                    "var_declaration",
                    r"\bvar\s+",
                    "Use let/const instead of var",
                ),
                CheckPattern::new("loose_equality", r"==", "Use strict equality (===) instead"),
                CheckPattern::new(
                    "throw_string",
                    r#"throw\s+["']"#,
                    "Throw Error objects, not strings",
                ),
            ],
        );

        // Python patterns
        patterns.insert(
            "python",
            vec![
                CheckPattern::new(
                    "bare_except",
                    r"except:",
                    "Bare except clause - specify exception type",
                ),
                CheckPattern::new(
                    "print_debug",
                    r"\bprint\(",
                    "Debug code: print() statement found",
                ),
                CheckPattern::new(
                    "import_star",
                    r"from\s+\w+\s+import\s+\*",
                    "Avoid wildcard imports",
                ),
                CheckPattern::new(
                    "lambda_assignment",
                    r"\w+\s*=\s*lambda",
                    "Use def instead of lambda assignment",
                ),
                CheckPattern::new(
                    "mutable_default",
                    r"def\s+\w+.*=\s*\[\]",
                    "Mutable default argument",
                ),
            ],
        );

        // Java patterns
        patterns.insert(
            "java",
            vec![
                CheckPattern::new(
                    "system_out",
                    r"System\.out\.print",
                    "Debug code: System.out found",
                ),
                CheckPattern::new(
                    "string_equality",
                    r#"==\s*"|\s*"\s*=="#,
                    "Use .equals() for string comparison",
                ),
                CheckPattern::new(
                    "raw_thread",
                    r"new\s+Thread\(",
                    "Consider ExecutorService instead of raw Thread",
                ),
                CheckPattern::new(
                    "finalize_method",
                    r"protected\s+void\s+finalize",
                    "Avoid finalize() method",
                ),
            ],
        );

        // Go patterns
        patterns.insert(
            "go",
            vec![
                CheckPattern::new("panic_call", r"\bpanic\(", "Explicit panic found"),
                CheckPattern::new("fmt_print", r"fmt\.Print", "Debug code: fmt.Print found"),
                CheckPattern::new(
                    "error_ignore",
                    r"_\s*=\s*\w+\(.*\)\s*$",
                    "Error potentially ignored",
                ),
                CheckPattern::new(
                    "empty_catch",
                    r"if\s+err\s*!=\s*nil\s*\{\s*\}",
                    "Empty error handling",
                ),
            ],
        );

        // C/C++ patterns
        patterns.insert(
            "c",
            vec![
                CheckPattern::new("printf_debug", r"\bprintf\(", "Debug code: printf found"),
                CheckPattern::new(
                    "malloc_without_free",
                    r"\bmalloc\(",
                    "malloc without corresponding free check",
                ),
                CheckPattern::new("gets_usage", r"\bgets\(", "Security risk: gets() is unsafe"),
                CheckPattern::new(
                    "strcpy_unsafe",
                    r"\bstrcpy\(",
                    "Use strncpy or safer alternatives",
                ),
            ],
        );

        patterns.insert(
            "cpp",
            vec![
                CheckPattern::new("cout_debug", r"std::cout", "Debug code: std::cout found"),
                CheckPattern::new(
                    "raw_pointer",
                    r"new\s+\w+",
                    "Consider smart pointers instead of raw new",
                ),
                CheckPattern::new(
                    "delete_mismatch",
                    r"\bdelete\s+",
                    "Ensure delete matches new/new[]",
                ),
                CheckPattern::new(
                    "using_namespace",
                    r"using\s+namespace\s+std",
                    "Avoid 'using namespace std'",
                ),
            ],
        );

        // C# patterns
        patterns.insert(
            "csharp",
            vec![
                CheckPattern::new(
                    "console_write",
                    r"Console\.Write",
                    "Debug code: Console.Write found",
                ),
                CheckPattern::new(
                    "catch_all",
                    r"catch\s*\(\s*Exception",
                    "Avoid catching generic Exception",
                ),
                CheckPattern::new(
                    "string_empty",
                    r#"==\s*""|\s*""\s*=="#,
                    "Use String.IsNullOrEmpty()",
                ),
                CheckPattern::new(
                    "dispose_missing",
                    r"new\s+\w+.*IDisposable",
                    "Consider using 'using' statement",
                ),
            ],
        );

        Self { patterns }
    }
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternRegistry {
    pub fn get_patterns(&self, language: &str) -> Option<&Vec<CheckPattern>> {
        self.patterns.get(language)
    }
}
