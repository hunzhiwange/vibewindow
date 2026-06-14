use super::*;
use iced::Element;
use crate::app::{App, Message};
use iced::widget::{slider, text};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn row_helpers_build_controls() {
    keep_element(field_row("标签", "说明", text("control")));
    keep_element(bool_row("启用", "说明", true, "开启", |value| {
        Message::Settings(message::SettingsMessage::CoordinationEnabledToggled(value))
    }));
    keep_element(text_row("主协调 Agent", "说明", "delegate-lead", "lead", |value| {
        Message::Settings(message::SettingsMessage::CoordinationLeadAgentChanged(value))
    }));
    keep_element(slider_row(
        "每 Agent 收件箱",
        "说明",
        slider(1.0..=10.0, 5.0, |_| Message::None),
        5,
    ));
}

#[test]
fn view_builds_default_populated_and_error_states() {
    let app = test_app();
    keep_element(view(&app));

    let mut app = test_app();
    app.coordination_settings.enabled = true;
    app.coordination_settings.lead_agent_input = "delegate-lead".to_string();
    app.coordination_settings.max_inbox_messages_per_agent = 256;
    app.coordination_settings.max_dead_letters = 128;
    app.coordination_settings.max_context_entries = 512;
    app.coordination_settings.max_seen_message_ids = 4096;
    keep_element(view(&app));

    app.coordination_settings.save_error = Some("保存失败".to_string());
    keep_element(view(&app));
}

#[test]
fn overlays_return_dialog_or_help_modal() {
    let mut app = test_app();
    keep_element(view_overlays(&app, text("dialog").into()));

    app.coordination_settings.show_help_modal = true;
    keep_element(view_overlays(&app, text("dialog").into()));
}
