//! Tailwind 渲染器测试模块，覆盖设计画布中布局、文本、媒体和视觉效果的回归场景。

use iced::{Point, Rectangle};

use super::{bounds_for_path, hit_test_path};
use crate::app::views::design::canvas::tailwind::dom::parse_html;

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.01, "expected {expected}, got {actual}");
}

#[test]
fn svg_without_explicit_size_uses_viewbox_dimensions() {
    let roots = parse_html("<svg viewBox=\"0 0 24 12\"><path d=\"M0 0 L24 0 L24 12 Z\" /></svg>");
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 240.0, height: 120.0 };

    let rect = bounds_for_path(&roots, bounds, 1.0, &[0]).expect("svg bounds");
    assert_close(rect.width, 24.0);
    assert_close(rect.height, 12.0);

    let hit = hit_test_path(&roots, bounds, 1.0, Point::new(12.0, 6.0));
    assert_eq!(hit, Some(vec![0]));
}

#[test]
fn svg_single_dimension_preserves_viewbox_aspect_ratio() {
    let roots = parse_html(
        "<svg class=\"w-20\" viewBox=\"0 0 24 12\"><path d=\"M0 0 L24 0 L24 12 Z\" /></svg>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 240.0, height: 120.0 };

    let rect = bounds_for_path(&roots, bounds, 1.0, &[0]).expect("svg bounds");
    assert_close(rect.width, 80.0);
    assert_close(rect.height, 40.0);
}

#[test]
fn img_single_dimension_falls_back_to_square_bounds() {
    let roots = parse_html("<img class=\"w-8\" src=\"hero.png\" />");
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 240.0, height: 120.0 };

    let rect = bounds_for_path(&roots, bounds, 1.0, &[0]).expect("img bounds");
    assert_close(rect.width, 32.0);
    assert_close(rect.height, 32.0);

    let hit = hit_test_path(&roots, bounds, 1.0, Point::new(16.0, 16.0));
    assert_eq!(hit, Some(vec![0]));
}
