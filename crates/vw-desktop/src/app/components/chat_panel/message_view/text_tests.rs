use super::text::{estimate_message_height_rough, estimate_text_height, should_prefer_plain_think_body, should_segment_text_block};

#[test]
fn estimate_text_height_grows_with_lines() {
    assert!(estimate_text_height("one\ntwo\nthree") > estimate_text_height("one"));
}

#[test]
fn should_segment_text_block_only_for_large_text() {
    assert!(!should_segment_text_block("short"));
    assert!(should_segment_text_block(&"line\n".repeat(120)));
}

#[test]
fn plain_think_body_is_preferred_for_streaming_or_thinking() {
    assert!(should_prefer_plain_think_body(true, true));
    assert!(should_prefer_plain_think_body(false, true));
    assert!(should_prefer_plain_think_body(true, false));
    assert!(!should_prefer_plain_think_body(false, false));
}

#[test]
fn estimate_message_height_rough_has_minimum() {
    assert!(estimate_message_height_rough("") > 0.0);
}
