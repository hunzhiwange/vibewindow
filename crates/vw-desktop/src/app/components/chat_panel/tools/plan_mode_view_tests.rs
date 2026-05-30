use super::plan_mode_view::{
    bool_field, derived_summary, is_plan_mode_tool, metadata_text, string_field, string_list_field,
    u64_field,
};
use serde_json::json;

#[test]
fn plan_mode_tool_names_are_explicit() {
    assert!(is_plan_mode_tool("enter_plan_mode"));
    assert!(is_plan_mode_tool("verify_plan_execution"));
    assert!(!is_plan_mode_tool("bash"));
}

#[test]
fn field_helpers_ignore_wrong_types_and_empty_strings() {
    let value = json!({"goal":" ship ","ok":true,"count":3,"items":["a", "", "b"]});
    let data = value.as_object();

    assert_eq!(string_field(data, "goal"), Some("ship"));
    assert!(bool_field(data, "ok"));
    assert_eq!(u64_field(data, "count"), 3);
    assert_eq!(string_list_field(data, "items"), vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn derived_summary_and_metadata_are_deterministic() {
    let value = json!({"ready":true,"pending_count":4,"todo_count":5,"in_progress_count":1,"goal":"finish"});
    let data = value.as_object();

    assert_eq!(derived_summary("verify_plan_execution", data), "Ready to execute 4 todo(s)");
    assert!(metadata_text("verify_plan_execution", data).contains("Pending: 4"));
}
