use super::send_controls::{
    bottom_bar, cancel_button, full_access_button, is_dark_theme, permission_access_button_style,
    pool_button, prominent_action_background, prominent_action_foreground, prominent_action_style,
    send_behavior_icon, send_behavior_popover, send_button, utility_cluster_style,
    workflow_mode_button,
};
use crate::app::Message;
use crate::app::assets::Icon;
use crate::app::state::ChatSendBehavior;
use iced::widget::{Space, button, text};
use iced::{Background, Element, Length, Theme};

fn spacer() -> Element<'static, Message> {
    Space::new().into()
}

#[test]
fn send_behavior_icons_match_each_mode() {
    assert_eq!(send_behavior_icon(ChatSendBehavior::Queue), Icon::ListUl);
    assert_eq!(send_behavior_icon(ChatSendBehavior::StopAndSend), Icon::Square);
    assert_eq!(send_behavior_icon(ChatSendBehavior::Guide), Icon::ChatTextFill);
}

#[test]
fn send_control_theme_helpers_cover_dark_and_light_palettes() {
    assert!(is_dark_theme(&Theme::Dark));
    assert!(!is_dark_theme(&Theme::Light));
    assert_ne!(
        prominent_action_background(&Theme::Dark),
        prominent_action_background(&Theme::Light)
    );
    assert_ne!(
        prominent_action_foreground(&Theme::Dark),
        prominent_action_foreground(&Theme::Light)
    );
}

#[test]
fn prominent_action_style_covers_enabled_disabled_and_statuses() {
    for enabled in [true, false] {
        for status in [
            iced::widget::button::Status::Active,
            iced::widget::button::Status::Hovered,
            iced::widget::button::Status::Pressed,
            iced::widget::button::Status::Disabled,
        ] {
            let style = prominent_action_style(&Theme::Dark, status, enabled);

            assert!(matches!(style.background, Some(Background::Color(_))));
            assert_eq!(style.border.width, 1.0);
        }
    }
}

#[test]
fn permission_access_button_style_covers_active_and_inactive() {
    let inactive = permission_access_button_style(
        &Theme::Light,
        iced::widget::button::Status::Active,
        true,
        false,
    );
    let active = permission_access_button_style(
        &Theme::Light,
        iced::widget::button::Status::Hovered,
        true,
        true,
    );

    assert!(inactive.background.is_some());
    assert!(active.background.is_some());
    assert_ne!(inactive.border.color, active.border.color);
}

#[test]
fn utility_cluster_style_sets_pill_background_for_both_themes() {
    let dark = utility_cluster_style(&Theme::Dark);
    let light = utility_cluster_style(&Theme::Light);

    assert!(dark.background.is_some());
    assert!(light.background.is_some());
    assert_eq!(dark.border.width, 0.0);
}

#[test]
fn pool_button_renders_enabled_disabled_and_task_mode_variants() {
    let disabled = pool_button(false, true, String::new(), "1".into(), "auto".into(), Vec::new());
    let simple = pool_button(true, false, "task".into(), "1".into(), "auto".into(), Vec::new());
    let task_mode = pool_button(
        true,
        true,
        "task".into(),
        "5".into(),
        "provider/model".into(),
        vec!["sub".into()],
    );

    assert_eq!(disabled.as_widget().children().len(), 2);
    assert_eq!(simple.as_widget().children().len(), 2);
    assert_eq!(task_mode.as_widget().children().len(), 2);
}

#[test]
fn access_workflow_cancel_and_send_buttons_render_tooltip_wrapped_controls() {
    let disabled_access = full_access_button(false, false);
    let enabled_access = full_access_button(true, false);
    let active_access = full_access_button(true, true);
    let workflow_off = workflow_mode_button(false);
    let workflow_on = workflow_mode_button(true);
    let cancel = cancel_button(0);
    let cancel_larger = cancel_button(5);

    for element in [
        disabled_access,
        enabled_access,
        active_access,
        workflow_off,
        workflow_on,
        cancel,
        cancel_larger,
    ] {
        assert_eq!(element.as_widget().children().len(), 2);
    }
}

#[test]
fn send_button_renders_idle_requesting_and_popover_modes() {
    let mut app = crate::app::App::new().0;

    let disabled = send_button(&app, false, false, false);
    let idle = send_button(&app, true, true, false);
    assert_eq!(disabled.as_widget().children().len(), 2);
    assert_eq!(idle.as_widget().children().len(), 2);
    drop(disabled);
    drop(idle);

    app.chat_send_behavior = ChatSendBehavior::Guide;
    app.show_send_mode_popover = true;
    let requesting = send_button(&app, true, true, true);
    assert_eq!(requesting.as_widget().size().width, Length::Shrink);
}

#[test]
fn send_behavior_popover_renders_all_behavior_options() {
    for behavior in
        [ChatSendBehavior::Queue, ChatSendBehavior::StopAndSend, ChatSendBehavior::Guide]
    {
        let element = send_behavior_popover(behavior);

        assert_eq!(element.as_widget().size().width, Length::Fixed(320.0));
    }
}

#[test]
fn bottom_bar_handles_optional_controls_and_task_mode_layouts() {
    let model_btn: Element<'_, Message> = button(text("model")).into();
    let usage_btn: Element<'_, Message> = button(text("usage")).into();
    let attach_btn: Element<'_, Message> = button(text("attach")).into();
    let normal = bottom_bar(
        Some(spacer()),
        Some(spacer()),
        Some(spacer()),
        spacer(),
        model_btn,
        usage_btn,
        attach_btn,
        Some(spacer()),
        Some(spacer()),
        Some(spacer()),
        false,
    );
    assert_eq!(normal.as_widget().size().width, Length::Fill);

    let task_mode = bottom_bar(
        None,
        None,
        None,
        spacer(),
        button(text("model")).into(),
        button(text("usage")).into(),
        button(text("attach")).into(),
        None,
        Some(spacer()),
        Some(spacer()),
        true,
    );
    assert_eq!(task_mode.as_widget().size().width, Length::Fill);
}
