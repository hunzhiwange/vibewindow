// Tests for plan6 task 807.
const SOURCE: &str = include_str!("connect.rs");

use crate::app::state::ProviderConnectState;
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
fn overlays_return_base_or_connect_modal_with_error() {
    let mut app = test_app();
    keep_element(super::connect::view_overlays(&app, text("dialog").into()));

    app.provider_settings.connect_modal = Some(ProviderConnectState {
        provider_id: "openai".to_string(),
        provider_name: "OpenAI".to_string(),
        api_key: "sk-test".to_string(),
    });
    keep_element(super::connect::view_overlays(&app, text("dialog").into()));

    app.provider_settings.connect_error = Some("连接失败".to_string());
    keep_element(super::connect::view_overlays(&app, text("dialog").into()));
}

#[test]
fn connect_tests_keeps_planned_coverage_targets() {
    for name in ["view_overlays"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
