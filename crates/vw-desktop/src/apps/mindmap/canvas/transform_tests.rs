use super::transform::{screen_from_world, world_from_screen};
use iced::{Point, Vector};

#[test]
fn screen_and_world_coordinates_round_trip() {
    let world = Point::new(-12.0, 48.5);
    let pan = Vector::new(30.0, -8.0);
    let zoom = 2.5;

    let screen = screen_from_world(world, pan, zoom);

    assert_eq!(world_from_screen(screen, pan, zoom), world);
}

#[test]
fn screen_from_world_applies_zoom_then_pan() {
    let screen = screen_from_world(Point::new(10.0, -4.0), Vector::new(5.0, 7.0), 3.0);

    assert_eq!(screen, Point::new(35.0, -5.0));
}
