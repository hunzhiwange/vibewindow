#[test]
fn text_width_counts_wide_chars_larger_than_ascii() {
    assert!(super::estimate_text_width("中", 10.0) > super::estimate_text_width("a", 10.0));
}

#[test]
fn text_width_sums_ascii_and_wide_char_estimates() {
    let width = super::estimate_text_width("ab中", 10.0);
    assert!((width - 20.6).abs() < 0.001);
}

#[test]
fn empty_label_has_zero_estimated_width() {
    assert_eq!(super::estimate_text_width("", 14.0), 0.0);
}
