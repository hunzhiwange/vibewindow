use super::styles::{
    is_dark_theme, message_body_text_color, message_meta_text_color, neutral_card_surface,
};
use iced::Theme;

#[test]
fn dark_theme_detection_tracks_palette_background() {
    assert!(is_dark_theme(&Theme::Dark));
    assert!(!is_dark_theme(&Theme::Light));
}

#[test]
fn message_colors_are_distinct_for_user_and_assistant() {
    assert_ne!(
        message_body_text_color(&Theme::Light, true),
        message_body_text_color(&Theme::Light, false)
    );
    assert_ne!(
        message_meta_text_color(&Theme::Dark, true),
        message_meta_text_color(&Theme::Dark, false)
    );
}

#[test]
fn neutral_card_surface_has_visible_border() {
    let (surface, border) = neutral_card_surface(&Theme::Dark);
    assert_ne!(surface, border);
}
