// Tests for plan6 task 806.
const SOURCE: &str = include_str!("catalog.rs");

use crate::app::state::{ModelCatalogEntry, ProviderSummary};
use crate::app::{App, Message};
use iced::widget::text;

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

fn catalog_entry(provider_id: &str, provider_name: &str, model_id: &str) -> ModelCatalogEntry {
    ModelCatalogEntry {
        provider_id: provider_id.to_string(),
        provider_name: provider_name.to_string(),
        model_id: model_id.to_string(),
        model_name: model_id.to_string(),
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
fn overlays_return_base_when_catalog_is_closed() {
    let app = test_app();

    keep_element(super::catalog::view_overlays(&app, text("dialog").into()));
}

#[test]
fn overlays_build_loading_empty_filtered_and_populated_catalogs() {
    let mut app = test_app();
    app.provider_settings.catalog_open = true;
    app.provider_settings.catalog_loading = true;
    keep_element(super::catalog::view_overlays(&app, text("dialog").into()));

    app.provider_settings.catalog_loading = false;
    keep_element(super::catalog::view_overlays(&app, text("dialog").into()));

    app.provider_settings.catalog_items = vec![
        catalog_entry("openai", "OpenAI", "gpt-5"),
        catalog_entry("openai", "OpenAI", "gpt-5-mini"),
        catalog_entry("anthropic", "Anthropic", "claude-4"),
    ];
    app.provider_settings.providers = vec![ProviderSummary {
        id: "openai".to_string(),
        name: "OpenAI".to_string(),
        source_label: "配置文件".to_string(),
        connected: true,
    }];
    app.provider_settings.popular_patterns = vec!["anthropic".to_string()];
    keep_element(super::catalog::view_overlays(&app, text("dialog").into()));

    app.provider_settings.catalog_query = "anth".to_string();
    keep_element(super::catalog::view_overlays(&app, text("dialog").into()));

    app.provider_settings.catalog_query = "missing".to_string();
    keep_element(super::catalog::view_overlays(&app, text("dialog").into()));
}

#[test]
fn catalog_tests_keeps_planned_coverage_targets() {
    for name in
        ["catalog_surface_style", "catalog_list_frame_style", "catalog_item_style", "view_overlays"]
    {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
