use super::{LeftOverlay, PointLeftOverlay, compute_left_position, compute_point_left_position};
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
fn left_overlay_new_sets_hidden_defaults() {
    let overlay = LeftOverlay::new(label("content"), label("overlay"));

    assert!(!overlay.show);
    assert_eq!(overlay.gap, 0.0);
    assert!(overlay.snap_within_viewport);
    assert!(overlay.on_close.is_none());
}

#[test]
fn left_overlay_builder_updates_all_flags() {
    let overlay = LeftOverlay::new(label("content"), label("overlay"))
        .show(true)
        .gap(6.0)
        .snap_within_viewport(false)
        .on_close(TestMessage::Close);

    assert!(overlay.show);
    assert_eq!(overlay.gap, 6.0);
    assert!(!overlay.snap_within_viewport);
    assert_eq!(overlay.on_close, Some(TestMessage::Close));
}

#[test]
fn point_left_overlay_new_sets_hidden_defaults() {
    let overlay = PointLeftOverlay::new(label("content"), label("overlay"));

    assert!(!overlay.show);
    assert_eq!(overlay.anchor, Point::ORIGIN);
    assert_eq!(overlay.gap, 0.0);
    assert!(overlay.snap_within_viewport);
    assert!(overlay.on_close.is_none());
}

#[test]
fn point_left_overlay_builder_updates_all_flags() {
    let overlay = PointLeftOverlay::new(label("content"), label("overlay"))
        .show(true)
        .anchor(Point::new(24.0, 80.0))
        .gap(10.0)
        .snap_within_viewport(false)
        .on_close(TestMessage::Close);

    assert!(overlay.show);
    assert_eq!(overlay.anchor, Point::new(24.0, 80.0));
    assert_eq!(overlay.gap, 10.0);
    assert!(!overlay.snap_within_viewport);
    assert_eq!(overlay.on_close, Some(TestMessage::Close));
}

#[test]
fn left_position_centers_on_target_height_without_snap() {
    let position = compute_left_position(
        Point::new(90.0, 40.0),
        Rectangle { x: 90.0, y: 40.0, width: 30.0, height: 50.0 },
        Rectangle::with_size(Size::new(200.0, 120.0)),
        Size::new(70.0, 20.0),
        8.0,
        false,
    );

    assert_eq!(position, Point::new(12.0, 55.0));
}

#[test]
fn left_position_snaps_inside_viewport() {
    let position = compute_left_position(
        Point::new(40.0, 112.0),
        Rectangle { x: 40.0, y: 112.0, width: 30.0, height: 50.0 },
        Rectangle::with_size(Size::new(120.0, 120.0)),
        Size::new(70.0, 40.0),
        8.0,
        true,
    );

    assert_eq!(position, Point::new(0.0, 80.0));
}

#[test]
fn point_left_position_centers_on_anchor() {
    let position = compute_point_left_position(
        Point::new(100.0, 60.0),
        Rectangle::with_size(Size::new(180.0, 120.0)),
        Size::new(70.0, 40.0),
        8.0,
        false,
    );

    assert_eq!(position, Point::new(22.0, 40.0));
}

#[test]
fn point_left_position_snaps_inside_viewport() {
    let position = compute_point_left_position(
        Point::new(30.0, 118.0),
        Rectangle::with_size(Size::new(180.0, 120.0)),
        Size::new(70.0, 40.0),
        8.0,
        true,
    );

    assert_eq!(position, Point::new(0.0, 80.0));
}

#[test]
fn left_overlays_convert_to_elements() {
    let _: Element<'static, TestMessage, Theme> =
        LeftOverlay::new(label("content"), label("overlay")).into();
    let _: Element<'static, TestMessage, Theme> =
        PointLeftOverlay::new(label("content"), label("overlay")).into();
}
