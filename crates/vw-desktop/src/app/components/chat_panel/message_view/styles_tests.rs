use super::styles::{
    is_dark_theme, message_body_text_color, message_meta_text_color, neutral_card_surface,
    subtle_card_shadow, think_block_text_color, thinking_status_text, user_bubble_surface,
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

#[test]
fn card_surfaces_change_between_light_and_dark_themes() {
    assert_ne!(neutral_card_surface(&Theme::Light), neutral_card_surface(&Theme::Dark));
    assert_ne!(user_bubble_surface(&Theme::Light), user_bubble_surface(&Theme::Dark));
}

#[test]
fn subtle_shadow_is_stronger_in_dark_theme() {
    let light = subtle_card_shadow(&Theme::Light);
    let dark = subtle_card_shadow(&Theme::Dark);

    assert!(dark.color.a > light.color.a);
    assert_eq!(dark.offset, light.offset);
    assert_eq!(dark.blur_radius, light.blur_radius);
}

#[test]
fn thinking_status_text_builds_for_empty_label() {
    let _ = thinking_status_text("", 1234, 2);
    assert_ne!(think_block_text_color(&Theme::Light), think_block_text_color(&Theme::Dark));
}
