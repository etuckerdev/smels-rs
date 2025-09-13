pub mod languages;
pub mod output;
pub mod patterns;

use crate::check::languages::LanguageDetector;
use crate::check::patterns::PatternRegistry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct CodeChecker {
    patterns: PatternRegistry,
}

impl CodeChecker {
    pub fn new() -> Self {
        Self {
            patterns: PatternRegistry::new(),
        }
    }

    pub async fn check_path(
        &self,
        path: &Path,
        filter: Option<&str>,
    ) -> Result<CheckResult, CheckError> {
        let source_files = LanguageDetector::find_source_files(path, filter)?;
        let mut issues = Vec::new();
        let mut files_by_language = HashMap::new();

        // Group files by language for statistics
        for (_file_path, language) in &source_files {
            *files_by_language.entry(*language).or_insert(0) += 1;
        }

        // Check each file
        for (file_path, language) in source_files.iter() {
            let content = std::fs::read_to_string(file_path)?;
            let file_issues = self.check_content(&content, file_path, language).await?;
            issues.extend(file_issues);
        }

        let passed = issues.is_empty();
        Ok(CheckResult {
            total_files: source_files.len(),
            files_by_language,
            issues,
            passed,
        })
    }

    async fn check_content(
        &self,
        content: &str,
        file: &Path,
        language: &str,
    ) -> Result<Vec<CheckIssue>, CheckError> {
        let mut issues = Vec::new();

        if let Some(patterns) = self.patterns.get_patterns(language) {
            for pattern in patterns {
                if let Some(mut issue) = pattern.check(content, file)? {
                    issue.language = language.to_string();
                    issues.push(issue);
                }
            }
        }

        Ok(issues)
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
    pub files_by_language: std::collections::HashMap<&'static str, usize>,
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
    pub language: String,
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
