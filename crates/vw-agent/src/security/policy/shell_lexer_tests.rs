use super::*;

#[test]
fn lexer_ignores_quoted_shell_metacharacters() {
    assert!(!contains_unquoted_char("echo ';'", ';'));
    assert!(contains_unquoted_char("echo hi; pwd", ';'));
    assert_eq!(strip_wrapping_quotes("'value'"), "value");
}

#[test]
fn env_assignment_prefix_is_skipped() {
    assert_eq!(skip_env_assignments("FOO=bar BAR=baz cargo test"), "cargo test");
}

#[test]
fn env_assignment_skipping_stops_at_invalid_names_or_command() {
    assert_eq!(skip_env_assignments("_FOO=bar echo hi"), "echo hi");
    assert_eq!(skip_env_assignments("1FOO=bar echo hi"), "1FOO=bar echo hi");
    assert_eq!(skip_env_assignments("FOO=bar"), "");
}

#[test]
fn split_segments_respects_quotes_escapes_and_control_operators() {
    assert_eq!(
        split_unquoted_segments(r#"echo "a;b" && printf 'x|y' || pwd; whoami"#),
        vec![r#"echo "a;b""#, "printf 'x|y'", "pwd", "whoami"]
    );
    assert_eq!(split_unquoted_segments(r#"echo a\;b | grep b"#), vec![r#"echo a\;b"#, "grep b"]);
    assert_eq!(split_unquoted_segments("cmd &"), vec!["cmd &"]);
    assert!(split_unquoted_segments(" ; \n ").is_empty());
}

#[test]
fn ampersand_detection_ignores_quotes_escapes_and_double_ampersand() {
    assert!(contains_unquoted_single_ampersand("sleep 1 &"));
    assert!(!contains_unquoted_single_ampersand("echo a && echo b"));
    assert!(!contains_unquoted_single_ampersand("echo '&'"));
    assert!(!contains_unquoted_single_ampersand(r#"echo \&"#));
    assert!(!contains_unquoted_single_ampersand(r#"echo "&""#));
}

#[test]
fn unquoted_char_detection_respects_double_quote_escapes() {
    assert!(contains_unquoted_char("cat < input", '<'));
    assert!(!contains_unquoted_char(r#"echo \"not quote\" > out"#, '"'));
    assert!(!contains_unquoted_char(r#"echo \>"#, '>'));
    assert!(!contains_unquoted_char(r#"echo ">""#, '>'));
}

#[test]
fn variable_expansion_detection_respects_quote_rules() {
    assert!(contains_unquoted_shell_variable_expansion("echo $HOME"));
    assert!(contains_unquoted_shell_variable_expansion(r#"echo "$HOME""#));
    assert!(contains_unquoted_shell_variable_expansion("echo ${HOME}"));
    assert!(contains_unquoted_shell_variable_expansion("echo $(whoami)"));
    assert!(contains_unquoted_shell_variable_expansion("echo $?"));
    assert!(!contains_unquoted_shell_variable_expansion("echo '$HOME'"));
    assert!(!contains_unquoted_shell_variable_expansion(r#"echo \$HOME"#));
    assert!(!contains_unquoted_shell_variable_expansion("cost is $"));
}

#[test]
fn strip_wrapping_quotes_trims_matching_quote_characters_from_edges() {
    assert_eq!(strip_wrapping_quotes(r#""value""#), "value");
    assert_eq!(strip_wrapping_quotes("'value'"), "value");
    assert_eq!(strip_wrapping_quotes(r#""value'"#), "value");
    assert_eq!(strip_wrapping_quotes("plain"), "plain");
}
