use super::{BorderStylePreview, LineStylePreview};
use crate::apps::mindmap::state::EdgeStyle;
use iced::Color;

#[test]
fn line_style_preview_preserves_all_edge_styles() {
    for style in [EdgeStyle::Solid, EdgeStyle::Dashed, EdgeStyle::Dotted] {
        let preview = LineStylePreview { style, color: Color::from_rgb(0.2, 0.4, 0.6) };
        let copied = preview;

        assert_eq!(copied.style, style);
        assert_eq!(copied.color, Color::from_rgb(0.2, 0.4, 0.6));
        assert!(format!("{copied:?}").contains("LineStylePreview"));
    }
}

#[test]
fn border_style_preview_preserves_all_edge_styles() {
    for style in [EdgeStyle::Solid, EdgeStyle::Dashed, EdgeStyle::Dotted] {
        let preview = BorderStylePreview { style, color: Color::from_rgba(0.7, 0.5, 0.3, 0.9) };
        let copied = preview;

        assert_eq!(copied.style, style);
        assert_eq!(copied.color, Color::from_rgba(0.7, 0.5, 0.3, 0.9));
        assert!(format!("{copied:?}").contains("BorderStylePreview"));
    }
}
