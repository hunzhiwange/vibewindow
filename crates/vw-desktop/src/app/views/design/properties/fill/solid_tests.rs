#[test]
fn task_1188_test_module_is_wired() {}

#[test]
fn parse_color_accepts_rgb_and_rgba_hex_and_rejects_invalid_input() {
    let red = super::parse_color("#ff0000").expect("rgb hex should parse");
    assert_eq!((red.r, red.g, red.b, red.a), (1.0, 0.0, 0.0, 1.0));

    let translucent = super::parse_color("#33669980").expect("rgba hex should parse");
    assert!((translucent.r - 0x33 as f32 / 255.0).abs() < 0.001);
    assert!((translucent.g - 0x66 as f32 / 255.0).abs() < 0.001);
    assert!((translucent.b - 0x99 as f32 / 255.0).abs() < 0.001);
    assert!((translucent.a - 128.0 / 255.0).abs() < 0.001);

    assert!(super::parse_color("336699").is_none());
    assert!(super::parse_color("#zz6699").is_none());
}

#[test]
fn rgba_helpers_round_and_fallback_consistently() {
    assert_eq!(super::format_rgba_to_hex(1.0, 0.5, 0.0, 0.25), "#FF800040");
    assert_eq!(super::parse_hex_to_rgba("bad"), (0.0, 0.0, 0.0, 1.0));

    let (r, g, b, a) = super::parse_hex_to_rgba("#0000ff40");
    assert_eq!((r, g, b), (0.0, 0.0, 1.0));
    assert!((a - 64.0 / 255.0).abs() < 0.001);
}

#[test]
fn render_builds_color_picker_element_for_valid_and_invalid_colors() {
    let _valid = super::render(
        "#123456".to_string(),
        0,
        vec![super::FillItem::Color("#123456".to_string())],
        "shape".to_string(),
        crate::app::views::design::models::ColorFormat::Hex,
        false,
    );
    let _fallback = super::render(
        "not-a-color".to_string(),
        0,
        Vec::new(),
        "shape".to_string(),
        crate::app::views::design::models::ColorFormat::Rgba,
        true,
    );
}
