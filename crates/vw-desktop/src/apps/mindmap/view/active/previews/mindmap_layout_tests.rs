use super::LayoutFormatPreview;
use crate::apps::mindmap::state::MindMapLayoutFormat;
use iced::Color;

#[test]
fn mindmap_layout_preview_preserves_each_layout_format() {
    for format in [
        MindMapLayoutFormat::RightAligned,
        MindMapLayoutFormat::LeftAligned,
        MindMapLayoutFormat::Bidirectional,
    ] {
        let preview = LayoutFormatPreview { format, color: Color::from_rgb(0.4, 0.2, 0.8) };
        let copied = preview;

        assert_eq!(copied.format, format);
        assert_eq!(copied.color, Color::from_rgb(0.4, 0.2, 0.8));
        assert!(format!("{copied:?}").contains("LayoutFormatPreview"));
    }
}
