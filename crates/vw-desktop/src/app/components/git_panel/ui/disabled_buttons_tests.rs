use iced::Element;
use iced::widget::text;

use crate::app::Message;
use crate::app::assets::Icon;

use super::disabled_buttons::{
    disabled_icon_button, disabled_square_content_button, disabled_square_content_button_tiny,
    disabled_square_icon_button, disabled_square_icon_button_small,
    disabled_square_icon_button_tiny,
};

#[test]
fn disabled_icon_buttons_build_elements() {
    let _: Element<'static, Message> = disabled_icon_button(Icon::Plus, "add disabled".into());
    let _: Element<'static, Message> =
        disabled_square_icon_button(Icon::Trash, "trash disabled".into());
    let _: Element<'static, Message> =
        disabled_square_icon_button_small(Icon::Image, "image disabled".into());
    let _: Element<'static, Message> =
        disabled_square_icon_button_tiny(Icon::FileText, "file disabled".into());
}

#[test]
fn disabled_content_buttons_build_elements() {
    let _: Element<'static, Message> =
        disabled_square_content_button(text("A"), "content disabled".into());
    let _: Element<'static, Message> =
        disabled_square_content_button_tiny(text("B"), "tiny content disabled".into());
}
