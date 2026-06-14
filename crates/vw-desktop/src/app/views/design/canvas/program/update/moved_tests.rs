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
        pan: iced::Vector::new(10.0, 20.0),
        zoom: 2.0,
        selected_id: None,
        selected_ids,
        selected_fill_index: None,
        editing_id: None,
        active_tool: DesignTool::Move,
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
fn hover_disabled_clears_hover_state() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.hover_disabled = true;
    let mut state = DesignCanvasState {
        hovered_id: Some("a".to_string()),
        hovered_tailwind_selection: Some(("tw".to_string(), vec![0])),
        ..Default::default()
    };

    assert!(canvas.handle_cursor_moved(&mut state, iced::Point::ORIGIN).is_some());
    assert!(state.hovered_id.is_none());
    assert!(state.hovered_tailwind_selection.is_none());
}

#[test]
fn pen_adds_world_point_only_after_distance_threshold() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Pen;
    let mut state = DesignCanvasState {
        brush_points_world: vec![iced::Point::new(0.0, 0.0)],
        ..Default::default()
    };

    assert!(canvas.handle_cursor_moved(&mut state, iced::Point::new(10.5, 20.0)).is_none());
    assert_eq!(state.brush_points_world.len(), 1);

    assert!(canvas.handle_cursor_moved(&mut state, iced::Point::new(20.0, 30.0)).is_some());
    assert_eq!(state.brush_points_world.last().copied(), Some(iced::Point::new(5.0, 5.0)));
}

#[test]
fn eraser_drag_publishes_world_erase_point() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Eraser;
    let mut state = DesignCanvasState { brush_erasing: true, ..Default::default() };

    let action = canvas
        .handle_cursor_moved(&mut state, iced::Point::new(30.0, 50.0))
        .expect("eraser drag should publish");

    match published(action) {
        crate::app::Message::Design(DesignMessage::EraseBrushAt(point, radius)) => {
            assert_eq!(point, iced::Point::new(10.0, 15.0));
            assert_eq!(radius, 15.0);
        }
        message => panic!("unexpected message: {message:?}"),
    }
    assert!(state.brush_erase_dirty);
}

#[test]
fn shape_preview_tracks_current_cursor() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Rectangle;
    let mut state = DesignCanvasState {
        tool_preview_start: Some(iced::Point::new(1.0, 2.0)),
        ..Default::default()
    };

    assert!(canvas.handle_cursor_moved(&mut state, iced::Point::new(30.0, 40.0)).is_some());
    assert_eq!(state.tool_preview_current, Some(iced::Point::new(30.0, 40.0)));
}

#[test]
fn moving_elements_waits_until_threshold_then_publishes_batch_update() {
    let doc = DesignDoc { children: vec![rect("a", 5.0, 6.0, 20.0, 20.0)], ..Default::default() };
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let mut state = DesignCanvasState {
        moving_elements: Some((
            vec![("a".to_string(), iced::Point::new(5.0, 6.0))],
            iced::Point::new(10.0, 10.0),
            false,
        )),
        ..Default::default()
    };

    assert!(canvas.handle_cursor_moved(&mut state, iced::Point::new(11.0, 10.0)).is_none());

    let action = canvas
        .handle_cursor_moved(&mut state, iced::Point::new(16.0, 14.0))
        .expect("movement after threshold should publish update");

    match published(action) {
        crate::app::Message::Design(DesignMessage::BatchPropertiesUpdateTransient(updates)) => {
            assert_eq!(updates[0].0, "a");
            assert_eq!(updates[0].1[0], ("x".to_string(), serde_json::json!(8.0)));
            assert_eq!(updates[0].1[1], ("y".to_string(), serde_json::json!(8.0)));
        }
        message => panic!("unexpected message: {message:?}"),
    }
    assert!(state.moving_elements.as_ref().is_some_and(|(_, _, moved)| *moved));
}

#[test]
fn resizing_publishes_transient_dimensions() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let mut state = DesignCanvasState {
        resizing: Some((
            "a".to_string(),
            Handle::BottomRight,
            iced::Rectangle::new(iced::Point::new(5.0, 6.0), iced::Size::new(20.0, 10.0)),
        )),
        drag_start: Some(iced::Point::new(10.0, 10.0)),
        ..Default::default()
    };

    let action = canvas
        .handle_cursor_moved(&mut state, iced::Point::new(14.0, 16.0))
        .expect("resize should publish dimensions");

    match published(action) {
        crate::app::Message::Design(DesignMessage::PropertiesUpdateTransient(id, updates)) => {
            assert_eq!(id, "a");
            assert_eq!(updates[2], ("width".to_string(), serde_json::json!(22.0)));
            assert_eq!(updates[3], ("height".to_string(), serde_json::json!(13.0)));
        }
        message => panic!("unexpected message: {message:?}"),
    }
}

#[test]
fn panning_publishes_new_pan_and_updates_last_cursor() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let mut state = DesignCanvasState {
        is_panning: true,
        last_cursor_pos: Some(iced::Point::new(12.0, 24.0)),
        ..Default::default()
    };

    let action = canvas
        .handle_cursor_moved(&mut state, iced::Point::new(15.0, 20.0))
        .expect("panning should publish pan");

    match published(action) {
        crate::app::Message::Design(DesignMessage::Pan(pan)) => {
            assert_eq!(pan, iced::Vector::new(13.0, 16.0));
        }
        message => panic!("unexpected message: {message:?}"),
    }
    assert_eq!(state.last_cursor_pos, Some(iced::Point::new(15.0, 20.0)));
}

#[test]
fn hover_hit_updates_hovered_id() {
    let doc = DesignDoc { children: vec![rect("a", 0.0, 0.0, 30.0, 30.0)], ..Default::default() };
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.pan = iced::Vector::new(0.0, 0.0);
    canvas.zoom = 1.0;
    let mut state = DesignCanvasState::default();

    assert!(canvas.handle_cursor_moved(&mut state, iced::Point::new(5.0, 5.0)).is_some());
    assert_eq!(state.hovered_id.as_deref(), Some("a"));
}

#[test]
fn selection_box_motion_requests_redraw() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let mut state = DesignCanvasState {
        selection_box_start: Some(iced::Point::new(1.0, 1.0)),
        ..Default::default()
    };

    assert!(canvas.handle_cursor_moved(&mut state, iced::Point::new(3.0, 4.0)).is_some());
}
