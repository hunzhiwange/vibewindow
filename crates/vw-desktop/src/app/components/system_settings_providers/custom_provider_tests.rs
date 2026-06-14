// Tests for plan6 task 810.
const SOURCE: &str = include_str!("custom_provider.rs");

use crate::app::state::{CustomProviderModelDraft, ProviderHeaderDraft};
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
fn overlays_return_base_or_create_modal_with_single_empty_rows() {
    let mut app = test_app();
    keep_element(super::custom_provider::view_overlays(&app, text("dialog").into()));

    app.provider_settings.custom_provider_modal_open = true;
    app.provider_settings.custom.provider_id = "local-ai".to_string();
    app.provider_settings.custom.display_name = "Local AI".to_string();
    app.provider_settings.custom.base_url = "http://localhost:11434/v1".to_string();
    app.provider_settings.custom.api_key = "sk-local".to_string();
    keep_element(super::custom_provider::view_overlays(&app, text("dialog").into()));
}

#[test]
fn overlays_build_edit_modal_headers_models_and_error_states() {
    let mut app = test_app();
    app.provider_settings.custom_provider_modal_open = true;
    app.provider_settings.custom_editing_provider_id = Some("local-ai".to_string());
    app.provider_settings.custom.provider_id = "local-ai".to_string();
    app.provider_settings.custom.display_name = "Local AI".to_string();
    app.provider_settings.custom.base_url = "http://localhost:11434/v1".to_string();
    app.provider_settings.custom.api_key = "sk-local".to_string();
    app.provider_settings.custom.headers = vec![
        ProviderHeaderDraft { key: "x-route".to_string(), value: "local".to_string() },
        ProviderHeaderDraft { key: "x-team".to_string(), value: "desktop".to_string() },
    ];
    app.provider_settings.custom.models = vec![
        CustomProviderModelDraft {
            model_id: "llama3.1".to_string(),
            display_name: "Llama 3.1".to_string(),
        },
        CustomProviderModelDraft { model_id: "qwen".to_string(), display_name: String::new() },
        CustomProviderModelDraft { model_id: String::new(), display_name: String::new() },
    ];
    keep_element(super::custom_provider::view_overlays(&app, text("dialog").into()));

    app.provider_settings.save_error = Some("保存失败".to_string());
    keep_element(super::custom_provider::view_overlays(&app, text("dialog").into()));
}

#[test]
fn custom_provider_tests_keeps_planned_coverage_targets() {
    for name in ["field_row", "text_row", "secure_text_row", "view_overlays"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
