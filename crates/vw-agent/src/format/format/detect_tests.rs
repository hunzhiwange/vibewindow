use super::detect::which;

#[test]
fn which_returns_none_for_missing_program() {
    assert!(which("vibewindow-format-command-that-should-not-exist").is_none());
}
