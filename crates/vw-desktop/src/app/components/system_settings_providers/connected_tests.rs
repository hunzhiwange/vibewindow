// Tests for plan6 task 808.
const SOURCE: &str = include_str!("connected.rs");

use crate::app::state::{ModelCatalogEntry, ProviderSummary};
use crate::app::{App, Message};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

fn provider(id: &str, name: &str, connected: bool) -> ProviderSummary {
    ProviderSummary {
        id: id.to_string(),
        name: name.to_string(),
        source_label: "配置文件".to_string(),
        connected,
    }
}

fn catalog_entry(provider_id: &str, provider_name: &str) -> ModelCatalogEntry {
    ModelCatalogEntry {
        provider_id: provider_id.to_string(),
        provider_name: provider_name.to_string(),
        model_id: "model".to_string(),
        model_name: "Model".to_string(),
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
fn view_builds_loading_sync_empty_connected_popular_and_error_states() {
    let mut app = test_app();
    keep_element(super::connected::view(&app));

    app.provider_settings.loading = true;
    keep_element(super::connected::view(&app));

    app.provider_settings.loading = false;
    app.provider_settings.models_syncing = true;
    app.provider_settings.models_sync_progress = 0.42;
    app.provider_settings.models_sync_label = "同步中".to_string();
    app.provider_settings.save_error = Some("保存失败".to_string());
    keep_element(super::connected::view(&app));

    app.provider_settings.models_syncing = false;
    app.provider_settings.providers =
        vec![provider("openai", "OpenAI", true), provider("anthropic", "Anthropic", false)];
    app.provider_settings.catalog_items =
        vec![catalog_entry("openai", "OpenAI"), catalog_entry("anthropic", "Anthropic")];
    app.provider_settings.popular_patterns =
        vec!["OpenAI".to_string(), "anth".to_string(), "missing".to_string()];
    keep_element(super::connected::view(&app));

    app.provider_settings.disconnect_confirm_provider_id = Some("openai".to_string());
    keep_element(super::connected::view(&app));

    app.provider_settings.popular_patterns.clear();
    keep_element(super::connected::view(&app));
}

#[test]
fn connected_tests_keeps_planned_coverage_targets() {
    for name in ["provider_action_panel_style", "provider_item_row", "view"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
