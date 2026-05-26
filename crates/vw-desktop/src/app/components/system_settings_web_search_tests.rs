// Tests for plan6 task 826.
const SOURCE: &str = include_str!("system_settings_web_search.rs");

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
fn system_settings_web_search_tests_keeps_planned_coverage_targets() {
    for name in [
        "field_row",
        "bool_row",
        "text_row",
        "hint_row",
        "shows_api_key",
        "shows_api_url",
        "shows_brave_api_key",
        "view",
        "view_overlays",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
