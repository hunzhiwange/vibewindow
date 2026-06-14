#[test]
fn task_1176_test_module_is_wired() {}

use iced::Color;

use crate::app::Message;
use crate::app::views::design::models::ColorFormat;

#[test]
fn render_mini_color_picker_constructs_all_format_variants_when_not_picking() {
    for format in [ColorFormat::Hex, ColorFormat::Rgba, ColorFormat::Hsl, ColorFormat::Css] {
        let _element = super::render_mini_color_picker(
            Color::from_rgba(0.25, 0.5, 0.75, 0.33),
            format,
            false,
            |_| Message::None,
            |_| Message::None,
            || Message::None,
        );
    }
}

#[test]
fn render_mini_color_picker_constructs_active_eyedropper_and_edge_colors() {
    let _transparent = super::render_mini_color_picker(
        Color::from_rgba(0.0, 0.0, 0.0, 0.0),
        ColorFormat::Hex,
        true,
        |_| Message::None,
        |_| Message::None,
        || Message::None,
    );
    let _opaque = super::render_mini_color_picker(
        Color::from_rgba(1.0, 1.0, 1.0, 1.0),
        ColorFormat::Hsl,
        true,
        |_| Message::None,
        |_| Message::None,
        || Message::None,
    );
}
