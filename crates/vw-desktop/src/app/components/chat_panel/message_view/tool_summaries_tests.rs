use super::tool_summaries::{
    count_code_blocks, normalized_visible_text, should_hide_explore_link_box,
    should_hide_post_explore_tool_block, special_text_blocks, tool_card_text_blocks,
};

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
