use crate::AnalysisResult;
use serde_json;
use std::collections::HashMap;

// Output format types
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Default,
    Compact,
    Verbose,
    Json,
    JsonPretty,
    Markdown,
    Html,
    Terminal, // With colors
}

// Color codes for terminal output
pub struct Colors;

impl Colors {
    pub const RESET: &'static str = "\x1b[0m";
    pub const BOLD: &'static str = "\x1b[1m";
    pub const DIM: &'static str = "\x1b[2m";
    pub const RED: &'static str = "\x1b[31m";
    pub const GREEN: &'static str = "\x1b[32m";
    pub const YELLOW: &'static str = "\x1b[33m";
    pub const BLUE: &'static str = "\x1b[34m";
    pub const MAGENTA: &'static str = "\x1b[35m";
    pub const CYAN: &'static str = "\x1b[36m";
    pub const WHITE: &'static str = "\x1b[37m";

    // Background colors
    pub const BG_RED: &'static str = "\x1b[41m";
    pub const BG_GREEN: &'static str = "\x1b[42m";
    pub const BG_YELLOW: &'static str = "\x1b[43m";
    pub const BG_BLUE: &'static str = "\x1b[44m";
}

// Main formatting function with format selection
pub fn format_result(result: &AnalysisResult, format: OutputFormat) -> String {
    match format {
        OutputFormat::Default => format_default(result),
        OutputFormat::Compact => format_compact(result),
        OutputFormat::Verbose => format_verbose(result),
        OutputFormat::Json => format_json(result),
        OutputFormat::JsonPretty => format_json_pretty(result),
        OutputFormat::Markdown => format_markdown(result),
        OutputFormat::Html => format_html(result),
        OutputFormat::Terminal => format_terminal(result),
    }
}

// Legacy function for backward compatibility
pub fn format_result_json(result: &AnalysisResult) -> String {
    format_json_pretty(result)
}

// Default format (your original implementation)
pub fn format_default(result: &AnalysisResult) -> String {
    let mut output = format!("Summary: {}\n", result.summary);

    if !result.root_causes.is_empty() {
        output.push_str("Root causes:\n");
        for cause in &result.root_causes {
            output.push_str(&format!("- {cause}\n"));
        }
    }

    if !result.fixes.is_empty() {
        output.push_str("Fixes:\n");
        for fix in &result.fixes {
            output.push_str(&format!("- {fix}\n"));
        }
    }

    if !result.related.is_empty() {
        output.push_str("Related:\n");
        for rel in &result.related {
            output.push_str(&format!("- {rel}\n"));
        }
    }

    output
}

// Compact format - single line summary with key info
pub fn format_compact(result: &AnalysisResult) -> String {
    let cause_count = result.root_causes.len();
    let fix_count = result.fixes.len();

    format!(
        "{} ({} causes, {} fixes)",
        result.summary, cause_count, fix_count
    )
}

// Verbose format with numbered items and additional details
pub fn format_verbose(result: &AnalysisResult) -> String {
    let mut output = String::new();

    // Header
    output.push_str("=== SMELS Error Analysis ===\n\n");
    output.push_str(&format!("Summary: {}\n\n", result.summary));

    // Root causes with numbering
    if !result.root_causes.is_empty() {
        output.push_str("Root Causes:\n");
        for (i, cause) in result.root_causes.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, cause));
        }
        output.push('\n');
    }

    // Fixes with numbering and priority indicators
    if !result.fixes.is_empty() {
        output.push_str("Recommended Fixes:\n");
        for (i, fix) in result.fixes.iter().enumerate() {
            let priority = if i == 0 { " [HIGH PRIORITY]" } else { "" };
            output.push_str(&format!("{}. {}{}\n", i + 1, fix, priority));
        }
        output.push('\n');
    }

    // Related links
    if !result.related.is_empty() {
        output.push_str("Related Documentation:\n");
        for (i, rel) in result.related.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, rel));
        }
        output.push('\n');
    }

    output.push_str("=== End Analysis ===\n");
    output
}

// JSON format (compact)
pub fn format_json(result: &AnalysisResult) -> String {
    serde_json::to_string(result)
        .unwrap_or_else(|_| r#"{"error":"Failed to serialize result"}"#.to_string())
}

// Pretty JSON format
pub fn format_json_pretty(result: &AnalysisResult) -> String {
    serde_json::to_string_pretty(result)
        .unwrap_or_else(|_| r#"{"error":"Failed to serialize result"}"#.to_string())
}

// Markdown format for documentation/web display
pub fn format_markdown(result: &AnalysisResult) -> String {
    let mut output = String::new();

    output.push_str("# Error Analysis\n\n");
    output.push_str(&format!("**Summary:** {}\n\n", result.summary));

    if !result.root_causes.is_empty() {
        output.push_str("## Root Causes\n\n");
        for cause in &result.root_causes {
            output.push_str(&format!("- {cause}\n"));
        }
        output.push('\n');
    }

    if !result.fixes.is_empty() {
        output.push_str("## Fixes\n\n");
        for fix in &result.fixes {
            output.push_str(&format!("- {fix}\n"));
        }
        output.push('\n');
    }

    if !result.related.is_empty() {
        output.push_str("## Related Links\n\n");
        for rel in &result.related {
            if rel.starts_with("http") {
                output.push_str(&format!("- [Link]({rel})\n"));
            } else {
                output.push_str(&format!("- {rel}\n"));
            }
        }
    }

    output
}

// HTML format for web display
pub fn format_html(result: &AnalysisResult) -> String {
    let mut output = String::new();

    output.push_str(r#"
<!DOCTYPE html>
<html>
<head>
    <title>SMELS Error Analysis</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 40px; }
        .summary { background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 20px; }
        .section { margin-bottom: 20px; }
        .section h3 { color: #495057; border-bottom: 2px solid #dee2e6; padding-bottom: 8px; }
        ul { padding-left: 20px; }
        li { margin-bottom: 8px; }
        .causes { background: #fff3cd; padding: 15px; border-radius: 6px; }
        .fixes { background: #d1ecf1; padding: 15px; border-radius: 6px; }
        .related { background: #e2e3e5; padding: 15px; border-radius: 6px; }
        a { color: #007bff; text-decoration: none; }
        a:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <h1>SMELS Error Analysis</h1>
    "#);

    output.push_str(&format!(
        r#"<div class="summary"><strong>Summary:</strong> {}</div>"#,
        html_escape(&result.summary)
    ));

    if !result.root_causes.is_empty() {
        output.push_str(r#"<div class="section causes"><h3>Root Causes</h3><ul>"#);
        for cause in &result.root_causes {
            output.push_str(&format!("<li>{}</li>", html_escape(cause)));
        }
        output.push_str("</ul></div>");
    }

    if !result.fixes.is_empty() {
        output.push_str(r#"<div class="section fixes"><h3>Fixes</h3><ul>"#);
        for fix in &result.fixes {
            output.push_str(&format!("<li>{}</li>", html_escape(fix)));
        }
        output.push_str("</ul></div>");
    }

    if !result.related.is_empty() {
        output.push_str(r#"<div class="section related"><h3>Related Links</h3><ul>"#);
        for rel in &result.related {
            if rel.starts_with("http") {
                output.push_str(&format!(
                    r#"<li><a href="{}" target="_blank">{}</a></li>"#,
                    html_escape(rel),
                    html_escape(rel)
                ));
            } else {
                output.push_str(&format!("<li>{}</li>", html_escape(rel)));
            }
        }
        output.push_str("</ul></div>");
    }

    output.push_str("</body></html>");
    output
}

// Terminal format with colors and icons
pub fn format_terminal(result: &AnalysisResult) -> String {
    let mut output = String::new();

    // Check if colors are supported
    let use_colors = supports_color();

    // Summary with icon
    if use_colors {
        output.push_str(&format!(
            "{}📋 Summary:{} {}\n",
            Colors::BOLD,
            Colors::RESET,
            result.summary
        ));
    } else {
        output.push_str(&format!("Summary: {}\n", result.summary));
    }

    // Root causes
    if !result.root_causes.is_empty() {
        if use_colors {
            output.push_str(&format!(
                "{}⚠️  Root causes:{}\n",
                Colors::YELLOW,
                Colors::RESET
            ));
        } else {
            output.push_str("Root causes:\n");
        }

        for cause in &result.root_causes {
            if use_colors {
                output.push_str(&format!("  {}•{} {}\n", Colors::RED, Colors::RESET, cause));
            } else {
                output.push_str(&format!("- {cause}\n"));
            }
        }
    }

    // Fixes
    if !result.fixes.is_empty() {
        if use_colors {
            output.push_str(&format!("{}🔧 Fixes:{}\n", Colors::GREEN, Colors::RESET));
        } else {
            output.push_str("Fixes:\n");
        }

        for fix in &result.fixes {
            if use_colors {
                output.push_str(&format!("  {}•{} {}\n", Colors::GREEN, Colors::RESET, fix));
            } else {
                output.push_str(&format!("- {fix}\n"));
            }
        }
    }

    // Related links
    if !result.related.is_empty() {
        if use_colors {
            output.push_str(&format!("{}🔗 Related:{}\n", Colors::BLUE, Colors::RESET));
        } else {
            output.push_str("Related:\n");
        }

        for rel in &result.related {
            if use_colors {
                output.push_str(&format!(
                    "  {}•{} {}{}{}\n",
                    Colors::BLUE,
                    Colors::RESET,
                    Colors::CYAN,
                    rel,
                    Colors::RESET
                ));
            } else {
                output.push_str(&format!("- {rel}\n"));
            }
        }
    }

    output
}

// Template system for custom formats
pub struct Template {
    template: String,
    variables: HashMap<String, String>,
}

impl Template {
    pub fn new(template: &str) -> Self {
        Self {
            template: template.to_string(),
            variables: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    pub fn render(&self, result: &AnalysisResult) -> String {
        let mut output = self.template.clone();

        // Replace built-in variables
        output = output.replace("{{summary}}", &result.summary);
        output = output.replace("{{cause_count}}", &result.root_causes.len().to_string());
        output = output.replace("{{fix_count}}", &result.fixes.len().to_string());

        // Replace causes list
        if result.root_causes.is_empty() {
            output = output.replace("{{causes}}", "None identified");
        } else {
            let causes = result.root_causes.join(", ");
            output = output.replace("{{causes}}", &causes);
        }

        // Replace fixes list
        if result.fixes.is_empty() {
            output = output.replace("{{fixes}}", "No fixes available");
        } else {
            let fixes = result.fixes.join(" | ");
            output = output.replace("{{fixes}}", &fixes);
        }

        // Replace custom variables
        for (key, value) in &self.variables {
            output = output.replace(&format!("{{{{{key}}}}}"), value);
        }

        output
    }
}

// Utility functions
fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn supports_color() -> bool {
    // Simple check for color support
    std::env::var("NO_COLOR").is_err()
        && (std::env::var("TERM")
            .map(|term| term != "dumb")
            .unwrap_or(false)
            || std::env::var("COLORTERM").is_ok())
}

// Formatting shortcuts
pub fn format_for_web(result: &AnalysisResult) -> String {
    format_result(result, OutputFormat::Html)
}

pub fn format_for_cli(result: &AnalysisResult, use_colors: bool) -> String {
    if use_colors {
        format_result(result, OutputFormat::Terminal)
    } else {
        format_result(result, OutputFormat::Default)
    }
}

pub fn format_for_api(result: &AnalysisResult) -> String {
    format_result(result, OutputFormat::Json)
}

// Builder pattern for complex formatting
pub struct ResponseBuilder {
    format: OutputFormat,
    template: Option<Template>,
    include_metadata: bool,
    max_items: Option<usize>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            format: OutputFormat::Default,
            template: None,
            include_metadata: false,
            max_items: None,
        }
    }

    pub fn format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    pub fn template(mut self, template: Template) -> Self {
        self.template = Some(template);
        self
    }

    pub fn include_metadata(mut self) -> Self {
        self.include_metadata = true;
        self
    }

    pub fn limit_items(mut self, max: usize) -> Self {
        self.max_items = Some(max);
        self
    }

    pub fn build(self, result: &AnalysisResult) -> String {
        let mut limited_result = result.clone();

        // Apply limits if specified
        if let Some(max) = self.max_items {
            limited_result.root_causes.truncate(max);
            limited_result.fixes.truncate(max);
            limited_result.related.truncate(max);
        }

        // Use template if provided
        if let Some(template) = &self.template {
            return template.render(&limited_result);
        }

        // Otherwise use format
        let mut output = format_result(&limited_result, self.format);

        // Add metadata if requested
        if self.include_metadata {
            output.push_str("\n--- Metadata ---\n");
            output.push_str(&format!(
                "Generated: {}\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            ));
            output.push_str(&format!("Version: SMELS v{}\n", env!("CARGO_PKG_VERSION")));
        }

        output
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_result() -> AnalysisResult {
        AnalysisResult {
            summary: "Test error detected".to_string(),
            root_causes: vec!["Cause 1".to_string(), "Cause 2".to_string()],
            fixes: vec!["Fix 1".to_string(), "Fix 2".to_string()],
            related: vec!["https://example.com".to_string()],
        }
    }

    #[test]
    fn test_default_format() {
        let result = sample_result();
        let output = format_default(&result);
        assert!(output.contains("Summary: Test error detected"));
        assert!(output.contains("- Cause 1"));
        assert!(output.contains("- Fix 1"));
    }

    #[test]
    fn test_compact_format() {
        let result = sample_result();
        let output = format_compact(&result);
        assert!(output.contains("Test error detected (2 causes, 2 fixes)"));
    }

    #[test]
    fn test_json_format() {
        let result = sample_result();
        let output = format_json(&result);
        assert!(output.contains("\"summary\":\"Test error detected\""));
    }

    #[test]
    fn test_template_system() {
        let result = sample_result();
        let mut template = Template::new("Error: {{summary}} with {{cause_count}} causes");
        template.set_variable("custom", "test");

        let output = template.render(&result);
        assert_eq!(output, "Error: Test error detected with 2 causes");
    }

    #[test]
    fn test_response_builder() {
        let result = sample_result();
        let output = ResponseBuilder::new()
            .format(OutputFormat::Compact)
            .limit_items(1)
            .build(&result);

        assert!(output.contains("Test error detected (1 causes, 1 fixes)"));
    }
}
