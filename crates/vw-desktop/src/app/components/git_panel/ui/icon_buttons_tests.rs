use iced::Element;

use crate::app::Message;
use crate::app::assets::Icon;

use super::icon_buttons::{
    icon_button, medium_icon_button, small_icon_button, small_plain_icon_button,
    square_icon_button, square_icon_button_micro, square_icon_button_small,
    square_icon_button_tiny,
};

#[test]
fn icon_buttons_build_elements() {
    let _: Element<'static, Message> = icon_button(Icon::Plus, "plus".into(), Message::None);
    let _: Element<'static, Message> =
        square_icon_button(Icon::Trash, "trash".into(), Message::None);
    let _: Element<'static, Message> =
        square_icon_button_small(Icon::Image, "image".into(), Message::None);
    let _: Element<'static, Message> =
        square_icon_button_tiny(Icon::FileText, "file".into(), Message::None);
    let _: Element<'static, Message> =
        square_icon_button_micro(Icon::Gear, "gear".into(), Message::None);
}

#[test]
fn compact_icon_buttons_build_elements() {
    let _: Element<'static, Message> =
        medium_icon_button(Icon::CloudUpload, "upload".into(), Message::None);
    let _: Element<'static, Message> =
        small_icon_button(Icon::CloudDownload, "download".into(), Message::None);
    let _: Element<'static, Message> =
        small_plain_icon_button(Some(Icon::CheckSquare), "checked".into(), Message::None);
    let _: Element<'static, Message> =
        small_plain_icon_button(None, "empty slot".into(), Message::None);
}
