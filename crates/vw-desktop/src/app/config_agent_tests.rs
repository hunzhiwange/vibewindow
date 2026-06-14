// Tests for plan6 task 840.
use std::collections::HashMap;

use serde_json::json;
use vw_config_types::{agent::DelegateAgentConfig, config::AcpAgentConfig};

use super::{
    build_acp_agent_enabled_patch, build_main_agent_overrides_patch, default_enabled_acp_config,
    resolve_enabled_acp_config,
};

const SOURCE: &str = include_str!("config_agent.rs");

fn source_declares_symbol(name: &str) -> bool {
    let needles = [
        format!("fn {name}"),
        format!("pub fn {name}"),
        format!("struct {name}"),
        format!("pub struct {name}"),
        format!("enum {name}"),
        format!("pub enum {name}"),
        format!("type {name}"),
        format!("pub type {name}"),
        format!("const {name}"),
        format!("pub const {name}"),
        format!("static {name}"),
        format!("pub static {name}"),
        format!("impl {name}"),
    ];

    needles.iter().any(|needle| SOURCE.contains(needle))
}

#[test]
fn config_agent_tests_keeps_planned_coverage_targets() {
    for name in [
        "fetch_agent_config_via_gateway",
        "fetch_global_agent_config_via_gateway",
        "fetch_global_acp_config_via_gateway",
        "patch_agent_config_via_gateway",
        "load_agent_config_via_gateway",
        "patch_agent_config",
        "load_full_agent_config_async",
        "load_browser_config_async",
        "load_gateway_config_result",
        "load_global_acp_config_result",
        "load_enabled_acp_config_result",
        "load_enabled_acp_config_async",
        "load_acp_settings_snapshot_async",
        "set_global_acp_agent_enabled_async",
        "build_acp_agent_enabled_patch",
        "build_main_agent_overrides_patch",
        "DEFAULT_ENABLED_ACP_AGENTS",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}

fn acp(command: &str) -> AcpAgentConfig {
    AcpAgentConfig { command: command.to_string(), ..AcpAgentConfig::default() }
}

fn catalog() -> HashMap<String, AcpAgentConfig> {
    HashMap::from([
        ("claude".to_string(), acp("claude")),
        ("gemini".to_string(), acp("gemini")),
        ("opencode".to_string(), acp("opencode")),
        ("codex".to_string(), acp("codex")),
        ("openclaw".to_string(), acp("openclaw")),
        ("copilot".to_string(), acp("copilot")),
        ("custom".to_string(), acp("custom")),
    ])
}

#[test]
fn default_enabled_acp_config_keeps_known_agents_in_catalog() {
    let catalog = catalog();

    let enabled = default_enabled_acp_config(&catalog);

    assert_eq!(enabled.len(), 6);
    assert_eq!(enabled.get("claude").map(|config| config.command.as_str()), Some("claude"));
    assert_eq!(enabled.get("custom").map(|config| config.command.as_str()), None);
}

#[test]
fn default_enabled_acp_config_skips_missing_catalog_entries() {
    let catalog = HashMap::from([("codex".to_string(), acp("codex"))]);

    let enabled = default_enabled_acp_config(&catalog);

    assert_eq!(enabled.len(), 1);
    assert!(enabled.contains_key("codex"));
}

#[test]
fn resolve_enabled_acp_config_uses_defaults_when_configured_is_empty() {
    let catalog = catalog();

    let enabled = resolve_enabled_acp_config(&catalog, HashMap::new());

    assert_eq!(enabled.len(), 6);
    assert!(enabled.contains_key("copilot"));
}

#[test]
fn resolve_enabled_acp_config_preserves_explicit_configured_agents() {
    let catalog = catalog();
    let configured = HashMap::from([("custom".to_string(), acp("configured-custom"))]);

    let enabled = resolve_enabled_acp_config(&catalog, configured);

    assert_eq!(enabled.len(), 1);
    assert_eq!(enabled["custom"].command, "configured-custom");
}

#[test]
fn build_acp_agent_enabled_patch_rejects_blank_name() {
    let err = build_acp_agent_enabled_patch("  ", true, None, &catalog(), HashMap::new())
        .expect_err("blank names should fail");

    assert_eq!(err, "acp agent name must not be empty");
}

#[test]
fn build_acp_agent_enabled_patch_rejects_unknown_agent_without_spec() {
    let err = build_acp_agent_enabled_patch("missing", true, None, &catalog(), HashMap::new())
        .expect_err("unknown catalog agents should fail");

    assert_eq!(err, "unknown acp agent: missing");
}

#[test]
fn build_acp_agent_enabled_patch_enables_catalog_agent_over_defaults() {
    let patch = build_acp_agent_enabled_patch(" custom ", true, None, &catalog(), HashMap::new())
        .expect("patch should build");

    assert_eq!(patch["acp"]["custom"]["command"], "custom");
    assert_eq!(patch["acp"]["claude"]["command"], "claude");
}

#[test]
fn build_acp_agent_enabled_patch_enables_single_explicit_agent_when_configured_exists() {
    let configured = HashMap::from([("claude".to_string(), acp("configured-claude"))]);
    let patch = build_acp_agent_enabled_patch(
        "custom",
        true,
        Some(acp("explicit-custom")),
        &catalog(),
        configured,
    )
    .expect("patch should build");

    assert_eq!(
        patch,
        json!({ "acp": { "custom": { "command": "explicit-custom", "args": [], "env": {} } } })
    );
}

#[test]
fn build_acp_agent_enabled_patch_disables_from_defaults_with_null_marker() {
    let patch = build_acp_agent_enabled_patch("claude", false, None, &catalog(), HashMap::new())
        .expect("patch should build");

    assert!(patch["acp"]["claude"].is_null());
    assert_eq!(patch["acp"]["gemini"]["command"], "gemini");
}

#[test]
fn build_acp_agent_enabled_patch_disables_single_configured_agent() {
    let configured = HashMap::from([
        ("claude".to_string(), acp("configured-claude")),
        ("custom".to_string(), acp("configured-custom")),
    ]);
    let patch = build_acp_agent_enabled_patch("custom", false, None, &catalog(), configured)
        .expect("patch should build");

    assert_eq!(patch, json!({ "acp": { "custom": null } }));
}

fn config_with_main_agent(provider: &str, model: &str, temperature: Option<f64>) -> super::Config {
    let mut config = super::Config::default();
    config.agents.insert(
        "main".to_string(),
        DelegateAgentConfig {
            provider: provider.to_string(),
            model: model.to_string(),
            temperature,
            ..DelegateAgentConfig::default()
        },
    );
    config
}

#[test]
fn build_main_agent_overrides_patch_returns_none_without_main_agent() {
    let patch = build_main_agent_overrides_patch(&super::Config::default())
        .expect("patch construction should not fail");

    assert!(patch.is_none());
}

#[test]
fn build_main_agent_overrides_patch_includes_provider_model_temperature_and_identity() {
    let config = config_with_main_agent(" openrouter ", " glm-5 ", Some(0.2));

    let patch = build_main_agent_overrides_patch(&config)
        .expect("patch construction should not fail")
        .expect("main agent should produce patch");

    assert_eq!(
        patch,
        json!({
            "default_provider": "openrouter",
            "default_model": "openrouter/glm-5",
            "default_temperature": 0.2,
            "identity": {
                "format": "openclaw",
                "aieos_path": null,
                "aieos_inline": null
            }
        })
    );
}

#[test]
fn build_main_agent_overrides_patch_skips_model_without_provider() {
    let config = config_with_main_agent(" ", "glm-5", Some(f64::NAN));

    let patch = build_main_agent_overrides_patch(&config)
        .expect("patch construction should not fail")
        .expect("main agent should produce identity patch");

    assert_eq!(
        patch,
        json!({
            "identity": {
                "format": "openclaw",
                "aieos_path": null,
                "aieos_inline": null
            }
        })
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn gateway_skey_db_path_for_config_dir_uses_vibewindow_gateway_sqlite() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = super::gateway_skey_db_path_for_config_dir(dir.path());

    assert_eq!(db_path, dir.path().join("gateway").join("skeys.sqlite"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn load_gateway_skeys_from_sqlite_reads_gateway_metadata() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("skeys.sqlite");
    let conn = rusqlite::Connection::open(&db_path).expect("sqlite open");
    conn.execute_batch(
        r#"
        CREATE TABLE gateway_skeys (
            skey_hash TEXT PRIMARY KEY NOT NULL,
            masked_skey TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            expires_at TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        INSERT INTO gateway_skeys (enabled, skey_hash, masked_skey, name, expires_at)
        VALUES (0, 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
                'sk-aaaa***************bbbbbbbbb', 'desktop', '2026-12-31T23:59:59Z');
        "#,
    )
    .expect("seed sqlite");

    let loaded = super::load_gateway_skeys_from_sqlite(&db_path).expect("load skeys");

    assert_eq!(loaded.len(), 1);
    assert!(!loaded[0].enabled);
    assert_eq!(loaded[0].name, "desktop");
    assert_eq!(loaded[0].masked_skey, "sk-aaaa***************bbbbbbbbb");
    assert_eq!(loaded[0].expires_at.as_deref(), Some("2026-12-31T23:59:59Z"));
    assert!(loaded[0].skey.is_none());
}
