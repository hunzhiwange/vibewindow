use super::store::{read_for_tool, write_todos};
use serde_json::json;

fn session(name: &str) -> String {
    format!("store-test-{name}-{}", std::process::id())
}

#[test]
fn write_todos_replaces_and_normalizes_items() {
    let session = session("replace");
    let output = write_todos(
        &session,
        json!({"todos": [{"id": "", "content": " task ", "status": "pending"}]}),
    )
    .expect("write succeeds");

    assert!(output.contains("\"content\": \"task\""));
    let todos = read_for_tool(&session).expect("read succeeds");
    assert_eq!(todos[0].id, "1");
}

#[test]
fn write_todos_merge_updates_existing_item() {
    let session = session("merge");
    write_todos(&session, json!({"todos": [{"id": 1, "content": "task"}]})).expect("seed succeeds");
    write_todos(&session, json!({"merge": true, "todos": [{"id": 1, "status": "completed"}]}))
        .expect("merge succeeds");

    let todos = read_for_tool(&session).expect("read succeeds");
    assert_eq!(todos[0].status, "completed");
}
