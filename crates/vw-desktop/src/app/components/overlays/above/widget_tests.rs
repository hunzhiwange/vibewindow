use super::{AboveOverlay, PointAboveOverlay};
use iced::advanced::{Widget, widget};
use iced::widget::text;
use iced::{Element, Theme};

#[derive(Debug, Clone)]
enum TestMessage {}

fn label(value: &'static str) -> Element<'static, TestMessage> {
    text(value).into()
}

#[test]
fn above_overlay_reports_content_and_overlay_children() {
    let overlay = AboveOverlay::new(label("content"), label("overlay"));

    let children =
        <AboveOverlay<'_, TestMessage> as Widget<TestMessage, Theme, iced::Renderer>>::children(
            &overlay,
        );

    assert_eq!(children.len(), 2);
}

#[test]
fn point_above_overlay_reports_content_and_overlay_children() {
    let overlay = PointAboveOverlay::new(label("content"), label("overlay"));

    let children =
        <PointAboveOverlay<'_, TestMessage> as Widget<TestMessage, Theme, iced::Renderer>>::children(
            &overlay,
        );

    assert_eq!(children.len(), 2);
}

#[test]
fn above_overlay_diff_keeps_two_child_trees() {
    let mut tree = widget::Tree::empty();
    tree.children.push(widget::Tree::empty());
    tree.children.push(widget::Tree::empty());

    let replacement = AboveOverlay::new(label("new content"), label("new overlay"));
    <AboveOverlay<'_, TestMessage> as Widget<TestMessage, Theme, iced::Renderer>>::diff(
        &replacement,
        &mut tree,
    );

    assert_eq!(tree.children.len(), 2);
}
