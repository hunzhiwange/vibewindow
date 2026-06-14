use iced::Element;

use crate::app::Message;

use super::glyph_buttons::{
    header_plain_glyph_button, medium_glyph_button, medium_plain_glyph_button, small_glyph_button,
    small_plain_glyph_button,
};

#[test]
fn tooltip_glyph_buttons_build_elements() {
    let _: Element<'static, Message> =
        medium_glyph_button("M", "medium glyph".into(), Message::None);
    let _: Element<'static, Message> = small_glyph_button("S", "small glyph".into(), Message::None);
    let _: Element<'static, Message> =
        header_plain_glyph_button("H", "header glyph".into(), Message::None);
}

#[test]
fn plain_glyph_buttons_build_elements_without_tooltip() {
    let _: Element<'static, Message> =
        medium_plain_glyph_button("M", "ignored".into(), Message::None);
    let _: Element<'static, Message> =
        small_plain_glyph_button("S", "ignored".into(), Message::None);
}
