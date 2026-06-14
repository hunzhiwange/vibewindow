use super::explore_summary_view::{
    SummarySegmentKind, explore_summary_expanded, explore_summary_is_running, latest_explore_items,
    right_aligned_slot_char, split_summary_segments, summary_animation_key,
    tool_explore_summary_view,
};
use super::types::{EXPLORE_GROUP_TOOL_IDX, ExploreItem};
use crate::app::{App, Message};
use std::collections::HashSet;

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn right_aligned_slot_char_pads_left_slots() {
    let chars = ['1', '2'];

    assert_eq!(right_aligned_slot_char(&chars, 0, 3), "");
    assert_eq!(right_aligned_slot_char(&chars, 1, 3), "1");
}

#[test]
fn summary_animation_key_packs_message_and_group() {
    assert_ne!(summary_animation_key(1, 2), summary_animation_key(1, 3));
}

#[test]
fn split_summary_segments_keeps_plain_text_and_numbers() {
    let segments = split_summary_segments("Read 12 files");

    assert_eq!(
        segments,
        vec![
            (SummarySegmentKind::Text, "Read "),
            (SummarySegmentKind::Number, "12"),
            (SummarySegmentKind::Text, " files"),
        ]
    );
    assert!(split_summary_segments("").is_empty());
}

#[test]
fn explore_summary_expanded_ignores_running_until_user_expands() {
    let key = 42;
    let mut expanded = HashSet::new();

    assert!(!explore_summary_expanded(true, key, &expanded));

    expanded.insert(key);
    assert!(explore_summary_expanded(true, key, &expanded));
}

#[test]
fn explore_summary_is_running_respects_following_block_closure() {
    assert!(explore_summary_is_running(true, false, false));
    assert!(explore_summary_is_running(false, true, false));
    assert!(!explore_summary_is_running(true, true, true));
    assert!(!explore_summary_is_running(false, false, false));
}

#[test]
fn latest_explore_items_keeps_last_item_for_same_dedupe_key() {
    let first = ExploreItem {
        tool_idx: 1,
        raw: r#"tool read
{"input":"{\"path\":\"src/main.rs\"}","status":"completed"}"#
            .to_string(),
    };
    let second = ExploreItem {
        tool_idx: 2,
        raw: r#"tool read
{"input":"{\"path\":\"src/main.rs\"}","status":"completed"}"#
            .to_string(),
    };
    let third = ExploreItem {
        tool_idx: 3,
        raw: r#"tool grep
{"input":"{\"pattern\":\"App\"}","status":"completed"}"#
            .to_string(),
    };
    let items = vec![first, second, third];

    let latest = latest_explore_items(&items);

    assert_eq!(latest.len(), 2);
    assert_eq!(latest[0].tool_idx, 2);
    assert_eq!(latest[1].tool_idx, 3);
}

#[test]
fn tool_explore_summary_view_returns_none_for_empty_items() {
    let app = test_app();

    assert!(tool_explore_summary_view(&app, 0, 0, &[], false, false).is_none());
}

#[test]
fn tool_explore_summary_view_builds_collapsed_and_expanded_summaries() {
    let mut app = test_app();
    let items = vec![
        ExploreItem {
            tool_idx: 1,
            raw: r#"tool read
{"input":"{\"path\":\"src/main.rs\"}","status":"completed"}"#
                .to_string(),
        },
        ExploreItem {
            tool_idx: 2,
            raw: r#"tool grep
{"input":"{\"pattern\":\"App\"}","status":"completed"}"#
                .to_string(),
        },
        ExploreItem {
            tool_idx: 3,
            raw: r#"tool glob
{"input":"{\"pattern\":\"*.rs\"}","status":"running"}"#
                .to_string(),
        },
    ];

    let collapsed =
        tool_explore_summary_view(&app, 1, 0, &items, false, false).expect("collapsed summary");
    keep_element(collapsed);

    let group_tool_idx = EXPLORE_GROUP_TOOL_IDX.saturating_sub(1);
    let key = ((1_u64) << 32) | (group_tool_idx as u64);
    app.chat_explore_expanded.insert(key);
    let expanded =
        tool_explore_summary_view(&app, 1, 0, &items, false, false).expect("expanded summary");
    keep_element(expanded);

    let closed = tool_explore_summary_view(&app, 1, 0, &items, false, true)
        .expect("closed summary still renders");
    keep_element(closed);
}
