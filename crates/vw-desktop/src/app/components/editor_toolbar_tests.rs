use super::editor_toolbar::{icon_button, icon_svg};
use crate::app::{assets::Icon, Message};
use iced::widget::tooltip::Position;

#[test]
fn toolbar_icon_svg_and_button_build() {
    let _ = icon_svg(Icon::Save);
    let _ = icon_button(Icon::Save, "保存", Position::Top, Message::PreviewLspTick, false);
}
