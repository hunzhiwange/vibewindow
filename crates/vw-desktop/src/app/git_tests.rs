// Tests for plan6 task 845.
const SOURCE: &str = include_str!("git.rs");

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
fn git_tests_keeps_planned_coverage_targets() {
    for name in [
        "git_stage_file",
        "sh_quote",
        "git_commit",
        "git_commit_with_body",
        "git_log",
        "git_discard_file",
        "git_diff_for_file",
        "get_file_content_pair",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
