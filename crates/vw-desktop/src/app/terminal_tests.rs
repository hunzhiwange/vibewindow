#[test]
fn shell_display_and_all_are_stable() {
    let shells = super::Shell::all();
    assert_eq!(shells, [super::Shell::Bash, super::Shell::Zsh]);
    assert_eq!(shells[0].to_string(), "bash");
    assert_eq!(shells[1].to_string(), "zsh");
}

#[test]
fn truncate_string_preserves_utf8_boundary() {
    assert_eq!(super::truncate_string_to_limit("abc", 4), "abc");
    assert_eq!(super::truncate_string_to_limit("a你好b", 5), "好b");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn color_channels_are_clamped_before_hex_conversion() {
    assert_eq!(super::color_channel_to_u8(-1.0), 0);
    assert_eq!(super::color_channel_to_u8(2.0), 255);
    assert_eq!(super::to_hex(iced::Color::from_rgb(1.0, 0.5, 0.0)), "#ff8000");
}
