use super::parse::hash_chat_content;
use super::render_cache::{
    assistant_render_blocks, build_render_cache_entry, effective_assistant_render_cache,
};
use crate::app::models::ParsedChatBlock;

#[test]
fn build_render_cache_entry_marks_special_blocks() {
    let raw = "<think>plan</think>\nvisible";
    let entry = build_render_cache_entry(raw, "visible", hash_chat_content("visible"), true);

    assert!(entry.has_special_blocks);
    assert!(matches!(entry.blocks.first(), Some(ParsedChatBlock::Think { .. })));
}

#[test]
fn effective_assistant_render_cache_rebuilds_stale_cache() {
    let old = build_render_cache_entry("old", "old", hash_chat_content("old"), true);
    let rebuilt = effective_assistant_render_cache(
        "tool bash\n{\"status\":\"completed\"}",
        &old,
        "tool bash",
        hash_chat_content("tool bash"),
        false,
        true,
    );

    assert!(matches!(rebuilt.blocks.first(), Some(ParsedChatBlock::Tool { .. })));
}

#[test]
fn assistant_render_blocks_uses_cache_when_fresh() {
    let raw = "plain text";
    let entry = build_render_cache_entry(raw, raw, hash_chat_content(raw), true);

    let (blocks, has_special) = assistant_render_blocks(raw, &entry, false);

    assert!(!has_special);
    assert!(
        matches!(blocks.first(), Some(ParsedChatBlock::Text { content }) if content == "plain text")
    );
}

#[test]
fn build_render_cache_entry_merges_explore_across_hidden_think() {
    let raw = concat!(
        "tool read\n",
        "{\"tool_call_id\":\"call-1\",\"input\":\"{\\\"filePath\\\":\\\"/tmp/a.rs\\\"}\",\"status\":\"completed\"}\n",
        "<think>done</think>\n",
        "tool read\n",
        "{\"tool_call_id\":\"call-2\",\"input\":\"{\\\"filePath\\\":\\\"/tmp/b.rs\\\"}\",\"status\":\"completed\"}\n"
    );
    let entry = build_render_cache_entry(raw, raw, hash_chat_content(raw), false);

    assert_eq!(entry.explore_summary_text_blocks.len(), 1);
    assert_eq!(entry.explore_summary_text_blocks[0].1, "2 次读取");
}
