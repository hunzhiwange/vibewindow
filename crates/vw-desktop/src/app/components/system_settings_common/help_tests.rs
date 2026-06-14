use crate::app::{App, Message};
use iced::widget::text;

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn help_tests_are_wired() {
    assert!(module_path!().contains("help_tests"));
}

#[test]
fn close_and_help_buttons_build_with_messages() {
    keep_element(super::help::settings_close_button(Message::GatewayHealthTick));
    keep_element(super::help::settings_help_button(Message::GatewayHealthTick));
}

#[test]
fn help_modal_uses_copy_and_copied_states() {
    let mut app = App::new().0;
    let help_text = "帮助内容\n\n```json\n{\"enabled\":true}\n```";

    keep_element(super::help::with_settings_help_modal(
        &app,
        text("base").into(),
        "帮助",
        help_text,
        Message::GatewayHealthTick,
    ));

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&help_text, &mut hasher);
    app.last_copied_code_hash = Some(std::hash::Hasher::finish(&hasher));

    keep_element(super::help::with_settings_help_modal(
        &app,
        text("base").into(),
        "帮助",
        help_text,
        Message::GatewayHealthTick,
    ));
}
