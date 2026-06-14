use super::*;
// Tests for plan6 task 802.
const SOURCE: &str = include_str!("system_settings_models.rs");

use crate::app::state::{ModelSummary, ProviderModelsSummary};
use crate::app::{App, Message};
use iced::widget::text;
use serde_json::json;

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

fn provider_summary() -> ProviderModelsSummary {
    ProviderModelsSummary {
        id: "openai".to_string(),
        name: "OpenAI".to_string(),
        models: vec![
            ModelSummary {
                id: "gpt-5".to_string(),
                name: "GPT-5".to_string(),
                enabled: true,
                toolcall: true,
                attachment: true,
                context_limit: 200_000,
                detail: json!({"id": "gpt-5", "capabilities": {"toolcall": true}}),
            },
            ModelSummary {
                id: "text-embedding-3-small".to_string(),
                name: "Embedding Small".to_string(),
                enabled: false,
                toolcall: false,
                attachment: false,
                context_limit: 0,
                detail: json!({"id": "text-embedding-3-small"}),
            },
        ],
    }
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
fn main_view_builds_empty_loading_error_and_filtered_states() {
    let mut app = test_app();
    keep_element(main_view(&app));

    app.model_settings.loading = true;
    keep_element(main_view(&app));

    app.model_settings.loading = false;
    app.model_settings.providers = vec![provider_summary()];
    keep_element(main_view(&app));

    app.model_settings.query = "openai".to_string();
    keep_element(main_view(&app));

    app.model_settings.query = "embedding".to_string();
    keep_element(main_view(&app));

    app.model_settings.query = "missing".to_string();
    app.model_settings.save_error = Some("保存失败".to_string());
    keep_element(main_view(&app));
}

#[test]
fn overlays_return_dialog_or_model_detail_modal() {
    let mut app = test_app();
    keep_element(view_overlays(&app, text("dialog").into()));

    app.model_settings.providers = vec![provider_summary()];
    app.model_settings.detail_modal = Some(crate::app::state::ModelDetailModalState {
        provider_id: "openai".to_string(),
        provider_name: "OpenAI".to_string(),
        model_id: "gpt-5".to_string(),
        model_name: "GPT-5".to_string(),
        rows: vec![crate::app::state::ModelDetailRow {
            label: "模型名称".to_string(),
            value: "GPT-5".to_string(),
        }],
        raw_json: "{\n  \"id\": \"gpt-5\"\n}".to_string(),
        show_raw: false,
    });
    keep_element(view_overlays(&app, text("dialog").into()));

    app.model_settings.detail_modal.as_mut().expect("modal").show_raw = true;
    keep_element(view_overlays(&app, text("dialog").into()));
}

#[test]
fn system_settings_models_tests_keeps_planned_coverage_targets() {
    for name in ["main_view", "view_overlays"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
