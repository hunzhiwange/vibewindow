use super::theme::{chat_secondary_text_color, is_dark_theme, mix_color};
use iced::{Color, Theme};

#[test]
fn theme_dark_detection_matches_builtin_themes() {
    assert!(is_dark_theme(&Theme::Dark));
    assert!(!is_dark_theme(&Theme::Light));
}

#[test]
fn mix_color_interpolates_channels() {
    let mixed = mix_color(Color::BLACK, Color::WHITE, 0.5);

    assert!(mixed.r > 0.0 && mixed.r < 1.0);
}

#[test]
fn secondary_text_color_is_visible() {
    assert!(chat_secondary_text_color(&Theme::Dark).a > 0.0);
}
