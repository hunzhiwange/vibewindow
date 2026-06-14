use super::nodes::dashed_stroke;
use crate::apps::mindmap::state::EdgeStyle;
use iced::Color;
use iced::widget::canvas::{LineCap, Style};

#[test]
fn dashed_stroke_keeps_solid_edges_plain() {
    let stroke = dashed_stroke(EdgeStyle::Solid, Color::from_rgb(0.1, 0.2, 0.3), 2.5, 1.0);

    assert_eq!(stroke.width, 2.5);
    assert_eq!(stroke.line_dash.segments, &[] as &[f32]);
    assert!(matches!(stroke.style, Style::Solid(color) if color == Color::from_rgb(0.1, 0.2, 0.3)));
}

#[test]
fn dashed_stroke_applies_dash_and_dot_patterns() {
    let dashed = dashed_stroke(EdgeStyle::Dashed, Color::BLACK, 1.0, 1.0);
    assert_eq!(dashed.line_dash.segments, &[12.0, 12.0]);

    let dotted = dashed_stroke(EdgeStyle::Dotted, Color::BLACK, 1.0, 1.0);
    assert_eq!(dotted.line_dash.segments, &[3.5, 10.5]);
    assert!(matches!(dotted.line_cap, LineCap::Round));
}
