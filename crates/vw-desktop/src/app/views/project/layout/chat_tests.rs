use iced::{Background, Color, Theme};

use crate::app::App;

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("chat_tests"));
}

fn render_chat_area(spacing: f32, corner_radius: f32, chat_content_pad: f32) {
    let (app, _task) = App::new();
    let element = super::chat_area(&app, spacing, corner_radius, chat_content_pad);

    std::hint::black_box(element);
}

#[test]
fn chat_area_builds_with_regular_layout_values() {
    render_chat_area(8.0, 12.0, 16.0);
}

#[test]
fn chat_area_builds_with_zero_layout_values() {
    render_chat_area(0.0, 0.0, 0.0);
}

#[test]
fn chat_area_builds_with_negative_radius() {
    render_chat_area(4.0, -2.0, 8.0);
}

#[test]
fn chat_area_style_uses_light_theme_surface() {
    let style = super::chat_area_style(&Theme::Light, 16.0);

    assert_eq!(style.background, Some(Background::Color(Color::from_rgba8(255, 255, 255, 0.985))));
    assert_eq!(style.border.width, 1.0);
    assert_eq!(style.border.color, Color::from_rgba8(224, 228, 236, 0.98));
    assert_eq!(style.border.radius.top_left, 16.0);
    assert_eq!(style.border.radius.top_right, 16.0);
    assert_eq!(style.border.radius.bottom_right, 16.0);
    assert_eq!(style.border.radius.bottom_left, 16.0);
}

#[test]
fn chat_area_style_uses_dark_theme_surface() {
    let style = super::chat_area_style(&Theme::Dark, 4.0);

    assert_eq!(style.background, Some(Background::Color(Color::from_rgba8(17, 18, 22, 0.985))));
    assert_eq!(style.border.width, 1.0);
    assert_ne!(style.border.color, Color::from_rgba8(224, 228, 236, 0.98));
    assert_eq!(style.border.radius.top_left, 4.0);
    assert_eq!(style.border.radius.top_right, 4.0);
    assert_eq!(style.border.radius.bottom_right, 4.0);
    assert_eq!(style.border.radius.bottom_left, 4.0);
}
