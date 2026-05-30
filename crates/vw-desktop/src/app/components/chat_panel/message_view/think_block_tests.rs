use super::think_block::{
    should_render_think_block, think_block_default_expanded, think_block_is_running,
    think_block_resolved_expanded,
};
use crate::app::models::ThinkTiming;
use std::collections::HashSet;

#[test]
fn think_block_is_running_uses_open_or_missing_end() {
    let running = ThinkTiming { start_ms: 1, end_ms: None, last_update_ms: 2 };
    let done = ThinkTiming { start_ms: 1, end_ms: Some(2), last_update_ms: 2 };

    assert!(think_block_is_running(false, Some(&running)));
    assert!(think_block_is_running(true, Some(&done)));
    assert!(!think_block_is_running(false, Some(&done)));
}

#[test]
fn manual_expand_and_collapse_override_default() {
    let expanded = HashSet::from([9]);
    let collapsed = HashSet::from([10]);

    assert!(think_block_resolved_expanded(false, 9, &expanded, &collapsed));
    assert!(!think_block_resolved_expanded(true, 10, &expanded, &collapsed));
}

#[test]
fn completed_hidden_when_reasoning_summary_disabled() {
    let done = ThinkTiming { start_ms: 1, end_ms: Some(2), last_update_ms: 2 };

    assert!(!think_block_default_expanded(false, false, Some(&done)));
    assert!(!should_render_think_block(false, 1, false, Some(&done)));
}
