use super::style::{
    dash_segments_px, default_edge_stroke_color, ideal_text_color, node_border_width_px,
    priority_color, rgba_u32_to_color,
};
use crate::apps::mindmap::state::EdgeStyle;
use iced::Color;

#[test]
fn node_border_width_px_clamps_zoom_range() {
    assert_eq!(node_border_width_px(0.25), 2.0);
    assert_eq!(node_border_width_px(1.5), 3.0);
    assert_eq!(node_border_width_px(4.0), 4.0);
}

#[test]
fn rgba_u32_to_color_preserves_channels() {
    let color = rgba_u32_to_color(0x33669980);

    assert_eq!(color, Color::from_rgba8(0x33, 0x66, 0x99, 128.0 / 255.0));
}

#[test]
fn color_helpers_return_deterministic_values() {
    assert_eq!(priority_color(0), Color::from_rgba8(107, 114, 128, 1.0));
    assert_eq!(ideal_text_color(Color::WHITE), Color::from_rgba8(17, 24, 39, 1.0));
    assert_eq!(ideal_text_color(Color::BLACK), Color::WHITE);
    assert_eq!(default_edge_stroke_color(&[1, 2]), default_edge_stroke_color(&[1, 2]));
}

#[test]
fn dash_segments_match_edge_style() {
    assert!(dash_segments_px(EdgeStyle::Solid, 1.0).is_none());
    assert_eq!(dash_segments_px(EdgeStyle::Dashed, 1.0), Some(&[12.0, 12.0][..]));
    assert_eq!(dash_segments_px(EdgeStyle::Dotted, 1.0), Some(&[3.5, 10.5][..]));
}
