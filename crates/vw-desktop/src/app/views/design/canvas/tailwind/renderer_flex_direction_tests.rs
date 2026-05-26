//! Tailwind 渲染器测试模块，覆盖设计画布中布局、文本、媒体和视觉效果的回归场景。

use iced::{Point, Rectangle};

use super::{
    bounds_for_path, child_layouts, effective_divide_x_reverse, effective_divide_y_reverse,
    hit_test_path,
};
use crate::app::views::design::canvas::tailwind::{TailwindParser, dom::parse_html};

#[test]
fn row_reverse_reorders_visual_positions_and_hit_test_paths() {
    let roots = parse_html(
        "<div class=\"flex flex-row-reverse justify-between w-20 h-4\"><div class=\"w-4 h-4\"></div><div class=\"w-4 h-4\"></div><div class=\"w-4 h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 80.0, height: 16.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");

    assert_eq!(children[0].1.x, 64.0);
    assert_eq!(children[1].1.x, 32.0);
    assert_eq!(children[2].1.x, 0.0);
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 0]), Some(children[0].1));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(4.0, 4.0)), Some(vec![0, 2]));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(68.0, 4.0)), Some(vec![0, 0]));
}

#[test]
fn column_reverse_reorders_visual_positions_and_keeps_cross_axis_alignment() {
    let roots = parse_html(
        "<div class=\"flex flex-col-reverse items-center w-20 h-20\"><div class=\"w-4 h-4\"></div><div class=\"w-4 h-8\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 80.0, height: 80.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");

    assert_eq!(children[0].1.x, 32.0);
    assert_eq!(children[0].1.y, 64.0);
    assert_eq!(children[1].1.x, 32.0);
    assert_eq!(children[1].1.y, 32.0);
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(36.0, 40.0)), Some(vec![0, 1]));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(36.0, 68.0)), Some(vec![0, 0]));
}

#[test]
fn reverse_flex_direction_flips_effective_divide_side() {
    let row_reverse = TailwindParser::parse("flex flex-row-reverse divide-x");
    assert!(effective_divide_x_reverse(&row_reverse));
    assert!(!effective_divide_y_reverse(&row_reverse));

    let column_reverse = TailwindParser::parse("flex flex-col-reverse divide-y");
    assert!(effective_divide_y_reverse(&column_reverse));
    assert!(!effective_divide_x_reverse(&column_reverse));

    let explicit_row_override =
        TailwindParser::parse("flex flex-row-reverse divide-x divide-x-reverse");
    assert!(!effective_divide_x_reverse(&explicit_row_override));
}

#[test]
fn reverse_layout_keeps_absolute_children_at_explicit_offsets() {
    let roots = parse_html(
        "<div class=\"relative flex flex-row-reverse w-20 h-20\"><div class=\"absolute left-0 top-0 w-4 h-4\"></div><div class=\"w-4 h-4\"></div><div class=\"w-4 h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 80.0, height: 80.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");

    assert_eq!(children[0].0, 0);
    assert_eq!(children[0].1.x, 0.0);
    assert_eq!(children[0].1.y, 0.0);
    assert_eq!(children[1].1.x, 64.0);
    assert_eq!(children[2].1.x, 48.0);
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(4.0, 4.0)), Some(vec![0, 0]));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(52.0, 4.0)), Some(vec![0, 2]));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(68.0, 4.0)), Some(vec![0, 1]));
}
