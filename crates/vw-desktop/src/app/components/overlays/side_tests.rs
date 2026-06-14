use super::side::{SideOverlay, compute_side_overlay_position, side_overlay_max_width};
use iced::widget::text;
use iced::{Element, Point, Rectangle, Size};

#[derive(Clone, Debug, PartialEq, Eq)]
enum TestMessage {
    Close,
}

#[test]
fn builder_converts_to_element_with_all_options() {
    let overlay = SideOverlay::new(text("content"), text("overlay"))
        .show(true)
        .gap(8.0)
        .min_x(12.0)
        .min_y(16.0)
        .align_y_start(true)
        .snap_within_viewport(false)
        .on_close(TestMessage::Close);

    let element: Element<'_, TestMessage> = overlay.into();

    std::hint::black_box(element);
}

#[test]
fn max_width_uses_larger_side_when_snapping() {
    let bounds = Size::new(320.0, 200.0);
    let position = Point::new(220.0, 40.0);
    let target = Rectangle { x: 220.0, y: 40.0, width: 60.0, height: 24.0 };

    let width = side_overlay_max_width(bounds, position, target, 8.0, true);

    assert_eq!(width, 212.0);
}

#[test]
fn max_width_uses_full_bounds_without_snapping() {
    let bounds = Size::new(320.0, 200.0);
    let position = Point::new(220.0, 40.0);
    let target = Rectangle { x: 220.0, y: 40.0, width: 60.0, height: 24.0 };

    let width = side_overlay_max_width(bounds, position, target, 8.0, false);

    assert_eq!(width, 320.0);
}

#[test]
fn position_prefers_right_when_there_is_room() {
    let point = compute_side_overlay_position(
        Point::new(40.0, 30.0),
        Rectangle { x: 40.0, y: 30.0, width: 80.0, height: 24.0 },
        Rectangle::with_size(Size::new(320.0, 220.0)),
        Size::new(120.0, 80.0),
        10.0,
        0.0,
        0.0,
        true,
        false,
    );

    assert_eq!(point, Point::new(130.0, 30.0));
}

#[test]
fn position_falls_back_left_when_right_side_is_tight() {
    let point = compute_side_overlay_position(
        Point::new(220.0, 50.0),
        Rectangle { x: 220.0, y: 50.0, width: 70.0, height: 24.0 },
        Rectangle::with_size(Size::new(320.0, 220.0)),
        Size::new(140.0, 80.0),
        8.0,
        0.0,
        0.0,
        true,
        false,
    );

    assert_eq!(point, Point::new(72.0, 50.0));
}

#[test]
fn position_clamps_to_minimums_and_viewport() {
    let point = compute_side_overlay_position(
        Point::new(8.0, 190.0),
        Rectangle { x: 8.0, y: 190.0, width: 24.0, height: 20.0 },
        Rectangle::with_size(Size::new(180.0, 220.0)),
        Size::new(220.0, 90.0),
        8.0,
        12.0,
        18.0,
        true,
        false,
    );

    assert_eq!(point, Point::new(12.0, 130.0));
}

#[test]
fn position_aligns_to_min_y_when_requested() {
    let point = compute_side_overlay_position(
        Point::new(40.0, 90.0),
        Rectangle { x: 40.0, y: 90.0, width: 30.0, height: 20.0 },
        Rectangle::with_size(Size::new(220.0, 180.0)),
        Size::new(60.0, 40.0),
        4.0,
        0.0,
        24.0,
        true,
        true,
    );

    assert_eq!(point, Point::new(74.0, 24.0));
}

#[test]
fn position_can_extend_without_snapping() {
    let point = compute_side_overlay_position(
        Point::new(190.0, 170.0),
        Rectangle { x: 190.0, y: 170.0, width: 40.0, height: 20.0 },
        Rectangle::with_size(Size::new(220.0, 180.0)),
        Size::new(80.0, 40.0),
        6.0,
        10.0,
        10.0,
        false,
        false,
    );

    assert_eq!(point, Point::new(104.0, 170.0));
}
