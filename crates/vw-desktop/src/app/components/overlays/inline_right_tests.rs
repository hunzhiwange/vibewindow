use super::{InlineRightOverlay, compute_inline_right_position};
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
fn inline_right_overlay_new_sets_hidden_defaults() {
    let overlay = InlineRightOverlay::new(label("content"), label("overlay"));

    assert!(!overlay.show);
    assert_eq!(overlay.gap, 0.0);
    assert!(overlay.snap_within_viewport);
    assert!(overlay.on_close.is_none());
}

#[test]
fn inline_right_overlay_builder_updates_all_flags() {
    let overlay = InlineRightOverlay::new(label("content"), label("overlay"))
        .show(true)
        .gap(12.0)
        .snap_within_viewport(false)
        .on_close(TestMessage::Close);

    assert!(overlay.show);
    assert_eq!(overlay.gap, 12.0);
    assert!(!overlay.snap_within_viewport);
    assert_eq!(overlay.on_close, Some(TestMessage::Close));
}

#[test]
fn inline_right_position_uses_right_edge_plus_gap_without_snap() {
    let position = compute_inline_right_position(
        Point::new(20.0, 40.0),
        Rectangle { x: 20.0, y: 40.0, width: 90.0, height: 30.0 },
        Rectangle::with_size(Size::new(180.0, 140.0)),
        Size::new(80.0, 70.0),
        8.0,
        false,
    );

    assert_eq!(position, Point::new(118.0, 40.0));
}

#[test]
fn inline_right_position_snaps_to_available_viewport() {
    let position = compute_inline_right_position(
        Point::new(140.0, 120.0),
        Rectangle { x: 140.0, y: 120.0, width: 70.0, height: 30.0 },
        Rectangle::with_size(Size::new(180.0, 140.0)),
        Size::new(80.0, 70.0),
        8.0,
        true,
    );

    assert_eq!(position, Point::new(100.0, 70.0));
}

#[test]
fn inline_right_overlay_converts_to_element() {
    let _: Element<'static, TestMessage, Theme> =
        InlineRightOverlay::new(label("content"), label("overlay")).into();
}
