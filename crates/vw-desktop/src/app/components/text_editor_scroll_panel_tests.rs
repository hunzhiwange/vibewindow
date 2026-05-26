// Tests for plan6 task 831.
const SOURCE: &str = include_str!("text_editor_scroll_panel.rs");

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
fn text_editor_scroll_panel_tests_keeps_planned_coverage_targets() {
    for name in [
        "TextEditorScrollPanelMetrics",
        "text_editor_scroll_panel",
        "wheel_interceptor",
        "WheelInterceptor",
        "children",
        "diff",
        "size",
        "layout",
        "operate",
        "update",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
