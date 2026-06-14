use std::collections::HashSet;

use super::*;
use axum::http::StatusCode;
use vw_api_types::tool::{GatewayRedisConnectionConfig, GatewayRedisConnectionUpsertBody};

fn body(value: serde_json::Value) -> GatewayRedisConnectionUpsertBody {
    serde_json::from_value(value).expect("body")
}

fn valid_body() -> GatewayRedisConnectionUpsertBody {
    body(serde_json::json!({
        "name": "  local  ",
        "host": "  127.0.0.1  ",
        "port": 6379,
        "db": 0,
        "username": " user@example.com ",
        "password": " p@ss word ",
        "key_pattern": " keys:* "
    }))
}

fn connection(value: serde_json::Value) -> GatewayRedisConnectionConfig {
    serde_json::from_value(value).expect("connection")
}

#[test]
fn normalize_upsert_body_trims_names_and_defaults_pattern() {
    let normalized = normalize_upsert_body(&body(serde_json::json!({
        "name": "  local  ",
        "host": "  127.0.0.1  ",
        "port": 6379,
        "db": 0,
        "key_pattern": "  "
    })))
    .expect("valid config");

    assert_eq!(normalized.name, "local");
    assert_eq!(normalized.host, "127.0.0.1");
    assert_eq!(normalized.key_pattern, "*");
}

#[test]
fn normalize_upsert_body_trims_nested_options_and_clamps_minimums() {
    let normalized = normalize_upsert_body(&body(serde_json::json!({
        "name": " tunnel ",
        "host": " redis.internal ",
        "port": 6379,
        "db": 0,
        "use_tls": true,
        "tls_cert": {
            "private_key_path": " /tmp/client.key ",
            "public_cert_path": " /tmp/client.crt ",
            "ca_path": " /tmp/ca.crt "
        },
        "ssh_tunnel": {
            "enabled": true,
            "host": " ssh.example.test ",
            "port": 0,
            "username": " deploy ",
            "password": " secret ",
            "private_key_path": " ~/.ssh/id_ed25519 ",
            "passphrase": " phrase ",
            "timeout_secs": 0
        },
        "sentinel": {
            "enabled": true,
            "master_name": " primary ",
            "node_password": " redis-secret "
        },
        "read_only": true,
        "key_pattern": " app:* "
    })))
    .expect("valid normalized body");

    assert_eq!(normalized.username, "");
    assert_eq!(normalized.password, "");
    assert_eq!(normalized.tls_cert.private_key_path, "/tmp/client.key");
    assert_eq!(normalized.tls_cert.public_cert_path, "/tmp/client.crt");
    assert_eq!(normalized.tls_cert.ca_cert_path, "/tmp/ca.crt");
    assert_eq!(normalized.ssh_tunnel.host, "ssh.example.test");
    assert_eq!(normalized.ssh_tunnel.port, 1);
    assert_eq!(normalized.ssh_tunnel.username, "deploy");
    assert_eq!(normalized.ssh_tunnel.timeout_secs, 1);
    assert_eq!(normalized.sentinel.master_name, "primary");
    assert_eq!(normalized.key_pattern, "app:*");
    assert!(normalized.read_only);
}

#[test]
fn normalize_upsert_body_rejects_required_and_conflicting_options() {
    let cases = [
        (
            serde_json::json!({"name": " ", "host": "127.0.0.1", "port": 6379, "db": 0}),
            "请输入连接名称",
        ),
        (
            serde_json::json!({"name": "local", "host": " ", "port": 6379, "db": 0}),
            "请输入 Redis 主机地址",
        ),
        (
            serde_json::json!({
                "name": "local",
                "host": "127.0.0.1",
                "port": 6379,
                "db": 0,
                "sentinel": {"enabled": true, "master_name": "mymaster"},
                "use_cluster": true
            }),
            "Sentinel 与 Cluster 不能同时启用",
        ),
        (
            serde_json::json!({
                "name": "local",
                "host": "127.0.0.1",
                "port": 6379,
                "db": 1,
                "use_cluster": true
            }),
            "Cluster 模式仅支持 DB 0",
        ),
        (
            serde_json::json!({
                "name": "local",
                "host": "127.0.0.1",
                "port": 6379,
                "db": 0,
                "ssh_tunnel": {"enabled": true, "username": "deploy"}
            }),
            "启用 SSH 时必须填写 SSH 地址",
        ),
        (
            serde_json::json!({
                "name": "local",
                "host": "127.0.0.1",
                "port": 6379,
                "db": 0,
                "ssh_tunnel": {"enabled": true, "host": "ssh.example.test", "username": " "}
            }),
            "启用 SSH 时必须填写 SSH 用户名",
        ),
        (
            serde_json::json!({
                "name": "local",
                "host": "127.0.0.1",
                "port": 6379,
                "db": 0,
                "sentinel": {"enabled": true, "master_name": " "}
            }),
            "启用 Sentinel 时必须填写 Master 组名称",
        ),
    ];

    for (input, message) in cases {
        let error = normalize_upsert_body(&body(input)).expect_err("invalid config");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.to_string(), message);
    }
}

#[test]
fn normalize_upsert_body_rejects_tls_material_without_tls() {
    let error = normalize_upsert_body(&body(serde_json::json!({
        "name": "local",
        "host": "127.0.0.1",
        "port": 6379,
        "db": 0,
        "use_tls": false,
        "tls_cert": {"ca_path": "/tmp/ca.pem"}
    })))
    .expect_err("tls material requires tls");

    assert_eq!(error.status, axum::http::StatusCode::BAD_REQUEST);
}

#[test]
fn new_and_updated_connection_from_upsert_normalize_and_preserve_identity_fields() {
    let created = new_connection_from_upsert(&valid_body(), 1234).expect("connection should build");
    assert!(created.id.starts_with("redis-1234-"));
    assert_eq!(created.name, "local");
    assert_eq!(created.host, "127.0.0.1");
    assert_eq!(created.username, "user@example.com");
    assert_eq!(created.password, "p@ss word");
    assert_eq!(created.key_pattern, "keys:*");
    assert_eq!(created.last_used_ms, Some(1234));
    assert_eq!(created.updated_at_ms, 1234);

    let mut existing = created.clone();
    existing.id = "redis-existing".to_string();
    existing.last_used_ms = Some(99);
    let updated =
        updated_connection_from_upsert(&existing, &valid_body(), 5678).expect("updated config");
    assert_eq!(updated.id, "redis-existing");
    assert_eq!(updated.last_used_ms, Some(99));
    assert_eq!(updated.updated_at_ms, 5678);
}

#[test]
fn normalize_import_connection_keeps_unique_id_and_fills_missing_times() {
    let mut seen = HashSet::new();
    let imported = normalize_import_connection(
        connection(serde_json::json!({
            "id": " redis-imported ",
            "name": " Imported ",
            "host": " redis.local ",
            "port": 6379,
            "db": 0,
            "updated_at_ms": 0,
            "last_used_ms": null
        })),
        42,
        &mut seen,
    )
    .expect("import should normalize");

    assert_eq!(imported.id, "redis-imported");
    assert_eq!(imported.name, "Imported");
    assert_eq!(imported.host, "redis.local");
    assert_eq!(imported.last_used_ms, Some(42));
    assert_eq!(imported.updated_at_ms, 42);
    assert!(seen.contains("redis-imported"));
}

#[test]
fn normalize_import_connection_regenerates_blank_and_duplicate_ids() {
    let mut seen = HashSet::from(["redis-existing".to_string()]);
    let duplicate = normalize_import_connection(
        connection(serde_json::json!({
            "id": " redis-existing ",
            "name": "Duplicate",
            "host": "127.0.0.1",
            "port": 6379,
            "db": 0,
            "updated_at_ms": 7,
            "last_used_ms": 6
        })),
        99,
        &mut seen,
    )
    .expect("duplicate id should regenerate");
    let blank = normalize_import_connection(
        connection(serde_json::json!({
            "id": " ",
            "name": "Blank",
            "host": "127.0.0.1",
            "port": 6379,
            "db": 0,
            "updated_at_ms": 8
        })),
        99,
        &mut seen,
    )
    .expect("blank id should regenerate");

    assert_ne!(duplicate.id, "redis-existing");
    assert!(duplicate.id.starts_with("redis-99-"));
    assert!(blank.id.starts_with("redis-99-"));
    assert_ne!(duplicate.id, blank.id);
    assert_eq!(duplicate.last_used_ms, Some(6));
    assert_eq!(duplicate.updated_at_ms, 7);
}

#[test]
fn build_connection_uri_encodes_supported_auth_shapes() {
    let mut config = connection(serde_json::json!({
        "id": "redis-1",
        "name": "Local",
        "host": "redis.local",
        "port": 6380,
        "db": 2,
        "username": "user name",
        "password": "p@ss word",
        "use_tls": true
    }));
    assert_eq!(
        build_connection_uri(&config).expect("uri"),
        "rediss://user%20name:p%40ss%20word@redis.local:6380/2"
    );

    config.use_tls = false;
    config.username.clear();
    assert_eq!(
        build_connection_uri(&config).expect("password only uri"),
        "redis://:p%40ss%20word@redis.local:6380/2"
    );

    config.username = "user name".to_string();
    config.password.clear();
    assert_eq!(
        build_connection_uri(&config).expect("username only uri"),
        "redis://user%20name@redis.local:6380/2"
    );

    config.username.clear();
    assert_eq!(build_connection_uri(&config).expect("no auth uri"), "redis://redis.local:6380/2");

    config.host = " ".to_string();
    let error = build_connection_uri(&config).expect_err("blank host");
    assert_eq!(error.status, StatusCode::BAD_REQUEST);
}

#[test]
fn load_tls_certificates_from_paths_loads_optional_material_and_rejects_partial_client_pair() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cert_path = temp.path().join("client.crt");
    let key_path = temp.path().join("client.key");
    let ca_path = temp.path().join("ca.crt");
    std::fs::write(&cert_path, b"cert").expect("write cert");
    std::fs::write(&key_path, b"key").expect("write key");
    std::fs::write(&ca_path, b"ca").expect("write ca");

    let loaded = load_tls_certificates_from_paths(&GatewayRedisTlsCertConfig {
        public_cert_path: cert_path.to_string_lossy().to_string(),
        private_key_path: key_path.to_string_lossy().to_string(),
        ca_cert_path: ca_path.to_string_lossy().to_string(),
    })
    .expect("tls material should load");

    let client = loaded.client_tls.expect("client tls");
    assert_eq!(client.client_cert, b"cert");
    assert_eq!(client.client_key, b"key");
    assert_eq!(loaded.root_cert.as_deref(), Some(&b"ca"[..]));

    let partial_result = load_tls_certificates_from_paths(&GatewayRedisTlsCertConfig {
        public_cert_path: cert_path.to_string_lossy().to_string(),
        private_key_path: String::new(),
        ca_cert_path: String::new(),
    });
    let partial = match partial_result {
        Ok(_) => panic!("partial client material should fail"),
        Err(error) => error,
    };
    assert_eq!(partial, "客户端证书和私钥必须同时提供，或同时留空");

    let missing_result = load_tls_certificates_from_paths(&GatewayRedisTlsCertConfig {
        public_cert_path: temp.path().join("missing.crt").to_string_lossy().to_string(),
        private_key_path: key_path.to_string_lossy().to_string(),
        ca_cert_path: String::new(),
    });
    let missing = match missing_result {
        Ok(_) => panic!("missing file should fail"),
        Err(error) => error,
    };
    assert!(missing.starts_with("读取客户端证书失败:"));
}

#[test]
fn has_custom_tls_material_detects_trimmed_paths() {
    assert!(!has_custom_tls_material(&GatewayRedisTlsCertConfig::default()));
    assert!(has_custom_tls_material(&GatewayRedisTlsCertConfig {
        ca_cert_path: " /tmp/ca.crt ".to_string(),
        ..GatewayRedisTlsCertConfig::default()
    }));
}
