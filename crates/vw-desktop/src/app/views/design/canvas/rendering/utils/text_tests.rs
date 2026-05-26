#[test]
fn line_width_counts_letter_spacing_between_chars_only() {
    assert_eq!(super::compute_line_width("ab", 10.0, 2.0), 14.0);
    assert_eq!(super::compute_line_width("中", 10.0, 2.0), 10.0);
}

#[test]
fn text_transform_handles_common_modes() {
    assert_eq!(super::apply_text_transform("hello", Some("uppercase")), "HELLO");
    assert_eq!(super::apply_text_transform("HELLO", Some("lowercase")), "hello");
    assert_eq!(super::apply_text_transform("hello world", Some("capitalize")), "Hello World");
}

#[test]
fn wrap_text_words_keeps_empty_input_empty() {
    let (lines, size) = super::wrap_text_words("", 100.0, 12.0, 0.0);
    assert!(lines.is_empty());
    assert_eq!((size.width, size.height), (0.0, 0.0));
}
