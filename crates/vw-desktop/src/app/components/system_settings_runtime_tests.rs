// Tests for plan6 task 815.
const SOURCE: &str = include_str!("system_settings_runtime.rs");

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
fn system_settings_runtime_tests_keeps_planned_coverage_targets() {
    for name in ["LabeledOption", "fmt", "field_row", "text_row", "hint_row", "view"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}

#[test]
fn runtime_reasoning_coverage_stays_outside_base_behavior_panel() {
    let base_section = SOURCE
        .find("settings_section_card(\"基础行为\"")
        .expect("base behavior section should exist");
    let base_panel = SOURCE[base_section..]
        .find("settings_panel(column![kind_row].spacing(0))")
        .map(|index| base_section + index)
        .expect("base behavior panel should only contain kind_row");
    let reasoning_section = SOURCE
        .find("settings_section_card(\n            \"推理覆盖\"")
        .expect("reasoning coverage section should exist");
    let reasoning_panel = SOURCE[reasoning_section..]
        .find("settings_panel(\n            column![reasoning_enabled_row")
        .map(|index| reasoning_section + index)
        .expect("reasoning coverage panel should exist");

    assert!(base_section < base_panel);
    assert!(base_panel < reasoning_section);
    assert!(reasoning_section < reasoning_panel);
}
