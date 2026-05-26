use super::*;
use crate::app::agent::util::MaybeSet;
use serde_json::json;

#[test]
fn parse_string_list_accepts_csv_and_arrays() {
    assert_eq!(
        ModelRoutingConfigTool::parse_string_list(&json!("a, b,,c"), "items").unwrap(),
        vec!["a", "b", "c"]
    );
    assert_eq!(
        ModelRoutingConfigTool::parse_string_list(&json!(["a", " ", "b"]), "items").unwrap(),
        vec!["a", "b"]
    );
}

#[test]
fn parse_non_empty_string_rejects_missing_and_blank() {
    assert_eq!(
        ModelRoutingConfigTool::parse_non_empty_string(&json!({"name":" agent "}), "name").unwrap(),
        "agent"
    );
    assert!(
        ModelRoutingConfigTool::parse_non_empty_string(&json!({"name":"   "}), "name").is_err()
    );
}

#[test]
fn optional_updates_distinguish_unset_null_and_values() {
    assert_eq!(
        ModelRoutingConfigTool::parse_optional_string_update(&json!({}), "model").unwrap(),
        MaybeSet::Unset
    );
    assert_eq!(
        ModelRoutingConfigTool::parse_optional_string_update(&json!({"model": null}), "model")
            .unwrap(),
        MaybeSet::Null
    );
    assert_eq!(
        ModelRoutingConfigTool::parse_optional_string_update(&json!({"model":" gpt "}), "model")
            .unwrap(),
        MaybeSet::Set("gpt".into())
    );
    assert_eq!(
        ModelRoutingConfigTool::parse_optional_bool(&json!({"enabled": true}), "enabled").unwrap(),
        Some(true)
    );
}
