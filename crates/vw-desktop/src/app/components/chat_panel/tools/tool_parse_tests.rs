use super::tool_parse::{
    explore_item_dedupe_key, explore_tool_kind, is_explore_tool, tool_call_id_from_raw,
    tool_name_from_raw, tool_status_from_raw,
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
