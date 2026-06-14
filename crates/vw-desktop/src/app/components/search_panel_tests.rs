use super::search_panel::{result_button_style, search_input_style};
use iced::widget::{button, text_input};
use iced::{Background, Theme};

fn bg(style: &iced::widget::button::Style) -> iced::Color {
    match style.background {
        Some(Background::Color(color)) => color,
        _ => panic!("button style should use a color background"),
    }
}

#[test]
fn result_button_style_changes_background_by_status() {
    let theme = Theme::Light;

    let active = result_button_style(&theme, button::Status::Active);
    let hovered = result_button_style(&theme, button::Status::Hovered);
    let pressed = result_button_style(&theme, button::Status::Pressed);

    assert_ne!(bg(&active), bg(&hovered));
    assert_ne!(bg(&hovered), bg(&pressed));
    assert_eq!(active.border.width, 0.0);
    assert_eq!(active.border.radius.top_left, 6.0);
}

#[test]
fn search_input_style_tracks_focus_and_hover() {
    let theme = Theme::Light;

    let active = search_input_style(&theme, text_input::Status::Active);
    let hovered = search_input_style(&theme, text_input::Status::Hovered);
    let focused = search_input_style(&theme, text_input::Status::Focused { is_hovered: false });
    let focused_hovered =
        search_input_style(&theme, text_input::Status::Focused { is_hovered: true });

    assert_ne!(active.border.color, hovered.border.color);
    assert_ne!(hovered.border.color, focused.border.color);
    assert_eq!(focused.border.color, focused_hovered.border.color);
    assert_eq!(active.border.width, 1.0);
    assert_eq!(active.border.radius.top_left, 8.0);
}
