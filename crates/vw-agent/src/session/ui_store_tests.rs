use super::*;

#[test]
fn session_scope_round_trips_and_clears() {
    set_session_scope(Some("/tmp/vw-scope"));
    assert_eq!(current_session_scope().as_deref(), Some("/tmp/vw-scope"));

    set_session_scope(None);
    assert_eq!(current_session_scope(), None);
}

struct ScopeReset;

impl Drop for ScopeReset {
    fn drop(&mut self) {
        set_session_scope(None);
    }
}

fn test_session(id: &str) -> ChatSession {
    ChatSession {
        id: id.to_string(),
        title: "Test session".to_string(),
        messages: vec![ChatMessage {
            role: ChatRole::User,
            content: "hello".to_string(),
            think_timing: Vec::new(),
        }],
        message_ids: vec![Some("msg-1".to_string())],
        calls: Vec::new(),
        steps: Vec::new(),
        created_ms: 1,
        updated_ms: 2,
    }
}

#[test]
fn scoped_todos_round_trip_through_shared_store() {
    let session_id = format!("ui-store-todos-{}", std::process::id());
    let scope = Some("project/ui-store-scope");
    let todos = vec![SessionTodoItem {
        id: "todo-1".to_string(),
        content: "cover ui store".to_string(),
        status: "pending".to_string(),
        priority: "medium".to_string(),
    }];

    let path = save_session_todos_scoped(&session_id, &todos, scope).expect("todo path");
    assert!(path.is_file());
    assert_eq!(load_session_todos_scoped(&session_id, scope).len(), 1);
}

#[test]
fn scoped_session_save_load_preview_append_and_delete() {
    let _scope_reset = ScopeReset;
    let session_id = format!("ui-store-session-{}", std::process::id());
    let scope = Some("project/ui-store-session-scope");
    let session = test_session(&session_id);

    set_session_scope(scope);
    let path = save_session_scoped(&session, scope).expect("session path");
    assert!(path.is_file());

    let loaded = load_session_scoped(&session_id, scope).expect("loaded session");
    assert_eq!(loaded.title, "Test session");
    let preview = session_preview_meta(&session_id).expect("preview meta");
    assert_eq!(preview.id, session_id);
    assert_eq!(preview.message_count, 1);

    let payload = serde_json::json!({"stream_id": 9, "delta": "first"});
    persist_ai_call_payload(&session_id, 0, &payload, scope).expect("persist call");
    let replacement = serde_json::json!({"stream_id": 9, "delta": "replacement"});
    persist_ai_call_payload(&session_id, 0, &replacement, scope).expect("replace call");
    let loaded = load_session_scoped(&session_id, scope).expect("loaded session with call");
    assert_eq!(loaded.calls.len(), 1);
    assert_eq!(loaded.calls[0]["delta"], "replacement");

    delete_session_scoped(&session_id, scope);
    assert!(load_session_scoped(&session_id, scope).is_none());
}

#[test]
fn archived_session_ids_are_scoped_by_project_key() {
    let mut ids = std::collections::HashSet::new();
    ids.insert("archived-one".to_string());
    let scope = Some("project/archive-scope");

    save_archived_session_ids_scoped(&ids, scope);
    assert!(load_archived_session_ids_scoped(scope).contains("archived-one"));
    assert!(
        !load_archived_session_ids_scoped(Some("project/other-scope")).contains("archived-one")
    );
}
