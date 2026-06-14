#[test]
fn format_bytes_uses_bytes_without_decimal_suffix() {
    assert_eq!(super::format_bytes(0), "0 B");
    assert_eq!(super::format_bytes(1023), "1023 B");
}

#[test]
fn format_bytes_scales_larger_units_with_two_decimals() {
    assert_eq!(super::format_bytes(1024), "1.00 KB");
    assert_eq!(super::format_bytes(1024 * 1024 + 512 * 1024), "1.50 MB");
    assert_eq!(super::format_bytes(1024_u64.pow(3) * 2), "2.00 GB");
    assert_eq!(super::format_bytes(1024_u64.pow(4) * 3), "3.00 TB");
}

#[test]
fn unsupported_platform_message_is_user_facing_and_specific() {
    let message = super::unsupported_platform_message();
    assert!(message.contains("暂不支持"));
    assert!(message.contains("macOS"));
    assert!(message.contains("Windows"));
}

#[test]
fn current_platform_matches_compile_target() {
    let expected = if cfg!(target_os = "macos") {
        Some(super::CleanerPlatform::MacOs)
    } else if cfg!(target_os = "windows") {
        Some(super::CleanerPlatform::Windows)
    } else {
        None
    };

    assert_eq!(super::current_platform(), expected);
}
