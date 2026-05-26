// Tests for plan6 task 843.
const SOURCE: &str = include_str!("config_system_settings.rs");

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
fn config_system_settings_tests_keeps_planned_coverage_targets() {
    for name in [
        "fetch_desktop_system_settings_via_gateway",
        "patch_desktop_system_settings_via_gateway",
        "load_legacy_system_settings_config_local",
        "normalize_system_settings_config",
        "gateway_client_bootstrap_cache_path",
        "load_gateway_client_bootstrap_config",
        "save_gateway_client_bootstrap_config",
        "load_gateway_client_config",
        "update_gateway_client_config",
        "load_system_settings_config_async",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
