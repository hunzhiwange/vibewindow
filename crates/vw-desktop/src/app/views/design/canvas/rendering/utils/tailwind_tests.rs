#[test]
fn square_with_large_radius_resolves_to_circle() {
    let (kind, radius) = super::resolved_shape_kind(
        iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(20.0, 20.0)),
        10.0,
    );
    assert_eq!(kind, "circle");
    assert_eq!(radius, 10.0);
}
