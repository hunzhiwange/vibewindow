use super::parse::{RenderBlock, borrowed_blocks, hash_chat_content, owned_blocks_from_raw};
use crate::app::models::ParsedChatBlock;

#[test]
fn owned_blocks_from_raw_splits_think_tool_and_text() {
    let raw = "<think>plan</think>\nanswer\ntool bash\n{\"status\":\"completed\"}\n";

    let blocks = owned_blocks_from_raw(raw);

    assert!(
        matches!(blocks.first(), Some(ParsedChatBlock::Think { content, open }) if content == "plan" && !open)
    );
    assert!(
        matches!(blocks.get(1), Some(ParsedChatBlock::Text { content }) if content.trim() == "answer")
    );
    assert!(
        matches!(blocks.get(2), Some(ParsedChatBlock::Tool { raw }) if raw.starts_with("tool bash"))
    );
}

#[test]
fn borrowed_blocks_preserve_block_kinds() {
    let blocks = vec![
        ParsedChatBlock::Text { content: "hello".to_string() },
        ParsedChatBlock::Think { content: "why".to_string(), open: true },
    ];

    let borrowed = borrowed_blocks(&blocks).collect::<Vec<_>>();

    assert!(matches!(borrowed[0], RenderBlock::Text { content } if content == "hello"));
    assert!(
        matches!(borrowed[1], RenderBlock::Think { content, open } if content == "why" && open)
    );
}

#[test]
fn hash_chat_content_changes_with_content() {
    assert_eq!(hash_chat_content("same"), hash_chat_content("same"));
    assert_ne!(hash_chat_content("same"), hash_chat_content("other"));
}

#[test]
fn think_tags_allow_attributes_and_ignore_prefix_collisions() {
    let raw = "pre <thinker>not a tag</thinker><think time=\"1s\">inside</think> tail";

    let blocks = owned_blocks_from_raw(raw);

    assert!(
        matches!(blocks.first(), Some(ParsedChatBlock::Text { content }) if content.contains("<thinker>"))
    );
    assert!(
        matches!(blocks.get(1), Some(ParsedChatBlock::Think { content, open }) if content == "inside" && !open)
    );
    assert!(matches!(blocks.get(2), Some(ParsedChatBlock::Text { content }) if content == " tail"));
}

#[test]
fn unclosed_think_splits_following_tool_blocks() {
    let raw = "<think>plan\ntool bash\n{\"status\":\"completed\",\"output\":\"ok\"}\nanswer";

    let blocks = owned_blocks_from_raw(raw);

    assert!(
        matches!(blocks.first(), Some(ParsedChatBlock::Think { content, open }) if content == "plan\n" && *open)
    );
    assert!(
        matches!(blocks.get(1), Some(ParsedChatBlock::Tool { raw }) if raw.starts_with("tool bash"))
    );
    assert!(
        matches!(blocks.get(2), Some(ParsedChatBlock::Text { content }) if content == "answer")
    );
}

#[test]
fn invalid_tool_like_text_stays_text() {
    let raw = "tool bash\nnot json\nstill not json";

    let blocks = owned_blocks_from_raw(raw);

    assert_eq!(blocks.len(), 1);
    assert!(matches!(blocks.first(), Some(ParsedChatBlock::Text { content }) if content == raw));
}

#[test]
fn tool_after_colon_is_parsed_as_tool_block() {
    let raw = "说明：tool file_write\n{\"status\":\"completed\",\"output\":\"ok\"}\n尾部";

    let blocks = owned_blocks_from_raw(raw);

    assert!(
        matches!(blocks.first(), Some(ParsedChatBlock::Text { content }) if content == "说明：")
    );
    assert!(
        matches!(blocks.get(1), Some(ParsedChatBlock::Tool { raw }) if raw.starts_with("tool file_write"))
    );
    assert!(matches!(blocks.get(2), Some(ParsedChatBlock::Text { content }) if content == "尾部"));
}
