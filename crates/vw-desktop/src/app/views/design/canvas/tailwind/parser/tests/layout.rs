//! Tailwind 类名解析测试模块，验证布局、间距、尺寸与不支持变体的解析结果保持稳定。

use super::*;

#[test]
fn test_parse_spacing_layout() {
    let style = TailwindParser::parse(
        "p-4 m-2 flex flex-col items-center justify-between gap-4 border-2 rounded-lg",
    );

    assert_eq!(style.padding, Some(16.0));
    assert_eq!(style.margin, Some(8.0));
    assert_eq!(style.display, Some("flex".to_string()));
    assert_eq!(style.flex_direction, Some("column".to_string()));
    assert_eq!(style.align_items, Some("center".to_string()));
    assert_eq!(style.justify_content, Some("space-between".to_string()));
    assert_eq!(style.gap_x, Some(16.0));
    assert_eq!(style.gap_y, Some(16.0));
    assert_eq!(style.border_width, Some(2.0));
    assert_eq!(style.border_radius, Some(TailwindColors::ROUNDED_LG));
}

#[test]
fn test_parse_reverse_flex_direction() {
    let row_reverse = TailwindParser::parse("flex flex-row-reverse");
    assert_eq!(row_reverse.flex_direction, Some("row-reverse".to_string()));

    let column_reverse = TailwindParser::parse("flex flex-col-reverse");
    assert_eq!(column_reverse.flex_direction, Some("column-reverse".to_string()));
}

#[test]
fn test_parse_flex_item_constraints() {
    let style = TailwindParser::parse("flex-1 grow-0 shrink-0 basis-8");

    assert_eq!(style.flex_grow, Some(0.0));
    assert_eq!(style.flex_shrink, Some(0.0));
    assert_eq!(style.flex_basis, Some(32.0));

    let flex_auto = TailwindParser::parse("flex-auto shrink");
    assert_eq!(flex_auto.flex_grow, Some(1.0));
    assert_eq!(flex_auto.flex_shrink, Some(1.0));
    assert_eq!(flex_auto.flex_basis, None);
}

#[test]
fn test_parse_position_utilities() {
    let style = TailwindParser::parse("relative absolute top-4 -left-2 right-8 bottom-px");

    assert_eq!(style.position, Some("absolute".to_string()));
    assert_eq!(style.top, Some(16.0));
    assert_eq!(style.left, Some(-8.0));
    assert_eq!(style.right, Some(32.0));
    assert_eq!(style.bottom, Some(1.0));
}

#[test]
fn test_parse_display_utilities() {
    assert_eq!(TailwindParser::parse("block").display, Some("block".to_string()));
    assert_eq!(TailwindParser::parse("inline-block").display, Some("inline-block".to_string()));
    assert_eq!(TailwindParser::parse("inline").display, Some("inline".to_string()));
    assert_eq!(TailwindParser::parse("inline-flex").display, Some("inline-flex".to_string()));
    assert_eq!(TailwindParser::parse("grid").display, Some("grid".to_string()));
    assert_eq!(TailwindParser::parse("hidden").display, Some("none".to_string()));
}
