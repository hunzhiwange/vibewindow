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

#[test]
fn running_think_block_stays_collapsed_until_manual_expand() {
    let running = ThinkTiming { start_ms: 1, end_ms: None, last_update_ms: 2 };

    assert!(!think_block_default_expanded(true, false, Some(&running)));
    assert!(!think_block_default_expanded(true, true, Some(&running)));
    assert!(!think_block_resolved_expanded(false, 7, &HashSet::new(), &HashSet::new()));
}

#[test]
fn render_decision_shows_running_without_summary() {
    let running = ThinkTiming { start_ms: 10, end_ms: None, last_update_ms: 10 };

    assert!(should_render_think_block(false, 3, false, Some(&running)));
    assert!(should_render_think_block(false, 3, true, None));
}

#[test]
fn render_decision_hides_finished_without_summary_and_shows_with_summary() {
    let done = ThinkTiming { start_ms: 10, end_ms: Some(40), last_update_ms: 40 };

    assert!(!should_render_think_block(false, 3, false, Some(&done)));
    assert!(should_render_think_block(true, 3, false, Some(&done)));
}

#[test]
fn collapsed_manual_state_wins_even_when_expanded_set_contains_key() {
    let expanded = HashSet::from([5]);
    let collapsed = HashSet::from([5]);

    assert!(!think_block_resolved_expanded(true, 5, &expanded, &collapsed));
}
