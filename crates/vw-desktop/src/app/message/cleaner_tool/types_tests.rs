#[test]
fn format_bytes_uses_bytes_without_decimal_suffix() {
    assert_eq!(super::format_bytes(0), "0 B");
    assert_eq!(super::format_bytes(1023), "1023 B");
}

#[test]
fn format_bytes_scales_larger_units_with_two_decimals() {
    assert_eq!(super::format_bytes(1024), "1.00 KB");
    assert_eq!(super::format_bytes(1024 * 1024 + 512 * 1024), "1.50 MB");
}

#[test]
fn unsupported_platform_message_is_user_facing_and_specific() {
    let message = super::unsupported_platform_message();
    assert!(message.contains("暂不支持"));
    assert!(message.contains("macOS"));
    assert!(message.contains("Windows"));
}
