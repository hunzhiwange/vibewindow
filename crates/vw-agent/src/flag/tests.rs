use super::{number, truthy};

#[test]
fn missing_flags_are_false_and_absent_numbers_are_none() {
    assert!(!truthy("VIBEWINDOW_TEST_FLAG_DOES_NOT_EXIST"));
    assert_eq!(number("VIBEWINDOW_TEST_NUMBER_DOES_NOT_EXIST"), None);
}
