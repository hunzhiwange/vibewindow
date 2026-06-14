use super::*;
use iced::Element;
use iced::widget::text;

#[test]
fn field_row_accepts_reliability_control() {
    let element: Element<'_, Message> =
        field_row("Provider 重试", "请求失败时的最大重试次数。", text("2"));
    drop(element);
}

#[test]
fn reliability_view_uses_expected_bounded_ranges() {
    let source = include_str!("system_settings_reliability.rs");

    assert!(source.contains("slider(0.0..=20.0, s.provider_retries as f32"));
    assert!(source.contains("slider(0.0..=60_000.0, s.provider_backoff_ms as f32"));
    assert!(source.contains("slider(1.0..=3600.0, s.channel_initial_backoff_secs as f32"));
    assert!(source.contains("s.channel_initial_backoff_secs as f32..=3600.0"));
    assert!(source.contains("slider(1.0..=3600.0, s.scheduler_poll_secs as f32"));
    assert!(source.contains("slider(0.0..=20.0, s.scheduler_retries as f32"));
}
