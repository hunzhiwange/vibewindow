#[test]
fn sanitize_base_rejects_out_of_range_values() {
    assert_eq!(super::sanitize_base(1), 10);
    assert_eq!(super::sanitize_base(2), 2);
    assert_eq!(super::sanitize_base(36), 36);
    assert_eq!(super::sanitize_base(37), 10);
}

#[test]
fn sanitize_input_keeps_only_digits_valid_for_base() {
    assert_eq!(super::sanitize_input(" -10aZ ", 16), "-10a");
    assert_eq!(super::sanitize_input("--10102", 2), "-1010");
    assert_eq!(super::sanitize_input("xyz", 36), "xyz");
}

#[test]
fn convert_handles_negative_values_and_base_36_digits() {
    assert_eq!(super::convert("-ff", 16, 10).unwrap(), "-255");
    assert_eq!(super::convert("35", 10, 36).unwrap(), "Z");
    assert_eq!(super::convert("Z", 36, 10).unwrap(), "35");
}

#[test]
fn convert_reports_invalid_base_or_digit() {
    assert_eq!(super::convert("10", 1, 10).unwrap_err(), "仅支持 2-36 进制");
    assert_eq!(super::convert("2", 2, 10).unwrap_err(), "非法数字或超出范围");
}

#[test]
fn digit_helpers_cover_bounds() {
    assert_eq!(super::digit_value('0'), Some(0));
    assert_eq!(super::digit_value('A'), Some(10));
    assert_eq!(super::digit_value('z'), Some(35));
    assert_eq!(super::digit_value('_'), None);
    assert_eq!(super::digit_char(35), 'Z');
    assert_eq!(super::digit_char(36), '?');
}
