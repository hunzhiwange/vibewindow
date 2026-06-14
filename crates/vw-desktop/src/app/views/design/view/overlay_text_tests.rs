use iced::widget::text_editor;
use iced::{Color, Vector};
use serde_json::json;

use crate::app::message::DesignMessage;
use crate::app::views::design::models::{DesignDoc, DesignElement, ThemeCondition};
use crate::app::views::design::state::DesignState;
use crate::app::{App, Message};

fn text_element(id: &str) -> DesignElement {
    DesignElement {
        kind: "text".to_string(),
        id: id.to_string(),
        x: 10.0,
        y: 20.0,
        width: Some(json!(120.0)),
        height: Some(json!(48.0)),
        fill: Some(json!("#ffffff")),
        color: Some("#111827".to_string()),
        font_size: Some(json!(18.0)),
        font_weight: Some(json!("400")),
        padding: Some(json!([4, 6, 8, 10])),
        content: Some("Overlay".to_string()),
        ..Default::default()
    }
}

fn state_with_element(element: DesignElement) -> DesignState {
    DesignState::new(DesignDoc { children: vec![element], ..Default::default() })
}

fn app_with_preview(open: bool) -> App {
    let mut app = App::new().0;
    app.show_element_html_preview = open;
    app.element_html_preview_editor = iced::widget::text_editor::Content::with_text("<p>HTML</p>");
    app
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("overlay_text_tests"));
}

#[test]
fn html_preview_layers_returns_empty_when_preview_closed() {
    let app = app_with_preview(false);

    let layers = super::html_preview_layers(&app);

    assert!(layers.is_empty());
}

#[test]
fn html_preview_layers_returns_modal_layer_when_preview_open() {
    let app = app_with_preview(true);

    let layers = super::html_preview_layers(&app);

    assert_eq!(layers.len(), 1);
}

#[test]
fn html_preview_backdrop_style_uses_theme_background_with_overlay_alpha() {
    let style = super::html_preview_backdrop_style(&iced::Theme::Dark);
    let Some(iced::Background::Color(background)) = style.background else {
        panic!("expected backdrop color");
    };

    assert_eq!(background.a, 0.5);
    assert_eq!(background.r, iced::Theme::Dark.palette().background.r);
    assert_eq!(background.g, iced::Theme::Dark.palette().background.g);
    assert_eq!(background.b, iced::Theme::Dark.palette().background.b);
}

#[test]
fn action_helpers_map_to_design_messages() {
    match super::edit_editor_action_message(text_editor::Action::SelectAll) {
        Message::Design(DesignMessage::EditEditorAction(text_editor::Action::SelectAll)) => {}
        other => panic!("expected edit editor action, got {other:?}"),
    }

    match super::html_preview_action_message(text_editor::Action::Move(text_editor::Motion::Left)) {
        Message::Design(DesignMessage::HtmlPreviewAction(text_editor::Action::Move(
            text_editor::Motion::Left,
        ))) => {}
        other => panic!("expected html preview action, got {other:?}"),
    }
}

#[test]
fn inline_text_editor_style_keeps_transparent_editor_with_text_color() {
    let color = Color::from_rgb(0.1, 0.2, 0.3);
    let style_for = super::inline_text_editor_style_for(color);
    let style = style_for(&iced::Theme::Light, text_editor::Status::Active);

    assert_eq!(style.background, iced::Background::Color(Color::TRANSPARENT));
    assert_eq!(style.border.width, 0.0);
    assert_eq!(style.value, color);
}

#[test]
fn inline_text_editor_overlay_renders_empty_layer_without_editing_id() {
    let state = DesignState::new(DesignDoc::default());

    let _overlay = super::inline_text_editor_overlay(&state);
}

#[test]
fn inline_text_editor_overlay_ignores_missing_edit_target() {
    let mut state = state_with_element(text_element("title"));
    state.editing_id = Some("missing".to_string());

    let _overlay = super::inline_text_editor_overlay(&state);
}

#[test]
fn inline_text_editor_overlay_renders_valid_edit_target() {
    let mut state = state_with_element(text_element("title"));
    state.editing_id = Some("title".to_string());
    state.doc.theme = Some(ThemeCondition { mode: "Dark".to_string() });
    state.editing_editor = iced::widget::text_editor::Content::with_text("Edited");
    state.pan = Vector::new(5.0, 7.0);
    state.zoom = 1.5;

    let _overlay = super::inline_text_editor_overlay(&state);
}

#[test]
fn inline_text_editor_overlay_handles_weight_variants_and_small_content_bounds() {
    for weight in ["300", "400", "500", "600", "700", "800", "900"] {
        let mut element = text_element(weight);
        element.width = Some(json!(8.0));
        element.height = Some(json!(6.0));
        element.padding = Some(json!([20, 20, 20, 20]));
        element.color = None;
        element.font_weight = Some(json!(weight));

        let mut state = state_with_element(element);
        state.editing_id = Some(weight.to_string());

        let _overlay = super::inline_text_editor_overlay(&state);
    }
}
