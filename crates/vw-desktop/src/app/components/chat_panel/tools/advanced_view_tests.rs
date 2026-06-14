use super::advanced_view::{
    format_tool_search_body, is_advanced_surface_tool, nested_string, result_data_message,
    tool_advanced_view,
};
use crate::app::App;
use serde_json::json;

#[test]
fn advanced_surface_tool_accepts_known_tools_and_mcp_prefixes() {
    assert!(is_advanced_surface_tool("AgentTool"));
    assert!(is_advanced_surface_tool("mcp_linear"));
    assert!(!is_advanced_surface_tool("bash"));
}

#[test]
fn nested_string_walks_object_path() {
    let value = json!({"result":{"data":{"message":"done"}}});
    assert_eq!(nested_string(&value, &["result", "data", "message"]), Some("done"));
}

#[test]
fn result_data_message_reads_nested_message() {
    assert_eq!(
        result_data_message(&json!({"result":{"data":{"message":"ok"}}})),
        Some("ok".to_string())
    );
    assert!(
        format_tool_search_body(&json!({
            "result": {"data": {"items": [{"display_name": "alpha"}]}}
        }))
        .is_some()
    );
}

#[test]
fn format_tool_search_body_handles_empty_and_truncated_results() {
    assert_eq!(
        format_tool_search_body(&json!({"result":{"data":{"items":[]}}})),
        Some("未找到匹配工具。".to_string())
    );
    assert!(
        format_tool_search_body(&json!({
            "result": {
                "data": {
                    "count": 8,
                    "items": [
                        {"display_name":"a","reason":"ra"},
                        {"id":"b","reason":"rb"},
                        {"display_name":"c"},
                        {"display_name":"d"},
                        {"display_name":"e"},
                        {"display_name":"f"}
                    ]
                }
            }
        }))
        .expect("tool search body")
        .contains("还有 2 个结果")
    );
    assert!(format_tool_search_body(&json!({"result":{"data":{"items":[42]}}})).is_none());
}

#[test]
fn advanced_view_rejects_unknown_tools_and_invalid_json() {
    let app = App::new().0;

    assert!(tool_advanced_view(&app, 0, 0, "tool bash\n{}").is_none());
    assert!(tool_advanced_view(&app, 0, 0, "tool browser\nnot-json").is_none());
}

#[test]
fn advanced_view_renders_agent_browser_tool_search_and_errors() {
    let mut app = App::new().0;
    app.chat_tool_hovered_idx = Some((4_u64 << 32) | 2);

    assert!(
        tool_advanced_view(
            &app,
            4,
            2,
            r#"tool AgentTool
{"input":"{\"agent\":\"reviewer\",\"prompt\":\"inspect the patch\"}","result":{"data":{"message":"done","agent":"reviewer","session_id":"s1"}}}"#
        )
        .is_some()
    );
    assert!(
        tool_advanced_view(
            &app,
            4,
            2,
            r#"tool browser
{"renderHint":{"metadata":{"action":"open","backend":"playwright"}},"result":{"data":{"result":{"title":"Example","url":"https://example.com"}}}}"#
        )
        .is_some()
    );
    assert!(
        tool_advanced_view(
            &app,
            4,
            2,
            r#"tool tool_search
{"result":{"data":{"items":[{"display_name":"Browser","reason":"navigation"}]}}}"#
        )
        .is_some()
    );
    assert!(
        tool_advanced_view(
            &app,
            4,
            2,
            r#"tool browser
{"status":"error","error":"navigation failed"}"#
        )
        .is_some()
    );
}

#[test]
fn advanced_view_renders_running_and_planned_explicit_surfaces() {
    let app = App::new().0;

    assert!(
        tool_advanced_view(
            &app,
            0,
            0,
            r#"tool verify_plan_execution
{"status":"running","input":"{}"}"#
        )
        .is_some()
    );
    assert!(
        tool_advanced_view(
            &app,
            0,
            0,
            r#"tool mcp_linear
{"input":"{}"}"#
        )
        .is_some()
    );
}
