use super::tool_text_support::{chat_text_font_name, is_safe_for_text_editor, tool_text_key};

#[test]
fn tool_text_key_packs_indices_without_collisions_for_adjacent_values() {
    assert_ne!(tool_text_key(1, 2, 3), tool_text_key(1, 2, 4));
    assert_ne!(tool_text_key(1, 2, 3), tool_text_key(1, 3, 3));
}

#[test]
fn text_editor_safety_rejects_oversized_lines() {
    assert!(is_safe_for_text_editor("short\ntext"));
    assert!(!is_safe_for_text_editor(&"x".repeat(2_001)));
}

#[test]
fn chat_text_font_name_is_configured() {
    assert!(!chat_text_font_name().is_empty());
}
