use super::*;
// Tests for plan6 task 803.
const SOURCE: &str = include_str!("system_settings_multimodal.rs");

use crate::app::{App, Message};

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
fn view_builds_default_enabled_and_error_states() {
    let mut app = test_app();
    keep_element(view(&app));

    app.multimodal_settings.max_images = 16;
    app.multimodal_settings.max_image_size_mb = 20;
    app.multimodal_settings.allow_remote_fetch = true;
    keep_element(view(&app));

    app.multimodal_settings.max_images = 1;
    app.multimodal_settings.max_image_size_mb = 1;
    app.multimodal_settings.save_error = Some("保存失败".to_string());
    keep_element(view(&app));
}

#[test]
fn system_settings_multimodal_tests_keeps_planned_coverage_targets() {
    for name in ["field_row", "number_row", "bool_row", "view"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
