use super::BracketLayoutFormatPreview;
use crate::apps::mindmap::state::BracketLayoutFormat;
use iced::Color;

#[test]
fn bracket_layout_preview_preserves_right_brace_inputs() {
    let preview = BracketLayoutFormatPreview {
        format: BracketLayoutFormat::BraceRight,
        color: Color::from_rgb(0.1, 0.2, 0.3),
    };

    assert_eq!(preview.format, BracketLayoutFormat::BraceRight);
    assert_eq!(preview.color, Color::from_rgb(0.1, 0.2, 0.3));
    assert!(format!("{preview:?}").contains("BraceRight"));
}

#[test]
fn bracket_layout_preview_preserves_left_brace_inputs() {
    let preview = BracketLayoutFormatPreview {
        format: BracketLayoutFormat::BraceLeft,
        color: Color::from_rgba(0.8, 0.7, 0.6, 0.5),
    };
    let copied = preview;

    assert_eq!(copied.format, BracketLayoutFormat::BraceLeft);
    assert_eq!(copied.color, Color::from_rgba(0.8, 0.7, 0.6, 0.5));
}
