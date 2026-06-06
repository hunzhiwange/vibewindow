#[test]
fn system_settings_gateway_tests_are_wired() {
    assert!(module_path!().contains("system_settings_gateway_tests"));
}

#[test]
fn paired_token_list_height_caps_at_ten_rows() {
    let expected = 10.0 * super::system_settings_gateway::GATEWAY_PAIRED_TOKEN_ROW_HEIGHT
        + 9.0 * super::system_settings_gateway::GATEWAY_PAIRED_TOKEN_ROW_SPACING;

    assert_eq!(super::system_settings_gateway::paired_token_list_max_height(10), expected);
    assert_eq!(super::system_settings_gateway::paired_token_list_max_height(11), expected);
}

#[test]
fn paired_token_scrollbar_width_is_four_pixels() {
    assert_eq!(super::system_settings_gateway::GATEWAY_PAIRED_TOKEN_SCROLLBAR_WIDTH, 4);
}
