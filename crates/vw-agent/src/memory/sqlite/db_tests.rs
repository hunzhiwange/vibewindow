use super::*;

#[test]
fn sqlite_open_timeout_cap_is_bounded() {
    assert_eq!(SQLITE_OPEN_TIMEOUT_CAP_SECS, 300);
}

