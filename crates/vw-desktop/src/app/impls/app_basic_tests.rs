// Tests for plan6 task 847.
const SOURCE: &str = include_str!("app_basic.rs");

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
fn app_basic_tests_keeps_planned_coverage_targets() {
    for name in [
        "normalize_file_search_query",
        "is_diff_file_expanded",
        "toggle_diff_file_expanded",
        "ensure_diff_file_expanded",
        "replace_expanded_files",
        "clear_expanded_files",
        "set_single_expanded_file",
        "is_file_tree_dir_expanded",
        "toggle_file_tree_dir_expanded",
        "ensure_file_tree_dir_expanded",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
