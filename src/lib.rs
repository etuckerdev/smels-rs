pub mod ai;
pub mod parsers;
pub mod rules;
pub mod templates;

#[cfg(feature = "web")]
pub mod web;

// /// # SMELS - AI-Powered Error Log Analyzer
// ///
// /// SMELS is an intelligent error log analyzer that uses AI to understand and provide
// /// actionable insights for errors across multiple programming languages.
// ///
// /// ## Features
// ///
// /// - **Multi-language Support**: Automatically detects and analyzes errors from Rust, JavaScript, Python, Java, and Go
// /// - **AI-Powered Analysis**: Uses local AI models for intelligent summaries and root cause analysis
// /// - **Actionable Fixes**: Provides specific solutions for common error patterns
// /// - **Curated Documentation**: Includes relevant documentation links
// ///
// /// ## Example
// ///

// /// use smels::{Analyzer, parsers::{GenericParser, rust::RustParser}};
// ///
// /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
// /// let mut analyzer = Analyzer::new();
// /// analyzer.add_parser(Box::new(GenericParser));
// /// analyzer.add_parser(Box::new(RustParser));
// ///
// /// let result = analyzer.analyze("thread 'main' panicked at 'called Option::unwrap() on a None value'").await;
// /// println!("Summary: {}", result.summary);
// /// # Ok(())
// /// # }

// Parser traits
/// Trait for parsing error messages from different sources
pub trait Parser: Send + Sync {
    /// Parse input text and return a list of detected errors
    fn parse(&self, input: &str) -> Vec<ErrorInfo>;
}

#[derive(Debug)]
/// Represents a detected programming language and error context
pub struct Signal {
    /// The detected programming language (e.g., "rust", "javascript", "python")
    pub language: String,
    /// The top stack frame or file location where the error occurred
    pub top_frame: Option<String>,
    /// The error code (e.g., "EADDRINUSE", "ENOENT")
    pub errno: Option<String>,
    /// The primary error message
    pub message: String,
    /// The last command that was executed (if available)
    pub last_command: Option<String>,
    /// List of top stack frames for context
    pub top_frames: Vec<String>,
}

impl Default for Signal {
    fn default() -> Self {
        Signal {
            language: "unknown".to_string(),
            top_frame: None,
            errno: None,
            message: "Unknown error".to_string(),
            last_command: None,
            top_frames: vec![],
        }
    }
}

#[derive(Debug)]
/// Information about a detected error
pub struct ErrorInfo {
    /// The error message text
    pub message: String,
    /// File location where the error occurred (if available)
    pub location: Option<String>,
    /// The programming language this error is from
    pub language: String,
}

#[derive(Debug, serde::Serialize, Clone)]
/// The result of an error analysis
pub struct AnalysisResult {
    /// Summary of the error analysis
    pub summary: String,
    /// List of identified root causes
    pub root_causes: Vec<String>,
    /// List of suggested fixes
    pub fixes: Vec<String>,
    /// List of relevant documentation links
    pub related: Vec<String>,
}

// Heuristics and summary engine
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
struct CacheEntry {
    input_hash: String,
    fixes: Vec<String>,
    timestamp: u64,
}

/// Main analyzer for processing error logs
///
/// The Analyzer coordinates parsing, rule application, and AI analysis
/// to provide comprehensive error diagnostics.
pub struct Analyzer {
    parsers: Vec<Box<dyn Parser>>,
    rules: Vec<Box<dyn rules::Rule>>,
    use_ai: bool,
    cache: HashMap<String, CacheEntry>,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    /// Create a new analyzer instance
    pub fn new() -> Self {
        let cache = Self::load_cache();
        Analyzer {
            parsers: vec![],
            rules: vec![],
            use_ai: false,
            cache,
        }
    }

    fn load_cache() -> HashMap<String, CacheEntry> {
        if let Ok(content) = fs::read_to_string("smels_cache.json") {
            if let Ok(cache) = serde_json::from_str(&content) {
                return cache;
            }
        }
        HashMap::new()
    }
    fn save_cache(&self) {
        if let Ok(json) = serde_json::to_string(&self.cache) {
            let _ = fs::write("smels_cache.json", json);
        }
    }

    /// Enable or disable AI-powered analysis
    ///
    /// # Arguments
    /// * `use_ai` - Whether to use AI analysis (requires Ollama to be running)
    pub fn with_ai(mut self, use_ai: bool) -> Self {
        self.use_ai = use_ai;
        self
    }

    /// Add a parser to the analyzer
    ///
    /// # Arguments
    /// * `parser` - The parser to add
    pub fn add_parser(&mut self, parser: Box<dyn Parser>) {
        self.parsers.push(parser);
    }

    /// Add a rule to the analyzer
    ///
    /// # Arguments
    /// * `rule` - The rule to add for pattern matching
    pub fn add_rule(&mut self, rule: Box<dyn rules::Rule>) {
        self.rules.push(rule);
    }

    /// Analyze an error message and return comprehensive results
    ///
    /// # Arguments
    /// * `input` - The error message or log to analyze
    ///
    /// # Returns
    /// An AnalysisResult containing summary, causes, fixes, and related links
    pub async fn analyze(&mut self, input: &str) -> AnalysisResult {
        // Check cache first
        let input_hash = format!("{:x}", Md5::digest(input));
        if let Some(cached) = self.cache.get(&input_hash) {
            let mut result = AnalysisResult {
                summary: "Previously resolved issue (cached)".to_string(),
                root_causes: vec!["Cached resolution".to_string()],
                fixes: cached.fixes.clone(),
                related: vec![],
            };
            result.related = self.get_curated_links(&Signal::default(), &result.root_causes);
            return result;
        }
        let mut all_errors = vec![];
        for parser in &self.parsers {
            all_errors.extend(parser.parse(input));
        }
        let mut all_causes = vec![];
        for rule in &self.rules {
            all_causes.extend(rule.apply(&all_errors));
        }
        // Sort by confidence
        all_causes.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Extract signal for AI and curated links
        let signal = self.extract_signal(input, &all_errors);

        let summary = if all_errors.is_empty() {
            "No errors detected".to_string()
        } else {
            format!("Detected {} error(s)", all_errors.len())
        };
        let root_causes: Vec<String> = all_causes.iter().map(|c| c.description.clone()).collect();
        let mut fixes: Vec<String> = all_causes.iter().flat_map(|c| c.fixes.clone()).collect();

        // Augment fixes with OS specific EADDRINUSE instructions if present
        if signal.errno.as_deref() == Some("EADDRINUSE") {
            let port_hint = self
                .extract_port(input)
                .map(|p| p.to_string())
                .unwrap_or_else(|| "<port>".into());
            fixes.push(format!(
                "Identify process using port: lsof -nP -iTCP:{port_hint} -sTCP:LISTEN"
            ));
            fixes.push("Kill process (Linux/macOS): kill -9 <PID> (verify first)".into());
            fixes.push(format!(
                "Windows find process: netstat -ano | findstr :{port_hint}"
            ));
            fixes.push("Windows kill: taskkill /PID <PID> /F".into());
            fixes.push(
                "Consider changing to an unused port or ensuring previous dev server shut down"
                    .into(),
            );
        }
        let mut result = AnalysisResult {
            summary,
            root_causes,
            fixes,
            related: vec![],
        };

        if self.use_ai && std::env::var("SMELS_NO_AI").is_err() {
            match self.try_ai_summarize(input, &all_errors).await {
                Ok(ai_summary) => {
                    // Merge AI results with deterministic fallback (no placeholders allowed)
                    result.summary = if ai_summary.summary.trim().is_empty()
                        || ai_summary.summary.to_lowercase().contains("root cause")
                    {
                        result.summary
                    } else {
                        ai_summary.summary
                    };
                    // Use AI causes if they pass specificity & probability filters
                    let mut ai_causes: Vec<String> = ai_summary
                        .causes
                        .into_iter()
                        .filter(|c| {
                            c.prob >= 0.3
                                && !c.label.to_lowercase().contains("root cause")
                                && c.label.len() > 3
                        })
                        .map(|c| c.label)
                        .collect();
                    ai_causes.truncate(4);

                    // Apply merge policy: deterministic > AI when conflict detected
                    if !ai_causes.is_empty() {
                        // Check for conflicts with deterministic signals
                        let has_port_conflict = signal.errno.as_deref() == Some("EADDRINUSE")
                            || all_causes.iter().any(|c| {
                                c.description.contains("port") || c.description.contains("address")
                            });
                        let has_unwrap_panic = signal.language == "rust"
                            && (signal.message.contains("unwrap")
                                || signal.message.contains("None"));

                        if has_port_conflict {
                            // For port conflicts, only keep AI causes that mention networking/port
                            ai_causes.retain(|c| {
                                c.to_lowercase().contains("port")
                                    || c.to_lowercase().contains("network")
                                    || c.to_lowercase().contains("address")
                            });
                        }

                        if has_unwrap_panic {
                            // For Rust unwrap panics, only keep AI causes that mention Option/unwrap
                            ai_causes.retain(|c| {
                                c.to_lowercase().contains("option")
                                    || c.to_lowercase().contains("unwrap")
                                    || c.to_lowercase().contains("none")
                            });
                        }

                        if !ai_causes.is_empty() {
                            result.root_causes = ai_causes;
                        }
                    }

                    // Merge fixes: prefer AI if non-empty, ensure uniqueness, append OS fixes if lost
                    if !ai_summary.fixes.is_empty() {
                        let mut merged: Vec<String> = ai_summary.fixes;
                        for f in &result.fixes {
                            if !merged.iter().any(|x| x == f) {
                                merged.push(f.clone());
                            }
                        }
                        result.fixes = merged;
                    }

                    // Curated links always based on merged root causes
                    result.related = self.get_curated_links(&signal, &result.root_causes);
                }
                Err(e) => {
                    eprintln!("AI unavailable; using deterministic analysis ({e})");
                    result.related = self.get_curated_links(&signal, &result.root_causes);
                }
            }
        } else {
            // Deterministic path still gets curated links
            result.related = self.get_curated_links(&signal, &result.root_causes);
        }

        // Save to cache for future use
        // Note: This requires mutable access, so you may need to adjust the method signature if you want to cache results.
        // Example (if you make `analyze` take &mut self):
        self.cache.insert(
            input_hash.clone(),
            CacheEntry {
                input_hash,
                fixes: result.fixes.clone(),
                timestamp: chrono::Utc::now().timestamp() as u64,
            },
        );
        self.save_cache();

        result
    }

    async fn try_ai_summarize(
        &self,
        input: &str,
        errors: &[ErrorInfo],
    ) -> Result<ai::AISummary, Box<dyn std::error::Error>> {
        let brief = self.build_prompt(input, errors);
        ai::ai_summarize(brief).await
    }

    fn build_prompt(&self, input: &str, errors: &[ErrorInfo]) -> String {
        let signal = self.extract_signal(input, errors);

        let mut prompt = format!("Language: {}\n", signal.language);
        prompt.push_str(&format!("Primary message: {}\n", signal.message));

        if let Some(frame) = &signal.top_frame {
            prompt.push_str(&format!("Top frame: {frame}\n"));
        }

        if let Some(errno) = &signal.errno {
            prompt.push_str(&format!("Error code: {errno}\n"));
        }

        if !signal.top_frames.is_empty() {
            prompt.push_str("Stack frames:\n");
            for frame in &signal.top_frames {
                prompt.push_str(&format!("- {frame}\n"));
            }
        }

        prompt.push_str(&format!("Full input: {input}\n"));

        // Add known patterns based on error type
        for err in errors {
            if err.message.contains("Panic detected") || err.message.contains("unwrap") {
                prompt.push_str("Known patterns: Option::unwrap() called on None value\n");
            } else if err.message.contains("Module not found") {
                prompt.push_str("Known patterns: Missing dependency or incorrect import path\n");
            } else if err.message.contains("EADDRINUSE") {
                prompt.push_str("Known patterns: Port already in use by another process\n");
            }
        }

        prompt
    }

    pub fn extract_signal(&self, input: &str, errors: &[ErrorInfo]) -> Signal {
        let language = errors
            .first()
            .map(|e| e.language.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Extract top frame from input (simple regex for file:line patterns)
        let top_frame = if let Ok(re) = regex::Regex::new(r"(\w+\.\w+:\d+)") {
            re.find(input).map(|cap| cap.as_str().to_string())
        } else {
            None
        };

        // Extract errno patterns
        let errno = if let Ok(re) = regex::Regex::new(r"(E[A-Z]+)") {
            re.find(input).map(|cap| cap.as_str().to_string())
        } else {
            None
        };

        let message = if let Some(err) = errors.first() {
            err.message.clone()
        } else {
            // Extract a meaningful message from the input when no parsers are available
            if let Some(panic_match) = input.find("panicked at '") {
                let start = panic_match + 13; // Length of "panicked at '"
                if let Some(end) = input[start..].find("'") {
                    input[start..start + end].to_string()
                } else {
                    "Unknown error".to_string()
                }
            } else if let Some(error_match) = input.find("Error:") {
                let start = error_match + 7; // Length of "Error: "
                if start < input.len() {
                    input[start..].trim().to_string()
                } else {
                    "Unknown error".to_string()
                }
            } else {
                "Unknown error".to_string()
            }
        };

        // Extract top frames (first few lines that look like stack frames)
        let top_frames: Vec<String> = input
            .lines()
            .filter(|line| {
                line.contains(".rs:")
                    || line.contains(".js:")
                    || line.contains(".py:")
                    || line.contains(".java:")
            })
            .take(3)
            .map(|s| s.to_string())
            .collect();

        Signal {
            language,
            top_frame,
            errno,
            message,
            last_command: None, // Could be extracted from environment or input
            top_frames,
        }
    }

    fn get_curated_links(&self, signal: &Signal, root_causes: &[String]) -> Vec<String> {
        let mut links = vec![];

        // Language-specific documentation
        match signal.language.as_str() {
            "rust" => {
                if root_causes
                    .iter()
                    .any(|c| c.contains("unwrap") || c.contains("None"))
                {
                    links.push(
                        "https://doc.rust-lang.org/std/option/struct.Option.html".to_string(),
                    );
                    links.push("https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html".to_string());
                }
            }
            "js" => {
                if root_causes
                    .iter()
                    .any(|c| c.contains("module") || c.contains("dependency"))
                {
                    links.push(
                        "https://nodejs.org/api/modules.html#modules_module_resolution".to_string(),
                    );
                    links.push("https://docs.npmjs.com/common-errors".to_string());
                }
                if signal.errno.as_deref() == Some("EADDRINUSE") {
                    links.push("https://nodejs.org/api/net.html#serverlisten".to_string());
                }
            }
            _ => {}
        }

        // OS-specific port conflict solutions
        if signal.errno.as_deref() == Some("EADDRINUSE") {
            #[cfg(target_os = "linux")]
            links.push("https://man7.org/linux/man-pages/man8/lsof.8.html".to_string());

            #[cfg(target_os = "macos")]
            links.push("https://ss64.com/osx/lsof.html".to_string());

            #[cfg(target_os = "windows")]
            links.push("https://docs.microsoft.com/en-us/windows-server/administration/windows-commands/netstat".to_string());
        }

        links
    }

    fn extract_port(&self, input: &str) -> Option<u16> {
        // Look for common patterns: :3000, PORT=3000, address already in use 3000
        if let Ok(re) = regex::Regex::new(r#":(\d{2,5})"#) {
            for cap in re.captures_iter(input) {
                if let Ok(p) = cap[1].parse::<u16>() {
                    return Some(p);
                }
            }
        }
        if let Ok(re) = regex::Regex::new(r#"PORT=?(\d{2,5})"#) {
            for cap in re.captures_iter(input) {
                if let Ok(p) = cap[1].parse::<u16>() {
                    return Some(p);
                }
            }
        }
        None
    }
}

impl Clone for Analyzer {
    fn clone(&self) -> Self {
        // Create a new analyzer with the same configuration but empty parsers/rules
        // This is necessary because trait objects can't be cloned directly
        Analyzer::new().with_ai(self.use_ai)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::GenericParser;
    use crate::rules::common::CommonRule;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = Analyzer::new();
        assert!(!analyzer.use_ai);
        assert!(analyzer.parsers.is_empty());
        assert!(analyzer.rules.is_empty());
    }

    #[test]
    fn test_analyzer_with_ai() {
        let analyzer = Analyzer::new().with_ai(true);
        assert!(analyzer.use_ai);
    }

    #[test]
    fn test_add_parser() {
        let mut analyzer = Analyzer::new();
        analyzer.add_parser(Box::new(GenericParser));
        assert_eq!(analyzer.parsers.len(), 1);
    }

    #[test]
    fn test_add_rule() {
        let mut analyzer = Analyzer::new();
        analyzer.add_rule(Box::new(CommonRule));
        assert_eq!(analyzer.rules.len(), 1);
    }

    #[test]
    fn test_extract_signal_rust_panic() {
        let analyzer = Analyzer::new();
        let input =
            "thread 'main' panicked at 'called Option::unwrap() on a None value', src/main.rs:10:5";
        let signal = analyzer.extract_signal(input, &vec![]);

        assert_eq!(signal.language, "unknown"); // No parsers added
        assert!(signal.top_frame.is_some());
        assert!(signal.message.contains("unwrap"));
    }

    #[test]
    fn test_extract_port() {
        let analyzer = Analyzer::new();

        // Test :port pattern
        let input1 = "Error: listen EADDRINUSE: address already in use :::3000";
        assert_eq!(analyzer.extract_port(input1), Some(3000));

        // Test PORT= pattern
        let input2 = "PORT=8080 npm start";
        assert_eq!(analyzer.extract_port(input2), Some(8080));

        // Test no port found
        let input3 = "Some error without port";
        assert_eq!(analyzer.extract_port(input3), None);
    }

    #[test]
    fn test_get_curated_links() {
        let analyzer = Analyzer::new();
        let signal = Signal {
            language: "rust".to_string(),
            top_frame: None,
            errno: None,
            message: "unwrap on None".to_string(),
            last_command: None,
            top_frames: vec![],
        };

        let links = analyzer.get_curated_links(&signal, &vec!["unwrap".to_string()]);
        assert!(!links.is_empty());
        assert!(links.iter().any(|l| l.contains("doc.rust-lang.org")));
    }

    #[tokio::test]
    async fn test_analyze_without_parsers() {
        let mut analyzer = Analyzer::new();
        // Use a unique input to avoid cache hits
        let unique_input = format!(
            "Unique test input {}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let result = analyzer.analyze(&unique_input).await;

        // When no parsers are added, no errors are detected
        assert!(result.summary.contains("No errors"));
        assert!(result.root_causes.is_empty());
        assert!(result.fixes.is_empty());
    }
}
