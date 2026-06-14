use super::*;
use crate::app::agent::storage;
use tokio::sync::Mutex;
use vw_api_types::tool::{
    GatewayRedisConnectionConfig, GatewayRedisHistoryRecord, GatewayRedisSettings,
};

static REDIS_STORAGE_SUPPORT_LOCK: Mutex<()> = Mutex::const_new(());

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

fn connection(
    id: &str,
    name: &str,
    updated_at_ms: u64,
    last_used_ms: Option<u64>,
) -> GatewayRedisConnectionConfig {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "name": name,
        "host": "127.0.0.1",
        "port": 6379,
        "db": 0,
        "updated_at_ms": updated_at_ms,
        "last_used_ms": last_used_ms
    }))
    .expect("connection")
}

fn history(command: &str, time_ms: u64) -> GatewayRedisHistoryRecord {
    GatewayRedisHistoryRecord {
        time_ms,
        connection_id: Some("redis-a".to_string()),
        connection_label: "Local".to_string(),
        command: command.to_string(),
        args: "args".to_string(),
        cost_ms: 3,
        is_write: false,
    }
}

#[tokio::test]
async fn load_helpers_return_defaults_when_storage_is_empty() {
    let _guard = REDIS_STORAGE_SUPPORT_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;

    assert_eq!(load_settings().await, GatewayRedisSettings::default());
    assert!(load_connections().await.is_empty());
    assert!(load_history().await.is_empty());

    snapshot.restore().await;
}

#[tokio::test]
async fn save_helpers_persist_round_trippable_values() {
    let _guard = REDIS_STORAGE_SUPPORT_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;

    let settings = GatewayRedisSettings {
        selected_connection_id: Some("redis-a".to_string()),
        default_load_count: 25,
        ..GatewayRedisSettings::default()
    };
    let connections = vec![connection("redis-a", "Alpha", 10, Some(30))];
    let records = vec![history("GET", 100), history("SET", 101)];

    save_settings(&settings).await.expect("settings should save");
    save_connections(&connections).await.expect("connections should save");
    save_history(&records).await.expect("history should save");

    assert_eq!(load_settings().await, settings);
    assert_eq!(load_connections().await, connections);
    assert_eq!(load_history().await, records);

    snapshot.restore().await;
}

#[tokio::test]
async fn append_history_best_effort_prepends_and_truncates_to_limit() {
    let _guard = REDIS_STORAGE_SUPPORT_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;
    let existing = (0..REDIS_HISTORY_LIMIT)
        .map(|index| history(&format!("CMD_{index}"), index as u64))
        .collect::<Vec<_>>();
    save_history(&existing).await.expect("seed history");

    append_history_best_effort(history("NEWEST", 999)).await;

    let records = load_history().await;
    assert_eq!(records.len(), REDIS_HISTORY_LIMIT);
    assert_eq!(records.first().map(|record| record.command.as_str()), Some("NEWEST"));
    assert_eq!(records.last().map(|record| record.command.as_str()), Some("CMD_198"));

    snapshot.restore().await;
}

#[tokio::test]
async fn load_connection_by_id_returns_match_or_not_found() {
    let _guard = REDIS_STORAGE_SUPPORT_LOCK.lock().await;
    let snapshot = RedisStorageSnapshot::capture().await;
    clear_redis_storage().await;
    save_connections(&[
        connection("redis-a", "Alpha", 10, None),
        connection("redis-b", "Beta", 20, None),
    ])
    .await
    .expect("seed connections");

    let found = load_connection_by_id("redis-b").await.expect("connection should exist");
    let missing = load_connection_by_id("missing").await.expect_err("missing connection");

    assert_eq!(found.name, "Beta");
    assert_eq!(missing.status, axum::http::StatusCode::NOT_FOUND);

    snapshot.restore().await;
}

#[test]
fn sort_connections_prefers_recent_use_then_name() {
    let mut connections = vec![
        connection("a", "zeta", 10, None),
        connection("b", "alpha", 20, Some(30)),
        connection("c", "beta", 20, Some(30)),
    ];

    sort_connections(&mut connections);

    let ids = connections.into_iter().map(|item| item.id).collect::<Vec<_>>();
    assert_eq!(ids, vec!["b", "c", "a"]);
}

#[test]
fn history_record_uses_global_label_without_connection() {
    let record = history_record(None, "UPDATE", "args".to_string(), 7, true);

    assert_eq!(record.connection_id, None);
    assert_eq!(record.connection_label, "全局配置");
    assert_eq!(record.command, "UPDATE");
    assert!(record.is_write);
}

#[test]
fn history_record_uses_connection_context_when_present() {
    let connection = connection("redis-a", "Alpha", 10, None);
    let record = history_record(Some(&connection), "GET", "key".to_string(), 4, false);

    assert_eq!(record.connection_id.as_deref(), Some("redis-a"));
    assert_eq!(record.connection_label, "Alpha");
    assert_eq!(record.command, "GET");
    assert_eq!(record.args, "key");
    assert_eq!(record.cost_ms, 4);
    assert!(!record.is_write);
    assert!(record.time_ms > 1_000_000_000_000);
}

#[test]
fn compact_connection_args_reports_connection_modes_without_secrets() {
    let mut connection = connection("redis-a", "Alpha", 10, None);
    connection.password = "secret".to_string();
    connection.use_tls = true;
    connection.ssh_tunnel.enabled = true;
    connection.sentinel.enabled = true;
    connection.use_cluster = true;
    connection.read_only = true;
    connection.key_pattern = "app:*".to_string();

    let args = compact_connection_args(&connection);

    assert_eq!(args, "127.0.0.1:6379 db=0 pattern=app:* mode=tls+ssh+sentinel+cluster+readonly");
    assert!(!args.contains("secret"));
}

#[test]
fn compact_connection_args_uses_direct_mode_by_default() {
    let args = compact_connection_args(&connection("redis-a", "Alpha", 10, None));

    assert_eq!(args, "127.0.0.1:6379 db=0 pattern=* mode=direct");
}

#[test]
fn now_ms_returns_epoch_milliseconds() {
    assert!(now_ms() > 1_000_000_000_000);
}
