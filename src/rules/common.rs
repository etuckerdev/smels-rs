use crate::rules::{Cause, Rule};
use crate::ErrorInfo;

pub struct CommonRule;

impl Rule for CommonRule {
    fn apply(&self, errors: &[ErrorInfo]) -> Vec<Cause> {
        let mut causes = vec![];
        for error in errors {
            if error.message.contains("Panic detected")
                || error.message.contains("panic")
                || error.message.contains("unwrap")
            {
                causes.push(Cause {
                    description: "Unchecked None value or panic condition".to_string(),
                    confidence: 0.9,
                    fixes: vec![
                        "Use expect() with context".to_string(),
                        "Handle None case".to_string(),
                    ],
                });
            } else if error.message.contains("Module not found") {
                causes.push(Cause {
                    description: "Missing dependency".to_string(),
                    confidence: 0.8,
                    fixes: vec![
                        "Run npm install".to_string(),
                        "Check import path".to_string(),
                    ],
                });
            } else if error.message.contains("Port already in use") {
                causes.push(Cause {
                    description: "Port conflict - another process is using the port".to_string(),
                    confidence: 0.95,
                    fixes: vec![
                        "Change to a different port".to_string(),
                        "Stop the conflicting process".to_string(),
                    ],
                });
            }
        }
        causes
    }
}
