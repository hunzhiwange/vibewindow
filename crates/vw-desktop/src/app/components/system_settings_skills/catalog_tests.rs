// Tests for plan6 task 820.
const SOURCE: &str = include_str!("catalog.rs");

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
fn catalog_tests_keeps_planned_coverage_targets() {
    for name in [
        "catalog_matches_query",
        "section_card_style",
        "skill_badge",
        "catalog_skill_initials",
        "section_copy",
        "source_label",
        "compact_source_path",
        "source_path_text",
        "catalog_item",
        "catalog_group_section",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
