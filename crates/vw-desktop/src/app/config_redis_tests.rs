use super::*;
use vw_gateway_client::vw_api_types::tool::{
    GatewayRedisCommandResponse, GatewayRedisConnectionConfig, GatewayRedisHistoryPage,
    GatewayRedisHistoryRecord, GatewayRedisInfoEntry,
    GatewayRedisKeyAnalysis as GatewayRedisKeyAnalysisDto, GatewayRedisKeyPage,
    GatewayRedisKeyspaceStat, GatewayRedisRuntimeOverview, GatewayRedisSentinelConfig,
    GatewayRedisSettings, GatewayRedisSshTunnelConfig, GatewayRedisTlsCertConfig,
};

fn gateway_connection() -> GatewayRedisConnectionConfig {
    GatewayRedisConnectionConfig {
        id: "conn-1".to_string(),
        name: "Primary".to_string(),
        host: "127.0.0.1".to_string(),
        port: 6379,
        db: 2,
        username: "app".to_string(),
        password: "secret".to_string(),
        use_tls: true,
        tls_cert: GatewayRedisTlsCertConfig {
            private_key_path: "/tmp/client.key".to_string(),
            public_cert_path: "/tmp/client.crt".to_string(),
            ca_cert_path: "/tmp/ca.crt".to_string(),
        },
        ssh_tunnel: GatewayRedisSshTunnelConfig {
            enabled: true,
            host: "bastion.internal".to_string(),
            port: 2222,
            username: "ssh-user".to_string(),
            password: "ssh-secret".to_string(),
            private_key_path: "/tmp/id_ed25519".to_string(),
            passphrase: "phrase".to_string(),
            timeout_secs: 45,
        },
        sentinel: GatewayRedisSentinelConfig {
            enabled: true,
            master_name: "mymaster".to_string(),
            node_password: "node-secret".to_string(),
        },
        use_cluster: true,
        read_only: true,
        key_pattern: "app:*".to_string(),
        last_used_ms: Some(111),
        updated_at_ms: 222,
    }
}

fn gateway_history_record() -> GatewayRedisHistoryRecord {
    GatewayRedisHistoryRecord {
        time_ms: 333,
        connection_id: Some("conn-1".to_string()),
        connection_label: "Primary".to_string(),
        command: "SET".to_string(),
        args: "app:key <redacted>".to_string(),
        cost_ms: 12,
        is_write: true,
    }
}

#[test]
fn default_redis_history_query_uses_first_page_defaults() {
    let query = default_redis_history_query();

    assert_eq!(query.offset, Some(0));
    assert_eq!(query.limit, Some(REDIS_HISTORY_PAGE_SIZE));
    assert_eq!(query.connection_id, None);
    assert_eq!(query.query, None);
    assert_eq!(query.only_write, Some(false));
}

#[test]
fn redis_snapshot_from_gateway_maps_state_and_page_metadata() {
    let snapshot = redis_snapshot_from_gateway(
        GatewayRedisSettings {
            schema_version: 7,
            default_load_count: 250,
            selected_connection_id: Some("conn-1".to_string()),
        },
        vec![gateway_connection()],
        GatewayRedisHistoryPage {
            items: vec![gateway_history_record()],
            offset: 50,
            limit: 25,
            total: 75,
            has_more: true,
        },
    );

    assert_eq!(snapshot.persisted_state.schema_version, 7);
    assert_eq!(snapshot.persisted_state.default_load_count, 250);
    assert_eq!(snapshot.persisted_state.selected_connection_id.as_deref(), Some("conn-1"));
    assert_eq!(snapshot.persisted_state.connections.len(), 1);
    assert_eq!(snapshot.persisted_state.history.len(), 1);
    assert_eq!(snapshot.history_offset, 50);
    assert_eq!(snapshot.history_limit, 25);
    assert_eq!(snapshot.history_total, 75);
    assert!(snapshot.history_has_more);
}

#[test]
fn redis_connection_from_gateway_preserves_basic_and_advanced_fields() {
    let connection = redis_connection_from_gateway(gateway_connection());

    assert_eq!(connection.id, "conn-1");
    assert_eq!(connection.name, "Primary");
    assert_eq!(connection.host, "127.0.0.1");
    assert_eq!(connection.port, 6379);
    assert_eq!(connection.db, 2);
    assert_eq!(connection.username, "app");
    assert_eq!(connection.password, "secret");
    assert!(connection.use_tls);
    assert_eq!(connection.tls_cert.private_key_path, "/tmp/client.key");
    assert_eq!(connection.tls_cert.public_cert_path, "/tmp/client.crt");
    assert_eq!(connection.tls_cert.ca_cert_path, "/tmp/ca.crt");
    assert!(connection.ssh_tunnel.enabled);
    assert_eq!(connection.ssh_tunnel.host, "bastion.internal");
    assert_eq!(connection.ssh_tunnel.port, 2222);
    assert_eq!(connection.ssh_tunnel.username, "ssh-user");
    assert_eq!(connection.ssh_tunnel.password, "ssh-secret");
    assert_eq!(connection.ssh_tunnel.private_key_path, "/tmp/id_ed25519");
    assert_eq!(connection.ssh_tunnel.passphrase, "phrase");
    assert_eq!(connection.ssh_tunnel.timeout_secs, 45);
    assert!(connection.sentinel.enabled);
    assert_eq!(connection.sentinel.master_name, "mymaster");
    assert_eq!(connection.sentinel.node_password, "node-secret");
    assert!(connection.use_cluster);
    assert!(connection.read_only);
    assert_eq!(connection.key_pattern, "app:*");
    assert_eq!(connection.last_used_ms, Some(111));
    assert_eq!(connection.updated_at_ms, 222);
}

#[test]
fn redis_history_from_gateway_preserves_record_fields() {
    let record = redis_history_from_gateway(gateway_history_record());

    assert_eq!(record.time_ms, 333);
    assert_eq!(record.connection_id.as_deref(), Some("conn-1"));
    assert_eq!(record.connection_label, "Primary");
    assert_eq!(record.command, "SET");
    assert_eq!(record.args, "app:key <redacted>");
    assert_eq!(record.cost_ms, 12);
    assert!(record.is_write);
}

#[test]
fn redis_runtime_overview_from_gateway_maps_nested_stats_and_info() {
    let overview = redis_runtime_overview_from_gateway(GatewayRedisRuntimeOverview {
        connection_id: "conn-1".to_string(),
        connection_label: "Primary".to_string(),
        server_version: "7.2.4".to_string(),
        os: "Darwin".to_string(),
        process_id: "1234".to_string(),
        used_memory_human: "1M".to_string(),
        used_memory_peak_human: "2M".to_string(),
        used_memory_lua_human: "32K".to_string(),
        connected_clients: 3,
        total_connections_received: 4,
        total_commands_processed: 5,
        keyspace: vec![GatewayRedisKeyspaceStat {
            db: "db0".to_string(),
            keys: 10,
            expires: 2,
            avg_ttl: 600,
        }],
        info_entries: vec![GatewayRedisInfoEntry {
            key: "redis_version".to_string(),
            value: "7.2.4".to_string(),
        }],
    });

    assert_eq!(overview.connection_id, "conn-1");
    assert_eq!(overview.connection_label, "Primary");
    assert_eq!(overview.server_version, "7.2.4");
    assert_eq!(overview.os, "Darwin");
    assert_eq!(overview.process_id, "1234");
    assert_eq!(overview.used_memory_human, "1M");
    assert_eq!(overview.used_memory_peak_human, "2M");
    assert_eq!(overview.used_memory_lua_human, "32K");
    assert_eq!(overview.connected_clients, 3);
    assert_eq!(overview.total_connections_received, 4);
    assert_eq!(overview.total_commands_processed, 5);
    assert_eq!(overview.keyspace[0].db, "db0");
    assert_eq!(overview.keyspace[0].keys, 10);
    assert_eq!(overview.keyspace[0].expires, 2);
    assert_eq!(overview.keyspace[0].avg_ttl, 600);
    assert_eq!(overview.info_entries[0].key, "redis_version");
    assert_eq!(overview.info_entries[0].value, "7.2.4");
}

#[test]
fn redis_key_page_from_gateway_preserves_scan_page() {
    let page = redis_key_page_from_gateway(GatewayRedisKeyPage {
        connection_id: "conn-1".to_string(),
        pattern: "app:*".to_string(),
        keys: vec!["app:1".to_string(), "app:2".to_string()],
        next_cursor: 42,
        has_more: true,
    });

    assert_eq!(page.connection_id, "conn-1");
    assert_eq!(page.pattern, "app:*");
    assert_eq!(page.keys, vec!["app:1".to_string(), "app:2".to_string()]);
    assert_eq!(page.next_cursor, 42);
    assert!(page.has_more);
}

#[test]
fn redis_key_analysis_from_gateway_preserves_preview_and_optional_memory() {
    let analysis = redis_key_analysis_from_gateway(GatewayRedisKeyAnalysisDto {
        connection_id: "conn-1".to_string(),
        key: "app:1".to_string(),
        key_type: "String".to_string(),
        ttl_secs: -1,
        memory_usage_bytes: Some(128),
        preview_command: "GET app:1".to_string(),
        preview_output: "\"value\"".to_string(),
    });

    assert_eq!(analysis.connection_id, "conn-1");
    assert_eq!(analysis.key, "app:1");
    assert_eq!(analysis.key_type, "String");
    assert_eq!(analysis.ttl_secs, -1);
    assert_eq!(analysis.memory_usage_bytes, Some(128));
    assert_eq!(analysis.preview_command, "GET app:1");
    assert_eq!(analysis.preview_output, "\"value\"");
}

#[test]
fn redis_key_analysis_from_gateway_preserves_missing_memory_usage() {
    let analysis = redis_key_analysis_from_gateway(GatewayRedisKeyAnalysisDto {
        memory_usage_bytes: None,
        ..Default::default()
    });

    assert_eq!(analysis.memory_usage_bytes, None);
}

#[test]
fn redis_command_output_from_gateway_preserves_response_and_sets_timestamp() {
    let before = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let entry = redis_command_output_from_gateway(GatewayRedisCommandResponse {
        command: "PING".to_string(),
        output: "PONG".to_string(),
        cost_ms: 9,
        is_error: true,
    });

    let after = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    assert_eq!(entry.command, "PING");
    assert_eq!(entry.output, "PONG");
    assert_eq!(entry.cost_ms, 9);
    assert!(entry.is_error);
    assert!((before..=after).contains(&entry.time_ms));
}
