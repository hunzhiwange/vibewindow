use super::*;

#[test]
fn strips_supported_dev_null_redirects() {
    assert_eq!(strip_supported_redirects("cargo test >/dev/null 2>&1"), "cargo test");
}

#[test]
fn parses_attached_short_option_values_and_redirect_targets() {
    assert_eq!(attached_short_option_value("-ofile"), Some("file"));
    assert_eq!(redirection_target(">out.log"), Some("out.log"));
}

