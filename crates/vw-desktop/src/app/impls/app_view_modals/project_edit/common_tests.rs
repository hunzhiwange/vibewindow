use super::*;
use crate::app::state::ProjectEditTab;
use iced::Color;

fn assert_color_near(actual: Color, expected: Color) {
    let epsilon = 0.001;
    assert!((actual.r - expected.r).abs() < epsilon, "red mismatch: {actual:?}");
    assert!((actual.g - expected.g).abs() < epsilon, "green mismatch: {actual:?}");
    assert!((actual.b - expected.b).abs() < epsilon, "blue mismatch: {actual:?}");
    assert!((actual.a - expected.a).abs() < epsilon, "alpha mismatch: {actual:?}");
}

#[test]
fn parse_hex_color_accepts_hashless_hash_and_whitespace() {
    assert_color_near(parse_hex_color("60a5fa").unwrap(), Color::from_rgb8(0x60, 0xa5, 0xfa));
    assert_color_near(parse_hex_color("#34d399").unwrap(), Color::from_rgb8(0x34, 0xd3, 0x99));
    assert_color_near(parse_hex_color("  #EF4444  ").unwrap(), Color::from_rgb8(0xef, 0x44, 0x44));
}

#[test]
fn parse_hex_color_rejects_invalid_lengths_and_digits() {
    assert!(parse_hex_color("").is_none());
    assert!(parse_hex_color("#12345").is_none());
    assert!(parse_hex_color("#1234567").is_none());
    assert!(parse_hex_color("#12xx56").is_none());
}

#[test]
fn format_hex_color_rounds_rgb_channels_to_lowercase_hex() {
    assert_eq!(format_hex_color(Color::from_rgb8(0x60, 0xa5, 0xfa)), "#60a5fa");
    assert_eq!(format_hex_color(Color::from_rgb(1.0, 0.5, 0.0)), "#ff8000");
}

#[test]
fn icon_image_handle_accepts_existing_plain_and_file_urls() {
    let path = std::env::temp_dir()
        .join(format!("vibe-window-project-edit-common-{}.png", std::process::id()));
    std::fs::write(&path, b"not a real png but enough for a path handle").unwrap();

    assert!(icon_image_handle(path.to_str().unwrap()).is_some());
    assert!(icon_image_handle(&format!("file://{}", path.display())).is_some());
    assert!(icon_image_handle(&format!("file:///{}", path.display())).is_some());

    let _ = std::fs::remove_file(path);
}

#[test]
fn icon_image_handle_rejects_blank_and_missing_paths() {
    assert!(icon_image_handle("   ").is_none());
    assert!(icon_image_handle("/definitely/missing/vibe-window/icon.png").is_none());
}

#[test]
fn tab_button_builds_for_selected_and_unselected_tabs() {
    let selected = tab_button("基础信息", ProjectEditTab::General, true);
    let unselected = tab_button("刷新策略", ProjectEditTab::Refresh, false);

    std::hint::black_box(selected);
    std::hint::black_box(unselected);
}
