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

#[test]
fn strip_redirects_handles_stream_merges_spacing_and_quotes() {
    assert_eq!(strip_supported_redirects("cmd 2> &1"), "cmd");
    assert_eq!(strip_supported_redirects("cmd 10>&2 arg"), "cmd  arg");
    assert_eq!(strip_supported_redirects(r#"echo "2>&1" 2>&1"#), r#"echo "2>&1""#);
    assert_eq!(strip_supported_redirects(r#"echo \2>&1"#), r#"echo \2"#);
}

#[test]
fn strip_redirects_handles_dev_null_forms_and_boundaries() {
    assert_eq!(strip_supported_redirects("cmd >/dev/null"), "cmd");
    assert_eq!(strip_supported_redirects("cmd 2>> /dev/null next"), "cmd  next");
    assert_eq!(strip_supported_redirects("cmd &>/dev/null"), "cmd");
    assert_eq!(strip_supported_redirects("cmd </dev/null"), "cmd");
    assert_eq!(strip_supported_redirects("cmd >/dev/nullish"), "cmd >/dev/nullish");
    assert_eq!(strip_supported_redirects("cmd >out.log"), "cmd >out.log");
}

#[test]
fn strip_redirects_preserves_unsupported_or_escaped_syntax() {
    assert_eq!(strip_supported_redirects("cmd |& grep x"), "cmd | grep x");
    assert_eq!(strip_supported_redirects(r#"cmd \>/dev/null"#), r#"cmd \>/dev/null"#);
    assert_eq!(strip_supported_redirects("cmd 2>&"), "cmd 2>&");
}

#[test]
fn attached_short_option_value_rejects_long_or_empty_options() {
    assert_eq!(attached_short_option_value("-o=file"), Some("file"));
    assert_eq!(attached_short_option_value("-o file"), Some("file"));
    assert_eq!(attached_short_option_value("-o"), None);
    assert_eq!(attached_short_option_value("--output=file"), None);
    assert_eq!(attached_short_option_value("output=file"), None);
}

#[test]
fn redirection_target_extracts_paths_and_ignores_fd_only_redirects() {
    assert_eq!(redirection_target("2>out.log"), Some("out.log"));
    assert_eq!(redirection_target(">> out.log"), Some("out.log"));
    assert_eq!(redirection_target("< input.txt"), Some("input.txt"));
    assert_eq!(redirection_target("2>&1"), None);
    assert_eq!(redirection_target("plain"), None);
}
