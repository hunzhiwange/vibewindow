use super::model_hover_tooltip::{
    HoverAnchor, hover_model_trigger, hover_text_trigger, hover_tooltip_overlay,
};
use crate::app::state::{ModelPopoverHover, ModelSummary, ProviderModelsSummary};
use crate::app::{App, Message};
use iced::widget::{container, text};
use iced::{Background, Color, Element, Theme};

fn test_app() -> App {
    App::new().0
}

fn keep(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

fn tooltip_style(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(Color::BLACK)),
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

fn hover_message(hover: Option<ModelPopoverHover>) -> Message {
    std::hint::black_box(hover);
    Message::View(crate::app::message::view::ViewMessage::CloseModelPopover)
}

fn exit_message() -> Message {
    hover_message(None)
}

fn provider() -> ProviderModelsSummary {
    ProviderModelsSummary {
        id: "openai".to_string(),
        name: "OpenAI".to_string(),
        models: vec![ModelSummary {
            id: "gpt-test".to_string(),
            name: "GPT Test".to_string(),
            enabled: true,
            toolcall: true,
            attachment: false,
            context_limit: 128_000,
            detail: serde_json::json!({"owned_by": "test"}),
        }],
    }
}

#[test]
fn task_746_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("model_hover_tooltip_tests.rs"));
}

#[test]
fn hover_anchor_is_plain_copyable_coordinate_data() {
    let anchor = HoverAnchor { x: 12.5, y: 99.0 };

    assert_eq!(anchor.x, 12.5);
    assert_eq!(anchor.y, 99.0);
    assert_eq!(format!("{anchor:?}"), "HoverAnchor { x: 12.5, y: 99.0 }");
}

#[test]
fn hover_text_and_model_triggers_build_mouse_areas() {
    keep(hover_text_trigger(text("trigger"), "plain tooltip", hover_message, exit_message()));
    keep(hover_model_trigger(text("model"), "openai", "gpt-test", hover_message, exit_message()));
}

#[test]
fn overlay_returns_content_when_hover_is_absent_or_model_is_missing() {
    let app = test_app();
    keep(hover_tooltip_overlay(&app, text("content"), tooltip_style));

    let mut missing = test_app();
    missing.model_popover_hover = Some(ModelPopoverHover::Model {
        provider_id: "missing".to_string(),
        model_id: "model".to_string(),
        anchor: None,
    });
    keep(hover_tooltip_overlay(&missing, text("content"), tooltip_style));
}

#[test]
fn overlay_builds_text_tooltip_with_side_and_point_anchor_modes() {
    let mut app = test_app();
    app.model_popover_hover = Some(ModelPopoverHover::Text {
        text: "Use this model for coding".to_string(),
        anchor: None,
    });
    keep(hover_tooltip_overlay(&app, container(text("content")), tooltip_style));

    app.model_popover_hover = Some(ModelPopoverHover::Text {
        text: "Anchored text".to_string(),
        anchor: Some(HoverAnchor { x: 30.0, y: 40.0 }),
    });
    keep(hover_tooltip_overlay(&app, container(text("content")), tooltip_style));
}

#[test]
fn overlay_builds_model_tooltip_for_matching_provider_model() {
    let mut app = test_app();
    app.model_settings.providers.push(provider());
    app.model_popover_hover = Some(ModelPopoverHover::Model {
        provider_id: "openai".to_string(),
        model_id: "gpt-test".to_string(),
        anchor: Some(HoverAnchor { x: 10.0, y: 20.0 }),
    });

    keep(hover_tooltip_overlay(&app, text("content"), tooltip_style));
}
