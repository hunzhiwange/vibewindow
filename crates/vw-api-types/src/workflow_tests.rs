use super::workflow::{WorkflowRunRequest, WorkflowRunStatus};
use serde_json::json;

#[test]
fn workflow_run_request_defaults_max_steps() {
    let request: WorkflowRunRequest = serde_json::from_value(json!({})).expect("request");

    assert_eq!(request.max_steps, 200);
    assert!(request.inputs.is_empty());
    assert_eq!(request.workflow_yaml, None);
    assert_eq!(WorkflowRunRequest::default().max_steps, 200);
}

#[test]
fn workflow_status_uses_snake_case_contract() {
    let value = serde_json::to_value(WorkflowRunStatus::Succeeded).expect("status");

    assert_eq!(value, json!("succeeded"));
}
