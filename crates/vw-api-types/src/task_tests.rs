use crate::task::{CommandTaskSpecDto, CreateTaskRequest, ListTasksRequest, TaskKind, TaskStatus};
use serde_json::json;

#[test]
fn task_requests_default_stream_and_filters() {
    let create: CreateTaskRequest = serde_json::from_value(json!({
        "project_id": "project-1",
        "kind": "command"
    }))
    .expect("valid create");
    assert!(!create.stream);
    assert_eq!(create.command, None);

    let list: ListTasksRequest = serde_json::from_value(json!({})).expect("valid list");
    assert_eq!(list.project_id, None);
    assert_eq!(list.status, None);

    let command = CommandTaskSpecDto {
        argv: vec!["echo".to_string()],
        cwd: ".".to_string(),
        env: Default::default(),
    };
    assert!(command.env.is_empty());
    assert_eq!(serde_json::to_value(TaskKind::Command).expect("serialize"), json!("command"));
    assert_eq!(serde_json::to_value(TaskStatus::Cancelled).expect("serialize"), json!("cancelled"));
}
