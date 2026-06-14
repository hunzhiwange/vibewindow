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

#[test]
fn build_render_cache_entry_falls_back_to_tool_text_when_visible_text_is_empty() {
    let raw = "tool bash\n{\"input\":\"pwd\",\"status\":\"completed\",\"output\":\"/tmp\"}\n";
    let entry = build_render_cache_entry(raw, "", hash_chat_content(""), true);

    assert_eq!(entry.display_text, "pwd");
    assert_eq!(entry.preview_text, "pwd");
    assert!(entry.has_special_blocks);
}

#[test]
fn build_render_cache_entry_marks_large_and_foldable_by_code_blocks() {
    let visible = "```text\nx\n```\n".repeat(6);
    let entry = build_render_cache_entry(&visible, &visible, hash_chat_content(&visible), true);

    assert!(entry.is_large_message);
    assert!(entry.foldable);
    assert!(entry.estimated_expanded_height >= entry.estimated_collapsed_height);
}

#[test]
fn effective_assistant_render_cache_borrows_fresh_cache() {
    let raw = "fresh";
    let entry = build_render_cache_entry(raw, raw, hash_chat_content(raw), false);
    let resolved =
        effective_assistant_render_cache(raw, &entry, raw, hash_chat_content(raw), false, false);

    assert!(matches!(resolved, std::borrow::Cow::Borrowed(_)));
}
