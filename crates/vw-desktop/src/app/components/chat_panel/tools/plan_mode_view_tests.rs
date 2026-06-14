use super::plan_mode_view::{
    bool_field, derived_summary, is_plan_mode_tool, metadata_text, string_field, string_list_field,
    tool_plan_mode_view, u64_field,
};
use crate::app::{App, Message};
use serde_json::json;

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

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

#[test]
fn derived_summary_covers_plan_mode_state_transitions_and_blockers() {
    let enter_active = json!({"active":true});
    assert_eq!(derived_summary("enter_plan_mode", enter_active.as_object()), "Plan mode enabled");

    let enter_already = json!({"already_active":true});
    assert_eq!(
        derived_summary("enter_plan_mode", enter_already.as_object()),
        "Plan mode remains active"
    );

    let exit_done = json!({"exited":true});
    assert_eq!(derived_summary("exit_plan_mode", exit_done.as_object()), "Plan mode disabled");

    let blocked = json!({"ready":false,"blockers":["missing todos","dirty plan"]});
    assert_eq!(
        derived_summary("verify_plan_execution", blocked.as_object()),
        "Blocked by 2 issue(s)"
    );
}

#[test]
fn metadata_text_truncates_goal_and_note_without_verify_counts() {
    let value = json!({"goal":"ship feature","note":"review first"});
    let metadata = metadata_text("enter_plan_mode", value.as_object());

    assert!(metadata.contains("Goal: ship feature"));
    assert!(metadata.contains("Note: review first"));
    assert!(!metadata.contains("Todo:"));
}

#[test]
fn tool_plan_mode_view_rejects_invalid_inputs_and_empty_completed_data() {
    let app = test_app();

    assert!(tool_plan_mode_view(&app, 0, 0, "tool read\n{}").is_none());
    assert!(tool_plan_mode_view(&app, 0, 0, "tool enter_plan_mode\nnot json").is_none());
    assert!(
        tool_plan_mode_view(
            &app,
            0,
            0,
            r#"tool enter_plan_mode
{"status":"completed","result":{"data":{}}}"#
        )
        .is_none()
    );
}

#[test]
fn tool_plan_mode_view_builds_enter_exit_verify_running_and_error_cards() {
    let app = test_app();

    let enter = tool_plan_mode_view(&app, 1, 1, r#"tool enter_plan_mode
{"status":"completed","result":{"data":{"active":true,"message":"Plan mode enabled","instructions":["inspect","plan"],"goal":"cover tests"}}}"#)
    .expect("enter view");
    keep_element(enter);

    let exit = tool_plan_mode_view(&app, 1, 2, r#"tool exit_plan_mode
{"status":"completed","result":{"data":{"exited":false,"active":true,"blockers":["finish plan"]}}}"#)
    .expect("exit view");
    keep_element(exit);

    let verify = tool_plan_mode_view(&app, 1, 3, r#"tool verify_plan_execution
{"status":"completed","result":{"data":{"ready":true,"todo_count":3,"pending_count":2,"in_progress_count":1}}}"#)
    .expect("verify view");
    keep_element(verify);

    let running = tool_plan_mode_view(
        &app,
        1,
        4,
        r#"tool enter_plan_mode
{"status":"running"}"#,
    )
    .expect("running view");
    keep_element(running);

    let error = tool_plan_mode_view(
        &app,
        1,
        5,
        r#"tool verify_plan_execution
{"status":"error","error":"blocked"}"#,
    )
    .expect("error view");
    keep_element(error);
}
