use super::todocards::{
    parse_json_from_value_or_json_string, parse_todoread_card_data, parse_todos_from_array,
    parse_todowrite_card_data, todo_status_symbol, tool_badge_cli,
};
use serde_json::json;

#[test]
fn maps_tool_badges_and_todo_symbols() {
    assert_eq!(tool_badge_cli("read").0, "[R]");
    assert_eq!(tool_badge_cli("apply_patch").0, "[E]");
    assert_eq!(tool_badge_cli("unknown").0, "[U]");
    assert_eq!(todo_status_symbol("completed"), "✓");
    assert_eq!(todo_status_symbol("in_progress"), "·");
    assert_eq!(todo_status_symbol("pending"), "○");
}

#[test]
fn parses_todo_array_with_defaults_and_trimming() {
    let todos = vec![
        json!({"status": " completed ", "content": " done "}),
        json!({"status": "in_progress", "content": " running "}),
        json!({"content": 9}),
    ];

    let data = parse_todos_from_array(&todos);

    assert_eq!(data.total, 3);
    assert_eq!(data.done, 1);
    assert_eq!(data.running, 1);
    assert_eq!(data.pending, 1);
    assert_eq!(data.items[0], ("completed".to_string(), "done".to_string()));
    assert_eq!(data.items[2], ("pending".to_string(), "(empty)".to_string()));
}

#[test]
fn parses_json_from_value_or_encoded_string() {
    let encoded = json!("{\"todos\":[{\"status\":\"completed\",\"content\":\"ok\"}]}");
    let direct = json!({"todos": []});

    assert!(parse_json_from_value_or_json_string(&encoded).unwrap()["todos"].is_array());
    assert_eq!(parse_json_from_value_or_json_string(&direct).unwrap(), direct);
    assert!(parse_json_from_value_or_json_string(&json!("not-json")).is_none());
}

#[test]
fn parses_todowrite_and_todoread_payloads() {
    let write_payload = json!({
        "input": "{\"todos\":[{\"status\":\"completed\",\"content\":\"ship\"}]}"
    })
    .to_string();
    let read_payload = json!({
        "output": [
            {"status":"completed","content":"ship"},
            {"status":"pending","content":"review"}
        ]
    })
    .to_string();

    let write_data = parse_todowrite_card_data(&write_payload).unwrap();
    let read_data = parse_todoread_card_data(&read_payload).unwrap();

    assert_eq!(write_data.done, 1);
    assert_eq!(read_data.total, 2);
    assert_eq!(read_data.pending, 1);
}
