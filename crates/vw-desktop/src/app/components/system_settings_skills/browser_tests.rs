// Tests for plan6 task 819.
const SOURCE: &str = include_str!("browser.rs");

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
fn browser_tests_keeps_planned_coverage_targets() {
    for name in [
        "DetailActionStyle",
        "search_bar_style",
        "header_panel_style",
        "catalog_panel_style",
        "status_banner",
        "scope_button",
        "refresh_button",
        "scope_source_matches",
        "scope_title",
        "scope_description",
        "active_scope_badge",
        "discovery_order_text",
        "loading_banner",
        "empty_state",
        "detail_source_note",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
