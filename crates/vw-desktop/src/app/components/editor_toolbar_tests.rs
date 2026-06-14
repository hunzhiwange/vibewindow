use super::editor_toolbar::{icon_button, icon_svg};
use crate::app::{App, Message, Screen, assets::Icon};
use iced::widget::tooltip::Position;

#[test]
fn toolbar_icon_svg_and_button_build() {
    let _ = icon_svg(Icon::Save);
    let _ = icon_button(Icon::Save, "保存", Position::Top, Message::PreviewLspTick, false);
    let _ = icon_button(Icon::Save, "保存", Position::Bottom, Message::PreviewLspTick, true);
}

#[test]
fn toolbar_view_handles_project_and_non_project_screens() {
    let mut app = App::new().0;
    let _ = super::editor_toolbar::view(&app, Some(Message::PreviewLspTick), true);

    app.screen = Screen::Project;
    let _ = super::editor_toolbar::view(&app, None, false);
}
