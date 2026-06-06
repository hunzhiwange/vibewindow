use super::tool_parse::{
    explore_item_dedupe_key, explore_tool_kind, is_explore_tool, tool_call_id_from_raw, tool_input,
    tool_input_path, tool_name_from_raw, tool_status_from_raw,
};

#[test]
fn tool_name_and_call_id_are_read_from_raw_tool_block() {
    let raw = r#"tool bash
{"call_id":"call-1","status":"completed","input":"echo ok"}"#;

    assert_eq!(tool_name_from_raw(raw), Some("bash".to_string()));
    assert_eq!(tool_call_id_from_raw(raw), Some("call-1".to_string()));
    assert_eq!(tool_status_from_raw(raw), Some("completed".to_string()));
}

#[test]
fn explore_tool_kind_classifies_read_like_tools() {
    assert!(is_explore_tool("read"));
    assert!(explore_tool_kind("read").is_some());
    assert!(
        explore_item_dedupe_key(
            r#"tool read
{"input":"{\"filePath\":\"src/main.rs\"}"}"#
        )
        .is_some()
    );
}

#[test]
fn tool_input_accepts_raw_input_alias() {
    let value = serde_json::json!({
        "rawInput": "{\"command\":\"date\"}"
    });

    assert_eq!(tool_input(&value), "{\"command\":\"date\"}");
}

#[test]
fn tool_input_path_accepts_claude_file_path_alias() {
    assert_eq!(
        tool_input_path(r#"{"file_path":"docs/demo.md"}"#),
        Some("docs/demo.md".to_string())
    );
}
