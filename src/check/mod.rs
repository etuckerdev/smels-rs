pub mod output;
pub mod patterns;

use patterns::CheckPattern;
use std::path::{Path, PathBuf};

pub struct CodeChecker {
    patterns: Vec<CheckPattern>,
}

impl CodeChecker {
    pub fn new() -> Self {
        Self {
            patterns: Self::load_check_patterns(),
        }
    }

    pub async fn check_path(&self, path: &Path) -> Result<CheckResult, CheckError> {
        let rust_files = self.find_rust_files(path)?;
        let mut issues = Vec::new();

        for file in &rust_files {
            let content = std::fs::read_to_string(file)?;
            let file_issues = self.check_content(&content, file).await?;
            issues.extend(file_issues);
        }

        let passed = issues.is_empty();
        Ok(CheckResult {
            total_files: rust_files.len(),
            issues,
            passed,
        })
    }

    async fn check_content(
        &self,
        content: &str,
        file: &Path,
    ) -> Result<Vec<CheckIssue>, CheckError> {
        let mut issues = Vec::new();

        // Proactive pattern checks
        for pattern in &self.patterns {
            if let Some(issue) = pattern.check(content, file)? {
                issues.push(issue);
            }
        }

        Ok(issues)
    }

    fn find_rust_files(&self, path: &Path) -> Result<Vec<PathBuf>, CheckError> {
        let mut files = Vec::new();
        Self::find_rust_files_recursive(path, &mut files)?;
        Ok(files)
    }

    fn find_rust_files_recursive(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), CheckError> {
        if path.is_file() {
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path.to_path_buf());
            }
            return Ok(());
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // Skip common directories that shouldn't be checked
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') || name == "target" || name == "node_modules" {
                        continue;
                    }
                }
                Self::find_rust_files_recursive(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }

        Ok(())
    }

    fn load_check_patterns() -> Vec<CheckPattern> {
        vec![
            CheckPattern::new(
                "unwrap_chain",
                r"\.unwrap\(\)",
                "Potential panic: .unwrap() call without error handling",
            ),
            CheckPattern::new(
                "todo_macro",
                r"todo!\(",
                "Unfinished code: todo!() macro found",
            ),
            CheckPattern::new(
                "panic_macro",
                r"panic!\(",
                "Explicit panic: panic!() macro found",
            ),
            CheckPattern::new(
                "unreachable_code",
                r"unreachable!\(",
                "Unreachable code marker found",
            ),
            CheckPattern::new("unsafe_block", r"unsafe\s*\{", "Unsafe block usage"),
            CheckPattern::new(
                "expect_none",
                r"\.expect\([^)]*\)",
                "Potential panic: .expect() call that could fail",
            ),
        ]
    }
}

impl Default for CodeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, serde::Serialize)]
pub struct CheckResult {
    pub total_files: usize,
    pub issues: Vec<CheckIssue>,
    pub passed: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct CheckIssue {
    pub file: PathBuf,
    pub line: Option<usize>,
    pub pattern: String,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum Severity {
    Warning,
    Error,
    Critical,
}

#[derive(Debug)]
pub enum CheckError {
    Io(std::io::Error),
    Regex(regex::Error),
}

impl From<std::io::Error> for CheckError {
    fn from(err: std::io::Error) -> Self {
        CheckError::Io(err)
    }
}

impl From<regex::Error> for CheckError {
    fn from(err: regex::Error) -> Self {
        CheckError::Regex(err)
    }
}
