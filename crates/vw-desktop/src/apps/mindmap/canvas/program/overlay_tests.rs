use super::super::HoverButtonKind;
use super::overlay::{hover_button_fill, point_in_circle};
use iced::{Color, Point};

#[test]
fn hover_button_fill_maps_add_buttons_and_skips_toggle() {
    assert_eq!(
        hover_button_fill(HoverButtonKind::AddChild),
        Some(Color::from_rgba8(34, 197, 94, 1.0))
    );
    assert_eq!(
        hover_button_fill(HoverButtonKind::AddSibling),
        Some(Color::from_rgba8(59, 130, 246, 1.0))
    );
    assert_eq!(hover_button_fill(HoverButtonKind::ToggleCollapse), None);
}

#[test]
fn point_in_circle_includes_boundary_and_rejects_outside_points() {
    let center = Point::new(10.0, 20.0);

    assert!(point_in_circle(center, center, 5.0));
    assert!(point_in_circle(Point::new(13.0, 24.0), center, 5.0));
    assert!(point_in_circle(Point::new(15.0, 20.0), center, 5.0));
    assert!(!point_in_circle(Point::new(15.1, 20.0), center, 5.0));
}
