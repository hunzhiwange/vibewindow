use axum::Json;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use super::*;
use crate::app::agent::storage;

static DESKTOP_STORAGE_LOCK: Mutex<()> = Mutex::const_new(());

struct DesktopStorageSnapshot {
    preferences: Option<Value>,
    json_tool_content: Option<ToolContentBody>,
    html_tool_content: Option<ToolContentBody>,
    mindmap_tabs: Option<Value>,
    project_preferences: Option<ProjectPreferencesBody>,
}

impl DesktopStorageSnapshot {
    async fn capture(project_path: &str) -> Self {
        let project_key = project_prefs_key(project_path);
        let project_key_refs = project_key.iter().map(String::as_str).collect::<Vec<_>>();

        Self {
            preferences: storage::read(&["desktop", "preferences"]).await.ok(),
            json_tool_content: storage::read(&["desktop", "tool_content", "json"]).await.ok(),
            html_tool_content: storage::read(&["desktop", "tool_content", "html"]).await.ok(),
            mindmap_tabs: storage::read(&["desktop", "mindmap_tabs"]).await.ok(),
            project_preferences: storage::read(&project_key_refs).await.ok(),
        }
    }

    async fn restore(self, project_path: &str) {
        restore_key(&["desktop", "preferences"], self.preferences).await;
        restore_key(&["desktop", "tool_content", "json"], self.json_tool_content).await;
        restore_key(&["desktop", "tool_content", "html"], self.html_tool_content).await;
        restore_key(&["desktop", "mindmap_tabs"], self.mindmap_tabs).await;

        let project_key = project_prefs_key(project_path);
        let project_key_refs = project_key.iter().map(String::as_str).collect::<Vec<_>>();
        restore_key(&project_key_refs, self.project_preferences).await;
    }
}

async fn restore_key<T: serde::Serialize>(key: &[&str], value: Option<T>) {
    match value {
        Some(value) => storage::write(key, &value).await.expect("restore desktop storage key"),
        None => storage::remove(key).await.expect("remove desktop storage key"),
    }
}

async fn clear_desktop_storage(project_path: &str) {
    storage::remove(&["desktop", "preferences"]).await.expect("clear preferences");
    storage::remove(&["desktop", "tool_content", "json"]).await.expect("clear json content");
    storage::remove(&["desktop", "tool_content", "html"]).await.expect("clear html content");
    storage::remove(&["desktop", "mindmap_tabs"]).await.expect("clear mindmap tabs");

    let project_key = project_prefs_key(project_path);
    let project_key_refs = project_key.iter().map(String::as_str).collect::<Vec<_>>();
    storage::remove(&project_key_refs).await.expect("clear project prefs");
}

#[test]
fn merge_preferences_patch_initializes_empty_preferences() {
    let mut current = Value::Null;

    merge_preferences_patch(
        &mut current,
        &json!({
            "model": "openai/gpt-5.1-codex-max",
            "auto_model": false
        }),
    );

    assert_eq!(
        current,
        json!({
            "model": "openai/gpt-5.1-codex-max",
            "auto_model": false
        })
    );
}

#[test]
fn merge_preferences_patch_null_removes_existing_key() {
    let mut current = json!({
        "model": "openai/gpt-5.1-codex-max",
        "auto_model": false
    });

    merge_preferences_patch(&mut current, &json!({ "model": null }));

    assert_eq!(current, json!({ "auto_model": false }));
}

#[test]
fn merge_preferences_patch_ignores_non_object_patch() {
    let mut current = json!({ "theme": "dark" });

    merge_preferences_patch(&mut current, &json!(["ignored"]));

    assert_eq!(current, json!({ "theme": "dark" }));
}

#[test]
fn service_command_normalization_allows_only_lifecycle_commands() {
    assert_eq!(normalize_service_command(" STATUS "), Some("status"));
    assert_eq!(normalize_service_command("install"), Some("install"));
    assert_eq!(normalize_service_command("start"), Some("start"));
    assert_eq!(normalize_service_command("stop"), Some("stop"));
    assert_eq!(normalize_service_command("restart"), Some("restart"));
    assert_eq!(normalize_service_command("uninstall"), Some("uninstall"));
    assert_eq!(normalize_service_command("daemon"), None);
    assert_eq!(normalize_service_command("../status"), None);
}

#[tokio::test]
async fn preferences_handlers_default_patch_and_persist_values() {
    let project_path = "/tmp/vw desktop prefs";
    let _guard = DESKTOP_STORAGE_LOCK.lock().await;
    let snapshot = DesktopStorageSnapshot::capture(project_path).await;
    clear_desktop_storage(project_path).await;

    let Json(defaults) = preferences_get().await.expect("preferences should load");
    assert_eq!(defaults, Value::Null);

    let Json(patched) = preferences_patch(Json(json!({
        "model": "openai/gpt-5.1-codex-max",
        "auto_model": false
    })))
    .await
    .expect("preferences should patch");
    assert_eq!(patched["model"], "openai/gpt-5.1-codex-max");

    let Json(patched) = preferences_patch(Json(json!({
        "model": null,
        "temperature": 0.2
    })))
    .await
    .expect("preferences should patch again");
    assert_eq!(patched, json!({ "auto_model": false, "temperature": 0.2 }));

    let Json(stored) = preferences_get().await.expect("preferences should read back");
    assert_eq!(stored, patched);

    snapshot.restore(project_path).await;
}

#[tokio::test]
async fn tool_content_handlers_validate_type_and_round_trip_content() {
    let project_path = "/tmp/vw tool content";
    let _guard = DESKTOP_STORAGE_LOCK.lock().await;
    let snapshot = DesktopStorageSnapshot::capture(project_path).await;
    clear_desktop_storage(project_path).await;

    let Json(default_content) =
        tool_content_get(Path(ToolContentPath { tool_type: "json".to_string() }))
            .await
            .expect("default content should load");
    assert_eq!(default_content, json!({ "content": "" }));

    let Json(saved) = tool_content_put(
        Path(ToolContentPath { tool_type: "html".to_string() }),
        Json(ToolContentBody { content: "<main>Hi</main>".to_string() }),
    )
    .await
    .expect("content should save");
    assert_eq!(saved, json!({ "content": "<main>Hi</main>" }));

    let Json(read_back) = tool_content_get(Path(ToolContentPath { tool_type: "html".to_string() }))
        .await
        .expect("saved content should load");
    assert_eq!(read_back, saved);

    let err = tool_content_get(Path(ToolContentPath { tool_type: "yaml".to_string() }))
        .await
        .expect_err("unsupported get should fail");
    assert_eq!(err.status, StatusCode::BAD_REQUEST);
    assert_eq!(err.to_string(), "unsupported tool content type");

    let err = tool_content_put(
        Path(ToolContentPath { tool_type: "yaml".to_string() }),
        Json(ToolContentBody { content: "x".to_string() }),
    )
    .await
    .expect_err("unsupported put should fail");
    assert_eq!(err.status, StatusCode::BAD_REQUEST);

    snapshot.restore(project_path).await;
}

#[tokio::test]
async fn mindmap_tabs_handlers_default_and_persist_any_json_shape() {
    let project_path = "/tmp/vw mindmap tabs";
    let _guard = DESKTOP_STORAGE_LOCK.lock().await;
    let snapshot = DesktopStorageSnapshot::capture(project_path).await;
    clear_desktop_storage(project_path).await;

    let Json(default_tabs) = mindmap_tabs_get().await.expect("tabs should load");
    assert_eq!(default_tabs, Value::Null);

    let body = json!([
        { "id": "tab-1", "title": "Roadmap" },
        { "id": "tab-2", "nodes": [1, 2, 3] }
    ]);
    let Json(saved) = mindmap_tabs_put(Json(body.clone())).await.expect("tabs should save");
    assert_eq!(saved, body);

    let Json(read_back) = mindmap_tabs_get().await.expect("tabs should read back");
    assert_eq!(read_back, body);

    snapshot.restore(project_path).await;
}

#[test]
fn project_prefs_key_normalizes_paths_into_single_storage_segment() {
    assert_eq!(
        project_prefs_key("/Users/Ada Lovelace/work.demo"),
        vec!["desktop", "project_prefs", "Users_Ada_Lovelace_work_demo"]
    );
    assert_eq!(project_prefs_key(" /.:\\ "), vec!["desktop", "project_prefs", "_root"]);
}

#[tokio::test]
async fn project_preferences_handlers_report_missing_and_round_trip() {
    let project_path = "/tmp/vw project prefs";
    let _guard = DESKTOP_STORAGE_LOCK.lock().await;
    let snapshot = DesktopStorageSnapshot::capture(project_path).await;
    clear_desktop_storage(project_path).await;

    let missing = project_preferences_get(Query(ProjectPreferencesQuery {
        project_path: project_path.to_string(),
    }))
    .await
    .expect_err("missing prefs should fail");
    assert_eq!(missing.status, StatusCode::NOT_FOUND);

    let body = ProjectPreferencesBody {
        model: "anthropic/claude-sonnet-4-5".to_string(),
        auto_model: true,
        acp_agent: Some("codex".to_string()),
    };
    let Json(saved) = project_preferences_put(
        Query(ProjectPreferencesQuery { project_path: project_path.to_string() }),
        Json(body.clone()),
    )
    .await
    .expect("prefs should save");
    assert_eq!(saved.model, body.model);
    assert!(saved.auto_model);
    assert_eq!(saved.acp_agent, Some("codex".to_string()));

    let Json(read_back) = project_preferences_get(Query(ProjectPreferencesQuery {
        project_path: project_path.to_string(),
    }))
    .await
    .expect("prefs should read back");
    assert_eq!(read_back.model, body.model);
    assert!(read_back.auto_model);
    assert_eq!(read_back.acp_agent, Some("codex".to_string()));

    snapshot.restore(project_path).await;
}

#[tokio::test]
async fn external_apps_get_reports_platform_and_finder() {
    let Json(response) = external_apps_get().await.expect("external apps should load");

    assert_eq!(response.platform, host_platform());
    assert!(response.apps.iter().any(|app| app.id == "finder" && app.available));
    assert_eq!(detect_external_apps().first().map(|app| app.id.as_str()), Some("finder"));
}

#[tokio::test]
async fn external_open_handler_rejects_unknown_target_without_spawning() {
    let err = external_apps_open_post(Json(ExternalOpenRequest {
        path: "/tmp".to_string(),
        target: "definitely-unsupported".to_string(),
    }))
    .await
    .expect_err("unknown target should fail");

    assert_eq!(err.status, StatusCode::BAD_REQUEST);
    assert_eq!(err.to_string(), "unsupported external app target");
}

#[test]
fn decode_file_url_path_decodes_plain_and_file_url_paths() {
    assert_eq!(decode_file_url_path("/tmp/space path"), "/tmp/space path");
    assert_eq!(decode_file_url_path("file:///tmp/space%20path"), "/tmp/space path");
    assert_eq!(decode_file_url_path("file://relative%2Fpath"), "relative/path");
}

#[test]
fn router_can_be_constructed_for_unit_state() {
    let _router = router::<()>();
}
