use super::text::{
    SessionControlSelection, estimate_message_height_rough, estimate_text_height,
    message_text_body, session_control_selection_card, should_prefer_plain_think_body,
    should_segment_text_block, split_session_control_selection,
};

#[test]
fn estimate_text_height_grows_with_lines() {
    assert!(estimate_text_height("one\ntwo\nthree") > estimate_text_height("one"));
}

#[test]
fn should_segment_text_block_only_for_large_text() {
    assert!(!should_segment_text_block("short"));
    assert!(should_segment_text_block(&"line\n".repeat(120)));
}

#[test]
fn plain_think_body_is_preferred_for_streaming_or_thinking() {
    assert!(should_prefer_plain_think_body(true, true));
    assert!(should_prefer_plain_think_body(false, true));
    assert!(should_prefer_plain_think_body(true, false));
    assert!(!should_prefer_plain_think_body(false, false));
}

#[test]
fn estimate_message_height_rough_has_minimum() {
    assert!(estimate_message_height_rough("") > 0.0);
}

#[test]
fn split_session_control_selection_extracts_context_card_data() {
    let raw = "修一下这里\n\n<session_control_selection>\n用户在会话控制中选中了以下上下文。选中不代表已经执行；请在本轮需要时再主动使用。\n工具：file_write, ls\n技能：writing-plans\n</session_control_selection>";

    let (text, selection) = split_session_control_selection(raw);
    let selection = selection.expect("selection context should parse");

    assert_eq!(text, "修一下这里");
    assert_eq!(selection.tools, vec!["file_write", "ls"]);
    assert_eq!(selection.skills, vec!["writing-plans"]);
}

#[test]
fn split_session_control_selection_supports_ascii_colons_and_chinese_commas() {
    let raw = "正文\n<session_control_selection>\n工具: read，grep\n技能: rust, testing\n</session_control_selection>\n";

    let (text, selection) = split_session_control_selection(raw);
    let selection = selection.expect("selection context should parse");

    assert_eq!(text, "正文");
    assert_eq!(selection.tools, vec!["read", "grep"]);
    assert_eq!(selection.skills, vec!["rust", "testing"]);
}

#[test]
fn split_session_control_selection_ignores_incomplete_or_empty_blocks() {
    assert_eq!(split_session_control_selection("a <session_control_selection> 工具: read").1, None);

    let (text, selection) = split_session_control_selection(
        "a <session_control_selection>\n说明\n</session_control_selection> b",
    );
    assert_eq!(text, "a  b");
    assert_eq!(selection, None);
}

#[test]
fn estimate_message_height_accounts_for_tools_and_thinks_but_clamps() {
    let plain = estimate_message_height_rough("hello");
    let rich = estimate_message_height_rough("<think>x</think>\ntool bash\n{}\nhello");
    let huge = estimate_message_height_rough(&"line\n".repeat(10_000));

    assert!(rich > plain);
    assert_eq!(huge, 720.0);
}

#[test]
fn text_and_session_cards_construct_for_segmented_and_empty_inputs() {
    let _ = message_text_body("line\n".repeat(120), false);
    let _ = message_text_body(String::new(), true);
    let _ = session_control_selection_card(SessionControlSelection {
        tools: vec!["read".to_string()],
        skills: vec!["testing".to_string()],
    });
}
