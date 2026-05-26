//! Tailwind 渲染器测试模块，覆盖设计画布中布局、文本、媒体和视觉效果的回归场景。

use iced::{Point, Rectangle};

use super::{bounds_for_path, child_layouts, hit_test_path};
use crate::app::views::design::canvas::tailwind::dom::parse_html;

#[test]
fn flex_one_grows_items_into_remaining_space() {
    let roots = parse_html(
        "<div class=\"flex w-20 h-4\"><div class=\"flex-1 h-4\"></div><div class=\"flex-1 h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 80.0, height: 16.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");
    let first = children[0].1;
    let second = children[1].1;

    assert_eq!(first.width, 40.0);
    assert_eq!(second.x, 40.0);
    assert_eq!(second.width, 40.0);
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 1]), Some(second));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(60.0, 4.0)), Some(vec![0, 1]));
}

#[test]
fn shrink_zero_preserves_fixed_item_while_sibling_shrinks() {
    let roots = parse_html(
        "<div class=\"flex w-10 h-4\"><div class=\"w-6 shrink-0 h-4\"></div><div class=\"w-6 h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 40.0, height: 16.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");
    let first = children[0].1;
    let second = children[1].1;

    assert_eq!(first.width, 24.0);
    assert_eq!(second.x, 24.0);
    assert_eq!(second.width, 16.0);
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 0]), Some(first));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(30.0, 4.0)), Some(vec![0, 1]));
}

#[test]
fn basis_and_grow_share_remaining_space_after_gap() {
    let roots = parse_html(
        "<div class=\"flex gap-x-4 w-24 h-4\"><div class=\"basis-4 grow h-4\"></div><div class=\"basis-8 grow h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 96.0, height: 16.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");
    let first = children[0].1;
    let second = children[1].1;

    assert_eq!(first.width, 32.0);
    assert_eq!(second.x, 48.0);
    assert_eq!(second.width, 48.0);
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 1]), Some(second));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(72.0, 4.0)), Some(vec![0, 1]));
}
