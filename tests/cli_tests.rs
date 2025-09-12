use std::process::Command;

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to run CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("SMELS") || stdout.contains("Usage"));
}

#[test]
fn test_cli_analyze_text() {
    let output = Command::new("cargo")
        .args(&["run", "--", "analyze"])
        .output()
        .expect("Failed to run CLI");

    // Should show error since no input provided
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No input provided") || stderr.contains("stdin"));
}
