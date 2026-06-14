use super::*;
use iced::Element;
use iced::widget::text;

#[test]
fn field_row_accepts_scheduler_control() {
    let element: Element<'_, Message> =
        field_row("最大任务", "单次轮询最多处理的任务数量。", text("64"));
    drop(element);
}

#[test]
fn scheduler_view_uses_expected_bounded_ranges_and_help_message() {
    let source = include_str!("system_settings_scheduler.rs");

    assert!(source.contains("slider(1.0..=10_000.0, s.max_tasks as f32"));
    assert!(source.contains("slider(1.0..=100.0, s.max_concurrent as f32"));
    assert!(source.contains("SchedulerHelpOpen"));
    assert!(source.contains("SchedulerHelpClose"));
}
