use iced::widget::button;
use iced::{Background, Color, Theme};

use crate::app::Message;
use crate::app::assets::Icon;

use super::shared::{
    compact_outlined_button_style, disabled_button_style, disabled_icon_svg,
    header_plain_glyph_button_style, icon_svg, icon_svg_sized, outlined_button_style,
    small_plain_icon_button_style, subtle_button_style, themed_icon_svg, with_tooltip,
};

fn radius(style: &button::Style) -> f32 {
    style.border.radius.top_left
}

#[test]
fn svg_helpers_accept_default_and_custom_sizes() {
    let _ = icon_svg(Icon::Plus);
    let _ = icon_svg_sized(Icon::Trash, 22.0);
    let _ = disabled_icon_svg(Icon::Image, 15.0);
    let _ = themed_icon_svg(Icon::FileText, 10.0);
}

#[test]
fn with_tooltip_builds_compact_and_regular_variants() {
    let _: iced::Element<'static, Message> =
        with_tooltip(iced::widget::text("regular"), "regular tip".into(), false, 6.0);
    let _: iced::Element<'static, Message> =
        with_tooltip(iced::widget::text("compact"), "compact tip".into(), true, 4.0);
}

#[test]
fn outlined_button_style_tracks_status_backgrounds() {
    let theme = Theme::Light;
    let active = outlined_button_style(&theme, button::Status::Active, 8.0);
    let hovered = outlined_button_style(&theme, button::Status::Hovered, 8.0);
    let pressed = outlined_button_style(&theme, button::Status::Pressed, 8.0);

    assert!(active.background.is_none());
    assert!(hovered.background.is_some());
    assert!(pressed.background.is_some());
    assert_eq!(active.border.width, 1.0);
    assert_eq!(radius(&active), 8.0);
}

#[test]
fn compact_outlined_button_style_covers_light_and_dark_branches() {
    let light_active = compact_outlined_button_style(&Theme::Light, button::Status::Active, 7.0);
    let light_hovered = compact_outlined_button_style(&Theme::Light, button::Status::Hovered, 7.0);
    let light_pressed = compact_outlined_button_style(&Theme::Light, button::Status::Pressed, 7.0);
    let dark_active = compact_outlined_button_style(&Theme::Dark, button::Status::Active, 7.0);
    let dark_hovered = compact_outlined_button_style(&Theme::Dark, button::Status::Hovered, 7.0);
    let dark_pressed = compact_outlined_button_style(&Theme::Dark, button::Status::Pressed, 7.0);

    assert!(light_active.background.is_none());
    assert!(light_hovered.background.is_some());
    assert!(light_pressed.background.is_some());
    assert!(dark_active.background.is_some());
    assert!(dark_hovered.background.is_some());
    assert!(dark_pressed.background.is_some());
    assert_eq!(radius(&dark_active), 7.0);
}

#[test]
fn disabled_and_subtle_styles_cover_all_statuses() {
    let disabled = disabled_button_style(&Theme::Light, 6.0);
    assert!(disabled.background.is_some());
    assert_eq!(disabled.border.width, 1.0);
    assert_eq!(radius(&disabled), 6.0);

    let active = subtle_button_style(button::Status::Active, 5.0);
    let hovered = subtle_button_style(button::Status::Hovered, 5.0);
    let pressed = subtle_button_style(button::Status::Pressed, 5.0);

    assert_eq!(active.background, Some(Background::Color(Color::TRANSPARENT)));
    assert_ne!(hovered.background, active.background);
    assert_ne!(pressed.background, active.background);
    assert_eq!(radius(&pressed), 5.0);
}

#[test]
fn plain_button_styles_cover_status_branches() {
    let icon_active = small_plain_icon_button_style(&Theme::Light, button::Status::Active);
    let icon_pressed = small_plain_icon_button_style(&Theme::Light, button::Status::Pressed);
    assert!(icon_active.background.is_none());
    assert!(icon_pressed.background.is_some());
    assert_eq!(radius(&icon_pressed), 6.0);

    let header_light_hovered =
        header_plain_glyph_button_style(&Theme::Light, button::Status::Hovered);
    let header_light_pressed =
        header_plain_glyph_button_style(&Theme::Light, button::Status::Pressed);
    let header_dark_hovered =
        header_plain_glyph_button_style(&Theme::Dark, button::Status::Hovered);
    let header_dark_pressed =
        header_plain_glyph_button_style(&Theme::Dark, button::Status::Pressed);
    let header_active = header_plain_glyph_button_style(&Theme::Light, button::Status::Active);

    assert!(header_active.background.is_none());
    assert!(header_light_hovered.background.is_some());
    assert!(header_light_pressed.background.is_some());
    assert!(header_dark_hovered.background.is_some());
    assert!(header_dark_pressed.background.is_some());
    assert_eq!(radius(&header_dark_pressed), 8.0);
}
