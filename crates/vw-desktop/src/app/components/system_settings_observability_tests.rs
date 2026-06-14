use super::*;
// Tests for plan6 task 804.
const SOURCE: &str = include_str!("system_settings_observability.rs");

use crate::app::{App, Message};
use iced::widget::text;

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

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
fn view_builds_backend_trace_and_error_states() {
    let mut app = test_app();
    keep_element(view(&app));

    app.observability_settings.backend = "otel".to_string();
    app.observability_settings.otel_endpoint_input = "http://collector:4318".to_string();
    app.observability_settings.otel_service_name_input = "vw-test".to_string();
    app.observability_settings.runtime_trace_mode = "rolling".to_string();
    app.observability_settings.runtime_trace_path_input = "state/runtime-trace.jsonl".to_string();
    app.observability_settings.runtime_trace_max_entries = 100_000;
    keep_element(view(&app));

    app.observability_settings.backend = "prometheus".to_string();
    app.observability_settings.runtime_trace_mode = "full".to_string();
    app.observability_settings.runtime_trace_max_entries = 1;
    app.observability_settings.save_error = Some("保存失败".to_string());
    keep_element(view(&app));
}

#[test]
fn overlays_return_base_dialog_or_help_modal() {
    let mut app = test_app();
    keep_element(view_overlays(&app, text("dialog").into()));

    app.observability_settings.show_help_modal = true;
    keep_element(view_overlays(&app, text("dialog").into()));
}

#[test]
fn system_settings_observability_tests_keeps_planned_coverage_targets() {
    for name in ["field_row", "text_row", "view", "view_overlays"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
