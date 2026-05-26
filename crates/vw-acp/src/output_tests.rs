use super::*;
use crate::types::{OutputFormat, OutputFormatterContext};
use serde_json::json;

#[test]
fn create_output_formatter_selects_requested_format() {
    assert!(matches!(
        create_output_formatter(OutputFormat::Quiet, Vec::new(), OutputFormatterOptions::default()),
        AnyOutputFormatter::Quiet(_)
    ));
    assert!(matches!(
        create_output_formatter(OutputFormat::Json, Vec::new(), OutputFormatterOptions::default()),
        AnyOutputFormatter::Json(_)
    ));
}

#[test]
fn tool_status_defaults_unknown_values_to_running() {
    assert_eq!(ToolStatus::from_value(Some("completed")).label(), "completed");
    assert_eq!(ToolStatus::from_value(Some("failed")).label(), "failed");
    assert_eq!(ToolStatus::from_value(Some("other")).label(), "running");
}

#[test]
fn truncate_json_preserves_valid_utf8_boundary() {
    let text = "好".repeat(MAX_OUTPUT_LENGTH);
    let rendered = truncate_text(&text);

    assert!(rendered.ends_with("..."));
    assert!(rendered.is_char_boundary(rendered.len()));
    assert!(!truncate_json(&json!({"a": 1})).is_empty());
}

#[test]
fn quiet_formatter_retains_inner_writer() {
    let formatter = create_output_formatter(
        OutputFormat::Quiet,
        Vec::<u8>::new(),
        OutputFormatterOptions {
            context: Some(OutputFormatterContext { session_id: "s1".to_string() }),
            suppress_reads: false,
            is_tty: false,
        },
    );

    assert!(formatter.into_inner().is_empty());
}
