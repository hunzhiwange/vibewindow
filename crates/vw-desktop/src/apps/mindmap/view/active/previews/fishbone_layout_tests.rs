use super::FishboneLayoutFormatPreview;
use crate::apps::mindmap::state::FishboneLayoutFormat;
use iced::Color;

#[test]
fn fishbone_layout_preview_preserves_head_right_inputs() {
    let preview = FishboneLayoutFormatPreview {
        format: FishboneLayoutFormat::HeadRight,
        color: Color::from_rgb(0.3, 0.5, 0.7),
    };

    assert_eq!(preview.format, FishboneLayoutFormat::HeadRight);
    assert_eq!(preview.color, Color::from_rgb(0.3, 0.5, 0.7));
    assert!(format!("{preview:?}").contains("HeadRight"));
}

#[test]
fn fishbone_layout_preview_preserves_head_left_inputs() {
    let preview = FishboneLayoutFormatPreview {
        format: FishboneLayoutFormat::HeadLeft,
        color: Color::from_rgba(0.7, 0.5, 0.3, 0.8),
    };
    let copied = preview;

    assert_eq!(copied.format, FishboneLayoutFormat::HeadLeft);
    assert_eq!(copied.color, Color::from_rgba(0.7, 0.5, 0.3, 0.8));
}
