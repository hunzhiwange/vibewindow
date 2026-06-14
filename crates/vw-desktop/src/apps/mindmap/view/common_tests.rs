use iced::{Color, Theme};

use super::common::{
    base_style, ideal_text_color, picker_style, priority_color, rgba_u32_to_color,
};

fn assert_color_close(actual: Color, expected: Color) {
    let epsilon = 0.0001;
    assert!((actual.r - expected.r).abs() < epsilon, "red differs: {actual:?}");
    assert!((actual.g - expected.g).abs() < epsilon, "green differs: {actual:?}");
    assert!((actual.b - expected.b).abs() < epsilon, "blue differs: {actual:?}");
    assert!((actual.a - expected.a).abs() < epsilon, "alpha differs: {actual:?}");
}

#[test]
fn rgba_u32_to_color_splits_all_channels() {
    assert_color_close(
        rgba_u32_to_color(0x12345680),
        Color::from_rgba8(0x12, 0x34, 0x56, 128.0 / 255.0),
    );
    assert_color_close(rgba_u32_to_color(0x00000000), Color::from_rgba8(0, 0, 0, 0.0));
    assert_color_close(rgba_u32_to_color(0xFFFFFFFF), Color::from_rgba8(255, 255, 255, 1.0));
}

#[test]
fn priority_color_maps_known_levels_and_fallback() {
    let cases = [
        (1, Color::from_rgba8(239, 68, 68, 1.0)),
        (2, Color::from_rgba8(249, 115, 22, 1.0)),
        (3, Color::from_rgba8(245, 158, 11, 1.0)),
        (4, Color::from_rgba8(234, 179, 8, 1.0)),
        (5, Color::from_rgba8(34, 197, 94, 1.0)),
        (6, Color::from_rgba8(20, 184, 166, 1.0)),
        (7, Color::from_rgba8(59, 130, 246, 1.0)),
        (8, Color::from_rgba8(99, 102, 241, 1.0)),
        (9, Color::from_rgba8(168, 85, 247, 1.0)),
        (10, Color::from_rgba8(34, 197, 94, 1.0)),
        (0, Color::from_rgba8(107, 114, 128, 1.0)),
        (11, Color::from_rgba8(107, 114, 128, 1.0)),
    ];

    for (level, expected) in cases {
        assert_color_close(priority_color(level), expected);
    }
}

#[test]
fn ideal_text_color_switches_at_luma_threshold() {
    assert_color_close(ideal_text_color(Color::WHITE), Color::from_rgba8(17, 24, 39, 1.0));
    assert_color_close(ideal_text_color(Color::BLACK), Color::WHITE);
    assert_color_close(ideal_text_color(Color::from_rgb(0.72, 0.72, 0.72)), Color::WHITE);
    assert_color_close(
        ideal_text_color(Color::from_rgb(0.73, 0.73, 0.73)),
        Color::from_rgba8(17, 24, 39, 1.0),
    );
}

#[test]
fn container_styles_use_theme_backgrounds() {
    let theme = Theme::Light;
    let picker = picker_style(&theme);
    let base = base_style(&theme);

    assert!(picker.background.is_some());
    assert_eq!(picker.border.width, 1.0);
    assert!(base.background.is_some());
    assert_eq!(base.border.width, 0.0);
    assert_color_close(base.border.color, Color::TRANSPARENT);
}
