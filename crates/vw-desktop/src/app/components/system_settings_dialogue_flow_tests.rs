use super::*;
use iced::Element;
use crate::app::{App, Message};
use iced::widget::text;

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn field_row_accepts_arbitrary_control() {
    keep_element(field_row("显示推理摘要", "说明", text("control")));
}

#[test]
fn view_builds_for_toggle_combinations() {
    let mut app = test_app();
    for (reasoning, shell, edit) in [(false, false, false), (true, false, true), (true, true, true)]
    {
        app.dialogue_flow_show_reasoning_summary = reasoning;
        app.dialogue_flow_expand_shell_tool_section = shell;
        app.dialogue_flow_expand_edit_tool_section = edit;
        keep_element(view(&app));
    }
}

#[test]
fn view_appends_success_or_error_save_message() {
    let mut app = test_app();
    app.dialogue_flow_settings_save_message = Some("已保存对话流配置".to_string());
    keep_element(view(&app));

    app.dialogue_flow_settings_save_message = Some("保存失败".to_string());
    keep_element(view(&app));
}
