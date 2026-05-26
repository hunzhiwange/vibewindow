//! Tailwind 渲染器测试模块，覆盖设计画布中布局、文本、媒体和视觉效果的回归场景。

use iced::{Point, Rectangle};

use super::{bounds_for_path, hit_test_path};
use crate::app::views::design::canvas::tailwind::dom::parse_html;

#[test]
fn absolute_right_bottom_geometry_matches_hit_test() {
    let roots = parse_html(
        "<div class=\"relative w-40 h-20\"><div class=\"absolute right-4 bottom-2 w-8 h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 160.0, height: 80.0 };

    let rect = bounds_for_path(&roots, bounds, 1.0, &[0, 0]).expect("child bounds");
    assert_eq!(rect.x, 112.0);
    assert_eq!(rect.y, 56.0);
    assert_eq!(rect.width, 32.0);
    assert_eq!(rect.height, 16.0);

    let hit = hit_test_path(&roots, bounds, 1.0, Point::new(120.0, 60.0));
    assert_eq!(hit, Some(vec![0, 0]));
}

#[test]
fn nested_layout_bounds_and_hit_test_stay_aligned() {
    let roots = parse_html(
        "<div class=\"p-4\"><div class=\"w-8 h-8\"></div><div class=\"w-10 h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 200.0, height: 200.0 };

    let first = bounds_for_path(&roots, bounds, 1.0, &[0, 0]).expect("first child bounds");
    let second = bounds_for_path(&roots, bounds, 1.0, &[0, 1]).expect("second child bounds");

    assert_eq!(
        hit_test_path(&roots, bounds, 1.0, Point::new(first.x + 1.0, first.y + 1.0)),
        Some(vec![0, 0])
    );
    assert_eq!(
        hit_test_path(&roots, bounds, 1.0, Point::new(second.x + 1.0, second.y + 1.0)),
        Some(vec![0, 1])
    );
}

#[test]
fn auto_margins_center_fixed_size_nodes() {
    let roots = parse_html("<div class=\"w-20 h-10 mx-auto my-auto\"></div>");
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 200.0, height: 100.0 };

    let rect = bounds_for_path(&roots, bounds, 1.0, &[0]).expect("root bounds");
    assert_eq!(rect.x, 60.0);
    assert_eq!(rect.y, 30.0);
    assert_eq!(rect.width, 80.0);
    assert_eq!(rect.height, 40.0);
}
