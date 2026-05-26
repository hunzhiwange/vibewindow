use super::tool_selector::{SESSION_SELECTOR_LIST_MAX_HEIGHT, SESSION_SELECTOR_SCROLLBAR_WIDTH};

#[test]
fn selector_scrollbar_width_matches_chat_design_contract() {
    assert_eq!(SESSION_SELECTOR_SCROLLBAR_WIDTH, 4.0);
}

#[test]
fn selector_list_has_positive_height_limit() {
    assert!(SESSION_SELECTOR_LIST_MAX_HEIGHT > 0.0);
}
