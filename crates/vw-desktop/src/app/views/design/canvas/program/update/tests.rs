use std::borrow::Cow;
use std::collections::HashSet;

use crate::app::views::design::canvas::types::DesignCanvasState;
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
        toolbar_icon_name: "arrow-up",
        mouse_wheel_zoom_enabled: false,
        show_slot_content: false,
        show_slot_overflow: false,
        color_picking: false,
        hover_disabled: false,
    }
}

fn frame(id: &str, x: f32, y: f32, width: f32, height: f32) -> DesignElement {
    DesignElement {
        id: id.to_string(),
        kind: "frame".to_string(),
        x,
        y,
        width: Some(serde_json::json!(width)),
        height: Some(serde_json::json!(height)),
        padding: Some(serde_json::json!(10.0)),
        ..Default::default()
    }
}

#[test]
fn drag_preview_supported_for_shape_tools_only() {
    assert!(super::tool_supports_drag_preview(DesignTool::Rectangle));
    assert!(super::tool_supports_drag_preview(DesignTool::StickyNote));
    assert!(!super::tool_supports_drag_preview(DesignTool::Move));
    assert!(!super::tool_supports_drag_preview(DesignTool::Text));
}

#[test]
fn root_frame_at_cursor_returns_topmost_frame_only() {
    let doc = DesignDoc {
        children: vec![
            frame("bottom", 0.0, 0.0, 100.0, 100.0),
            DesignElement {
                id: "shape".to_string(),
                kind: "rectangle".to_string(),
                x: 0.0,
                y: 0.0,
                width: Some(serde_json::json!(200.0)),
                height: Some(serde_json::json!(200.0)),
                ..Default::default()
            },
            frame("top", 20.0, 20.0, 100.0, 100.0),
        ],
        ..Default::default()
    };
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);

    assert_eq!(super::root_frame_at_cursor(&canvas, iced::Point::new(80.0, 90.0)), Some("top"));
    assert_eq!(super::root_frame_at_cursor(&canvas, iced::Point::new(12.0, 22.0)), Some("bottom"));
    assert!(super::root_frame_at_cursor(&canvas, iced::Point::new(500.0, 500.0)).is_none());
}

#[test]
fn frame_child_world_position_applies_pan_zoom_and_padding() {
    let doc =
        DesignDoc { children: vec![frame("frame", 10.0, 10.0, 100.0, 80.0)], ..Default::default() };
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);

    let point = super::frame_child_world_position(&canvas, "frame", iced::Point::new(50.0, 70.0));

    assert_eq!(point, iced::Point::new(0.0, 5.0));
}

#[test]
fn preview_world_rect_uses_parent_coordinates_when_present() {
    let doc =
        DesignDoc { children: vec![frame("frame", 10.0, 10.0, 100.0, 80.0)], ..Default::default() };
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let state = DesignCanvasState {
        tool_preview_start: Some(iced::Point::new(50.0, 70.0)),
        tool_preview_current: Some(iced::Point::new(70.0, 90.0)),
        tool_preview_parent_id: Some("frame".to_string()),
        ..Default::default()
    };

    let (start, current) = super::preview_world_rect(&canvas, &state).unwrap();

    assert_eq!(start, iced::Point::new(0.0, 5.0));
    assert_eq!(current, iced::Point::new(10.0, 15.0));
}

#[test]
fn preview_world_rect_defaults_current_to_start() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let state = DesignCanvasState {
        tool_preview_start: Some(iced::Point::new(30.0, 60.0)),
        ..Default::default()
    };

    let (start, current) = super::preview_world_rect(&canvas, &state).unwrap();

    assert_eq!(start, iced::Point::new(10.0, 20.0));
    assert_eq!(current, iced::Point::new(10.0, 20.0));
}

#[test]
fn build_created_element_uses_drag_size_for_rectangle() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Rectangle;
    let state = DesignCanvasState {
        tool_preview_start: Some(iced::Point::new(30.0, 60.0)),
        tool_preview_current: Some(iced::Point::new(70.0, 100.0)),
        ..Default::default()
    };

    let (element, parent_id, start_editing) =
        super::build_created_element(&canvas, &state, iced::Point::new(70.0, 100.0)).unwrap();

    assert_eq!(element.kind, "rectangle");
    assert_eq!(element.x, 10.0);
    assert_eq!(element.y, 20.0);
    assert_eq!(element.width, Some(serde_json::json!(20.0)));
    assert_eq!(element.height, Some(serde_json::json!(20.0)));
    assert!(parent_id.is_none());
    assert!(!start_editing);
}

#[test]
fn build_created_element_uses_default_size_for_tiny_line_drag() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Line;
    let state = DesignCanvasState {
        tool_preview_start: Some(iced::Point::new(30.0, 60.0)),
        tool_preview_current: Some(iced::Point::new(30.2, 60.2)),
        ..Default::default()
    };

    let (element, _, _) =
        super::build_created_element(&canvas, &state, iced::Point::new(30.2, 60.2)).unwrap();

    assert_eq!(element.kind, "line");
    assert_eq!(element.width, Some(serde_json::json!(160.0)));
    assert_eq!(element.height, Some(serde_json::json!(2.0)));
}

#[test]
fn build_created_text_inside_frame_returns_parent_and_local_position() {
    let doc =
        DesignDoc { children: vec![frame("frame", 10.0, 10.0, 100.0, 80.0)], ..Default::default() };
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Text;
    let state = DesignCanvasState::default();

    let (element, parent_id, start_editing) =
        super::build_created_element(&canvas, &state, iced::Point::new(50.0, 70.0)).unwrap();

    assert_eq!(element.kind, "text");
    assert_eq!(element.x, 0.0);
    assert_eq!(element.y, 5.0);
    assert_eq!(parent_id.as_deref(), Some("frame"));
    assert!(start_editing);
}
