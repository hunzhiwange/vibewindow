//! Tailwind 渲染器测试模块，覆盖设计画布中布局、文本、媒体和视觉效果的回归场景。

use iced::{Point, Rectangle};

use super::{bounds_for_path, hit_test_path};
use crate::app::views::design::canvas::tailwind::dom::parse_html;

#[test]
fn outline_expands_visual_bounds_and_hit_test() {
    let roots =
        parse_html("<div class=\"w-8 h-8 outline-2 outline-blue-500 outline-offset-1\"></div>");
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 200.0, height: 200.0 };

    let rect = bounds_for_path(&roots, bounds, 1.0, &[0]).expect("outline bounds");
    assert_eq!(rect.x, -3.0);
    assert_eq!(rect.y, -3.0);
    assert_eq!(rect.width, 38.0);
    assert_eq!(rect.height, 38.0);

    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(-1.0, 12.0)), Some(vec![0]));
}

#[test]
fn shadow_expands_visual_bounds_and_hit_test() {
    let roots = parse_html("<div class=\"w-8 h-8 shadow-md\"></div>");
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 200.0, height: 200.0 };

    let rect = bounds_for_path(&roots, bounds, 1.0, &[0]).expect("shadow bounds");
    assert_eq!(rect.x, -3.0);
    assert_eq!(rect.y, 0.0);
    assert_eq!(rect.width, 38.0);
    assert_eq!(rect.height, 39.0);

    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(34.0, 36.0)), Some(vec![0]));
}
