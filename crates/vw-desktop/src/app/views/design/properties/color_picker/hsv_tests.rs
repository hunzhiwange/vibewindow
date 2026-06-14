#[test]
fn task_1171_test_module_is_wired() {}

use iced::Color;

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.001, "expected {actual} to be close to {expected}");
}

#[test]
fn from_color_handles_grayscale_and_black() {
    let gray = super::Hsv::from_color(Color::from_rgb(0.4, 0.4, 0.4));
    assert_close(gray.h, 0.0);
    assert_close(gray.s, 0.0);
    assert_close(gray.v, 0.4);

    let black = super::Hsv::from_color(Color::from_rgb(0.0, 0.0, 0.0));
    assert_close(black.h, 0.0);
    assert_close(black.s, 0.0);
    assert_close(black.v, 0.0);
}

#[test]
fn from_color_covers_red_green_blue_and_negative_red_hue() {
    let red = super::Hsv::from_color(Color::from_rgb(1.0, 0.0, 0.0));
    assert_close(red.h, 0.0);
    assert_close(red.s, 1.0);
    assert_close(red.v, 1.0);

    let green = super::Hsv::from_color(Color::from_rgb(0.0, 1.0, 0.0));
    assert_close(green.h, 120.0);

    let blue = super::Hsv::from_color(Color::from_rgb(0.0, 0.0, 1.0));
    assert_close(blue.h, 240.0);

    let magenta = super::Hsv::from_color(Color::from_rgb(1.0, 0.0, 0.5));
    assert_close(magenta.h, 330.0);
}

#[test]
fn to_color_covers_each_hue_sector_and_grayscale() {
    let cases = [
        (0.0, (1.0, 0.0, 0.0)),
        (60.0, (1.0, 1.0, 0.0)),
        (120.0, (0.0, 1.0, 0.0)),
        (180.0, (0.0, 1.0, 1.0)),
        (240.0, (0.0, 0.0, 1.0)),
        (300.0, (1.0, 0.0, 1.0)),
    ];

    for (h, (r, g, b)) in cases {
        let color = super::Hsv { h, s: 1.0, v: 1.0 }.to_color();
        assert_close(color.r, r);
        assert_close(color.g, g);
        assert_close(color.b, b);
    }

    let gray = super::Hsv { h: 25.0, s: 0.0, v: 0.35 }.to_color();
    assert_close(gray.r, 0.35);
    assert_close(gray.g, 0.35);
    assert_close(gray.b, 0.35);
    assert_close(gray.a, 1.0);
}
