use crate::check::CheckResult;

pub fn print_check_result(result: &CheckResult, strict: bool) {
    // Show language breakdown
    println!("📊 Analyzed {} files:", result.total_files);
    for (lang, count) in &result.files_by_language {
        let icon = match *lang {
            "rust" => "🦀",
            "javascript" => "📜",
            "python" => "🐍",
            "java" => "☕",
            "go" => "🐹",
            "c" => "⚙️",
            "cpp" => "⚡",
            "csharp" => "💎",
            _ => "📄",
        };
        println!("  {icon} {lang} files: {count}");
    }

    if result.passed {
        println!("✅ All checks passed!");
        return;
    }

    // Group issues by language
    let mut issues_by_lang = std::collections::HashMap::new();
    for issue in &result.issues {
        issues_by_lang
            .entry(&issue.language)
            .or_insert(Vec::new())
            .push(issue);
    }

    println!("\n❌ Found {} issues:", result.issues.len());
    for (lang, issues) in issues_by_lang {
        let icon = match lang.as_str() {
            "rust" => "🦀",
            "javascript" => "📜",
            "python" => "🐍",
            "java" => "☕",
            "go" => "🐹",
            "c" => "⚙️",
            "cpp" => "⚡",
            "csharp" => "💎",
            _ => "📄",
        };

        println!("\n{} {} ({} issues):", icon, lang, issues.len());
        for issue in issues {
            println!(
                "  🟠 {}:{} {}",
                issue.file.display(),
                issue.line.unwrap_or(0),
                issue.message
            );
        }
    }

    if strict {
        println!("\n⚠️  Strict mode: build not recommended");
    } else {
        println!("\n💡 Run with --strict to fail CI on issues");
    }
}
