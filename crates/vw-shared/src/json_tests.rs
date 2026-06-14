use serde_json::json;

use super::extract_json_values;

#[test]
fn extract_json_values_empty_input_returns_empty_values() {
    assert!(extract_json_values("").is_empty());
    assert!(extract_json_values(" \n\t ").is_empty());
}

#[test]
fn extract_json_values_whole_json_object_returns_single_value() {
    let values = extract_json_values(r#" { "name": "vibe", "enabled": true } "#);

    assert_eq!(values, vec![json!({"name": "vibe", "enabled": true})]);
}

#[test]
fn extract_json_values_whole_json_array_returns_single_value() {
    let values = extract_json_values(r#" [1, {"nested": ["ok"]}] "#);

    assert_eq!(values, vec![json!([1, {"nested": ["ok"]}])]);
}

#[test]
fn extract_json_values_embedded_json_values_preserves_order() {
    let values =
        extract_json_values(r#"prefix {"id":1} middle [true,false] suffix {"name":"窗口"} tail"#);

    assert_eq!(values, vec![json!({"id": 1}), json!([true, false]), json!({"name": "窗口"})]);
}

#[test]
fn extract_json_values_skips_invalid_json_starters() {
    let values = extract_json_values(r#"bad { nope [ still-bad {"valid":2}"#);

    assert_eq!(values, vec![json!({"valid": 2})]);
}

#[test]
fn extract_json_values_plain_text_returns_empty_values() {
    assert!(extract_json_values("no structured values here").is_empty());
}
