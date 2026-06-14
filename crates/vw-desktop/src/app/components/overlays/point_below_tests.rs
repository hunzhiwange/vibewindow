use super::{PointBelowOverlay, compute_point_below_position};
use iced::widget::text;
use iced::{Element, Point, Rectangle, Size, Theme};

#[derive(Debug, Clone, PartialEq)]
enum TestMessage {
    Close,
}

fn label(value: &'static str) -> Element<'static, TestMessage> {
    text(value).into()
}

#[test]
fn point_below_overlay_new_sets_hidden_defaults() {
    let overlay = PointBelowOverlay::new(label("content"), label("overlay"));

    assert!(!overlay.show);
    assert_eq!(overlay.anchor, Point::ORIGIN);
    assert_eq!(overlay.gap, 0.0);
    assert!(overlay.snap_within_viewport);
    assert!(!overlay.snap_within_target_bounds);
    assert!(overlay.on_close.is_none());
    assert!(overlay.capture_outside_click);
}

#[test]
fn point_below_overlay_builder_updates_all_flags() {
    let overlay = PointBelowOverlay::new(label("content"), label("overlay"))
        .show(true)
        .anchor(Point::new(18.0, 24.0))
        .gap(5.0)
        .snap_within_viewport(false)
        .snap_within_target_bounds(true)
        .capture_outside_click(false)
        .on_close(TestMessage::Close);

    assert!(overlay.show);
    assert_eq!(overlay.anchor, Point::new(18.0, 24.0));
    assert_eq!(overlay.gap, 5.0);
    assert!(!overlay.snap_within_viewport);
    assert!(overlay.snap_within_target_bounds);
    assert!(!overlay.capture_outside_click);
    assert_eq!(overlay.on_close, Some(TestMessage::Close));
}

#[test]
fn point_below_position_uses_anchor_plus_gap_without_snap() {
    let position = compute_point_below_position(
        Point::new(90.0, 40.0),
        Rectangle { x: 30.0, y: 20.0, width: 120.0, height: 80.0 },
        Rectangle::with_size(Size::new(200.0, 160.0)),
        Size::new(70.0, 40.0),
        8.0,
        false,
        false,
    );

    assert_eq!(position, Point::new(90.0, 48.0));
}

#[test]
fn point_below_position_snaps_to_viewport() {
    let position = compute_point_below_position(
        Point::new(190.0, 150.0),
        Rectangle { x: 30.0, y: 20.0, width: 120.0, height: 80.0 },
        Rectangle::with_size(Size::new(200.0, 160.0)),
        Size::new(70.0, 40.0),
        8.0,
        true,
        false,
    );

    assert_eq!(position, Point::new(130.0, 120.0));
}

#[test]
fn point_below_position_target_snap_takes_precedence() {
    let position = compute_point_below_position(
        Point::new(190.0, 150.0),
        Rectangle { x: 30.0, y: 20.0, width: 120.0, height: 80.0 },
        Rectangle::with_size(Size::new(200.0, 160.0)),
        Size::new(70.0, 40.0),
        8.0,
        true,
        true,
    );

    assert_eq!(position, Point::new(80.0, 60.0));
}

#[test]
fn point_below_overlay_converts_to_element() {
    let _: Element<'static, TestMessage, Theme> =
        PointBelowOverlay::new(label("content"), label("overlay")).into();
}
