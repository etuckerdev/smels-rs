use smels::{Analyzer, parsers::{GenericParser, rust::RustParser, js::JsParser}, rules::common::CommonRule};

#[tokio::test]
async fn test_rust_panic_analysis() {
    let mut analyzer = Analyzer::new();
    analyzer.add_parser(Box::new(GenericParser));
    analyzer.add_parser(Box::new(RustParser));
    analyzer.add_rule(Box::new(CommonRule));

    let input = format!("thread 'main' panicked at 'called Option::unwrap() on a None value', src/main.rs:10:5 - {}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let result = analyzer.analyze(&input).await;

    assert!(result.summary.contains("panic") || result.summary.contains("error") || result.summary.contains("Detected"));
    assert!(!result.root_causes.is_empty());
    assert!(!result.fixes.is_empty());
}

#[tokio::test]
async fn test_javascript_module_error() {
    let mut analyzer = Analyzer::new();
    analyzer.add_parser(Box::new(GenericParser));
    analyzer.add_parser(Box::new(JsParser));
    analyzer.add_rule(Box::new(CommonRule));

    let input = format!("Error: Cannot find module 'express' at Function.Module._resolveFilename - {}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let result = analyzer.analyze(&input).await;

    assert!(result.summary.contains("error") || result.summary.contains("module"));
    assert!(!result.root_causes.is_empty());
}

#[tokio::test]
async fn test_port_conflict_error() {
    let mut analyzer = Analyzer::new();
    analyzer.add_parser(Box::new(GenericParser));
    analyzer.add_rule(Box::new(CommonRule));

    let input = format!("Error: listen EADDRINUSE: address already in use :::3000 - {}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let result = analyzer.analyze(&input).await;

    assert!(result.summary.contains("error") || result.summary.contains("port"));
    assert!(result.fixes.iter().any(|f| f.contains("port") || f.contains("kill")));
}

#[tokio::test]
async fn test_empty_input() {
    let mut analyzer = Analyzer::new();
    analyzer.add_parser(Box::new(GenericParser));

    let result = analyzer.analyze("").await;
    assert!(result.summary.contains("No errors") || result.summary.contains("empty"));
}

#[tokio::test]
async fn test_cache_functionality() {
    let mut analyzer = Analyzer::new();
    analyzer.add_parser(Box::new(GenericParser));

    let input = "Test error message";
    let result1 = analyzer.analyze(input).await;
    let result2 = analyzer.analyze(input).await;

    // Second result should be cached
    assert_eq!(result2.summary, "Previously resolved issue (cached)");
    assert_eq!(result2.root_causes, vec!["Cached resolution".to_string()]);
    assert_eq!(result1.fixes, result2.fixes); // Fixes should be the same
}
