use super::ui::{
    bold_font, capped_scroll_height, chat_context_menu, chat_context_target_key,
    copy_tooltip_content,
};
use iced::Length;

#[test]
fn chat_context_target_key_packs_optional_tool_index() {
    assert_eq!(chat_context_target_key(3, None), 3_u64 << 32);
    assert_eq!(chat_context_target_key(3, Some(4)), (3_u64 << 32) | 5);
}

#[test]
fn capped_scroll_height_keeps_reasonable_bounds() {
    assert_eq!(capped_scroll_height("short text", 120.0), Length::Shrink);
    assert_eq!(capped_scroll_height(&"line\n".repeat(100), 120.0), Length::Fixed(120.0));
}

#[test]
fn bold_font_is_configured() {
    assert_eq!(bold_font().weight, iced::font::Weight::Bold);
}

#[test]
fn menu_and_tooltip_build_only_when_requested() {
    assert!(chat_context_menu(false).is_none());
    assert!(chat_context_menu(true).is_some());
    let _ = copy_tooltip_content("复制");
}
