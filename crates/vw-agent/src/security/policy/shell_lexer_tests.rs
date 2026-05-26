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

