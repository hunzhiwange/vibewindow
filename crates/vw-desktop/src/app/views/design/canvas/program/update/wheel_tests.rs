use std::borrow::Cow;
use std::collections::HashSet;

use crate::app::message::DesignMessage;

fn canvas<'a>(
    doc: &'a crate::app::views::design::models::DesignDoc,
    cache: &'a iced::widget::canvas::Cache,
    selected_ids: &'a HashSet<String>,
) -> super::DesignCanvas<'a> {
    super::DesignCanvas {
        doc: Cow::Borrowed(doc),
        cache,
        pan: iced::Vector::new(10.0, 20.0),
        zoom: 1.0,
        selected_id: None,
        selected_ids,
        selected_fill_index: None,
        editing_id: None,
        active_tool: super::DesignTool::Move,
        brush_color_hex: "#000000",
        brush_width_px: 4.0,
        toolbar_icon_family: "phosphor",
        toolbar_icon_name: "star",
        mouse_wheel_zoom_enabled: false,
        show_slot_content: false,
        show_slot_overflow: false,
        color_picking: false,
        hover_disabled: false,
    }
}

fn published(action: iced::widget::canvas::Action<crate::app::Message>) -> crate::app::Message {
    action.into_inner().0.expect("action should publish a message")
}

#[test]
fn wheel_zoom_publishes_zoom_around_cursor() {
    let doc = Default::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.mouse_wheel_zoom_enabled = true;

    let action = canvas
        .handle_wheel_scrolled(
            iced::Point::new(12.0, 24.0),
            &iced::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
        )
        .expect("positive scroll should zoom");

    match published(action) {
        crate::app::Message::Design(DesignMessage::Zoom(factor, Some(point))) => {
            assert_eq!(factor, 1.1);
            assert_eq!(point, iced::Point::new(12.0, 24.0));
        }
        message => panic!("unexpected message: {message:?}"),
    }
}

#[test]
fn wheel_zoom_ignores_zero_vertical_scroll() {
    let doc = Default::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.mouse_wheel_zoom_enabled = true;

    assert!(
        canvas
            .handle_wheel_scrolled(
                iced::Point::ORIGIN,
                &iced::mouse::ScrollDelta::Pixels { x: 40.0, y: 0.0 },
            )
            .is_none()
    );
}

#[test]
fn wheel_pan_uses_line_delta_scale() {
    let doc = Default::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);

    let action = canvas
        .handle_wheel_scrolled(
            iced::Point::ORIGIN,
            &iced::mouse::ScrollDelta::Lines { x: 1.0, y: -2.0 },
        )
        .expect("non-zero scroll should pan");

    match published(action) {
        crate::app::Message::Design(DesignMessage::Pan(pan)) => {
            assert_eq!(pan, iced::Vector::new(70.0, -100.0));
        }
        message => panic!("unexpected message: {message:?}"),
    }
}

#[test]
fn wheel_pan_ignores_zero_delta() {
    let doc = Default::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);

    assert!(
        canvas
            .handle_wheel_scrolled(
                iced::Point::ORIGIN,
                &iced::mouse::ScrollDelta::Pixels { x: 0.0, y: 0.0 },
            )
            .is_none()
    );
}
