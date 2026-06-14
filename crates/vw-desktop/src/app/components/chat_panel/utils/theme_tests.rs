use super::theme::{
    chat_secondary_muted_text_color, chat_secondary_subtle_text_color, chat_secondary_text_color,
    is_dark_theme, mix_color, muted_icon_color,
};
use iced::{Color, Theme};

#[test]
fn theme_dark_detection_matches_builtin_themes() {
    assert!(is_dark_theme(&Theme::Dark));
    assert!(!is_dark_theme(&Theme::Light));
}

#[test]
fn mix_color_interpolates_channels() {
    let mixed = mix_color(Color::BLACK, Color::WHITE, 0.5);

    assert_eq!(mixed, Color::from_rgba(0.5, 0.5, 0.5, 1.0));
}

#[test]
fn secondary_text_color_is_visible() {
    assert!(chat_secondary_text_color(&Theme::Dark).a > 0.0);
    assert!(chat_secondary_muted_text_color(&Theme::Dark).a > 0.0);
    assert!(chat_secondary_subtle_text_color(&Theme::Light).a > 0.0);
    assert!(muted_icon_color(&Theme::Light).a > 0.0);
}
