use axum::Json;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use tokio::sync::Mutex;
use vw_api_types::tool::{
    GatewayRedisConfigBundle, GatewayRedisConnectionConfig, GatewayRedisConnectionUpsertBody,
    GatewayRedisHistoryListQuery, GatewayRedisHistoryRecord, GatewayRedisKeyAnalysisRequest,
    GatewayRedisKeyCreateRequest, GatewayRedisKeyListQuery, GatewayRedisSettings,
    GatewayRedisSettingsUpdateBody,
};

use super::*;
use crate::app::agent::storage;

static REDIS_STORAGE_LOCK: Mutex<()> = Mutex::const_new(());

struct RedisStorageSnapshot {
    settings: Option<GatewayRedisSettings>,
    connections: Option<Vec<GatewayRedisConnectionConfig>>,
    history: Option<Vec<GatewayRedisHistoryRecord>>,
}

impl RedisStorageSnapshot {
    async fn capture() -> Self {
        Self {
            settings: storage::read(&["redis", "settings"]).await.ok(),
            connections: storage::read(&["redis", "connections"]).await.ok(),
            history: storage::read(&["redis", "history"]).await.ok(),
        }
    }

    async fn restore(self) {
        restore_key(&["redis", "settings"], self.settings).await;
        restore_key(&["redis", "connections"], self.connections).await;
        restore_key(&["redis", "history"], self.history).await;
    }
}

async fn restore_key<T: serde::Serialize>(key: &[&str], value: Option<T>) {
    match value {
        Some(value) => storage::write(key, &value).await.expect("restore redis storage key"),
        None => storage::remove(key).await.expect("remove redis storage key"),
    }
}

async fn clear_redis_storage() {
    storage::remove(&["redis", "settings"]).await.expect("clear redis settings");
    storage::remove(&["redis", "connections"]).await.expect("clear redis connections");
    storage::remove(&["redis", "history"]).await.expect("clear redis history");
}

fn connection(id: &str, name: &str, updated_at_ms: u64) -> GatewayRedisConnectionConfig {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "name": name,
        "host": "127.0.0.1",
        "port": 6379,
        "db": 0,
        "key_pattern": "prefix:*",
        "updated_at_ms": updated_at_ms
    }))
    .expect("connection")
}

fn upsert_body(name: &str, host: &str) -> GatewayRedisConnectionUpsertBody {
    serde_json::from_value(serde_json::json!({
        "name": name,
        "host": host,
        "port": 6380,
        "db": 2,
        "username": " user ",
        "password": " secret ",
        "key_pattern": " keys:* "
    }))
    .expect("upsert body")
}

fn history_record(
    connection_id: Option<&str>,
    connection_label: &str,
    command: &str,
    args: &str,
    is_write: bool,
    time_ms: u64,
) -> GatewayRedisHistoryRecord {
    GatewayRedisHistoryRecord {
        time_ms,
        connection_id: connection_id.map(str::to_string),
        connection_label: connection_label.to_string(),
        command: command.to_string(),
        args: args.to_string(),
        cost_ms: 3,
        is_write,
    }
}

#[test]
fn redis_handler_functions_are_available() {
    let _ = redis_settings_get;
    let _ = redis_settings_put;
    let _ = redis_connections_list;
    let _ = redis_connection_get;
    let _ = redis_connection_create;
    let _ = redis_connection_update;
    let _ = redis_connection_delete;
    let _ = redis_connection_activate;
    let _ = redis_connection_test;
    let _ = redis_connection_overview;
    let _ = redis_connection_keys;
    let _ = redis_connection_key_create;
    let _ = redis_connection_key_analyze;
    let _ = redis_command_execute;
    let _ = redis_history_list;
    let _ = redis_import;
    let _ = redis_export;
}

#[tokio::test]
async fn redis_settings_get_returns_default_when_storage_is_empty() {
    let _guard = REDIS_STORAGE_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;

    let Json(settings) = redis_settings_get().await.expect("settings should load");

    assert_eq!(settings, GatewayRedisSettings::default());
    snapshot.restore().await;
}

#[tokio::test]
async fn redis_settings_put_clamps_value_and_records_history() {
    let _guard = REDIS_STORAGE_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;

    let Json(settings) =
        redis_settings_put(Json(GatewayRedisSettingsUpdateBody { default_load_count: 20_000 }))
            .await
            .expect("settings should save");

    let history = storage_support::load_history().await;
    assert_eq!(settings.default_load_count, 10_000);
    assert_eq!(history[0].command, "UPDATE_SETTINGS");
    assert!(history[0].is_write);
    snapshot.restore().await;
}

#[tokio::test]
async fn redis_connection_create_saves_selected_connection_and_history() {
    let _guard = REDIS_STORAGE_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;

    let Json(created) = redis_connection_create(Json(upsert_body(" local ", " 127.0.0.1 ")))
        .await
        .expect("connection should create");

    let settings = storage_support::load_settings().await;
    let connections = storage_support::load_connections().await;
    let history = storage_support::load_history().await;
    assert_eq!(created.name, "local");
    assert_eq!(created.host, "127.0.0.1");
    assert_eq!(created.key_pattern, "keys:*");
    assert_eq!(settings.selected_connection_id.as_deref(), Some(created.id.as_str()));
    assert_eq!(connections.len(), 1);
    assert_eq!(history[0].command, "SAVE_CONFIG");
    assert!(!history[0].args.contains("secret"));
    snapshot.restore().await;
}

#[tokio::test]
async fn redis_connections_list_sorts_by_recent_activity() {
    let _guard = REDIS_STORAGE_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;
    let mut alpha = connection("alpha", "Alpha", 10);
    alpha.last_used_ms = Some(90);
    let beta = connection("beta", "Beta", 80);
    storage_support::save_connections(&[beta, alpha]).await.expect("seed connections");

    let Json(connections) = redis_connections_list().await.expect("connections should load");

    let ids = connections.into_iter().map(|item| item.id).collect::<Vec<_>>();
    assert_eq!(ids, vec!["alpha", "beta"]);
    snapshot.restore().await;
}

#[tokio::test]
async fn redis_connection_get_returns_match_or_not_found() {
    let _guard = REDIS_STORAGE_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;
    storage_support::save_connections(&[connection("one", "One", 10)])
        .await
        .expect("seed connections");

    let Json(found) = redis_connection_get(Path("one".to_string())).await.expect("found");
    let missing = redis_connection_get(Path("missing".to_string())).await.expect_err("missing");

    assert_eq!(found.id, "one");
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    snapshot.restore().await;
}

#[tokio::test]
async fn redis_connection_update_preserves_id_and_selects_connection() {
    let _guard = REDIS_STORAGE_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;
    let mut existing = connection("one", "One", 10);
    existing.last_used_ms = Some(12);
    storage_support::save_connections(&[existing]).await.expect("seed connections");

    let Json(updated) =
        redis_connection_update(Path("one".to_string()), Json(upsert_body("Two", "redis.local")))
            .await
            .expect("update");

    let settings = storage_support::load_settings().await;
    assert_eq!(updated.id, "one");
    assert_eq!(updated.name, "Two");
    assert_eq!(updated.last_used_ms, Some(12));
    assert_eq!(settings.selected_connection_id.as_deref(), Some("one"));
    snapshot.restore().await;
}

#[tokio::test]
async fn redis_connection_delete_moves_selection_to_first_remaining_connection() {
    let _guard = REDIS_STORAGE_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;
    let mut keep = connection("keep", "Keep", 20);
    keep.last_used_ms = Some(80);
    storage_support::save_connections(&[connection("delete", "Delete", 30), keep])
        .await
        .expect("seed connections");
    storage_support::save_settings(&GatewayRedisSettings {
        selected_connection_id: Some("delete".to_string()),
        ..GatewayRedisSettings::default()
    })
    .await
    .expect("seed settings");

    let Json(response) =
        redis_connection_delete(Path("delete".to_string())).await.expect("delete connection");

    let settings = storage_support::load_settings().await;
    let connections = storage_support::load_connections().await;
    assert_eq!(response.deleted_id, "delete");
    assert_eq!(settings.selected_connection_id.as_deref(), Some("keep"));
    assert_eq!(connections.len(), 1);
    snapshot.restore().await;
}

#[tokio::test]
async fn redis_connection_activate_updates_last_used_and_history() {
    let _guard = REDIS_STORAGE_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;
    storage_support::save_connections(&[connection("one", "One", 10)])
        .await
        .expect("seed connections");

    let Json(settings) =
        redis_connection_activate(Path("one".to_string())).await.expect("activate connection");

    let connections = storage_support::load_connections().await;
    let history = storage_support::load_history().await;
    assert_eq!(settings.selected_connection_id.as_deref(), Some("one"));
    assert!(connections[0].last_used_ms.is_some_and(|value| value >= 10));
    assert_eq!(history[0].command, "OPEN_CONNECTION");
    assert!(!history[0].is_write);
    snapshot.restore().await;
}

#[tokio::test]
async fn redis_runtime_handlers_reject_missing_connection_before_network_access() {
    let _guard = REDIS_STORAGE_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;

    let test_error =
        redis_connection_test(Path("missing".to_string())).await.expect_err("missing test");
    let overview_error =
        redis_connection_overview(Path("missing".to_string())).await.expect_err("missing overview");
    let keys_error = redis_connection_keys(
        Path("missing".to_string()),
        Query(GatewayRedisKeyListQuery::default()),
    )
    .await
    .expect_err("missing keys");

    assert_eq!(test_error.status, StatusCode::NOT_FOUND);
    assert_eq!(overview_error.status, StatusCode::NOT_FOUND);
    assert_eq!(keys_error.status, StatusCode::NOT_FOUND);
    snapshot.restore().await;
}

#[tokio::test]
async fn redis_key_and_command_handlers_validate_blank_input_before_network_access() {
    let _guard = REDIS_STORAGE_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;
    storage_support::save_connections(&[connection("one", "One", 10)])
        .await
        .expect("seed connections");

    let create_error = redis_connection_key_create(
        Path("one".to_string()),
        Json(GatewayRedisKeyCreateRequest { key: " ".to_string(), key_type: "String".to_string() }),
    )
    .await
    .expect_err("blank key");
    let create_type_error = redis_connection_key_create(
        Path("one".to_string()),
        Json(GatewayRedisKeyCreateRequest { key: "key".to_string(), key_type: " ".to_string() }),
    )
    .await
    .expect_err("blank type");
    let analyze_error = redis_connection_key_analyze(
        Path("one".to_string()),
        Json(GatewayRedisKeyAnalysisRequest { key: " ".to_string() }),
    )
    .await
    .expect_err("blank analyze key");
    let command_error = redis_command_execute(
        Path("one".to_string()),
        Json(vw_api_types::tool::GatewayRedisCommandRequest { command: " ".to_string() }),
    )
    .await
    .expect_err("blank command");

    assert_eq!(create_error.status, StatusCode::BAD_REQUEST);
    assert_eq!(create_type_error.status, StatusCode::BAD_REQUEST);
    assert_eq!(analyze_error.status, StatusCode::BAD_REQUEST);
    assert_eq!(command_error.status, StatusCode::BAD_REQUEST);
    snapshot.restore().await;
}

#[tokio::test]
async fn redis_history_list_filters_paginates_and_clamps_limit() {
    let _guard = REDIS_STORAGE_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;
    storage_support::save_history(&[
        history_record(Some("one"), "Local", "GET", "key=read", false, 3),
        history_record(Some("one"), "Local", "SET", "key=write", true, 2),
        history_record(Some("two"), "Remote", "DEL", "key=write", true, 1),
    ])
    .await
    .expect("seed history");

    let Json(page) = redis_history_list(Query(GatewayRedisHistoryListQuery {
        offset: Some(0),
        limit: Some(500),
        connection_id: Some(" one ".to_string()),
        query: Some("write".to_string()),
        only_write: Some(true),
    }))
    .await
    .expect("history page");

    assert_eq!(page.limit, storage_support::REDIS_HISTORY_PAGE_LIMIT_MAX);
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].command, "SET");
    assert!(!page.has_more);
    snapshot.restore().await;
}

#[tokio::test]
async fn redis_import_normalizes_connections_and_records_history() {
    let _guard = REDIS_STORAGE_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;
    let mut first = connection("same", " First ", 0);
    first.key_pattern = " ".to_string();
    let second = connection("same", "Second", 15);

    let Json(response) = redis_import(Json(GatewayRedisConfigBundle {
        schema_version: 0,
        default_load_count: 0,
        connections: vec![first, second],
    }))
    .await
    .expect("import");

    let settings = storage_support::load_settings().await;
    let connections = storage_support::load_connections().await;
    let history = storage_support::load_history().await;
    assert_eq!(response.imported_count, 2);
    assert_eq!(response.default_load_count, 1);
    assert_eq!(settings.schema_version, 1);
    assert_eq!(connections[0].key_pattern, "*");
    assert_ne!(connections[0].id, connections[1].id);
    assert_eq!(history[0].command, "IMPORT_CONFIG");
    snapshot.restore().await;
}

#[tokio::test]
async fn redis_export_sorts_connections_and_records_history() {
    let _guard = REDIS_STORAGE_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;
    let mut first = connection("first", "First", 10);
    first.last_used_ms = Some(20);
    let mut second = connection("second", "Second", 30);
    second.last_used_ms = Some(80);
    storage_support::save_settings(&GatewayRedisSettings {
        schema_version: 4,
        default_load_count: 77,
        selected_connection_id: Some("first".to_string()),
    })
    .await
    .expect("seed settings");
    storage_support::save_connections(&[first, second]).await.expect("seed connections");

    let Json(bundle) = redis_export().await.expect("export");

    let history = storage_support::load_history().await;
    assert_eq!(bundle.schema_version, 4);
    assert_eq!(bundle.default_load_count, 77);
    assert_eq!(bundle.connections[0].id, "second");
    assert_eq!(history[0].command, "EXPORT_CONFIG");
    assert!(!history[0].is_write);
    snapshot.restore().await;
}
