use super::color_ops::rgba_u32_from_color;
use iced::Color;

#[test]
fn rgba_u32_from_color_clamps_and_rounds_channels() {
    let color = Color { r: 1.2, g: 0.5, b: -0.2, a: 0.0 };

    assert_eq!(rgba_u32_from_color(color), 0xFF800000);
}
