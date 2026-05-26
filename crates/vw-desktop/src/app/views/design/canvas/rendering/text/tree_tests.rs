#[test]
fn clamp_child_size_keeps_non_negative_bounds() {
    let size = super::clamp_child_size_to_content(
        iced::Size::new(20.0, 30.0),
        100.0,
        120.0,
        iced::Size::new(200.0, 200.0),
    );
    assert!(size.width >= 0.0);
    assert!(size.height >= 0.0);
}
