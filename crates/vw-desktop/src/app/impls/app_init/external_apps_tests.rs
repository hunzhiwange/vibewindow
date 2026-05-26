// Tests for plan6 task 849.
const SOURCE: &str = include_str!("external_apps.rs");

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
fn external_apps_tests_keeps_planned_coverage_targets() {
    for name in ["resolve_external_apps", "configured_external_app", "priority_external_apps"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
