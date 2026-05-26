//! Tailwind 渲染器测试模块，覆盖设计画布中布局、文本、媒体和视觉效果的回归场景。

use iced::Rectangle;

use super::bounds_for_path;
use crate::app::views::design::canvas::tailwind::dom::parse_html;

#[test]
fn max_width_constraints_center_with_auto_margins() {
    let roots = parse_html("<div class=\"max-w-screen-lg mx-auto h-4\"></div>");
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 1400.0, height: 100.0 };

    let rect = bounds_for_path(&roots, bounds, 1.0, &[0]).expect("root bounds");
    assert_eq!(rect.x, 188.0);
    assert_eq!(rect.width, 1024.0);
    assert_eq!(rect.height, 16.0);
}

#[test]
fn full_width_still_respects_max_width_constraint() {
    let roots = parse_html("<div class=\"w-full max-w-screen-lg mx-auto h-4\"></div>");
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 1400.0, height: 100.0 };

    let rect = bounds_for_path(&roots, bounds, 1.0, &[0]).expect("root bounds");
    assert_eq!(rect.x, 188.0);
    assert_eq!(rect.width, 1024.0);
    assert_eq!(rect.height, 16.0);
}
