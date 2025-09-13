use crate::check::{CheckError, CheckIssue, Severity};
use regex::Regex;
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
            }))
        } else {
            Ok(None)
        }
    }
}
