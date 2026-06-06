use super::{
    VibeWindowSseEvent, router, workflow_response_to_chat_response, workflow_response_to_sse_events,
};
use serde_json::json;
use std::collections::BTreeMap;
use vw_api_types::workflow::{WorkflowRunResponse, WorkflowRunStatus};

#[test]
fn workflow_router_is_wired() {
    let _router = router();
}

#[test]
fn chat_response_maps_workflow_answer_and_conversation() {
    let response = WorkflowRunResponse {
        run_id: "run-1".to_string(),
        status: WorkflowRunStatus::Succeeded,
        answer: None,
        outputs: BTreeMap::from([("answer".to_string(), json!("ok"))]),
        nodes: Vec::new(),
        error: None,
        pause: None,
    };

    let chat = workflow_response_to_chat_response(&response, Some("conv-1".to_string()));

    assert_eq!(chat.task_id, "run-1");
    assert_eq!(chat.conversation_id, "conv-1");
    assert_eq!(chat.answer, "ok");
    assert_eq!(chat.mode, "advanced-chat");
}

#[test]
fn streaming_events_keep_conversation_id() {
    let response = WorkflowRunResponse {
        run_id: "run-1".to_string(),
        status: WorkflowRunStatus::Succeeded,
        answer: Some("done".to_string()),
        outputs: BTreeMap::new(),
        nodes: Vec::new(),
        error: None,
        pause: None,
    };

    let events = workflow_response_to_sse_events(response, Some("conv-1".to_string()));

    assert!(events.iter().any(|event| matches!(
        event,
        VibeWindowSseEvent::Message { conversation_id, .. } if conversation_id == "conv-1"
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        VibeWindowSseEvent::MessageEnd { conversation_id, .. } if conversation_id == "conv-1"
    )));
}
