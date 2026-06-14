use super::*;
use crate::app::agent::config::Config;
use crate::app::agent::gateway::{GatewayRateLimiter, IdempotencyStore};
use crate::app::agent::memory::NoneMemory;
use crate::app::agent::providers::Provider;
use crate::app::agent::security::pairing::PairingGuard;
use crate::app::agent::{project, session};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue};
use std::path::Path as FsPath;
use std::sync::Arc;
use std::sync::Once;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::broadcast;
use vw_api_types::session::{
    GatewaySessionCreateBody, GatewaySessionPatchBody, GatewaySessionPatchTime,
    GatewaySessionTodoItem, GatewaySessionTodoPutBody,
};
use vw_api_types::todo::{TodoPriority, TodoStatus};

struct StaticProvider;

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Provider for StaticProvider {
    async fn chat_with_system(
        &self,
        _system_prompt: Option<&str>,
        message: &str,
        _model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        Ok(message.to_string())
    }
}

static TEST_HOME: Once = Once::new();
static SESSION_OPS_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn ensure_test_home() {
    TEST_HOME.call_once(|| {
        let dir = std::env::temp_dir().join(format!("vw-session-ops-tests-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("test home should be created");
        unsafe {
            std::env::set_var("VIBEWINDOW_TEST_HOME", dir);
        }
    });
}

fn headers_for(directory: &FsPath) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-vibewindow-directory",
        HeaderValue::from_str(&directory.to_string_lossy()).expect("valid directory header"),
    );
    headers
}

fn query_for(directory: &FsPath) -> InstanceQuery {
    InstanceQuery { directory: Some(directory.to_string_lossy().to_string()) }
}

fn state() -> AppState {
    let (event_tx, _) = broadcast::channel(16);
    AppState {
        config: Arc::new(parking_lot::Mutex::new(Config::default())),
        provider: Arc::new(StaticProvider),
        model: "test-model".to_string(),
        temperature: 0.0,
        mem: Arc::new(NoneMemory::new()),
        auto_save: false,
        webhook_secret_hash: None,
        pairing: Arc::new(PairingGuard::new(false, &[])),
        trust_forwarded_headers: false,
        rate_limiter: Arc::new(GatewayRateLimiter::new(100, 100, 100)),
        idempotency_store: Arc::new(IdempotencyStore::new(Duration::from_secs(300), 1000)),
        whatsapp: None,
        whatsapp_app_secret: None,
        linq: None,
        linq_signing_secret: None,
        nextcloud_talk: None,
        nextcloud_talk_webhook_secret: None,
        wati: None,
        qq: None,
        qq_webhook_enabled: false,
        observer: Arc::new(crate::app::agent::observability::NoopObserver),
        tools_registry: Arc::new(Vec::new()),
        tools_registry_exec: Arc::new(Vec::new()),
        multimodal: crate::app::agent::config::MultimodalConfig::default(),
        max_tool_iterations: 10,
        event_tx,
        session_query_engines: Default::default(),
    }
}

async fn create_session_in(directory: &FsPath, title: &str) -> agent_session::session::Info {
    let body = UiSessionCreateBody {
        session: GatewaySessionCreateBody { parent_id: None, title: Some(title.to_string()) },
        permission: None,
    };
    ui_session_create(Query(query_for(directory)), HeaderMap::new(), Json(Some(body)))
        .await
        .expect("session should be created")
        .0
}

#[test]
fn session_handlers_are_available() {
    let _ = ui_session_list;
    let _ = ui_session_status;
    let _ = ui_session_get;
    let _ = ui_session_create;
    let _ = ui_session_patch;
    let _ = ui_session_delete;
    let _ = session_children;
    let _ = session_todo_get;
    let _ = session_todo_put;
    let _ = session_fork;
    let _ = session_reset;
    let _ = session_diff;
    let _ = session_summarize;
    let _ = session_title_generate;
}

#[tokio::test]
async fn session_ops_cover_lifecycle_filters_and_instance_boundaries() {
    let _guard = SESSION_OPS_TEST_LOCK.lock().await;
    ensure_test_home();
    project::instance::dispose_all().await;
    session::ui_store::set_session_scope(None);

    let project_a = TempDir::new().expect("project a dir");
    let project_b = TempDir::new().expect("project b dir");
    let project_a_path = project_a.path();
    let project_b_path = project_b.path();

    let root = create_session_in(project_a_path, "Alpha Root").await;
    let child_body = UiSessionCreateBody {
        session: GatewaySessionCreateBody {
            parent_id: Some(root.id.clone()),
            title: Some("Alpha Child".to_string()),
        },
        permission: None,
    };
    let child = ui_session_create(
        Query(query_for(project_a_path)),
        HeaderMap::new(),
        Json(Some(child_body)),
    )
    .await
    .expect("child should be created")
    .0;
    let other = create_session_in(project_b_path, "Beta Root").await;

    let Json(listed) = ui_session_list(
        Query(UiSessionListQuery {
            directory: Some(project_a_path.to_string_lossy().to_string()),
            roots: Some(true),
            start: Some(0),
            search: Some("alpha".to_string()),
            limit: Some(10),
        }),
        HeaderMap::new(),
    )
    .await
    .expect("list should succeed");
    assert_eq!(listed.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), vec![root.id.as_str()]);

    let Json(limited) = ui_session_list(
        Query(UiSessionListQuery {
            directory: Some("  ".to_string()),
            roots: None,
            start: None,
            search: None,
            limit: Some(1),
        }),
        HeaderMap::new(),
    )
    .await
    .expect("blank directory list should succeed");
    assert_eq!(limited.len(), 1);

    let Json(statuses) = ui_session_status(Query(query_for(project_a_path)), HeaderMap::new())
        .await
        .expect("status should succeed");
    assert!(matches!(statuses.get(&root.id), Some(agent_session::status::Info::Idle)));
    assert!(statuses.contains_key(&child.id));
    assert!(!statuses.contains_key(&other.id));

    let Json(global_get) = ui_session_get(
        Path(root.id.clone()),
        Query(InstanceQuery { directory: None }),
        HeaderMap::new(),
    )
    .await
    .expect("global get should succeed");
    assert_eq!(global_get.id, root.id);

    let Json(scoped_get) = ui_session_get(
        Path(root.id.clone()),
        Query(InstanceQuery { directory: None }),
        headers_for(project_a_path),
    )
    .await
    .expect("header scoped get should succeed");
    assert_eq!(scoped_get.directory, project_a_path.to_string_lossy());

    let missing_scoped =
        ui_session_get(Path(other.id.clone()), Query(query_for(project_a_path)), HeaderMap::new())
            .await;
    assert!(missing_scoped.is_err());

    let Json(patched) = ui_session_patch(
        Path(root.id.clone()),
        Query(InstanceQuery { directory: None }),
        HeaderMap::new(),
        Json(GatewaySessionPatchBody {
            title: Some("  Alpha Renamed  ".to_string()),
            time: Some(GatewaySessionPatchTime { archived: Some(42) }),
        }),
    )
    .await
    .expect("global patch should succeed");
    assert_eq!(patched.title, "Alpha Renamed");
    assert_eq!(patched.time.archived, Some(42));

    let Json(unarchived) = ui_session_patch(
        Path(root.id.clone()),
        Query(query_for(project_a_path)),
        HeaderMap::new(),
        Json(GatewaySessionPatchBody {
            title: Some(" ".to_string()),
            time: Some(GatewaySessionPatchTime { archived: Some(0) }),
        }),
    )
    .await
    .expect("scoped patch should succeed");
    assert_eq!(unarchived.title, "Alpha Renamed");
    assert_eq!(unarchived.time.archived, None);

    let Json(children) =
        session_children(Path(root.id.clone()), Query(query_for(project_a_path)), HeaderMap::new())
            .await
            .expect("children should be listed");
    assert_eq!(children.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), vec![child.id.as_str()]);

    let todo = GatewaySessionTodoItem {
        id: "todo-1".to_string(),
        content: "ship tests".to_string(),
        status: TodoStatus::InProgress,
        priority: TodoPriority::High,
    };
    let Json(saved_todos) = session_todo_put(
        Path(root.id.clone()),
        Query(query_for(project_a_path)),
        HeaderMap::new(),
        Json(GatewaySessionTodoPutBody { todos: vec![todo.clone()] }),
    )
    .await
    .expect("todos should be saved");
    assert!(saved_todos);

    let Json(todos) =
        session_todo_get(Path(root.id.clone()), Query(query_for(project_a_path)), HeaderMap::new())
            .await
            .expect("todos should be loaded");
    assert_eq!(todos, vec![todo]);

    let Json(forked) = session_fork(
        State(state()),
        Path(root.id.clone()),
        Query(query_for(project_a_path)),
        HeaderMap::new(),
        Json(None),
    )
    .await
    .expect("fork should succeed");
    assert_eq!(forked.title, "Alpha Renamed (fork #1)");
    assert_eq!(forked.directory, project_a_path.to_string_lossy());

    let Json(deleted) = ui_session_delete(
        State(state()),
        Path(forked.id.clone()),
        Query(query_for(project_a_path)),
        HeaderMap::new(),
    )
    .await
    .expect("scoped delete should succeed");
    assert!(deleted);
    assert!(
        ui_session_get(Path(forked.id), Query(query_for(project_a_path)), HeaderMap::new())
            .await
            .is_err()
    );

    let Json(deleted_other) = ui_session_delete(
        State(state()),
        Path(other.id.clone()),
        Query(InstanceQuery { directory: None }),
        HeaderMap::new(),
    )
    .await
    .expect("global delete should succeed");
    assert!(deleted_other);
}

#[tokio::test]
async fn session_ops_cover_diff_summary_reset_and_title_errors() {
    let _guard = SESSION_OPS_TEST_LOCK.lock().await;
    ensure_test_home();
    project::instance::dispose_all().await;
    session::ui_store::set_session_scope(None);

    let project_dir = TempDir::new().expect("project dir");
    let project_path = project_dir.path();
    let info = create_session_in(project_path, "Error Paths").await;

    let Json(diffs) = session_diff(
        Path(info.id.clone()),
        Query(GatewaySessionDiffQuery {
            directory: Some(project_path.to_string_lossy().to_string()),
            message_id: None,
        }),
        HeaderMap::new(),
    )
    .await
    .expect("diff should return an empty list outside git changes");
    assert!(diffs.is_empty());

    let Json(summarized) = session_summarize(
        Path(info.id.clone()),
        Query(query_for(project_path)),
        HeaderMap::new(),
        Json(GatewaySessionSummarizeBody { message_id: "missing-message".to_string() }),
    )
    .await
    .expect("summarize should safely handle a missing message");
    assert!(summarized);

    let Json(reset) = session_reset(
        State(state()),
        Path(info.id.clone()),
        Query(query_for(project_path)),
        HeaderMap::new(),
        Json(GatewaySessionResetBody {
            message_id: "missing-message".to_string(),
            revert_code: false,
        }),
    )
    .await
    .expect("reset without revert should safely handle a missing message");
    assert_eq!(reset.id, info.id);

    let title_result = session_title_generate(
        Path(info.id),
        Json(GatewaySessionTitleGenerateBody {
            content: "".to_string(),
            preferred_model: None,
            acp_agent: None,
        }),
    )
    .await;
    assert!(title_result.is_err());
}
