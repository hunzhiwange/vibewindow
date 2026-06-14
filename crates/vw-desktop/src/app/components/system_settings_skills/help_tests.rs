// Tests for plan6 task 821.
const SOURCE: &str = include_str!("help.rs");

use super::help::help_text;

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
fn help_text_documents_core_fields_and_local_skill_toggle_marker() {
    let text = help_text();

    for expected in [
        "open_skills_enabled",
        "open_skills_dir",
        "prompt_injection_mode",
        "compact",
        "full",
        "SKILL.disabled",
    ] {
        assert!(text.contains(expected), "help text should contain {expected}");
    }
}

#[test]
fn help_tests_keeps_planned_coverage_targets() {
    for name in ["view_overlays"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
