//! Tailwind 渲染器测试模块，覆盖设计画布中布局、文本、媒体和视觉效果的回归场景。

use iced::{Point, Rectangle};

use super::{bounds_for_path, hit_test_path, resolve_text_layout};
use crate::app::views::design::canvas::rendering::utils::{
    apply_text_transform, compute_line_width,
};
use crate::app::views::design::canvas::tailwind::dom::parse_html;
use crate::app::views::design::canvas::tailwind::parser::TailwindParser;

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.01, "expected {expected}, got {actual}");
}

#[test]
fn resolve_text_layout_applies_tailwind_text_details() {
    let style = TailwindParser::parse(
        "text-2xl leading-tight tracking-wide underline uppercase text-center text-blue-500 opacity-75",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 240.0, height: 120.0 };

    let layout = resolve_text_layout("hello world", bounds, 1.0, &style);

    assert_eq!(apply_text_transform("hello world", style.text_transform.as_deref()), "HELLO WORLD");
    let expected_color = style.text_color.expect("text color");
    assert_close(layout.color.a, expected_color.a * 0.75);
    assert_close(layout.font_size, style.font_size.expect("font size"));
    assert_close(layout.letter_spacing, style.letter_spacing.expect("letter spacing"));
    assert_close(layout.line_height, layout.font_size * style.line_height.expect("line height"));
    assert_eq!(layout.decoration.as_deref(), Some("underline"));
    assert_eq!(layout.lines, vec!["HELLO WORLD".to_string()]);
    assert_close(
        compute_line_width("HELLO WORLD", layout.font_size, layout.letter_spacing),
        super::text_layout_size(&layout).width,
    );
}

#[test]
fn nested_text_bounds_and_hit_test_inherit_parent_typography() {
    let html = concat!(
        "<div class=\"w-[140px] text-2xl leading-tight tracking-wide uppercase\">",
        "<span><span>hello world tailwind</span></span>",
        "</div>"
    );
    let roots = parse_html(html);
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 140.0, height: 240.0 };
    let style = TailwindParser::parse("w-[140px] text-2xl leading-tight tracking-wide uppercase");
    let expected = resolve_text_layout(
        "hello world tailwind",
        Rectangle { x: 0.0, y: 0.0, width: 140.0, height: f32::INFINITY },
        1.0,
        &style,
    );

    let rect = bounds_for_path(&roots, bounds, 1.0, &[0, 0, 0, 0]).expect("text bounds");
    assert_close(rect.width, super::text_layout_size(&expected).width);
    assert_close(rect.height, super::text_layout_size(&expected).height);

    let hit = hit_test_path(
        &roots,
        bounds,
        1.0,
        Point::new(rect.x + rect.width / 2.0, rect.y + rect.height - 1.0),
    );
    assert_eq!(hit, Some(vec![0, 0, 0, 0]));
}
