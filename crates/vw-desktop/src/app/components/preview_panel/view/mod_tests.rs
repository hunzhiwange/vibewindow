use super::{fullscreen_icon_style, fullscreen_tooltip_style, overlay_icon_button_style, view};
use crate::app::components::editor::Editor;
use crate::app::{App, Message, PreviewTab, Screen};
use iced::widget::button;
use iced::{Background, Color, Theme, widget::Id};

fn app() -> App {
    App::new().0
}

fn tab(path: &str, server_key: Option<&'static str>) -> PreviewTab {
    PreviewTab {
        path: path.to_string(),
        title: path.rsplit('/').next().unwrap_or(path).to_string(),
        content: "fn main() {}".to_string(),
        is_dirty: false,
        truncated: false,
        auto_save_revision: 0,
        editor: Editor::new("fn main() {}", "rust"),
        scroll_id: Id::unique(),
        #[cfg(not(target_arch = "wasm32"))]
        lsp_server_key: server_key,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_uri: Some(format!("file://{path}")),
        #[cfg(not(target_arch = "wasm32"))]
        lsp_language_id: Some("rust".to_string()),
    }
}

fn keep(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn view_builds_project_preview_with_tabs_and_content() {
    let mut app = app();
    app.screen = Screen::Project;
    app.active_preview_path = Some("/tmp/main.rs".to_string());
    app.preview_tabs = vec![tab("/tmp/main.rs", None)];

    keep(view(&app));
}

#[test]
fn view_builds_settings_overlay_when_enabled() {
    let mut app = app();
    app.screen = Screen::Preview;
    app.show_preview_settings = true;

    keep(view(&app));
}

#[test]
fn view_builds_changes_layout_without_preview_header() {
    let mut app = app();
    app.screen = Screen::Project;
    app.file_manager_show_changes = true;

    keep(view(&app));
}

#[test]
fn overlay_icon_button_style_distinguishes_statuses() {
    let theme = Theme::Light;
    let active = overlay_icon_button_style(&theme, button::Status::Active);
    let hovered = overlay_icon_button_style(&theme, button::Status::Hovered);
    let pressed = overlay_icon_button_style(&theme, button::Status::Pressed);

    assert!(active.background.is_none());
    assert_eq!(active.border.color, Color::TRANSPARENT);
    assert!(matches!(hovered.background, Some(Background::Color(_))));
    assert!(matches!(pressed.background, Some(Background::Color(_))));
    assert_ne!(hovered.border.color, pressed.border.color);
}

#[test]
fn fullscreen_tooltip_style_switches_between_light_and_dark_palettes() {
    let light = fullscreen_tooltip_style(&Theme::Light);
    let dark = fullscreen_tooltip_style(&Theme::Dark);

    assert_eq!(light.text_color, Some(Color::WHITE));
    assert_ne!(light.background, dark.background);
    assert!(light.shadow.blur_radius > dark.shadow.blur_radius);
}

#[test]
fn fullscreen_icon_style_uses_visible_theme_color() {
    let light = fullscreen_icon_style(&Theme::Light);
    let dark = fullscreen_icon_style(&Theme::Dark);

    assert!(light.color.is_some());
    assert!(dark.color.is_some());
    assert_ne!(light.color, dark.color);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn active_preview_lsp_badge_requires_matching_active_tab() {
    let mut app = app();
    assert!(super::active_preview_lsp_badge(&app).is_none());

    app.active_preview_path = Some("/tmp/main.rs".to_string());
    assert!(super::active_preview_lsp_badge(&app).is_none());

    app.preview_tabs = vec![tab("/tmp/main.rs", Some("rust-analyzer"))];
    assert!(super::active_preview_lsp_badge(&app).is_some());
}
