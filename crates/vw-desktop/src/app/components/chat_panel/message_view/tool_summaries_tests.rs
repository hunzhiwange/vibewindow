use super::tool_summaries::{
    count_code_blocks, normalized_visible_text, should_hide_explore_link_box,
    should_hide_post_explore_tool_block, special_text_blocks, tool_card_text_blocks,
    trailing_tool_tail_text_source_block_idx,
};
use crate::app::models::ParsedChatBlock;

#[test]
fn count_code_blocks_counts_fenced_pairs() {
    assert_eq!(count_code_blocks("```rust\nfn main() {}\n```\n```text\nx\n```"), 2);
}

#[test]
fn normalized_visible_text_filters_empty_or_trace_like_text() {
    assert_eq!(normalized_visible_text("  visible  "), Some("visible".to_string()));
    assert_eq!(normalized_visible_text(""), None);
}

#[test]
fn special_text_blocks_keep_text_after_tool() {
    let raw = "tool bash\n{\"status\":\"completed\",\"output\":\"ok\"}\nfinal answer";

    assert_eq!(special_text_blocks(raw), vec!["final answer".to_string()]);
    assert_eq!(tool_card_text_blocks(raw).len(), 1);
}

#[test]
fn explore_link_box_hide_rules_are_explicit() {
    assert!(should_hide_explore_link_box("- src/main.rs\n- src/lib.rs"));
    assert!(!should_hide_explore_link_box("summary text"));
    assert!(!should_hide_post_explore_tool_block("plain text"));
}

#[test]
fn tool_card_text_blocks_cover_todo_and_browser_previews() {
    let todo = tool_card_text_blocks(
        "tool todowrite\n{\"input\":\"{\\\"todos\\\":[{\\\"content\\\":\\\"done\\\",\\\"status\\\":\\\"completed\\\"},{\\\"content\\\":\\\"next\\\",\\\"status\\\":\\\"pending\\\"}]}\"}",
    );
    assert_eq!(todo, vec![vec!["✓ done\n○ next".to_string()]]);

    let browser = tool_card_text_blocks(
        "tool browser_open\n{\"result\":{\"data\":{\"browser\":\"Chrome\",\"url\":\"https://example.test\"}},\"status\":\"completed\",\"output\":\"opened\"}",
    );
    assert_eq!(browser, vec![vec!["Chrome · https://example.test".to_string()]]);
}

#[test]
fn tool_card_text_blocks_cover_advanced_tool_previews() {
    let blocks = tool_card_text_blocks(
        "tool tool_search\n{\"result\":{\"data\":{\"items\":[{\"display_name\":\"GitHub\",\"reason\":\"repo access\"},{\"id\":\"slack\",\"reason\":\"messages\"}]}},\"status\":\"completed\",\"output\":\"matches\"}",
    );

    assert!(blocks[0][0].starts_with("状态:"));
    assert!(blocks[0].contains(&"GitHub: repo access".to_string()));
    assert!(blocks[0].contains(&"slack: messages".to_string()));
}

#[test]
fn tool_card_text_blocks_cover_brief_attachments_and_overflow() {
    let blocks = tool_card_text_blocks(
        "tool brief\n{\"result\":{\"data\":{\"message\":\"ready\",\"attachments\":[{\"path\":\"/tmp/a/image.png\",\"isImage\":true,\"size\":2048},{\"path\":\"/tmp/b/doc.txt\",\"size\":5},{\"path\":\"/tmp/c/notes.md\",\"size\":1048576},{\"path\":\"/tmp/d/more.md\",\"size\":1}]}}}",
    );

    assert_eq!(blocks[0][0], "ready");
    assert!(blocks[0][1].contains("[image] a/image.png (2.0 KB)"));
    assert!(blocks[0][3].contains("1.0 MB"));
    assert_eq!(blocks[0][4], "... 还有 1 个附件");
}

#[test]
fn normalized_visible_text_strips_internal_traces() {
    assert_eq!(
        normalized_visible_text("tool bash({\"command\":\"pwd\"})\nfinal"),
        Some("final".to_string())
    );
}

#[test]
fn trailing_tool_tail_text_source_block_idx_requires_trailing_explore_tool() {
    let blocks = vec![
        ParsedChatBlock::Text { content: "summary".to_string() },
        ParsedChatBlock::Tool {
            raw: "tool read\n{\"input\":\"{\\\"filePath\\\":\\\"/tmp/a.rs\\\"}\",\"status\":\"completed\"}"
                .to_string(),
        },
        ParsedChatBlock::Text { content: "\n".to_string() },
    ];

    assert_eq!(trailing_tool_tail_text_source_block_idx(&blocks), Some(0));

    let no_explore = vec![
        ParsedChatBlock::Text { content: "summary".to_string() },
        ParsedChatBlock::Tool {
            raw: "tool bash\n{\"input\":\"pwd\",\"status\":\"completed\"}".to_string(),
        },
    ];
    assert_eq!(trailing_tool_tail_text_source_block_idx(&no_explore), None);
}
