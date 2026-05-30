use super::advanced_view::{
    format_tool_search_body, is_advanced_surface_tool, nested_string, result_data_message,
};
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
