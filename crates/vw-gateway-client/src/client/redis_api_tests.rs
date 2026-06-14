use serde_json::json;
use vw_api_types::tool::{
    GatewayRedisCommandRequest, GatewayRedisConfigBundle, GatewayRedisConnectionUpsertBody,
    GatewayRedisHistoryListQuery, GatewayRedisKeyAnalysisRequest, GatewayRedisKeyCreateRequest,
    GatewayRedisKeyListQuery, GatewayRedisSettingsUpdateBody,
};

use crate::client::test_support;

fn connection_json(id: &str) -> serde_json::Value {
    json!({
        "id": id,
        "name": "Local",
        "host": "127.0.0.1",
        "port": 6379,
        "db": 0,
        "updated_at_ms": 42
    })
}

fn upsert_body() -> GatewayRedisConnectionUpsertBody {
    GatewayRedisConnectionUpsertBody {
        name: "Local".to_string(),
        host: "127.0.0.1".to_string(),
        port: 6379,
        db: 0,
        username: "default".to_string(),
        password: "secret".to_string(),
        use_tls: false,
        tls_cert: Default::default(),
        ssh_tunnel: Default::default(),
        sentinel: Default::default(),
        use_cluster: false,
        read_only: true,
        key_pattern: "cache:*".to_string(),
    }
}

#[tokio::test]
async fn redis_api_routes_settings_connections_keys_history_and_import_export() {
    let server = test_support::server(vec![
        (
            200,
            json!({"schema_version": 1, "default_load_count": 100, "selected_connection_id": "r1"}),
        ),
        (
            200,
            json!({"schema_version": 1, "default_load_count": 200, "selected_connection_id": null}),
        ),
        (200, json!([connection_json("r1")])),
        (200, connection_json("r1")),
        (200, connection_json("r2")),
        (200, connection_json("r1")),
        (200, json!({"deleted_id": "r1"})),
        (
            200,
            json!({"schema_version": 1, "default_load_count": 200, "selected_connection_id": "r2"}),
        ),
        (200, json!({"ok": true, "message": "PONG", "latency_ms": 3})),
        (200, json!({"connection_id": "r2", "connection_label": "Local", "server_version": "7.2"})),
        (
            200,
            json!({"connection_id": "r2", "pattern": "cache:*", "keys": ["cache:a"], "next_cursor": 8, "has_more": true}),
        ),
        (
            200,
            json!({"connection_id": "r2", "pattern": "*", "keys": [], "next_cursor": 0, "has_more": false}),
        ),
        (
            200,
            json!({"connection_id": "r2", "key": "cache:a", "key_type": "String", "ttl_secs": -1, "preview_output": "value"}),
        ),
        (200, json!({"connection_id": "r2", "key": "cache:b", "key_type": "Hash", "ttl_secs": -1})),
        (200, json!({"command": "PING", "output": "PONG", "cost_ms": 1, "is_error": false})),
        (200, json!({"items": [], "offset": 10, "limit": 5, "total": 0, "has_more": false})),
        (200, json!({"items": [], "offset": 0, "limit": 50, "total": 0, "has_more": false})),
        (
            200,
            json!({"schema_version": 1, "default_load_count": 200, "connections": [connection_json("r2")]}),
        ),
        (
            200,
            json!({"imported_count": 1, "default_load_count": 200, "selected_connection_id": "r2"}),
        ),
    ]);
    let upsert = upsert_body();

    assert_eq!(
        server.client().redis_settings_get().await.expect("settings").default_load_count,
        100
    );
    assert_eq!(
        server
            .client()
            .redis_settings_put(&GatewayRedisSettingsUpdateBody { default_load_count: 200 })
            .await
            .expect("settings put")
            .default_load_count,
        200
    );
    assert_eq!(server.client().redis_connections_list().await.expect("connections")[0].id, "r1");
    assert_eq!(server.client().redis_connection_get("r1").await.expect("get").host, "127.0.0.1");
    assert_eq!(server.client().redis_connection_create(&upsert).await.expect("create").id, "r2");
    assert!(
        !server.client().redis_connection_update("r1", &upsert).await.expect("update").read_only
    );
    assert_eq!(
        server.client().redis_connection_delete("r1").await.expect("delete").deleted_id,
        "r1"
    );
    assert_eq!(
        server
            .client()
            .redis_connection_activate("r2")
            .await
            .expect("activate")
            .selected_connection_id,
        Some("r2".to_string())
    );
    assert!(server.client().redis_connection_test("r2").await.expect("test").ok);
    assert_eq!(
        server.client().redis_connection_overview("r2").await.expect("overview").server_version,
        "7.2"
    );
    assert_eq!(
        server
            .client()
            .redis_connection_keys(
                "r2",
                &GatewayRedisKeyListQuery {
                    cursor: Some(7),
                    count: Some(25),
                    pattern: Some("cache:*".to_string()),
                },
            )
            .await
            .expect("keys")
            .keys,
        vec!["cache:a".to_string()]
    );
    assert!(
        server
            .client()
            .redis_connection_keys(
                "r2",
                &GatewayRedisKeyListQuery {
                    cursor: None,
                    count: None,
                    pattern: Some("   ".to_string()),
                },
            )
            .await
            .expect("keys without params")
            .keys
            .is_empty()
    );
    assert_eq!(
        server
            .client()
            .redis_connection_key_analyze(
                "r2",
                &GatewayRedisKeyAnalysisRequest { key: "cache:a".to_string() },
            )
            .await
            .expect("analyze")
            .preview_output,
        "value"
    );
    assert_eq!(
        server
            .client()
            .redis_connection_key_create(
                "r2",
                &GatewayRedisKeyCreateRequest {
                    key: "cache:b".to_string(),
                    key_type: "Hash".to_string(),
                },
            )
            .await
            .expect("create key")
            .key_type,
        "Hash"
    );
    assert_eq!(
        server
            .client()
            .redis_command_execute(
                "r2",
                &GatewayRedisCommandRequest { command: "PING".to_string() }
            )
            .await
            .expect("command")
            .output,
        "PONG"
    );
    assert_eq!(
        server
            .client()
            .redis_history_list(&GatewayRedisHistoryListQuery {
                offset: Some(10),
                limit: Some(5),
                connection_id: Some("r2".to_string()),
                query: Some("PING".to_string()),
                only_write: Some(false),
            })
            .await
            .expect("history")
            .offset,
        10
    );
    assert_eq!(
        server
            .client()
            .redis_history_list(&GatewayRedisHistoryListQuery {
                offset: None,
                limit: None,
                connection_id: Some(" ".to_string()),
                query: Some("\t".to_string()),
                only_write: None,
            })
            .await
            .expect("empty history")
            .limit,
        50
    );
    let bundle = server.client().redis_export().await.expect("export");
    assert_eq!(bundle.connections[0].id, "r2");
    assert_eq!(
        server
            .client()
            .redis_import(&GatewayRedisConfigBundle::default())
            .await
            .expect("import")
            .imported_count,
        1
    );

    assert_eq!(server.take_request().path, "/v1/redis/settings");
    let request = server.take_request();
    assert_eq!(request.method, "PUT");
    assert_eq!(request.path, "/v1/redis/settings");
    assert_eq!(request.body["default_load_count"], 200);
    assert_eq!(server.take_request().path, "/v1/redis/connections");
    assert_eq!(server.take_request().path, "/v1/redis/connections/r1");
    let request = server.take_request();
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/v1/redis/connections");
    assert_eq!(request.body["read_only"], true);
    let request = server.take_request();
    assert_eq!(request.method, "PUT");
    assert_eq!(request.path, "/v1/redis/connections/r1");
    assert_eq!(request.body["key_pattern"], "cache:*");
    let request = server.take_request();
    assert_eq!(request.method, "DELETE");
    assert_eq!(request.path, "/v1/redis/connections/r1");
    assert_eq!(request.body, json!({}));
    assert_eq!(server.take_request().path, "/v1/redis/connections/r2/activate");
    assert_eq!(server.take_request().path, "/v1/redis/connections/r2/test");
    assert_eq!(server.take_request().path, "/v1/redis/connections/r2/overview");
    assert_eq!(
        server.take_request().path,
        "/v1/redis/connections/r2/keys?cursor=7&count=25&pattern=cache%3A*"
    );
    assert_eq!(server.take_request().path, "/v1/redis/connections/r2/keys");
    let request = server.take_request();
    assert_eq!(request.path, "/v1/redis/connections/r2/keys/analyze");
    assert_eq!(request.body["key"], "cache:a");
    let request = server.take_request();
    assert_eq!(request.path, "/v1/redis/connections/r2/keys");
    assert_eq!(request.body["key_type"], "Hash");
    let request = server.take_request();
    assert_eq!(request.path, "/v1/redis/connections/r2/command");
    assert_eq!(request.body["command"], "PING");
    assert_eq!(
        server.take_request().path,
        "/v1/redis/history?offset=10&limit=5&connection_id=r2&query=PING&only_write=false"
    );
    assert_eq!(server.take_request().path, "/v1/redis/history");
    assert_eq!(server.take_request().path, "/v1/redis/export");
    let request = server.take_request();
    assert_eq!(request.path, "/v1/redis/import");
    assert_eq!(request.body["connections"], json!([]));
    server.join();
}
