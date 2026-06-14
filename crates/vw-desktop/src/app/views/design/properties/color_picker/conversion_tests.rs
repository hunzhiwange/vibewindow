#[test]
fn task_1170_test_module_is_wired() {}

use iced::Color;

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.001, "expected {actual} to be close to {expected}");
}

#[test]
fn rgba_formatters_round_channels_and_keep_alpha_precision() {
    assert_eq!(super::format_rgba_to_hex(0.0, 0.5, 1.0, 0.25), "#0080FF40");
    assert_eq!(super::format_rgba_to_hex(1.0, 0.0, 0.003, 1.0), "#FF0001FF");
    assert_eq!(super::format_rgba_to_css(0.0, 0.5, 1.0, 0.257), "rgba(0, 128, 255, 0.26)");
}

#[test]
fn css_parser_accepts_rgb_and_rgba_with_whitespace() {
    let rgb = super::parse_css_color(" rgb( 12, 34, 56 ) ").expect("rgb should parse");
    assert_close(rgb.r, 12.0 / 255.0);
    assert_close(rgb.g, 34.0 / 255.0);
    assert_close(rgb.b, 56.0 / 255.0);
    assert_close(rgb.a, 1.0);

    let rgba = super::parse_css_color("rgba(255, 128, 0, 0.5)").expect("rgba should parse");
    assert_close(rgba.r, 1.0);
    assert_close(rgba.g, 128.0 / 255.0);
    assert_close(rgba.b, 0.0);
    assert_close(rgba.a, 0.5);
}

#[test]
fn css_parser_rejects_unknown_or_incomplete_values() {
    assert!(super::parse_css_color("hsl(0, 100%, 50%)").is_none());
    assert!(super::parse_css_color("rgb(1, 2)").is_none());
    assert!(super::parse_css_color("rgba(1, 2, 3)").is_none());
    assert!(super::parse_css_color("rgba(1, nope, 3, 1)").is_none());
    assert!(super::parse_css_color("rgba(1, 2, 3, nope)").is_none());
}

#[test]
fn hex_parser_accepts_rgb_rgba_and_rejects_bad_hex() {
    let rgb = super::parse_color("#336699").expect("rgb hex should parse");
    assert_close(rgb.r, 0x33 as f32 / 255.0);
    assert_close(rgb.g, 0x66 as f32 / 255.0);
    assert_close(rgb.b, 0x99 as f32 / 255.0);
    assert_close(rgb.a, 1.0);

    let rgba = super::parse_color("#33669980").expect("rgba hex should parse");
    assert_close(rgba.a, 128.0 / 255.0);

    assert!(super::parse_color("336699").is_none());
    assert!(super::parse_color("#zz6699").is_none());
    assert!(super::parse_color("#3366zz").is_none());
    assert!(super::parse_color("#336699zz").is_none());
}

#[test]
fn rgba_to_hsla_handles_gray_and_primary_branches() {
    let gray = super::rgba_to_hsla(Color::from_rgba(0.4, 0.4, 0.4, 0.75));
    assert_close(gray.0, 0.0);
    assert_close(gray.1, 0.0);
    assert_close(gray.2, 0.4);
    assert_close(gray.3, 0.75);

    let red = super::rgba_to_hsla(Color::from_rgb(1.0, 0.0, 0.0));
    assert_close(red.0, 0.0);
    assert_close(red.1, 1.0);
    assert_close(red.2, 0.5);

    let green = super::rgba_to_hsla(Color::from_rgb(0.0, 1.0, 0.0));
    assert_close(green.0, 120.0);

    let blue = super::rgba_to_hsla(Color::from_rgb(0.0, 0.0, 1.0));
    assert_close(blue.0, 240.0);

    let magenta = super::rgba_to_hsla(Color::from_rgb(1.0, 0.0, 0.5));
    assert_close(magenta.0, 330.0);
}

#[test]
fn hsla_to_rgba_covers_all_hue_sectors() {
    let cases = [
        (0.0, (1.0, 0.0, 0.0)),
        (60.0, (1.0, 1.0, 0.0)),
        (120.0, (0.0, 1.0, 0.0)),
        (180.0, (0.0, 1.0, 1.0)),
        (240.0, (0.0, 0.0, 1.0)),
        (300.0, (1.0, 0.0, 1.0)),
    ];

    for (h, (r, g, b)) in cases {
        let color = super::hsla_to_rgba(h, 1.0, 0.5, 0.3);
        assert_close(color.r, r);
        assert_close(color.g, g);
        assert_close(color.b, b);
        assert_close(color.a, 0.3);
    }
}

#[test]
fn format_percent_trims_trailing_zeroes() {
    assert_eq!(super::format_percent(100.0), "100");
    assert_eq!(super::format_percent(12.50), "12.5");
    assert_eq!(super::format_percent(0.125), "0.12");
    assert_eq!(super::format_percent(0.126), "0.13");
}
