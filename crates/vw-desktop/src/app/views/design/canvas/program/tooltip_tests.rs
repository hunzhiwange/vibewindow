#[test]
fn text_width_counts_wide_chars_larger_than_ascii() {
    assert!(super::estimate_text_width("中", 10.0) > super::estimate_text_width("a", 10.0));
}
