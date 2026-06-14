use super::style::{
    dash_segments_px, default_edge_stroke_color, hsv_to_rgb, ideal_text_color,
    node_border_width_px, priority_color, rgba_u32_to_color,
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
    assert_eq!(priority_color(1), Color::from_rgba8(239, 68, 68, 1.0));
    assert_eq!(priority_color(2), Color::from_rgba8(249, 115, 22, 1.0));
    assert_eq!(priority_color(3), Color::from_rgba8(245, 158, 11, 1.0));
    assert_eq!(priority_color(4), Color::from_rgba8(234, 179, 8, 1.0));
    assert_eq!(priority_color(5), Color::from_rgba8(34, 197, 94, 1.0));
    assert_eq!(priority_color(6), Color::from_rgba8(20, 184, 166, 1.0));
    assert_eq!(priority_color(7), Color::from_rgba8(59, 130, 246, 1.0));
    assert_eq!(priority_color(8), Color::from_rgba8(99, 102, 241, 1.0));
    assert_eq!(priority_color(9), Color::from_rgba8(168, 85, 247, 1.0));
    assert_eq!(priority_color(10), Color::from_rgba8(34, 197, 94, 1.0));
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

#[test]
fn hsv_to_rgb_covers_each_hue_sector_and_clamps_inputs() {
    assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), Color::from_rgb(1.0, 0.0, 0.0));
    assert_eq!(hsv_to_rgb(60.0, 1.0, 1.0), Color::from_rgb(1.0, 1.0, 0.0));
    assert_eq!(hsv_to_rgb(120.0, 1.0, 1.0), Color::from_rgb(0.0, 1.0, 0.0));
    assert_eq!(hsv_to_rgb(180.0, 1.0, 1.0), Color::from_rgb(0.0, 1.0, 1.0));
    assert_eq!(hsv_to_rgb(240.0, 1.0, 1.0), Color::from_rgb(0.0, 0.0, 1.0));
    assert_eq!(hsv_to_rgb(300.0, 1.0, 1.0), Color::from_rgb(1.0, 0.0, 1.0));
    assert_eq!(hsv_to_rgb(-60.0, 2.0, 2.0), Color::from_rgb(1.0, 0.0, 1.0));
    assert_eq!(hsv_to_rgb(30.0, -1.0, 0.5), Color::from_rgb(0.5, 0.5, 0.5));
}
