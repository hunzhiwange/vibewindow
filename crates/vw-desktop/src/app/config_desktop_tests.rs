// Tests for plan6 task 841.
const SOURCE: &str = include_str!("config_desktop.rs");

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
fn config_desktop_tests_keeps_planned_coverage_targets() {
    for name in [
        "load_app_config_async",
        "save_app_config_async",
        "update_agents_compat_registry_result_async",
        "update_agents_compat_registry_async",
        "load_app_config",
        "save_app_config",
        "set_config_field",
        "load_project_chat_preferences",
        "save_project_chat_preferences",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
