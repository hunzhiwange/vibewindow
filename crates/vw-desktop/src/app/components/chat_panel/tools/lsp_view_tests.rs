use super::lsp_view::{LspToolData, action_title, meta_text, summary_from_data, tool_lsp_view};
use crate::app::{App, Message};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

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

#[test]
fn lsp_meta_text_covers_operation_specific_labels() {
    for (operation, expected) in [
        ("hover", "无悬停信息"),
        ("documentSymbol", "0 个文档符号"),
        ("workspaceSymbol", "0 个工作区符号 / 2 个文件"),
        ("prepareCallHierarchy", "0 个调用层级项"),
        ("incomingCalls", "0 个入向调用 / 2 个文件"),
        ("outgoingCalls", "0 个出向调用 / 2 个文件"),
        ("unknown", "0 个结果 / 2 个文件"),
    ] {
        let data: LspToolData = serde_json::from_value(serde_json::json!({
            "success": true,
            "operation": operation,
            "result_count": 0,
            "file_count": 2,
            "payload": {"kind": "message", "message": "ok"}
        }))
        .expect("valid lsp payload");

        assert_eq!(meta_text(&data), expected);
    }
}

#[test]
fn lsp_summary_falls_back_to_query_then_file_path() {
    let with_query: LspToolData = serde_json::from_value(serde_json::json!({
        "success": true,
        "operation": "workspaceSymbol",
        "query": "Widget",
        "payload": {"kind": "message", "message": "ok"}
    }))
    .expect("valid lsp payload");
    assert_eq!(summary_from_data(&with_query), "Widget");

    let with_file: LspToolData = serde_json::from_value(serde_json::json!({
        "success": true,
        "operation": "hover",
        "file_path": "src/main.rs",
        "payload": {"kind": "message", "message": "ok"}
    }))
    .expect("valid lsp payload");
    assert_eq!(summary_from_data(&with_file), "src/main.rs");
}

#[test]
fn lsp_action_title_covers_known_operations_and_default() {
    assert_eq!(action_title("goToDefinition"), "跳转定义");
    assert_eq!(action_title("findReferences"), "查找引用");
    assert_eq!(action_title("documentSymbol"), "文档符号");
    assert_eq!(action_title("workspaceSymbol"), "工作区符号");
    assert_eq!(action_title("goToImplementation"), "跳转实现");
    assert_eq!(action_title("prepareCallHierarchy"), "调用层级");
    assert_eq!(action_title("incomingCalls"), "入向调用");
    assert_eq!(action_title("outgoingCalls"), "出向调用");
    assert_eq!(action_title("other"), "LSP");
}

#[test]
fn tool_lsp_view_rejects_non_lsp_or_invalid_json() {
    let app = test_app();

    assert!(tool_lsp_view(&app, 0, 0, "tool read\n{}").is_none());
    assert!(tool_lsp_view(&app, 0, 0, "tool lsp\nnot json").is_none());
}

#[test]
fn tool_lsp_view_builds_running_error_text_and_structured_bodies() {
    let app = test_app();

    let running = tool_lsp_view(
        &app,
        1,
        1,
        r#"tool lsp
{"status":"running","input":"{}"}"#,
    )
    .expect("running lsp view");
    keep_element(running);

    let error = tool_lsp_view(
        &app,
        1,
        2,
        r#"tool lsp
{"status":"error","error":"server failed"}"#,
    )
    .expect("error lsp view");
    keep_element(error);

    let text = tool_lsp_view(
        &app,
        1,
        3,
        r#"tool lsp
{"status":"completed","output":"plain lsp output"}"#,
    )
    .expect("text lsp view");
    keep_element(text);

    let structured = tool_lsp_view(&app, 1, 4, r#"tool lsp
{"status":"completed","result":{"data":{"success":true,"operation":"goToDefinition","result_count":1,"file_count":1,"payload":{"kind":"locations","items":[{"path":"src/main.rs","absolute_path":"/tmp/src/main.rs","line":3,"character":4,"name":"main","kind":"fn","detail":"detail","container_name":"crate"}]}}}}"#)
    .expect("structured lsp view");
    keep_element(structured);
}
