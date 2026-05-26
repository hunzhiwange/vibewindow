use super::locale::{datetime, time_short, today_time_or_datetime};

#[test]
fn formats_known_timestamp_without_empty_output() {
    assert!(time_short(0).contains(':'));
    assert!(datetime(0).contains("1970"));
    assert!(!today_time_or_datetime(0).is_empty());
}
