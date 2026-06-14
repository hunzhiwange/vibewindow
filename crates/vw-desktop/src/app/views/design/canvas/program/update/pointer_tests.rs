use std::borrow::Cow;
use std::collections::HashSet;

use crate::app::message::DesignMessage;
use crate::app::views::design::canvas::types::DesignCanvasState;
use crate::app::views::design::models::{DesignDoc, DesignElement};

fn canvas<'a>(
    doc: &'a DesignDoc,
    cache: &'a iced::widget::canvas::Cache,
    selected_ids: &'a HashSet<String>,
) -> super::DesignCanvas<'a> {
    super::DesignCanvas {
        doc: Cow::Borrowed(doc),
        cache,
        pan: iced::Vector::new(0.0, 0.0),
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

fn bounds() -> iced::Rectangle {
    iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(300.0, 200.0))
}

#[test]
fn pointer_outside_clears_hover() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let mut state =
        DesignCanvasState { hovered_id: Some("node".to_string()), ..Default::default() };

    let action = canvas.update_pointer_event(
        &mut state,
        &iced::widget::canvas::Event::Mouse(iced::mouse::Event::CursorMoved {
            position: iced::Point::new(400.0, 400.0),
        }),
        bounds(),
        iced::mouse::Cursor::Unavailable,
    );

    assert!(action.is_some());
    assert!(state.hovered_id.is_none());
}

#[test]
fn pointer_outside_clears_drag_preview_parent() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let mut state = DesignCanvasState {
        tool_preview_start: Some(iced::Point::new(10.0, 10.0)),
        tool_preview_current: Some(iced::Point::new(20.0, 20.0)),
        tool_preview_parent_id: Some("frame".to_string()),
        ..Default::default()
    };

    let action = canvas.update_pointer_event(
        &mut state,
        &iced::widget::canvas::Event::Mouse(iced::mouse::Event::CursorMoved {
            position: iced::Point::new(400.0, 400.0),
        }),
        bounds(),
        iced::mouse::Cursor::Unavailable,
    );

    assert!(action.is_some());
    assert_eq!(state.tool_preview_start, Some(iced::Point::new(10.0, 10.0)));
    assert!(state.tool_preview_current.is_none());
    assert!(state.tool_preview_parent_id.is_none());
}

#[test]
fn color_picker_left_click_publishes_pick_color() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.color_picking = true;
    let mut state = DesignCanvasState::default();

    let action = canvas
        .update_pointer_event(
            &mut state,
            &iced::widget::canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Left,
            )),
            bounds(),
            iced::mouse::Cursor::Available(iced::Point::new(18.0, 24.0)),
        )
        .expect("left click should pick color");

    match published(action) {
        crate::app::Message::Design(DesignMessage::PickColor(point)) => {
            assert_eq!(point, iced::Point::new(18.0, 24.0));
        }
        message => panic!("unexpected message: {message:?}"),
    }
}

#[test]
fn right_click_publishes_context_menu_with_hit_id() {
    let doc = DesignDoc {
        children: vec![DesignElement {
            id: "rect".to_string(),
            kind: "rect".to_string(),
            x: 10.0,
            y: 10.0,
            width: Some(serde_json::json!(40.0)),
            height: Some(serde_json::json!(30.0)),
            ..Default::default()
        }],
        ..Default::default()
    };
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let mut state = DesignCanvasState::default();

    let action = canvas
        .update_pointer_event(
            &mut state,
            &iced::widget::canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Right,
            )),
            bounds(),
            iced::mouse::Cursor::Available(iced::Point::new(15.0, 15.0)),
        )
        .expect("right click should open context menu");

    match published(action) {
        crate::app::Message::Design(DesignMessage::CanvasContextMenuOpen(point, hit)) => {
            assert_eq!(point, iced::Point::new(15.0, 15.0));
            assert_eq!(hit.as_deref(), Some("rect"));
        }
        message => panic!("unexpected message: {message:?}"),
    }
}

#[test]
fn middle_button_toggles_panning_state() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let mut state = DesignCanvasState::default();

    assert!(
        canvas
            .update_pointer_event(
                &mut state,
                &iced::widget::canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(
                    iced::mouse::Button::Middle,
                )),
                bounds(),
                iced::mouse::Cursor::Available(iced::Point::new(7.0, 9.0)),
            )
            .is_some()
    );
    assert!(state.is_panning);
    assert_eq!(state.last_cursor_pos, Some(iced::Point::new(7.0, 9.0)));

    assert!(
        canvas
            .update_pointer_event(
                &mut state,
                &iced::widget::canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(
                    iced::mouse::Button::Middle,
                )),
                bounds(),
                iced::mouse::Cursor::Available(iced::Point::new(7.0, 9.0)),
            )
            .is_some()
    );
    assert!(!state.is_panning);
    assert!(state.last_cursor_pos.is_none());
}
