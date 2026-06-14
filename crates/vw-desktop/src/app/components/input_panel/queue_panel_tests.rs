use super::queue_panel::{
    format_queue_time, is_dark_theme, queue_item_style, queue_panel, queue_panel_style,
};
use crate::app::QueueItem;
use crate::app::state::{AcpHistoryReplayMode, ChatSendBehavior};
use iced::{Background, Length, Theme};

fn queue_item(query: &str, created_ms: u64) -> QueueItem {
    QueueItem {
        created_ms,
        query: query.to_string(),
        attachments: Vec::new(),
        root: None,
        model: None,
        acp_test: false,
        acp_agent: None,
        acp_allowed_tools: None,
        agent: None,
        allowed_tools: None,
        acp_force_new_session: false,
        acp_history_mode: AcpHistoryReplayMode::Discard,
        acp_recent_count: 3,
        full_access_enabled: false,
        send_behavior: ChatSendBehavior::Queue,
        request_history_override: None,
        resume_history_only: false,
        workflow_mode_enabled: false,
    }
}

#[test]
fn format_queue_time_formats_epoch_millis_and_rejects_out_of_range_values() {
    assert_eq!(format_queue_time(0), "1970-01-01 00:00:00");
    assert_eq!(format_queue_time(1_000), "1970-01-01 00:00:01");
    assert_eq!(format_queue_time(u64::MAX), "");
}

#[test]
fn queue_theme_detection_distinguishes_light_and_dark() {
    assert!(is_dark_theme(&Theme::Dark));
    assert!(!is_dark_theme(&Theme::Light));
}

#[test]
fn queue_item_style_changes_for_next_item_and_theme() {
    let next_dark = queue_item_style(&Theme::Dark, true);
    let waiting_dark = queue_item_style(&Theme::Dark, false);
    let next_light = queue_item_style(&Theme::Light, true);

    assert!(matches!(next_dark.background, Some(Background::Color(_))));
    assert!(matches!(waiting_dark.background, Some(Background::Color(_))));
    assert_ne!(next_dark.border.color, waiting_dark.border.color);
    assert_ne!(next_dark.border.color, next_light.border.color);
    assert_eq!(next_dark.border.width, 1.0);
}

#[test]
fn queue_panel_style_uses_theme_specific_container_background() {
    let dark = queue_panel_style(&Theme::Dark);
    let light = queue_panel_style(&Theme::Light);

    assert!(dark.background.is_some());
    assert!(light.background.is_some());
    assert_ne!(dark.border.color, light.border.color);
    assert_eq!(dark.border.width, 1.0);
}

#[test]
fn queue_panel_renders_empty_short_and_long_queues() {
    let empty = queue_panel(Vec::new(), false);
    assert_eq!(empty.as_widget().size().width, Length::Fill);

    let short = queue_panel(vec![queue_item("first line\nsecond line", 0)], true);
    assert_eq!(short.as_widget().size().width, Length::Fill);

    let long_label = "这是一条很长很长很长很长很长很长很长很长很长的任务标题";
    let long =
        queue_panel(vec![queue_item(long_label, 1_000), queue_item("waiting", 2_000)], false);
    assert_eq!(long.as_widget().size().width, Length::Fill);
}
