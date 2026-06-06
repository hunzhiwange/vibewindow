use super::tool_selector::{
    SESSION_SELECTOR_LIST_MAX_HEIGHT, SESSION_SELECTOR_SCROLLBAR_WIDTH,
    SESSION_SELECTOR_SKILL_DESCRIPTION_CHARS, ellipsize_text,
};

#[test]
fn selector_scrollbar_width_matches_chat_design_contract() {
    assert_eq!(SESSION_SELECTOR_SCROLLBAR_WIDTH, 4.0);
}

#[test]
fn selector_list_has_positive_height_limit() {
    assert!(SESSION_SELECTOR_LIST_MAX_HEIGHT > 0.0);
}

#[test]
fn skill_description_preview_stays_compact() {
    assert!(SESSION_SELECTOR_SKILL_DESCRIPTION_CHARS <= 28);
}

#[test]
fn ellipsize_text_compacts_whitespace_and_uses_ascii_dots() {
    assert_eq!(ellipsize_text("alpha\n beta   gamma", 12), "alpha bet...");
}
