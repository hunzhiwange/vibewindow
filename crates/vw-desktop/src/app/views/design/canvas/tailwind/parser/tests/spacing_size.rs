//! Tailwind 类名解析测试模块，验证布局、间距、尺寸与不支持变体的解析结果保持稳定。

use super::*;

#[test]
fn test_parse_spacing_shorthand_and_directional_utilities() {
    let style = TailwindParser::parse("p-4 px-6 py-2 pt-8 pr-10 pb-12 pl-1");

    assert_eq!(style.padding, Some(16.0));
    assert_eq!(style.padding_top, Some(32.0));
    assert_eq!(style.padding_right, Some(40.0));
    assert_eq!(style.padding_bottom, Some(48.0));
    assert_eq!(style.padding_left, Some(4.0));
}

#[test]
fn test_parse_margin_gap_directional_and_special_utilities() {
    let style = TailwindParser::parse("m-10 mx-auto my-6 mb-16 gap-8 gap-x-4 gap-y-10");

    assert_eq!(style.margin, Some(40.0));
    assert_eq!(style.margin_left, Some(-1.0));
    assert_eq!(style.margin_right, Some(-1.0));
    assert_eq!(style.margin_top, Some(24.0));
    assert_eq!(style.margin_bottom, Some(64.0));
    assert_eq!(style.gap_x, Some(16.0));
    assert_eq!(style.gap_y, Some(40.0));
}

#[test]
fn test_parse_offset_and_size_numeric_utilities() {
    let offset_style = TailwindParser::parse("-top-4 right-8 bottom-2 left-0");
    assert_eq!(offset_style.top, Some(-16.0));
    assert_eq!(offset_style.right, Some(32.0));
    assert_eq!(offset_style.bottom, Some(8.0));
    assert_eq!(offset_style.left, Some(0.0));

    let size_style = TailwindParser::parse("w-24 h-12 max-w-screen-lg");
    assert_eq!(size_style.width, Some(96.0));
    assert_eq!(size_style.height, Some(48.0));
    assert_eq!(size_style.max_width, Some(1024.0));
}

#[test]
fn test_parse_restricted_arbitrary_utilities() {
    let style = TailwindParser::parse(
        "w-[320px] h-[160px] top-[12px] right-[24px] bottom-[8px] left-[4px] bg-[#1f2937]",
    );

    assert_eq!(style.width, Some(320.0));
    assert_eq!(style.height, Some(160.0));
    assert_eq!(style.top, Some(12.0));
    assert_eq!(style.right, Some(24.0));
    assert_eq!(style.bottom, Some(8.0));
    assert_eq!(style.left, Some(4.0));
    assert_eq!(style.background_color, Some(iced::Color::from_rgb8(0x1f, 0x29, 0x37)));
}

#[test]
fn test_parse_size_special_utilities() {
    let screen_style = TailwindParser::parse("w-screen h-screen");
    assert_eq!(screen_style.width, Some(-1.0));
    assert_eq!(screen_style.height, Some(-1.0));

    let auto_style = TailwindParser::parse("w-24 h-12 w-auto h-auto");
    assert_eq!(auto_style.width, None);
    assert_eq!(auto_style.height, None);
}
