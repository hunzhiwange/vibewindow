#[test]
fn polyline_geometry_requires_at_least_two_points() {
    assert!(super::build_polyline_geometry(&[]).is_none());
    let geometry =
        super::build_polyline_geometry(&[iced::Point::new(1.0, 2.0), iced::Point::new(3.0, 4.0)]);
    assert_eq!(geometry.as_deref(), Some("M 1.00 2.00 L 3.00 4.00"));
}
