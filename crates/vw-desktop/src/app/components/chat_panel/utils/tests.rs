use super::{truncate_chars, truncate_lines_middle};

#[test]
fn utils_reexports_text_helpers() {
    assert_eq!(truncate_chars("hello", 10), "hello");
    assert_eq!(truncate_lines_middle("a\nb", 3, 10), "a\nb");
}
