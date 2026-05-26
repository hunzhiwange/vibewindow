//! Tailwind 渲染器测试模块，覆盖设计画布中布局、文本、媒体和视觉效果的回归场景。

use iced::{Point, Rectangle};

use super::{bounds_for_path, hit_test_path};
use crate::app::views::design::canvas::tailwind::dom::parse_html;

#[test]
fn translate_offsets_bounds_and_hit_test_consistently() {
    let roots = parse_html("<div class=\"w-8 h-8 translate-x-4 -translate-y-2\"></div>");
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 200.0, height: 200.0 };

    let rect = bounds_for_path(&roots, bounds, 1.0, &[0]).expect("translated bounds");
    assert_eq!(rect.x, 16.0);
    assert_eq!(rect.y, -8.0);
    assert_eq!(rect.width, 32.0);
    assert_eq!(rect.height, 32.0);

    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(20.0, 4.0)), Some(vec![0]));
}

#[test]
fn translate_moves_entire_subtree() {
    let roots = parse_html(
        "<div class=\"translate-x-4 translate-y-2\"><div class=\"w-8 h-8\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 200.0, height: 200.0 };

    let rect = bounds_for_path(&roots, bounds, 1.0, &[0, 0]).expect("translated child bounds");
    assert_eq!(rect.x, 16.0);
    assert_eq!(rect.y, 8.0);
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(17.0, 9.0)), Some(vec![0, 0]));
}
