use serde_json::json;
use vw_api_types::session::{
    GatewaySessionCreateBody, GatewaySessionDiffQuery, GatewaySessionForkBody,
    GatewaySessionMessageListQuery, GatewaySessionPatchBody, GatewaySessionPatchTime,
    GatewaySessionResetBody, GatewaySessionSummarizeBody, GatewaySessionTitleGenerateBody,
    GatewaySessionTodoItem, GatewaySessionTodoPutBody,
};
use vw_api_types::todo::{TodoPriority, TodoStatus};
use vw_shared::session::ui_types::{ChatMessage, ChatRole, ChatSession};

use crate::client::test_support;

fn chat_session() -> ChatSession {
    ChatSession {
        id: "s1".to_string(),
        title: "Session".to_string(),
        messages: vec![ChatMessage {
            role: ChatRole::User,
            content: "hello".to_string(),
            think_timing: Vec::new(),
        }],
        message_ids: vec![Some("m1".to_string())],
        calls: Vec::new(),
        steps: Vec::new(),
        created_ms: 1,
        updated_ms: 2,
    }
}

#[tokio::test]
async fn session_api_routes_lifecycle_messages_ui_scope_and_title_calls() {
    let server = test_support::server(vec![
        (200, json!({"items": [{"id": "s1"}]})),
        (200, json!({"id": "s1", "title": "Session"})),
        (200, json!([{"id": "m1"}])),
        (200, json!([{"id": "m2"}])),
        (200, json!({"id": "s2"})),
        (200, json!({"id": "s1", "title": "Renamed"})),
        (204, json!(null)),
        (200, json!({"id": "forked"})),
        (200, json!({"id": "s1", "reset": true})),
        (200, json!(true)),
        (200, json!(true)),
        (200, json!([{"content": "Do it", "status": "pending", "priority": "high", "id": 7}])),
        (200, json!({"files": []})),
        (200, json!({"files": []})),
        (200, serde_json::to_value(chat_session()).expect("chat session json")),
        (404, json!({"missing": true})),
        (200, json!(true)),
        (
            200,
            json!([{
                "id": "s1",
                "title": "Session",
                "updated_ms": 2,
                "message_count": 1,
                "call_count": 0,
                "last_content": "hello"
            }]),
        ),
        (404, json!({"missing": true})),
        (404, json!({"missing": true})),
        (200, json!(["s-old"])),
        (200, json!(true)),
        (200, json!("global-scope")),
        (200, json!(true)),
        (200, json!({"title": "Generated"})),
    ]);

    let list: serde_json::Value = server.client().session_list(Some("/repo")).await.expect("list");
    assert_eq!(list["items"][0]["id"], "s1");
    let get: serde_json::Value =
        server.client().session_get("s1", Some("/repo")).await.expect("get");
    assert_eq!(get["title"], "Session");
    let messages: serde_json::Value =
        server.client().session_messages("s1", Some("/repo")).await.expect("messages");
    assert_eq!(messages[0]["id"], "m1");
    let queried: serde_json::Value = server
        .client()
        .session_messages_query(
            "s1",
            &GatewaySessionMessageListQuery {
                directory: Some("/repo".to_string()),
                limit: Some(25),
            },
        )
        .await
        .expect("messages query");
    assert_eq!(queried[0]["id"], "m2");
    let created: serde_json::Value = server
        .client()
        .session_create(
            "/repo",
            &Some(GatewaySessionCreateBody {
                parent_id: Some("parent".to_string()),
                title: Some("New".to_string()),
            }),
        )
        .await
        .expect("create");
    assert_eq!(created["id"], "s2");
    let updated: serde_json::Value = server
        .client()
        .session_update(
            "s1",
            Some("/repo"),
            &GatewaySessionPatchBody {
                title: Some("Renamed".to_string()),
                time: Some(GatewaySessionPatchTime { archived: Some(123) }),
            },
        )
        .await
        .expect("update");
    assert_eq!(updated["title"], "Renamed");
    server.client().session_delete("s1", Some("/repo")).await.expect("delete");
    let forked: serde_json::Value = server
        .client()
        .session_fork(
            "s1",
            Some("/repo"),
            &Some(GatewaySessionForkBody { message_id: Some("m1".to_string()) }),
        )
        .await
        .expect("fork");
    assert_eq!(forked["id"], "forked");
    let reset: serde_json::Value = server
        .client()
        .session_reset(
            "s1",
            Some("/repo"),
            &GatewaySessionResetBody { message_id: "m1".to_string(), revert_code: true },
        )
        .await
        .expect("reset");
    assert!(reset["reset"].as_bool().unwrap());
    assert!(
        server
            .client()
            .session_summarize(
                "s1",
                Some("/repo"),
                &GatewaySessionSummarizeBody { message_id: "m1".to_string() },
            )
            .await
            .expect("summarize")
    );
    assert!(
        server
            .client()
            .session_todo_update(
                "s1",
                Some("/repo"),
                &GatewaySessionTodoPutBody {
                    todos: vec![GatewaySessionTodoItem {
                        id: "todo-1".to_string(),
                        content: "Do it".to_string(),
                        status: TodoStatus::Pending,
                        priority: TodoPriority::High,
                    }],
                },
            )
            .await
            .expect("todo update")
    );
    let todos = server.client().session_todo_get("s1", Some("/repo")).await.expect("todos");
    assert_eq!(todos[0].id, "7");
    let diff: serde_json::Value = server
        .client()
        .session_diff(
            "s1",
            &GatewaySessionDiffQuery {
                directory: Some("/repo".to_string()),
                message_id: Some("m1".to_string()),
            },
        )
        .await
        .expect("diff");
    assert_eq!(diff["files"], json!([]));
    let diff_without_message: serde_json::Value = server
        .client()
        .session_diff(
            "s1",
            &GatewaySessionDiffQuery { directory: None, message_id: Some("   ".to_string()) },
        )
        .await
        .expect("diff without message");
    assert_eq!(diff_without_message["files"], json!([]));
    assert_eq!(
        server.client().session_ui_get("s1", Some("/repo")).await.expect("ui get").expect("ui").id,
        "s1"
    );
    assert!(server.client().session_ui_get_any("missing").await.expect("ui any").is_none());
    assert!(
        server
            .client()
            .session_ui_save("s1", Some("/repo"), &chat_session())
            .await
            .expect("ui save")
    );
    assert_eq!(
        server.client().session_ui_previews(Some("/repo")).await.expect("previews")[0].last_content,
        Some("hello".to_string())
    );
    assert!(
        server
            .client()
            .session_preview_meta_get("missing", Some("/repo"))
            .await
            .expect("preview")
            .is_none()
    );
    assert!(
        server.client().session_path_get("missing", Some("/repo")).await.expect("path").is_none()
    );
    assert_eq!(
        server.client().session_archived_get(Some("/repo")).await.expect("archived"),
        vec!["s-old".to_string()]
    );
    assert!(
        server
            .client()
            .session_archived_put(Some("/repo"), &["s-old".to_string()])
            .await
            .expect("archived put")
    );
    assert_eq!(
        server.client().session_scope_get(Some("/ignored"), Some("project")).await.expect("scope"),
        Some("global-scope".to_string())
    );
    assert!(server.client().session_scope_put(Some("global-scope")).await.expect("scope put"));
    assert_eq!(
        server
            .client()
            .session_title_generate(
                "s1",
                &GatewaySessionTitleGenerateBody {
                    content: "hello world".to_string(),
                    preferred_model: Some("gpt".to_string()),
                    acp_agent: None,
                },
            )
            .await
            .expect("title")
            .title,
        "Generated"
    );

    assert_eq!(server.take_request().path, "/v1/session?directory=%2Frepo");
    assert_eq!(server.take_request().path, "/v1/session/s1?directory=%2Frepo");
    assert_eq!(server.take_request().path, "/v1/session/s1/message?directory=%2Frepo");
    assert_eq!(server.take_request().path, "/v1/session/s1/message?directory=%2Frepo&limit=25");
    let request = server.take_request();
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/v1/session?directory=%2Frepo");
    assert_eq!(request.body["parentID"], "parent");
    let request = server.take_request();
    assert_eq!(request.method, "PATCH");
    assert_eq!(request.path, "/v1/session/s1?directory=%2Frepo");
    assert_eq!(request.body["time"]["archived"], 123);
    let request = server.take_request();
    assert_eq!(request.method, "DELETE");
    assert_eq!(request.path, "/v1/session/s1?directory=%2Frepo");
    let request = server.take_request();
    assert_eq!(request.path, "/v1/session/s1/fork?directory=%2Frepo");
    assert_eq!(request.body["messageID"], "m1");
    let request = server.take_request();
    assert_eq!(request.path, "/v1/session/s1/reset?directory=%2Frepo");
    assert_eq!(request.body["revertCode"], true);
    assert_eq!(server.take_request().path, "/v1/session/s1/summarize?directory=%2Frepo");
    let request = server.take_request();
    assert_eq!(request.method, "PUT");
    assert_eq!(request.path, "/v1/session/s1/todo?directory=%2Frepo");
    assert_eq!(request.body["todos"][0]["priority"], "high");
    assert_eq!(server.take_request().path, "/v1/session/s1/todo?directory=%2Frepo");
    assert_eq!(server.take_request().path, "/v1/session/s1/diff?directory=%2Frepo&messageID=m1");
    assert_eq!(server.take_request().path, "/v1/session/s1/diff");
    assert_eq!(server.take_request().path, "/v1/session/s1/ui?directory=%2Frepo");
    assert_eq!(server.take_request().path, "/v1/session/missing/any");
    let request = server.take_request();
    assert_eq!(request.method, "PUT");
    assert_eq!(request.path, "/v1/session/s1/ui?directory=%2Frepo");
    assert_eq!(request.body["messages"][0]["content"], "hello");
    assert_eq!(server.take_request().path, "/v1/session/ui-previews?directory=%2Frepo");
    assert_eq!(server.take_request().path, "/v1/session/missing/preview?directory=%2Frepo");
    assert_eq!(server.take_request().path, "/v1/session/missing/path?directory=%2Frepo");
    assert_eq!(server.take_request().path, "/v1/session/archived?directory=%2Frepo");
    let request = server.take_request();
    assert_eq!(request.method, "PUT");
    assert_eq!(request.path, "/v1/session/archived?directory=%2Frepo");
    assert_eq!(request.body, json!(["s-old"]));
    assert_eq!(server.take_request().path, "/v1/session/scope");
    let request = server.take_request();
    assert_eq!(request.method, "PUT");
    assert_eq!(request.path, "/v1/session/scope");
    assert_eq!(request.body["scope"], "global-scope");
    let request = server.take_request();
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/v1/session/s1/title");
    assert_eq!(request.body["preferred_model"], "gpt");
    server.join();
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn session_todo_get_blocking_uses_same_route_and_directory_query() {
    let server = test_support::server(vec![(
        200,
        json!([{"content": "Blocking", "status": "done", "priority": "low", "id": "todo"}]),
    )]);

    let todos =
        server.client().session_todo_get_blocking("s1", Some("/repo")).expect("blocking todos");

    assert_eq!(todos[0].content, "Blocking");
    assert_eq!(server.take_request().path, "/v1/session/s1/todo?directory=%2Frepo");
    server.join();
}
