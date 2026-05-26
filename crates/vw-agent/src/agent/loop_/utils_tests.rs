use super::utils::scrub_credentials;

#[test]
fn scrub_credentials_redacts_sensitive_values_without_touching_plain_text() {
    let input = r#"token: "abcdef123456" api_key=xyz987654321 plain=value"#;
    let scrubbed = scrub_credentials(input);

    assert!(scrubbed.contains("abcd*[REDACTED]"));
    assert!(scrubbed.contains("xyz9*[REDACTED]"));
    assert!(scrubbed.contains("plain=value"));
    assert!(!scrubbed.contains("ef123456"));
}
