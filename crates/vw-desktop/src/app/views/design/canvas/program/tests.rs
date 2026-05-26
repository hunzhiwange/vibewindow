#[test]
fn frame_header_label_uses_default_for_blank_values() {
    assert_eq!(super::frame_header_label(None), "画板");
    assert_eq!(super::frame_header_label(Some("   ")), "画板");
    assert_eq!(super::frame_header_label(Some("Home")), "Home");
}

#[test]
fn mix_color_clamps_blend_amount() {
    let base = iced::Color::from_rgb(0.0, 0.0, 0.0);
    let overlay = iced::Color::from_rgb(1.0, 0.5, 0.25);
    let color = super::mix_color(base, overlay, 2.0);
    assert_eq!((color.r, color.g, color.b), (1.0, 0.5, 0.25));
}
