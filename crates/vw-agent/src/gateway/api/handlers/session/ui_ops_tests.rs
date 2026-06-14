use super::*;
use crate::app::agent::gateway::instance::InstanceQuery;
use crate::session::ui_store;
use axum::extract::{Path, Query};
use axum::http::{HeaderMap, HeaderValue};
use std::path::Path as FsPath;
use std::sync::Once;
use tempfile::TempDir;
use vw_api_types::session::GatewaySessionScopeBody;

static TEST_HOME: Once = Once::new();
static UI_OPS_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn ensure_test_home() {
    TEST_HOME.call_once(|| {
        let dir = std::env::temp_dir().join(format!("vw-ui-ops-tests-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("test home should be created");
        unsafe {
            std::env::set_var("VIBEWINDOW_TEST_HOME", dir);
        }
    });
}

fn query_for(directory: &FsPath) -> InstanceQuery {
    InstanceQuery { directory: Some(directory.to_string_lossy().to_string()) }
}

fn headers_for(directory: &FsPath) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-vibewindow-directory",
        HeaderValue::from_str(&directory.to_string_lossy()).expect("valid directory header"),
    );
    headers
}

fn session(id: &str, title: &str) -> ui_models::ChatSession {
    ui_models::ChatSession {
        id: id.to_string(),
        title: title.to_string(),
        messages: vec![ui_models::ChatMessage {
            role: ui_models::ChatRole::User,
            content: "hello".to_string(),
            think_timing: Vec::new(),
        }],
        message_ids: vec![Some("message-1".to_string())],
        calls: vec![serde_json::json!({"tool": "noop"})],
        steps: vec![ui_models::ChatSessionStep {
            index: 0,
            started_ms: 1,
            finished_ms: Some(2),
            start_snapshot_path: None,
            finish_snapshot_path: None,
            usage: ui_models::TokenUsage {
                input_tokens: 3,
                output_tokens: 5,
                cached_tokens: 0,
                reasoning_tokens: 1,
            },
            cost_usd: Some(0.01),
            finish_reason: Some("stop".to_string()),
            model: Some("test-model".to_string()),
        }],
        created_ms: 10,
        updated_ms: 20,
    }
}

#[test]
fn ui_handlers_are_available() {
    let _ = session_ui_get;
    let _ = session_ui_save;
    let _ = session_ui_previews;
    let _ = session_ui_preview_meta;
    let _ = session_archived_get;
    let _ = session_archived_put;
    let _ = session_path_get;
    let _ = session_scope_get;
    let _ = session_scope_put;
    let _ = session_ui_get_any;
}

#[tokio::test(flavor = "multi_thread")]
async fn ui_snapshot_handlers_round_trip_scoped_sessions_and_metadata() {
    let _guard = UI_OPS_TEST_LOCK.lock().await;
    ensure_test_home();
    ui_store::set_session_scope(None);

    let project = TempDir::new().expect("project dir");
    let project_path = project.path();
    let body = session("ui-round-trip", "UI Round Trip");

    let Json(saved) = session_ui_save(
        Path(body.id.clone()),
        Query(query_for(project_path)),
        HeaderMap::new(),
        Json(body.clone()),
    )
    .await
    .expect("session ui save should succeed");
    assert!(saved);

    let Json(loaded) =
        session_ui_get(Path(body.id.clone()), Query(query_for(project_path)), HeaderMap::new())
            .await
            .expect("session ui get should succeed");
    let loaded = loaded.expect("saved session should be loaded");
    assert_eq!(loaded.id, body.id);
    assert_eq!(loaded.messages[0].content, "hello");

    let Json(previews) =
        session_ui_previews(Query(InstanceQuery { directory: None }), headers_for(project_path))
            .await
            .expect("scoped previews should load");
    assert!(previews.iter().any(|preview| {
        preview.id == body.id
            && preview.title == body.title
            && preview.message_count == 1
            && preview.call_count == 1
            && preview.last_content.as_deref() == Some("hello")
    }));

    let scope = ui_store::resolve_session_scope_id(Some(&project_path.to_string_lossy()), None)
        .expect("directory should resolve to a scope");
    ui_store::set_session_scope(Some(&scope));

    let Json(meta) =
        session_ui_preview_meta(Path(body.id.clone())).await.expect("preview meta should load");
    assert_eq!(meta.expect("preview meta").id, body.id);

    let Json(path) =
        session_path_get(Path(body.id.clone())).await.expect("session path should load");
    let path = path.expect("session path");
    assert!(std::path::Path::new(&path).exists());

    let Json(any_session) =
        session_ui_get_any(Path(body.id.clone())).await.expect("any session should load");
    assert_eq!(any_session.expect("any session").title, body.title);

    ui_store::set_session_scope(None);
}

#[tokio::test(flavor = "multi_thread")]
async fn ui_snapshot_handlers_cover_archived_scope_and_errors() {
    let _guard = UI_OPS_TEST_LOCK.lock().await;
    ensure_test_home();
    ui_store::set_session_scope(None);

    let project = TempDir::new().expect("project dir");
    let project_path = project.path();

    let mismatch = session_ui_save(
        Path("route-id".to_string()),
        Query(query_for(project_path)),
        HeaderMap::new(),
        Json(session("body-id", "Mismatch")),
    )
    .await
    .expect_err("body id mismatch should fail");
    assert_eq!(mismatch.status, axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(mismatch.to_string(), "body.id does not match session_id");

    let Json(saved_archived) = session_archived_put(
        Query(query_for(project_path)),
        HeaderMap::new(),
        Json(vec!["archived-a".to_string(), "archived-a".to_string(), "archived-b".to_string()]),
    )
    .await
    .expect("archived ids should save");
    assert!(saved_archived);

    let Json(mut archived) = session_archived_get(Query(query_for(project_path)), HeaderMap::new())
        .await
        .expect("archived ids should load");
    archived.sort();
    assert_eq!(archived, vec!["archived-a".to_string(), "archived-b".to_string()]);

    let Json(initial_scope) = session_scope_get().await.expect("current scope should load");
    assert_eq!(initial_scope, None);

    let Json(scope_saved) = session_scope_put(Json(GatewaySessionScopeBody {
        scope: Some("manual-scope".to_string()),
    }))
    .await
    .expect("scope should save");
    assert!(scope_saved);

    let Json(scope) = session_scope_get().await.expect("scope should load");
    assert_eq!(scope.as_deref(), Some("manual-scope"));

    let Json(cleared) = session_scope_put(Json(GatewaySessionScopeBody { scope: None }))
        .await
        .expect("scope should clear");
    assert!(cleared);
    assert_eq!(session_scope_get().await.expect("scope get").0, None);
}
