use super::render_cache::{assistant_render_blocks, build_render_cache_entry, effective_assistant_render_cache};
use super::parse::hash_chat_content;
use crate::app::models::ParsedChatBlock;

#[test]
fn build_render_cache_entry_marks_special_blocks() {
    let raw = "<think>plan</think>\nvisible";
    let entry = build_render_cache_entry(raw, "visible", hash_chat_content("visible"));

    assert!(entry.has_special_blocks);
    assert!(matches!(entry.blocks.first(), Some(ParsedChatBlock::Think { .. })));
}

#[test]
fn effective_assistant_render_cache_rebuilds_stale_cache() {
    let old = build_render_cache_entry("old", "old", hash_chat_content("old"));
    let rebuilt = effective_assistant_render_cache(
        "tool bash\n{\"status\":\"completed\"}",
        &old,
        "tool bash",
        hash_chat_content("tool bash"),
        false,
    );

    assert!(matches!(rebuilt.blocks.first(), Some(ParsedChatBlock::Tool { .. })));
}

#[test]
fn assistant_render_blocks_uses_cache_when_fresh() {
    let raw = "plain text";
    let entry = build_render_cache_entry(raw, raw, hash_chat_content(raw));

    let (blocks, has_special) = assistant_render_blocks(raw, &entry, false);

    assert!(!has_special);
    assert!(matches!(blocks.first(), Some(ParsedChatBlock::Text { content }) if content == "plain text"));
}
