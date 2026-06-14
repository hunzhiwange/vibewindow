use super::*;
use iced::Element;
use crate::app::message::settings::{SettingsMessage, WebSearchMessage};
use crate::app::{App, Message};
use iced::widget::text;

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn provider_visibility_helpers_match_supported_providers() {
    assert!(!shows_api_key("duckduckgo"));
    assert!(!shows_api_url("duckduckgo"));
    assert!(!shows_brave_api_key("duckduckgo"));

    assert!(shows_api_key("brave"));
    assert!(shows_api_url("brave"));
    assert!(shows_brave_api_key("brave"));

    assert!(shows_api_key("serper"));
    assert!(shows_api_url("serper"));
    assert!(!shows_brave_api_key("serper"));
}

#[test]
fn row_helpers_build_controls() {
    keep_element(field_row("标签", "说明", text("control")));
    keep_element(bool_row("启用", "说明", true, "启用搜索", |value| {
        Message::Settings(SettingsMessage::WebSearch(WebSearchMessage::EnabledToggled(value)))
    }));
    keep_element(text_row("密钥", "说明", "placeholder", "key", |value| {
        Message::Settings(SettingsMessage::WebSearch(WebSearchMessage::ApiKeyChanged(value)))
    }));
    keep_element(hint_row("提示"));
}

#[test]
fn view_builds_provider_specific_sections_and_error_banner() {
    let mut app = test_app();
    for provider in ["duckduckgo", "brave", "serper", "google", "bing", "unknown"] {
        app.web_search_settings.provider = provider.to_string();
        app.web_search_settings.enabled = true;
        app.web_search_settings.api_key_input = "api-key".to_string();
        app.web_search_settings.brave_api_key_input = "brave-key".to_string();
        app.web_search_settings.api_url_input = "https://search.example".to_string();
        app.web_search_settings.max_results_input = "10".to_string();
        app.web_search_settings.timeout_secs_input = "30".to_string();
        app.web_search_settings.user_agent = "VibeWindow/Test".to_string();
        keep_element(view(&app));
    }

    app.web_search_settings.save_error = Some("保存失败".to_string());
    keep_element(view(&app));
}

#[test]
fn overlays_return_dialog_or_help_modal() {
    let mut app = test_app();
    keep_element(view_overlays(&app, text("dialog").into()));

    app.web_search_settings.show_help_modal = true;
    keep_element(view_overlays(&app, text("dialog").into()));
}
