use serde_json::json;
use tokio::sync::Mutex;
use vw_api_types::data::{
    AiDataConnectionDto, AiDataConnectionKind, AiDataReportDto, AiDataSettings, AiDataSourceMode,
};

use super::*;
use crate::app::agent::storage;

static AI_DATA_STORAGE_LOCK: Mutex<()> = Mutex::const_new(());

struct AiDataStorageSnapshot {
    settings: Option<AiDataSettings>,
    connections: Option<Vec<AiDataConnectionDto>>,
    reports: Option<Vec<AiDataReportDto>>,
}

impl AiDataStorageSnapshot {
    async fn capture() -> Self {
        Self {
            settings: storage::read(&["ai_data", "settings"]).await.ok(),
            connections: storage::read(&["ai_data", "connections"]).await.ok(),
            reports: storage::read(&["ai_data", "reports"]).await.ok(),
        }
    }

    async fn restore(self) {
        restore_key(&["ai_data", "settings"], self.settings).await;
        restore_key(&["ai_data", "connections"], self.connections).await;
        restore_key(&["ai_data", "reports"], self.reports).await;
    }
}

async fn restore_key<T: serde::Serialize>(key: &[&str], value: Option<T>) {
    match value {
        Some(value) => storage::write(key, &value).await.expect("restore ai-data storage key"),
        None => storage::remove(key).await.expect("remove ai-data storage key"),
    }
}

async fn clear_ai_data_storage() {
    storage::remove(&["ai_data", "settings"]).await.expect("clear ai-data settings");
    storage::remove(&["ai_data", "connections"]).await.expect("clear ai-data connections");
    storage::remove(&["ai_data", "reports"]).await.expect("clear ai-data reports");
}

fn connection(
    id: &str,
    name: &str,
    updated_at_ms: u64,
    last_used_ms: Option<u64>,
) -> AiDataConnectionDto {
    serde_json::from_value(json!({
        "id": id,
        "name": name,
        "kind": "sqlite",
        "sqlite_path": "/tmp/data.sqlite",
        "updated_at_ms": updated_at_ms,
        "last_used_ms": last_used_ms
    }))
    .expect("connection")
}

fn report(id: &str, name: &str, updated_at_ms: u64) -> AiDataReportDto {
    AiDataReportDto {
        id: id.to_string(),
        name: name.to_string(),
        slug: id.to_string(),
        data_source: AiDataSourceMode::Normal,
        default_source_key: None,
        report_config: json!({ "columns": [] }),
        sources: Vec::new(),
        updated_at_ms,
    }
}

#[tokio::test]
async fn load_helpers_return_defaults_when_storage_is_empty() {
    let _guard = AI_DATA_STORAGE_LOCK.lock().await;
    let snapshot = AiDataStorageSnapshot::capture().await;
    clear_ai_data_storage().await;

    assert_eq!(load_settings().await, AiDataSettings::default());
    assert!(load_connections().await.is_empty());
    assert!(load_reports().await.is_empty());

    snapshot.restore().await;
}

#[tokio::test]
async fn save_helpers_persist_round_trippable_values() {
    let _guard = AI_DATA_STORAGE_LOCK.lock().await;
    let snapshot = AiDataStorageSnapshot::capture().await;
    clear_ai_data_storage().await;

    let settings = AiDataSettings {
        default_limit: 250,
        default_timeout_secs: 12,
        selected_connection_id: Some("conn-a".to_string()),
        ..AiDataSettings::default()
    };
    let connections = vec![
        connection("conn-a", "alpha", 10, Some(30)),
        AiDataConnectionDto {
            id: "conn-b".to_string(),
            name: "beta".to_string(),
            kind: AiDataConnectionKind::Http,
            description: Some("HTTP source".to_string()),
            enabled: true,
            read_only: false,
            base_url: Some("https://example.test".to_string()),
            connection_url: None,
            sqlite_path: None,
            default_path: Some("/api".to_string()),
            auth_token: None,
            headers: Default::default(),
            schema_hint: None,
            updated_at_ms: 20,
            last_used_ms: None,
        },
    ];
    let reports = vec![report("report-a", "Revenue", 100), report("report-b", "Costs", 50)];

    save_settings(&settings).await.expect("settings should save");
    save_connections(&connections).await.expect("connections should save");
    save_reports(&reports).await.expect("reports should save");

    assert_eq!(load_settings().await, settings);
    assert_eq!(load_connections().await, connections);
    assert_eq!(load_reports().await, reports);

    snapshot.restore().await;
}

#[test]
fn sort_connections_prefers_last_used_then_updated_at_then_name() {
    let mut connections = vec![
        connection("old", "zeta", 10, None),
        connection("tie-b", "bravo", 20, Some(80)),
        connection("tie-a", "alpha", 20, Some(80)),
        connection("updated", "omega", 70, None),
    ];

    sort_connections(&mut connections);

    let ids = connections.into_iter().map(|item| item.id).collect::<Vec<_>>();
    assert_eq!(ids, vec!["tie-a", "tie-b", "updated", "old"]);
}

#[test]
fn sort_reports_prefers_recent_update_then_name() {
    let mut reports =
        vec![report("old", "zeta", 10), report("tie-b", "bravo", 20), report("tie-a", "alpha", 20)];

    sort_reports(&mut reports);

    let ids = reports.into_iter().map(|item| item.id).collect::<Vec<_>>();
    assert_eq!(ids, vec!["tie-a", "tie-b", "old"]);
}

#[test]
fn now_ms_returns_epoch_milliseconds() {
    assert!(now_ms() > 1_000_000_000_000);
}
