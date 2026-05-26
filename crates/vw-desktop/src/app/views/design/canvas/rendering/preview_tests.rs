#[test]
fn normalized_rect_uses_positive_size() {
    let rect = super::normalized_rect(iced::Point::new(10.0, 20.0), iced::Point::new(5.0, 7.0));
    assert_eq!((rect.x, rect.y, rect.width, rect.height), (5.0, 7.0, 5.0, 13.0));
}

#[test]
fn parse_hex_color_accepts_rgb_and_rejects_invalid() {
    let color = super::parse_hex_color("#ff8000").expect("valid color");
    assert_eq!((color.r, color.g, color.b, color.a), (1.0, 128.0 / 255.0, 0.0, 1.0));
    assert!(super::parse_hex_color("#xyz").is_none());
}
