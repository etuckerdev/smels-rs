use crate::check::{CheckResult, Severity};

pub fn print_check_result(result: &CheckResult, strict: bool) {
    if result.passed {
        println!("✅ All checks passed ({} files)", result.total_files);
        return;
    }

    println!(
        "❌ Found {} issues in {} files:",
        result.issues.len(),
        result.total_files
    );

    for issue in &result.issues {
        let location = match issue.line {
            Some(line) => format!("{}:{}", issue.file.display(), line),
            None => issue.file.display().to_string(),
        };

        println!(
            "  {} {} {}",
            severity_icon(issue.severity.clone()),
            location,
            issue.message
        );
    }

    if strict {
        println!("\n⚠️  Strict mode: build not recommended");
    } else {
        println!("\n💡 Run with --strict to fail CI on issues");
    }
}

fn severity_icon(severity: Severity) -> &'static str {
    match severity {
        Severity::Critical => "🔴",
        Severity::Error => "🟡",
        Severity::Warning => "🟠",
    }
}
