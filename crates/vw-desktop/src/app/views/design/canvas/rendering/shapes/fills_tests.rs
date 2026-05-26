#[test]
fn lerp_and_bilerp_interpolate_midpoints() {
    assert_eq!(super::lerp(2.0, 6.0, 0.25), 3.0);
    let point = super::bilerp_point(
        iced::Point::new(0.0, 0.0),
        iced::Point::new(10.0, 0.0),
        iced::Point::new(0.0, 10.0),
        iced::Point::new(10.0, 10.0),
        0.5,
        0.5,
    );
    assert_eq!((point.x, point.y), (5.0, 5.0));
}

#[test]
fn lerp_color_interpolates_channels() {
    let color = super::lerp_color(iced::Color::BLACK, iced::Color::WHITE, 0.5);
    assert_eq!((color.r, color.g, color.b, color.a), (0.5, 0.5, 0.5, 1.0));
}
