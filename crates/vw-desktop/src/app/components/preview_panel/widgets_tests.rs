use super::widgets::{DraggableArea, PreviewOverlayHost};
use crate::app::Message;
use iced::advanced::{Widget, widget};
use iced::widget::{container, text};
use iced::{Element, Length};

#[test]
fn preview_overlay_host_builder_sets_children_and_size_from_content() {
    let content: Element<'_, Message> =
        container(text("content")).width(Length::Fixed(120.0)).height(Length::Fixed(80.0)).into();
    let overlay: Element<'_, Message> = container(text("overlay")).into();

    let host = PreviewOverlayHost::new(content, overlay)
        .show(true)
        .pos(Some((12.0, 24.0)))
        .on_close(Message::None);

    let children = host.children();
    assert_eq!(children.len(), 2);
    assert_eq!(host.size().width, Length::Fixed(120.0));
    assert_eq!(host.size().height, Length::Fixed(80.0));
}

#[test]
fn draggable_area_exposes_single_child_state_and_content_size() {
    let content: Element<'_, Message> =
        container(text("drag")).width(Length::Fixed(64.0)).height(Length::Fixed(32.0)).into();
    let area = DraggableArea::new(content, Message::None, Message::None);

    assert_eq!(area.children().len(), 1);
    assert_eq!(area.size().width, Length::Fixed(64.0));
    assert_eq!(area.size().height, Length::Fixed(32.0));
    assert_eq!(area.tag(), widget::tree::Tag::of::<super::widgets::DraggableAreaState>());
}
