//! 配置 I/O 兼容性归一化测试。
//!
//! 这些测试覆盖旧配置别名到当前字段名的迁移行为，确保用户升级后不会因为历史键名而丢失 ACP 配置。

use super::config_io::{normalize_legacy_alias_conflicts, normalize_top_level_table_aliases};

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
