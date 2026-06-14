use std::borrow::Cow;
use std::collections::HashSet;

use crate::app::message::DesignMessage;
use crate::app::views::design::canvas::types::{
    DesignCanvasState, MeshDragKind, MeshDragState, SelectedMeshHandle,
};
use crate::app::views::design::models::{DesignDoc, DesignElement};

fn canvas<'a>(
    doc: &'a DesignDoc,
    cache: &'a iced::widget::canvas::Cache,
    selected_ids: &'a HashSet<String>,
    selected_id: Option<&'a str>,
) -> super::DesignCanvas<'a> {
    super::DesignCanvas {
        doc: Cow::Borrowed(doc),
        cache,
        pan: iced::Vector::new(0.0, 0.0),
        zoom: 1.0,
        selected_id,
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

fn mesh_element() -> DesignElement {
    DesignElement {
        id: "mesh".to_string(),
        kind: "rect".to_string(),
        width: Some(serde_json::json!(100.0)),
        height: Some(serde_json::json!(100.0)),
        fill: Some(serde_json::json!([{
            "type": "mesh_gradient",
            "enabled": true,
            "columns": 2,
            "rows": 2,
            "colors": ["#000000", "#111111", "#222222", "#333333"],
            "points": [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
            "handles": [
                [0.2, 0.2, 0.3, 0.3, 0.4, 0.4, 0.5, 0.5],
                [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]
            ],
            "selected_point_index": 0
        }])),
        ..Default::default()
    }
}

fn published(action: iced::widget::canvas::Action<crate::app::Message>) -> crate::app::Message {
    action.into_inner().0.expect("action should publish a message")
}

#[test]
fn escape_clears_interaction_state_without_selection() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids, None);
    let mut state = DesignCanvasState {
        mesh_drag: Some(MeshDragState {
            element_id: "mesh".to_string(),
            fill_index: 0,
            point_index: 0,
            kind: MeshDragKind::Point,
            has_moved: true,
            start_cursor_u: 0.0,
            start_cursor_v: 0.0,
            start_point_x: 0.0,
            start_point_y: 0.0,
            start_handles: [0.0; 8],
        }),
        selected_mesh_handle: Some(SelectedMeshHandle {
            element_id: "mesh".to_string(),
            fill_index: 0,
            point_index: 0,
            handle_index: 0,
        }),
        tool_preview_start: Some(iced::Point::new(1.0, 1.0)),
        tool_preview_current: Some(iced::Point::new(2.0, 2.0)),
        tool_preview_parent_id: Some("frame".to_string()),
        brush_points_world: vec![iced::Point::new(3.0, 3.0)],
        brush_erasing: true,
        brush_erase_dirty: true,
        ..Default::default()
    };

    let action = canvas.update_key_pressed(
        &mut state,
        &iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
    );

    assert!(action.is_some());
    assert!(state.mesh_drag.is_none());
    assert!(state.selected_mesh_handle.is_none());
    assert!(state.tool_preview_start.is_none());
    assert!(state.tool_preview_current.is_none());
    assert!(state.tool_preview_parent_id.is_none());
    assert!(state.brush_points_world.is_empty());
    assert!(!state.brush_erasing);
    assert!(!state.brush_erase_dirty);
}

#[test]
fn escape_unselects_mesh_point_with_transient_fill_update() {
    let doc = DesignDoc { children: vec![mesh_element()], ..Default::default() };
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids, Some("mesh"));
    let mut state = DesignCanvasState::default();

    let action = canvas
        .update_key_pressed(
            &mut state,
            &iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
        )
        .expect("escape should publish mesh update");

    match published(action) {
        crate::app::Message::Design(DesignMessage::PropertyUpdateTransient(id, key, value)) => {
            assert_eq!(id, "mesh");
            assert_eq!(key, "fill");
            assert_eq!(value[0]["selected_point_index"], serde_json::Value::Null);
        }
        message => panic!("unexpected message: {message:?}"),
    }
}

#[test]
fn delete_without_selected_element_is_ignored() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids, None);
    let mut state = DesignCanvasState::default();

    assert!(
        canvas
            .update_key_pressed(
                &mut state,
                &iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete),
            )
            .is_none()
    );
}

#[test]
fn delete_resets_selected_mesh_point_handles() {
    let doc = DesignDoc { children: vec![mesh_element()], ..Default::default() };
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids, Some("mesh"));
    let mut state = DesignCanvasState::default();

    let action = canvas
        .update_key_pressed(
            &mut state,
            &iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete),
        )
        .expect("delete should publish fill update");

    match published(action) {
        crate::app::Message::Design(DesignMessage::PropertyUpdate(id, key, value)) => {
            assert_eq!(id, "mesh");
            assert_eq!(key, "fill");
            assert_eq!(
                value[0]["handles"][0],
                serde_json::json!([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
            );
        }
        message => panic!("unexpected message: {message:?}"),
    }
}

#[test]
fn backspace_resets_only_selected_mesh_handle() {
    let doc = DesignDoc { children: vec![mesh_element()], ..Default::default() };
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids, Some("mesh"));
    let mut state = DesignCanvasState {
        selected_mesh_handle: Some(SelectedMeshHandle {
            element_id: "mesh".to_string(),
            fill_index: 0,
            point_index: 0,
            handle_index: 1,
        }),
        ..Default::default()
    };

    let action = canvas
        .update_key_pressed(
            &mut state,
            &iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace),
        )
        .expect("backspace should publish fill update");

    match published(action) {
        crate::app::Message::Design(DesignMessage::PropertyUpdate(_, _, value)) => {
            assert_eq!(
                value[0]["handles"][0],
                serde_json::json!([0.2, 0.2, 0.0, 0.0, 0.4, 0.4, 0.5, 0.5])
            );
        }
        message => panic!("unexpected message: {message:?}"),
    }
    assert!(state.selected_mesh_handle.is_none());
}

#[test]
fn unrelated_key_is_not_handled() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids, None);
    let mut state = DesignCanvasState::default();

    assert!(
        canvas
            .update_key_pressed(&mut state, &iced::keyboard::Key::Character("x".into()))
            .is_none()
    );
}
