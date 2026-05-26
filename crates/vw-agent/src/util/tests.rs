use super::{MaybeSet, floor_utf8_char_boundary, truncate_with_ellipsis};

#[test]
fn utf8_helpers_preserve_character_boundaries() {
    assert_eq!(truncate_with_ellipsis("hello world", 5), "hello...");
    assert_eq!(truncate_with_ellipsis("hi", 5), "hi");
    assert_eq!(floor_utf8_char_boundary("你好", 1), 0);
    assert_eq!(floor_utf8_char_boundary("你好", 4), 3);
    assert_eq!(MaybeSet::Set(1), MaybeSet::Set(1));
}
