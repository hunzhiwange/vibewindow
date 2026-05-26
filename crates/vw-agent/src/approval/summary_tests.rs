use super::summary::summarize_args;

#[test]
fn summarize_args_formats_objects_and_truncates_long_strings() {
    let summary = summarize_args(&serde_json::json!({
        "path": "src/main.rs",
        "content": "x".repeat(100)
    }));

    assert!(summary.contains("path: src/main.rs"));
    assert!(summary.contains("content: "));
    assert!(summary.contains('…'));
}
