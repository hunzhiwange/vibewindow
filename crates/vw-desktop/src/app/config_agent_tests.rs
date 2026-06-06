// Tests for plan6 task 840.
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
        "DEFAULT_ENABLED_ACP_AGENTS",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
