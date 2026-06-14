#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("tree_layout_tests"));
}

use super::tree_layout::TreeLayoutFormatPreview;
use crate::apps::mindmap::state::TreeLayoutFormat;
use iced::Color;

#[test]
fn tree_layout_preview_stores_format_and_color() {
    for format in [
        TreeLayoutFormat::FanDown,
        TreeLayoutFormat::SymmetricSplit,
        TreeLayoutFormat::LeftAligned,
        TreeLayoutFormat::RightAligned,
    ] {
        let preview = TreeLayoutFormatPreview { format, color: Color::from_rgb(0.7, 0.3, 0.2) };

        assert_eq!(preview.format, format);
        assert_eq!(preview.color, Color::from_rgb(0.7, 0.3, 0.2));
    }
}

#[test]
fn tree_layout_formats_expose_expected_labels() {
    assert_eq!(TreeLayoutFormat::SymmetricSplit.label(), "左右对称分支");
    assert_eq!(TreeLayoutFormat::FanDown.label(), "顶部中心多分支");
    assert_eq!(TreeLayoutFormat::LeftAligned.label(), "左侧单边分支");
    assert_eq!(TreeLayoutFormat::RightAligned.label(), "右侧单边分支");
}
