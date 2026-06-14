use super::overlay_element::{compute_above_position, compute_point_above_position};
use iced::{Point, Rectangle, Size};

#[test]
fn above_position_keeps_raw_coordinates_without_snap() {
    let position = compute_above_position(
        Point::new(260.0, 40.0),
        Rectangle { x: 260.0, y: 40.0, width: 32.0, height: 20.0 },
        Rectangle::with_size(Size::new(300.0, 200.0)),
        Size::new(120.0, 80.0),
        6.0,
        false,
    );

    assert_eq!(position, Point::new(260.0, -46.0));
}

#[test]
fn above_position_snaps_to_viewport_edges() {
    let position = compute_above_position(
        Point::new(260.0, 30.0),
        Rectangle { x: 260.0, y: 30.0, width: 32.0, height: 20.0 },
        Rectangle { x: 10.0, y: 12.0, width: 300.0, height: 200.0 },
        Size::new(120.0, 80.0),
        6.0,
        true,
    );

    assert_eq!(position, Point::new(190.0, 12.0));
}

#[test]
fn point_above_position_centers_on_anchor_without_snap() {
    let position = compute_point_above_position(
        Point::new(80.0, 90.0),
        Rectangle::with_size(Size::new(300.0, 200.0)),
        Size::new(60.0, 40.0),
        10.0,
        false,
    );

    assert_eq!(position, Point::new(50.0, 40.0));
}

#[test]
fn point_above_position_snaps_when_anchor_is_near_edges() {
    let position = compute_point_above_position(
        Point::new(12.0, 8.0),
        Rectangle::with_size(Size::new(100.0, 80.0)),
        Size::new(40.0, 30.0),
        6.0,
        true,
    );

    assert_eq!(position, Point::new(0.0, 0.0));
}
