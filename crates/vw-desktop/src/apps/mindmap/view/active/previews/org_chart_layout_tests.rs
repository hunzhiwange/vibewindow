use super::OrgChartLayoutFormatPreview;
use crate::apps::mindmap::state::OrgChartLayoutFormat;
use iced::Color;

#[test]
fn org_chart_layout_preview_preserves_top_down_inputs() {
    let preview = OrgChartLayoutFormatPreview {
        format: OrgChartLayoutFormat::TopDown,
        color: Color::from_rgb(0.2, 0.6, 0.4),
    };

    assert_eq!(preview.format, OrgChartLayoutFormat::TopDown);
    assert_eq!(preview.color, Color::from_rgb(0.2, 0.6, 0.4));
    assert!(format!("{preview:?}").contains("TopDown"));
}

#[test]
fn org_chart_layout_preview_preserves_left_right_inputs() {
    let preview = OrgChartLayoutFormatPreview {
        format: OrgChartLayoutFormat::LeftRight,
        color: Color::from_rgba(0.9, 0.1, 0.2, 0.7),
    };
    let copied = preview;

    assert_eq!(copied.format, OrgChartLayoutFormat::LeftRight);
    assert_eq!(copied.color, Color::from_rgba(0.9, 0.1, 0.2, 0.7));
}
