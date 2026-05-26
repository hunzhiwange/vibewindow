//! Tailwind 渲染器测试模块，覆盖设计画布中布局、文本、媒体和视觉效果的回归场景。

use iced::{Point, Rectangle};

use super::{bounds_for_path, child_layouts, hit_test_path};
use crate::app::views::design::canvas::tailwind::dom::parse_html;

#[test]
fn row_items_center_offsets_children_on_cross_axis() {
    let roots = parse_html(
        "<div class=\"flex items-center w-20 h-20\"><div class=\"w-4 h-4\"></div><div class=\"w-4 h-8\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 80.0, height: 80.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");
    let first = children[0].1;
    let second = children[1].1;

    assert_eq!(first.y, 32.0);
    assert_eq!(second.y, 24.0);
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 0]), Some(first));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(4.0, 36.0)), Some(vec![0, 0]));
}

#[test]
fn row_default_stretch_expands_auto_height_children() {
    let roots = parse_html(
        "<div class=\"flex w-20 h-20\"><div class=\"w-4\"></div><div class=\"w-4 h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 80.0, height: 80.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");
    let first = children[0].1;
    let second = children[1].1;

    assert_eq!(first.height, 80.0);
    assert_eq!(second.height, 16.0);
    assert_eq!(second.y, 0.0);
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 0]), Some(first));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(4.0, 64.0)), Some(vec![0, 0]));
}

#[test]
fn column_items_center_offsets_children_horizontally() {
    let roots = parse_html(
        "<div class=\"flex flex-col items-center w-20 h-20\"><div class=\"w-4 h-4\"></div><div class=\"w-8 h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 80.0, height: 80.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");
    let first = children[0].1;
    let second = children[1].1;

    assert_eq!(first.x, 32.0);
    assert_eq!(second.x, 24.0);
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 1]), Some(second));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(28.0, 20.0)), Some(vec![0, 1]));
}

#[test]
fn column_items_end_offsets_children_to_cross_end() {
    let roots = parse_html(
        "<div class=\"flex flex-col items-end w-20 h-20\"><div class=\"w-4 h-4\"></div><div class=\"w-8 h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 80.0, height: 80.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");
    let first = children[0].1;
    let second = children[1].1;

    assert_eq!(first.x, 64.0);
    assert_eq!(second.x, 48.0);
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 0]), Some(first));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(52.0, 20.0)), Some(vec![0, 1]));
}

#[test]
fn column_default_stretch_keeps_auto_width_children_full_width() {
    let roots =
        parse_html("<div class=\"flex flex-col w-20 h-20\"><div class=\"h-4\"></div></div>");
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 80.0, height: 80.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");
    let child = children[0].1;

    assert_eq!(child.x, 0.0);
    assert_eq!(child.width, 80.0);
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 0]), Some(child));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(40.0, 4.0)), Some(vec![0, 0]));
}
