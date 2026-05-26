#[test]
fn formatted_outputs_preserve_alpha_percent() {
    let outputs = super::format_outputs(iced::Color::from_rgba8(0x11, 0x22, 0x33, 128.0 / 255.0));
    assert_eq!(outputs.hex, "#11223380");
    assert_eq!(outputs.alpha_percent, 50);
}
