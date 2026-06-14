#[test]
fn task_1172_test_module_is_wired() {}

use iced::Color;
use iced::widget::image;

fn rgba_pixels(handle: image::Handle) -> (u32, u32, Vec<u8>) {
    match handle {
        image::Handle::Rgba { width, height, pixels, .. } => (width, height, pixels.to_vec()),
        _ => panic!("expected rgba handle"),
    }
}

fn pixel(pixels: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let idx = ((y * width + x) * 4) as usize;
    [pixels[idx], pixels[idx + 1], pixels[idx + 2], pixels[idx + 3]]
}

#[test]
fn sv_image_maps_corners_to_expected_saturation_and_value() {
    let (width, height, pixels) = rgba_pixels(super::sv_image_handle(0.0));
    assert_eq!((width, height), (256, 256));
    assert_eq!(pixels.len(), (width * height * 4) as usize);

    assert_eq!(pixel(&pixels, width, 0, 0), [255, 255, 255, 255]);
    assert_eq!(pixel(&pixels, width, 255, 0), [255, 0, 0, 255]);
    assert_eq!(pixel(&pixels, width, 0, 255), [0, 0, 0, 255]);
    assert_eq!(pixel(&pixels, width, 255, 255), [0, 0, 0, 255]);
}

#[test]
fn sv_image_uses_requested_hue() {
    let (width, _height, pixels) = rgba_pixels(super::sv_image_handle(120.0));
    assert_eq!(pixel(&pixels, width, 255, 0), [0, 255, 0, 255]);
}

#[test]
fn hue_image_spans_red_green_blue_and_back_to_red() {
    let (width, height, pixels) = rgba_pixels(super::hue_image_handle());
    assert_eq!((width, height), (256, 16));
    assert_eq!(pixel(&pixels, width, 0, 0), [255, 0, 0, 255]);
    assert_eq!(pixel(&pixels, width, 85, 0), [0, 255, 0, 255]);
    assert_eq!(pixel(&pixels, width, 170, 0), [0, 0, 255, 255]);
    assert_eq!(pixel(&pixels, width, 255, 15), [255, 0, 0, 255]);
}

#[test]
fn alpha_image_preserves_rgb_and_gradates_alpha() {
    let rgb = Color::from_rgba8(10, 20, 30, 0.25);
    let (width, height, pixels) = rgba_pixels(super::alpha_image_handle(rgb));
    assert_eq!((width, height), (256, 16));
    assert_eq!(pixel(&pixels, width, 0, 0), [10, 20, 30, 0]);
    assert_eq!(pixel(&pixels, width, 255, 15), [10, 20, 30, 255]);
}
