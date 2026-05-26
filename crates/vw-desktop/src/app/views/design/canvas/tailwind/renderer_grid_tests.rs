//! Tailwind 渲染器测试模块，覆盖设计画布中布局、文本、媒体和视觉效果的回归场景。

use iced::{Point, Rectangle};

use super::{bounds_for_path, child_layouts, hit_test_path};
use crate::app::views::design::canvas::tailwind::dom::parse_html;

#[test]
fn grid_auto_placement_wraps_last_row_and_keeps_root_height() {
    let roots = parse_html(
        "<div class=\"grid grid-cols-2 gap-2 w-20\"><div class=\"h-4\"></div><div class=\"h-4\"></div><div class=\"h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 80.0, height: 200.0 };

    let root = bounds_for_path(&roots, bounds, 1.0, &[0]).expect("root bounds");
    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");

    assert_eq!(root.height, 40.0);
    assert_eq!(children[0].1.width, 36.0);
    assert_eq!(children[1].1.x, 44.0);
    assert_eq!(children[2].1.y, 24.0);
    assert_eq!(bounds_for_path(&roots, bounds, 1.0, &[0, 2]), Some(children[2].1));
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(4.0, 28.0)), Some(vec![0, 2]));
}

#[test]
fn grid_cells_keep_uniform_width_across_rows() {
    let roots = parse_html(
        "<div class=\"grid grid-cols-3 gap-x-4 w-32\"><div class=\"h-4\"></div><div class=\"h-8\"></div><div class=\"h-4\"></div><div class=\"h-4\"></div></div>",
    );
    let bounds = Rectangle { x: 0.0, y: 0.0, width: 128.0, height: 200.0 };

    let (_, children) = child_layouts(&roots[0], bounds, 1.0).expect("child layout");

    assert_eq!(children[0].1.width, 32.0);
    assert_eq!(children[1].1.x, 48.0);
    assert_eq!(children[2].1.x, 96.0);
    assert_eq!(children[3].1.x, 0.0);
    assert_eq!(children[3].1.y, 32.0);
    assert_eq!(hit_test_path(&roots, bounds, 1.0, Point::new(4.0, 36.0)), Some(vec![0, 3]));
}
