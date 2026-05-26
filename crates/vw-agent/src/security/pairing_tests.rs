use super::*;

#[test]
fn constant_time_eq_matches_equal_strings_only() {
    assert!(constant_time_eq("same", "same"));
    assert!(!constant_time_eq("same", "diff"));
}

#[test]
fn public_bind_detection_is_explicit() {
    assert!(is_public_bind("0.0.0.0"));
    assert!(!is_public_bind("127.0.0.1"));
}

