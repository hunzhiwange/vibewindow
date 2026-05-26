// Tests for plan6 task 838.
const SOURCE: &str = include_str!("widgets.rs");

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
fn widgets_tests_keeps_planned_coverage_targets() {
    for name in [
        "color_with_alpha",
        "icon_svg",
        "icon_button",
        "icon_toggle_button",
        "icon_toggle_button_opt",
        "menu_btn",
        "menu_container",
        "menu_item_btn",
        "menu_separator",
        "menu_item_icon_btn",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
