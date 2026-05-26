use super::*;

#[test]
fn redact_never_returns_full_secret() {
    assert_eq!(redact("abc"), "***");
    assert_eq!(redact("abcdef"), "abcd***");
}

