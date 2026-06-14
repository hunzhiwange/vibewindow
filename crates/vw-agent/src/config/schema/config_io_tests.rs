//! 配置 I/O 兼容性归一化测试。
//!
//! 这些测试覆盖旧配置别名到当前字段名的迁移行为，确保用户升级后不会因为历史键名而丢失 ACP 配置。

use super::config_io::{
    extract_config_payload, normalize_legacy_alias_conflicts, normalize_top_level_table_aliases,
    read_config_root_json, upsert_config_payload,
};

#[test]
/// JSON payload 缺少 `acp` 时，应将旧 `acp2` 字段迁移过去。
fn normalize_json_acp2_moves_to_acp_when_acp_missing() {
    let mut payload = serde_json::json!({
        "acp2": {
            "demo": {
                "command": "codex-acp"
            }
        }
    });

    normalize_legacy_alias_conflicts(&mut payload);

    assert!(payload.get("acp2").is_none());
    assert_eq!(payload["acp"]["demo"]["command"], "codex-acp");
}

#[test]
/// JSON payload 同时存在新旧字段时，应丢弃旧 `acp2`，避免覆盖新配置。
fn normalize_json_acp2_is_dropped_when_acp_exists() {
    let mut payload = serde_json::json!({
        "acp": {},
        "acp2": {
            "demo": {
                "command": "codex-acp"
            }
        }
    });

    normalize_legacy_alias_conflicts(&mut payload);

    assert!(payload.get("acp2").is_none());
    assert_eq!(payload["acp"], serde_json::json!({}));
}

#[test]
/// TOML 顶层 `[acp2]` 表在没有 `[acp]` 时应迁移为 `[acp]`。
fn normalize_toml_acp2_moves_to_acp_when_acp_missing() {
    let mut raw = toml::from_str::<toml::Value>(
        r#"
        [acp2.demo]
        command = "codex-acp"
        "#,
    )
    .expect("valid toml");

    normalize_top_level_table_aliases(&mut raw);

    let root = raw.as_table().expect("root table");
    assert!(!root.contains_key("acp2"));
    assert_eq!(root["acp"]["demo"]["command"].as_str(), Some("codex-acp"));
}

#[test]
fn normalize_json_model_aliases_prefer_current_keys() {
    let mut payload = serde_json::json!({
        "default_model": "new",
        "model": "old",
        "default_provider": "provider-new",
        "model_provider": "provider-old"
    });

    normalize_legacy_alias_conflicts(&mut payload);

    assert!(payload.get("model").is_none());
    assert!(payload.get("model_provider").is_none());
    assert_eq!(payload["default_model"], "new");
}

#[test]
fn upsert_config_payload_removes_legacy_keys_and_merges_payload() {
    let mut root = serde_json::json!({
        "agent": {"legacy": true},
        "model": "old",
        "model_provider": "old-provider",
        "retained": true
    });

    upsert_config_payload(
        &mut root,
        serde_json::json!({"default_model": "new", "default_provider": "new-provider"}),
    );

    assert!(root.get("agent").is_none());
    assert!(root.get("model").is_none());
    assert!(root.get("model_provider").is_none());
    assert_eq!(root["retained"], true);
    assert_eq!(root["default_model"], "new");
}

#[test]
fn extract_config_payload_treats_empty_or_non_object_roots_as_absent() {
    assert!(extract_config_payload(&serde_json::json!({})).is_none());
    assert!(extract_config_payload(&serde_json::json!(null)).is_none());
    assert_eq!(
        extract_config_payload(&serde_json::json!({"default_model": "m"})).unwrap()["default_model"],
        "m"
    );
}

#[tokio::test]
async fn read_config_root_json_handles_missing_empty_and_invalid_files() {
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("missing.json");
    assert_eq!(read_config_root_json(&missing).await.unwrap(), serde_json::json!({}));

    let empty = tmp.path().join("empty.json");
    tokio::fs::write(&empty, "   ").await.unwrap();
    assert_eq!(read_config_root_json(&empty).await.unwrap(), serde_json::json!({}));

    let invalid = tmp.path().join("invalid.json");
    tokio::fs::write(&invalid, "{").await.unwrap();
    assert!(read_config_root_json(&invalid).await.unwrap_err().to_string().contains("parse"));
}
