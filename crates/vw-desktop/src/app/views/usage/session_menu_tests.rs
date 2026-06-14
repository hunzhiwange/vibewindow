use iced::Point;

use crate::app::{App, Message};

#[test]
fn session_menu_button_and_menu_can_be_constructed() {
    let _button = super::session_menu_button("重命名", Message::None);
    let _menu = super::build_session_menu("session-1".to_string());
}

#[test]
fn kv_with_menu_uses_spacer_when_session_id_is_missing() {
    let (app, _task) = App::new();

    let _row = super::kv_with_menu(&app, "会话", "暂无".to_string(), None);
}

#[test]
fn kv_with_menu_builds_right_click_area_for_closed_menu() {
    let (app, _task) = App::new();

    let _row =
        super::kv_with_menu(&app, "会话", "Session".to_string(), Some("session-1".to_string()));
}

#[test]
fn kv_with_menu_wraps_button_in_overlay_when_menu_is_open() {
    let (mut app, _task) = App::new();
    app.session_menu_id = Some("session-1".to_string());
    app.session_menu_anchor = Some(Point::new(10.0, 20.0));

    let _row =
        super::kv_with_menu(&app, "会话", "Session".to_string(), Some("session-1".to_string()));
}
