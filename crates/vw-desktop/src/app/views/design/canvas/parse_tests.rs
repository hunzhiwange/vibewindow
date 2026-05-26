#[test]
fn fallback_char_width_matches_ascii_and_wide_chars() {
    assert_eq!(super::fallback_char_advance_px('a', 10.0), 6.0);
    assert_eq!(super::fallback_char_advance_px('中', 10.0), 10.0);
}

#[test]
fn parse_size_accepts_numbers_and_pixel_strings() {
    assert_eq!(
        super::parse_size(&Some(serde_json::json!(12)), &std::collections::HashMap::new(), None),
        Some(12.0)
    );
    assert_eq!(
        super::parse_size(&Some(serde_json::json!("14")), &std::collections::HashMap::new(), None),
        Some(14.0)
    );
    assert_eq!(
        super::parse_size(&Some(serde_json::json!("bad")), &std::collections::HashMap::new(), None),
        None
    );
}
