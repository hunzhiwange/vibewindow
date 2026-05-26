// Tests for plan6 task 814.
const SOURCE: &str = include_str!("system_settings_research.rs");

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
fn system_settings_research_tests_keeps_planned_coverage_targets() {
    for name in
        ["field_row", "text_row", "trigger_label", "parse_trigger_label", "view", "view_overlays"]
    {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
