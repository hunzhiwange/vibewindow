use super::ui::{bold_font, capped_scroll_height, chat_context_target_key};

#[test]
fn chat_context_target_key_packs_optional_tool_index() {
    assert_eq!(chat_context_target_key(3, None), 3_u64 << 32);
    assert_eq!(chat_context_target_key(3, Some(4)), (3_u64 << 32) | 5);
}

#[test]
fn capped_scroll_height_keeps_reasonable_bounds() {
    let _ = capped_scroll_height("short text", 120.0);
    let _ = capped_scroll_height(&"line\n".repeat(100), 120.0);
}

#[test]
fn bold_font_is_configured() {
    let _ = bold_font();
}
