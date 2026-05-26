use super::truncate::truncate_chars_cli;

#[test]
fn truncate_chars_cli_is_unicode_safe() {
    assert_eq!(truncate_chars_cli("abcdef", 3), "abc…");
    assert_eq!(truncate_chars_cli("你好世界", 2), "你好…");
    assert_eq!(truncate_chars_cli("keep", 8), "keep");
    assert_eq!(truncate_chars_cli("drop", 0), "");
}
