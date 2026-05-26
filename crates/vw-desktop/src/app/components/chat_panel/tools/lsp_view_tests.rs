use super::lsp_view::{LspToolData, action_title, meta_text, summary_from_data};

#[test]
fn lsp_summary_and_meta_use_structured_counts() {
    let data: LspToolData = serde_json::from_value(serde_json::json!({
        "success": true,
        "operation": "definition",
        "file_path": "src/main.rs",
        "result_count": 2,
        "file_count": 1,
        "payload": {"kind": "message", "message": "ok"}
    }))
    .expect("valid lsp payload");

    assert!(summary_from_data(&data).contains("2"));
    assert!(meta_text(&data).contains("2"));
    assert_eq!(action_title("hover"), "悬停信息");
}
