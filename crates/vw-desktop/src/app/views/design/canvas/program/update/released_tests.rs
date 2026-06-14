use std::borrow::Cow;
use std::collections::HashSet;

use crate::app::message::DesignMessage;
use crate::app::views::design::canvas::types::{DesignCanvasState, Handle};
use crate::app::views::design::models::{DesignDoc, DesignElement, DesignTool};

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
        active_tool: DesignTool::Move,
        brush_color_hex: "#123456",
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

fn rect(id: &str, x: f32, y: f32, width: f32, height: f32) -> DesignElement {
    DesignElement {
        id: id.to_string(),
        kind: "rect".to_string(),
        x,
        y,
        width: Some(serde_json::json!(width)),
        height: Some(serde_json::json!(height)),
        ..Default::default()
    }
}

#[test]
fn pen_release_creates_brush_path_and_clears_state() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Pen;
    let mut state = DesignCanvasState {
        brush_points_world: vec![
            iced::Point::new(0.0, 0.0),
            iced::Point::new(10.0, 10.0),
            iced::Point::new(20.0, 0.0),
        ],
        brush_erasing: true,
        brush_erase_dirty: true,
        ..Default::default()
    };

    let action = canvas
        .handle_left_released(&mut state, iced::Point::new(20.0, 0.0))
        .expect("pen release should create path or redraw");

    match published(action) {
        crate::app::Message::Design(DesignMessage::CreateElement {
            element,
            start_editing,
            ..
        }) => {
            assert_eq!(element.kind, "path");
            assert!(!start_editing);
        }
        message => panic!("unexpected message: {message:?}"),
    }
    assert!(state.brush_points_world.is_empty());
    assert!(!state.brush_erasing);
    assert!(!state.brush_erase_dirty);
}

#[test]
fn eraser_release_with_dirty_state_publishes_snapshot() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Eraser;
    let mut state =
        DesignCanvasState { brush_erasing: true, brush_erase_dirty: true, ..Default::default() };

    let action = canvas
        .handle_left_released(&mut state, iced::Point::ORIGIN)
        .expect("eraser release should publish snapshot");

    match published(action) {
        crate::app::Message::Design(DesignMessage::Snapshot) => {}
        message => panic!("unexpected message: {message:?}"),
    }
    assert!(!state.brush_erasing);
    assert!(!state.brush_erase_dirty);
}

#[test]
fn shape_preview_release_creates_element_and_clears_preview() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Rectangle;
    let mut state = DesignCanvasState {
        tool_preview_start: Some(iced::Point::new(10.0, 20.0)),
        tool_preview_current: Some(iced::Point::new(60.0, 80.0)),
        ..Default::default()
    };

    let action = canvas
        .handle_left_released(&mut state, iced::Point::new(60.0, 80.0))
        .expect("preview release should create element");

    match published(action) {
        crate::app::Message::Design(DesignMessage::CreateElement {
            element, parent_id, ..
        }) => {
            assert_eq!(element.kind, "rectangle");
            assert_eq!(element.x, 10.0);
            assert_eq!(element.y, 20.0);
            assert_eq!(element.width, Some(serde_json::json!(50.0)));
            assert_eq!(element.height, Some(serde_json::json!(60.0)));
            assert!(parent_id.is_none());
        }
        message => panic!("unexpected message: {message:?}"),
    }
    assert!(state.tool_preview_start.is_none());
    assert!(state.tool_preview_current.is_none());
}

#[test]
fn selection_box_release_publishes_intersecting_ids() {
    let doc = DesignDoc {
        children: vec![
            rect("inside", 10.0, 10.0, 20.0, 20.0),
            rect("outside", 100.0, 100.0, 20.0, 20.0),
        ],
        ..Default::default()
    };
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let mut state = DesignCanvasState {
        selection_box_start: Some(iced::Point::new(0.0, 0.0)),
        ..Default::default()
    };

    let action = canvas
        .handle_left_released(&mut state, iced::Point::new(40.0, 40.0))
        .expect("selection release should publish");

    match published(action) {
        crate::app::Message::Design(DesignMessage::MultiSelect(ids)) => {
            assert_eq!(ids, vec!["inside".to_string()]);
        }
        message => panic!("unexpected message: {message:?}"),
    }
    assert!(state.selection_box_start.is_none());
}

#[test]
fn moving_release_without_movement_only_clears_drag_state() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let mut state = DesignCanvasState {
        moving_elements: Some((
            vec![("a".to_string(), iced::Point::new(1.0, 2.0))],
            iced::Point::new(0.0, 0.0),
            false,
        )),
        drop_target_frame_id: Some("frame".to_string()),
        ..Default::default()
    };

    assert!(canvas.handle_left_released(&mut state, iced::Point::ORIGIN).is_some());
    assert!(state.moving_elements.is_none());
    assert!(state.drop_target_frame_id.is_none());
}

#[test]
fn moving_release_reparents_moved_items() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let mut state = DesignCanvasState {
        moving_elements: Some((
            vec![
                ("a".to_string(), iced::Point::new(1.0, 2.0)),
                ("b".to_string(), iced::Point::new(3.0, 4.0)),
            ],
            iced::Point::new(0.0, 0.0),
            true,
        )),
        drop_target_frame_id: Some("frame".to_string()),
        ..Default::default()
    };

    let action = canvas
        .handle_left_released(&mut state, iced::Point::ORIGIN)
        .expect("moved release should publish reparent");

    match published(action) {
        crate::app::Message::Design(DesignMessage::ReparentElements(ids, parent)) => {
            assert_eq!(ids, vec!["a".to_string(), "b".to_string()]);
            assert_eq!(parent.as_deref(), Some("frame"));
        }
        message => panic!("unexpected message: {message:?}"),
    }
    assert!(state.drop_target_frame_id.is_none());
}

#[test]
fn resize_or_rotate_release_clears_state_and_snapshots() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let mut state = DesignCanvasState {
        resizing: Some((
            "a".to_string(),
            Handle::Right,
            iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(10.0, 10.0)),
        )),
        drag_start: Some(iced::Point::new(1.0, 1.0)),
        ..Default::default()
    };

    let action = canvas
        .handle_left_released(&mut state, iced::Point::ORIGIN)
        .expect("resize release should snapshot");

    match published(action) {
        crate::app::Message::Design(DesignMessage::Snapshot) => {}
        message => panic!("unexpected message: {message:?}"),
    }
    assert!(state.resizing.is_none());
    assert!(state.rotating.is_none());
    assert!(state.drag_start.is_none());
}

#[test]
fn hand_release_stops_panning() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Hand;
    let mut state = DesignCanvasState {
        is_panning: true,
        last_cursor_pos: Some(iced::Point::new(2.0, 3.0)),
        ..Default::default()
    };

    assert!(canvas.handle_left_released(&mut state, iced::Point::ORIGIN).is_some());
    assert!(!state.is_panning);
    assert!(state.last_cursor_pos.is_none());
}
