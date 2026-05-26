#[test]
fn parse_fill_items_returns_empty_for_missing_or_invalid_fill() {
    assert!(super::parse_fill_items(&None).is_empty());
    assert!(super::parse_fill_items(&Some(serde_json::json!(true))).is_empty());
}

#[test]
fn cursor_to_uv_raw_clamps_to_rect() {
    let rect = iced::Rectangle::new(iced::Point::new(10.0, 20.0), iced::Size::new(100.0, 200.0));
    let uv = super::cursor_to_uv_raw(60.0, 120.0, rect);
    assert!((uv.0 - 0.5).abs() < 0.0001);
    assert!((uv.1 - 0.5).abs() < 0.0001);
}
