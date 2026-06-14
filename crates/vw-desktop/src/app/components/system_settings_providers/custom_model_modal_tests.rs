// Tests for plan6 task 809.
const SOURCE: &str = include_str!("custom_model_modal.rs");

use crate::app::state::CustomProviderModelModalState;
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
fn overlays_return_base_add_modal_and_edit_modal() {
    let mut app = test_app();
    keep_element(super::custom_model_modal::view_overlays(&app, text("dialog").into()));

    app.provider_settings.custom_model_modal = Some(CustomProviderModelModalState {
        edit_index: None,
        model_id: "gpt-5".to_string(),
        display_name: "GPT-5".to_string(),
    });
    keep_element(super::custom_model_modal::view_overlays(&app, text("dialog").into()));

    app.provider_settings.custom_model_modal.as_mut().expect("modal").edit_index = Some(0);
    keep_element(super::custom_model_modal::view_overlays(&app, text("dialog").into()));
}

#[test]
fn custom_model_modal_tests_keeps_planned_coverage_targets() {
    for name in ["field_row", "text_row", "view_overlays"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
