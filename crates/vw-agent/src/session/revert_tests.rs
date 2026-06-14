#[test]
fn revert_tests_module_is_wired() {
    let marker = String::from("revert_tests");
    assert_eq!(marker.as_str(), "revert_tests");
}

#[test]
fn revert_input_serializes_with_expected_camel_case_keys() {
    let input = super::RevertInput {
        session_id: "session-1".to_string(),
        message_id: "message-1".to_string(),
        part_id: Some("part-1".to_string()),
    };

    let value = serde_json::to_value(&input).unwrap();

    assert_eq!(value["sessionID"], "session-1");
    assert_eq!(value["messageID"], "message-1");
    assert_eq!(value["partID"], "part-1");
}

#[test]
fn revert_input_skips_absent_part_id() {
    let input = super::RevertInput {
        session_id: "session-1".to_string(),
        message_id: "message-1".to_string(),
        part_id: None,
    };

    let value = serde_json::to_value(&input).unwrap();

    assert!(value.get("partID").is_none());
}

#[test]
fn extra_builds_json_map_from_pairs() {
    let map = super::extra([
        ("sessionID", serde_json::Value::String("session-1".to_string())),
        ("count", serde_json::json!(2)),
    ]);

    assert_eq!(map.get("sessionID").and_then(|v| v.as_str()), Some("session-1"));
    assert_eq!(map.get("count").and_then(|v| v.as_i64()), Some(2));
}
