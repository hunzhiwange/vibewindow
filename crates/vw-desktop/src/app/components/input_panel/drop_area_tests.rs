use iced::advanced::Widget;
use iced::widget::text;
use iced::{Element, Length};

use crate::app::Message;

use super::drop_area::{DropArea, DropAreaElement};

#[test]
fn drop_area_new_wraps_content_and_exposes_child_tree() {
    let content: Element<'static, u8> = text("drop").into();
    let area = DropArea::new(content, 7, Some((1, 2)), true);

    let children = area.children();
    assert_eq!(children.len(), 1);
}

#[test]
fn drop_area_size_delegates_to_content() {
    let content: Element<'static, u8> =
        iced::widget::container(text("sized")).width(Length::Fixed(123.0)).into();
    let area = DropArea::new(content, 9, None, false);

    let size = area.size();
    assert!(matches!(size.width, Length::Fixed(width) if width == 123.0));
}

#[test]
fn drop_area_can_be_wrapped_as_iced_element_and_app_alias() {
    let content: Element<'static, u8> = text("element").into();
    let _: Element<'static, u8> = Element::new(DropArea::new(content, 1, None, false));

    let app_content: Element<'static, Message> = text("app element").into();
    let _: DropAreaElement<'static> = DropArea::new(app_content, Message::None, None, false);
}
