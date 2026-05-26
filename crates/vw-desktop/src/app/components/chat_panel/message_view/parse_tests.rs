use super::parse::{RenderBlock, borrowed_blocks, hash_chat_content, owned_blocks_from_raw};
use crate::app::models::ParsedChatBlock;

#[test]
fn owned_blocks_from_raw_splits_think_tool_and_text() {
    let raw = "<think>plan</think>\nanswer\ntool bash\n{\"status\":\"completed\"}\n";

    let blocks = owned_blocks_from_raw(raw);

    assert!(matches!(blocks.first(), Some(ParsedChatBlock::Think { content, open }) if content == "plan" && !open));
    assert!(matches!(blocks.get(1), Some(ParsedChatBlock::Text { content }) if content.trim() == "answer"));
    assert!(matches!(blocks.get(2), Some(ParsedChatBlock::Tool { raw }) if raw.starts_with("tool bash")));
}

#[test]
fn borrowed_blocks_preserve_block_kinds() {
    let blocks = vec![
        ParsedChatBlock::Text { content: "hello".to_string() },
        ParsedChatBlock::Think { content: "why".to_string(), open: true },
    ];

    let borrowed = borrowed_blocks(&blocks).collect::<Vec<_>>();

    assert!(matches!(borrowed[0], RenderBlock::Text { content } if content == "hello"));
    assert!(matches!(borrowed[1], RenderBlock::Think { content, open } if content == "why" && open));
}

#[test]
fn hash_chat_content_changes_with_content() {
    assert_eq!(hash_chat_content("same"), hash_chat_content("same"));
    assert_ne!(hash_chat_content("same"), hash_chat_content("other"));
}
