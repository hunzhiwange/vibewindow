//! Tailwind 渲染器测试模块，覆盖设计画布中布局、文本、媒体和视觉效果的回归场景。

use iced::{Point, Rectangle};

use super::{bounds_for_path, child_layouts, hit_test_path};
use crate::app::views::design::canvas::tailwind::dom::parse_html;

#[test]
fn justify_center_row_layout_matches_bounds_and_hit_test() {
    let roots = parse_html(
        "<div class=\"flex justify-center w-40 h-10\"><div class=\"w-8 h-4\"></div><div class=\"w-4 h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 160.0, height: 40.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");
    let first = children[0].1;
    let second = children[1].1;

    assert_eq!(first.x, 56.0);
    assert_eq!(first.y, 0.0);
    assert_eq!(second.x, 88.0);
    assert_eq!(second.y, 0.0);
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 0]), Some(first));
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 1]), Some(second));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(60.0, 4.0)), Some(vec![0, 0]));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(92.0, 4.0)), Some(vec![0, 1]));
}

#[test]
fn justify_between_adds_remaining_space_on_top_of_gap() {
    let roots = parse_html(
        "<div class=\"flex justify-between gap-x-4 w-40 h-10\"><div class=\"w-4 h-4\"></div><div class=\"w-4 h-4\"></div><div class=\"w-4 h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 160.0, height: 40.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");
    let first = children[0].1;
    let second = children[1].1;
    let third = children[2].1;

    assert_eq!(first.x, 0.0);
    assert_eq!(second.x, 72.0);
    assert_eq!(third.x, 144.0);
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 2]), Some(third));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(8.0, 4.0)), Some(vec![0, 0]));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(80.0, 4.0)), Some(vec![0, 1]));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(152.0, 4.0)), Some(vec![0, 2]));
}

#[test]
fn justify_between_single_column_child_stays_at_start() {
    let roots = parse_html(
        "<div class=\"flex flex-col justify-between w-20 h-20\"><div class=\"w-4 h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 80.0, height: 80.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");
    let child = children[0].1;

    assert_eq!(child.x, 0.0);
    assert_eq!(child.y, 0.0);
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 0]), Some(child));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(4.0, 4.0)), Some(vec![0, 0]));
}

#[test]
fn justify_between_column_auto_height_has_no_extra_distribution() {
    let roots = parse_html(
        "<div class=\"flex flex-col justify-between w-20\"><div class=\"w-4 h-4\"></div><div class=\"w-4 h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 80.0, height: 120.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");
    let first = children[0].1;
    let second = children[1].1;
    let root = bounds_for_path(&roots, bounds, 1.0, &[0]).expect("root bounds");

    assert_eq!(root.height, 32.0);
    assert_eq!(first.y, 0.0);
    assert_eq!(second.y, 16.0);
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 1]), Some(second));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(4.0, 20.0)), Some(vec![0, 1]));
}
