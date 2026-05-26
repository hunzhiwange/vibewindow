use super::*;
use vw_api_types::tool::GatewayRedisConnectionConfig;

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
