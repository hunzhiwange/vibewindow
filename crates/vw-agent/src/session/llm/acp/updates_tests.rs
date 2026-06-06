#[test]
fn updates_tests_module_is_wired() {
    let marker = String::from("updates_tests");
    assert_eq!(marker.as_str(), "updates_tests");
}

#[test]
fn transcript_tool_delta_accepts_input_output_aliases() {
    let update = serde_json::json!({
        "sessionUpdate": "tool_call_update",
        "toolCallId": "call-1",
        "kind": "execute",
        "title": "Terminal",
        "status": "completed",
        "input": {
            "command": "date '+%Y-%m-%d %H:%M:%S'",
            "description": "Get current date and time"
        },
        "output": "2026-06-05 12:34:56\n"
    });
    let update = update.as_object().expect("update should be an object");

    let delta = super::transcript_tool_delta_from_update(update).expect("delta should be built");

    assert!(delta.starts_with("tool execute\n"));
    assert!(delta.contains(r#""input":"{\"command\":\"date '+%Y-%m-%d %H:%M:%S'\""#));
    assert!(delta.contains(r#""output":"2026-06-05 12:34:56\n""#));
}
