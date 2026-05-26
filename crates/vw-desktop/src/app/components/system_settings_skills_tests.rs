// Tests for plan6 task 818.
const SOURCE: &str = include_str!("system_settings_skills.rs");

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
fn system_settings_skills_tests_keeps_planned_coverage_targets() {
    for name in [
        "field_row",
        "settings_tab_button",
        "plugins_placeholder",
        "system_config_view",
        "view",
        "view_overlays",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
