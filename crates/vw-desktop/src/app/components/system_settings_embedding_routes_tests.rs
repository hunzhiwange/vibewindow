use super::*;
use iced::Element;
use crate::app::state::EmbeddingRouteDraft;
use crate::app::{App, Message};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn rows_build_plain_and_secure_inputs() {
    keep_element(field_row("匹配模式", "说明", "semantic", "memory", |value| {
        Message::Settings(message::SettingsMessage::EmbeddingRoutes(
            EmbeddingRoutesMessage::PatternChanged(0, value),
        ))
    }));
    keep_element(secure_field_row("API Key", "说明", "KEY", "secret", |value| {
        Message::Settings(message::SettingsMessage::EmbeddingRoutes(
            EmbeddingRoutesMessage::ApiKeyChanged(0, value),
        ))
    }));
}

#[test]
fn view_builds_empty_success_error_and_route_cards() {
    let app = test_app();
    keep_element(view(&app));

    let mut app = test_app();
    app.embedding_routes_settings.save_success = true;
    keep_element(view(&app));

    app.embedding_routes_settings.save_success = false;
    app.embedding_routes_settings.save_error = Some("保存失败".to_string());
    keep_element(view(&app));

    app.embedding_routes_settings.save_error = None;
    app.embedding_routes_settings.routes.push(EmbeddingRouteDraft {
        pattern: "memory".to_string(),
        provider: "alibaba-cn".to_string(),
        model: "text-embedding-v4".to_string(),
        dimensions: "1024".to_string(),
        api_key_input: "dashscope-key".to_string(),
    });
    app.embedding_routes_settings.routes.push(EmbeddingRouteDraft {
        pattern: "code".to_string(),
        provider: "openai".to_string(),
        model: "text-embedding-3-large".to_string(),
        dimensions: String::new(),
        api_key_input: String::new(),
    });
    keep_element(view(&app));
}
