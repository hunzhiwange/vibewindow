#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("timeline_layout_tests"));
}

use super::timeline_layout::TimelineLayoutFormatPreview;
use crate::apps::mindmap::state::TimelineLayoutFormat;
use iced::Color;

#[test]
fn timeline_layout_preview_stores_format_and_color() {
    for format in
        [TimelineLayoutFormat::UpDown, TimelineLayoutFormat::AllUp, TimelineLayoutFormat::AllDown]
    {
        let preview = TimelineLayoutFormatPreview { format, color: Color::from_rgb(0.2, 0.4, 0.6) };

        assert_eq!(preview.format, format);
        assert_eq!(preview.color, Color::from_rgb(0.2, 0.4, 0.6));
    }
}

#[test]
fn timeline_layout_formats_expose_expected_labels() {
    assert_eq!(TimelineLayoutFormat::UpDown.label(), "上下");
    assert_eq!(TimelineLayoutFormat::AllUp.label(), "全上");
    assert_eq!(TimelineLayoutFormat::AllDown.label(), "全下");
}
