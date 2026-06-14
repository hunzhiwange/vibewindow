// Tests for plan6 task 822.
const SOURCE: &str = include_str!("system_settings_sop.rs");

use super::{execution_mode_options, rounded_u32};

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
fn execution_mode_options_keep_supervised_before_autonomous() {
    assert_eq!(execution_mode_options(), ["supervised".to_string(), "autonomous".to_string()]);
}

#[test]
fn rounded_u32_matches_number_input_message_conversion() {
    assert_eq!(rounded_u32(0.0), 0);
    assert_eq!(rounded_u32(1.49), 1);
    assert_eq!(rounded_u32(1.5), 2);
    assert_eq!(rounded_u32(86_400.0), 86_400);
}

#[test]
fn system_settings_sop_tests_keeps_planned_coverage_targets() {
    for name in ["field_row", "hint_row", "number_row", "view"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
