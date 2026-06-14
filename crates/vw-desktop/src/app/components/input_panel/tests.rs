use super::{
    acp_history_controls, acp_selector_button, bottom_bar_round_toggle, dark_tooltip_content,
    popover_item_style, selected_context_card, session_control_button, summarize_names, view,
    with_dark_tooltip,
};
use crate::app::assets::Icon;
use crate::app::components::input_panel::icons::icon_svg;
use crate::app::state::{AcpHistoryReplayMode, ChatSendBehavior};
use crate::app::{Message, message};
use iced::widget::{button, text};
use iced::{Element, Length, Theme};

#[test]
fn summarize_names_lists_up_to_three_names_and_hidden_count() {
    assert_eq!(summarize_names(&[]), "");
    assert_eq!(summarize_names(&["a".into(), "b".into()]), "a、b");
    assert_eq!(summarize_names(&["a".into(), "b".into(), "c".into(), "d".into()]), "a、b、c +1");
}

#[test]
fn popover_item_style_delegates_selected_and_unselected_states() {
    let selected = popover_item_style(&Theme::Dark, iced::widget::button::Status::Hovered, true);
    let unselected = popover_item_style(&Theme::Dark, iced::widget::button::Status::Active, false);

    assert_ne!(selected.background, unselected.background);
}

#[test]
fn tooltip_helpers_wrap_content_with_shrink_size() {
    let tip = dark_tooltip_content("提示");
    assert_eq!(tip.as_widget().size().width, Length::Shrink);

    let wrapped = with_dark_tooltip(button(text("content")).into(), "tip");
    assert_eq!(wrapped.as_widget().children().len(), 2);
}

#[test]
fn bottom_bar_round_toggle_builds_tooltip_wrapped_icon_button() {
    let icon: Element<'_, Message> = icon_svg(Icon::Sliders, 14.0).into();
    let element = bottom_bar_round_toggle(
        icon,
        "toggle",
        Message::View(message::ViewMessage::ToggleSessionToolSelectorPopover),
    );

    assert_eq!(element.as_widget().children().len(), 2);
}

#[test]
fn acp_history_controls_render_all_modes_and_recent_input() {
    for mode in [
        AcpHistoryReplayMode::Discard,
        AcpHistoryReplayMode::Summary,
        AcpHistoryReplayMode::Full,
        AcpHistoryReplayMode::Recent,
    ] {
        let element = acp_history_controls(mode, 5);

        assert_eq!(element.as_widget().size().width, Length::Fill);
    }
}

#[test]
fn optional_selector_buttons_reflect_app_state() {
    let mut app = crate::app::App::new().0;

    assert!(session_control_button(&app).is_some());
    assert!(acp_selector_button(&app, None).is_some());

    app.acp_agents = vec!["Codex".to_string(), "Claude Code".to_string()];
    app.show_acp_popover = true;
    let acp = acp_selector_button(&app, Some("Codex".to_string()));
    assert!(acp.is_some());
}

#[test]
fn selected_context_card_is_absent_without_runtime_manual_context() {
    let app = crate::app::App::new().0;

    assert!(selected_context_card(&app).is_none());
}

#[test]
fn input_panel_view_renders_default_task_queue_and_overlay_states() {
    let mut app = crate::app::App::new().0;
    let default = view(&app);
    assert_eq!(default.as_widget().size().width, Length::Fill);
    drop(default);

    app.input_editor = iced::widget::text_editor::Content::with_text("@src/main.rs:1\nhello");
    app.files.push("/tmp/a.txt".to_string());
    app.show_model_popover = true;
    app.show_file_search = true;
    app.project_path = Some("/repo".to_string());
    app.file_search_query = "src".to_string();
    app.set_file_index("/repo", vec!["src/main.rs".to_string()]);
    let with_overlays = view(&app);
    assert_eq!(with_overlays.as_widget().size().width, Length::Fill);
}

#[test]
fn input_panel_view_renders_drag_hover_and_send_mode_state() {
    let mut app = crate::app::App::new().0;
    app.dragging_file_paths = vec!["src/main.rs".to_string()];
    app.input_drop_hovered = true;
    app.show_send_mode_popover = true;
    app.chat_send_behavior = ChatSendBehavior::StopAndSend;

    let element = view(&app);

    assert_eq!(element.as_widget().size().width, Length::Fill);
}
