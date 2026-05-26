#[test]
fn wrap_text_lines_respects_invalid_dimensions_and_width() {
    assert!(super::wrap_text_lines("abc", 0.0, 12.0).is_empty());
    assert_eq!(super::wrap_text_lines("abcd", 12.0, 10.0), vec!["ab".to_string(), "cd".to_string()]);
}
