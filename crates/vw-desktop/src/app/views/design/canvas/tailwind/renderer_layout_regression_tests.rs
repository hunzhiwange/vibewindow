//! Tailwind 渲染器测试模块，覆盖设计画布中布局、文本、媒体和视觉效果的回归场景。

use iced::{Point, Rectangle};

use super::{bounds_for_path, child_layouts, hit_test_path};
use crate::app::views::design::canvas::tailwind::dom::parse_html;

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.01, "expected {expected}, got {actual}");
}

fn assert_rect_close(actual: Rectangle, expected: Rectangle) {
    assert_close(actual.x, expected.x);
    assert_close(actual.y, expected.y);
    assert_close(actual.width, expected.width);
    assert_close(actual.height, expected.height);
}

fn assert_bounds_and_hit_test(
    roots: &[crate::app::views::design::canvas::tailwind::dom::TailwindNode],
    bounds: Rectangle,
    expected_path: &[usize],
    expected_rect: Rectangle,
    hit_point: Point,
) {
    let actual = bounds_for_path(roots, bounds, 1.0, expected_path).expect("bounds for path");
    assert_rect_close(actual, expected_rect);
    assert_eq!(hit_test_path(roots, bounds, 1.0, hit_point), Some(expected_path.to_vec()));
}

#[test]
fn flex_geometry_regression_matches_bounds_and_hit_test() {
    let flex_roots = parse_html(
        "<div class=\"flex justify-center w-40 h-10\"><div class=\"w-8 h-4\"></div><div class=\"w-4 h-4\"></div></div>",
    );
    let flex_bounds = Rectangle { x: 0.0, y: 0.0, width: 160.0, height: 40.0 };
    let (_, flex_children) =
        child_layouts(&flex_roots[0], flex_bounds, 1.0).expect("flex child layout");

    assert_rect_close(flex_children[0].1, Rectangle { x: 56.0, y: 0.0, width: 32.0, height: 16.0 });
    assert_rect_close(flex_children[1].1, Rectangle { x: 88.0, y: 0.0, width: 16.0, height: 16.0 });
    assert_bounds_and_hit_test(
        &flex_roots,
        flex_bounds,
        &[0, 0],
        flex_children[0].1,
        Point::new(60.0, 4.0),
    );
    assert_bounds_and_hit_test(
        &flex_roots,
        flex_bounds,
        &[0, 1],
        flex_children[1].1,
        Point::new(92.0, 4.0),
    );
}

#[test]
fn grid_geometry_regression_matches_bounds_and_hit_test() {
    let grid_roots = parse_html(
        "<div class=\"grid grid-cols-2 gap-2 w-20\"><div class=\"h-4\"></div><div class=\"h-4\"></div><div class=\"h-4\"></div></div>",
    );
    let grid_bounds = Rectangle { x: 0.0, y: 0.0, width: 80.0, height: 200.0 };
    let (_, grid_children) =
        child_layouts(&grid_roots[0], grid_bounds, 1.0).expect("grid child layout");

    assert_rect_close(grid_children[0].1, Rectangle { x: 0.0, y: 0.0, width: 36.0, height: 16.0 });
    assert_rect_close(grid_children[1].1, Rectangle { x: 44.0, y: 0.0, width: 36.0, height: 16.0 });
    assert_rect_close(grid_children[2].1, Rectangle { x: 0.0, y: 24.0, width: 36.0, height: 16.0 });
    assert_bounds_and_hit_test(
        &grid_roots,
        grid_bounds,
        &[0, 2],
        grid_children[2].1,
        Point::new(4.0, 28.0),
    );
}

#[test]
fn absolute_geometry_regression_matches_bounds_and_hit_test() {
    let absolute_roots = parse_html(
        "<div class=\"relative w-40 h-20\"><div class=\"absolute right-4 bottom-2 w-8 h-4\"></div></div>",
    );
    let absolute_bounds = Rectangle { x: 0.0, y: 0.0, width: 160.0, height: 80.0 };
    let (_, absolute_children) =
        child_layouts(&absolute_roots[0], absolute_bounds, 1.0).expect("absolute child layout");

    assert_rect_close(
        absolute_children[0].1,
        Rectangle { x: 112.0, y: 56.0, width: 32.0, height: 16.0 },
    );
    assert_bounds_and_hit_test(
        &absolute_roots,
        absolute_bounds,
        &[0, 0],
        absolute_children[0].1,
        Point::new(120.0, 60.0),
    );
}

#[test]
fn svg_geometry_regression_matches_bounds_and_hit_test() {
    let svg_roots = parse_html(
        "<svg class=\"w-20\" viewBox=\"0 0 24 12\"><path d=\"M0 0 L24 0 L24 12 Z\" /></svg>",
    );
    let media_bounds = Rectangle { x: 0.0, y: 0.0, width: 240.0, height: 120.0 };
    assert_bounds_and_hit_test(
        &svg_roots,
        media_bounds,
        &[0],
        Rectangle { x: 0.0, y: 0.0, width: 80.0, height: 40.0 },
        Point::new(12.0, 6.0),
    );
}

#[test]
fn img_geometry_regression_matches_bounds_and_hit_test() {
    let img_roots = parse_html("<img class=\"w-8\" src=\"hero.png\" />");
    let media_bounds = Rectangle { x: 0.0, y: 0.0, width: 240.0, height: 120.0 };
    assert_bounds_and_hit_test(
        &img_roots,
        media_bounds,
        &[0],
        Rectangle { x: 0.0, y: 0.0, width: 32.0, height: 32.0 },
        Point::new(16.0, 16.0),
    );
}
