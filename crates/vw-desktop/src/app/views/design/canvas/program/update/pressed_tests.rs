use std::borrow::Cow;
use std::collections::HashSet;

use crate::app::message::DesignMessage;
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
fn pen_press_starts_brush_path_in_world_space() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Pen;
    let mut state = DesignCanvasState {
        brush_points_world: vec![iced::Point::new(99.0, 99.0)],
        brush_erasing: true,
        brush_erase_dirty: true,
        ..Default::default()
    };

    assert!(canvas.handle_left_pressed(&mut state, iced::Point::new(30.0, 60.0), false).is_some());
    assert_eq!(state.brush_points_world, vec![iced::Point::new(10.0, 20.0)]);
    assert!(!state.brush_erasing);
    assert!(!state.brush_erase_dirty);
}

#[test]
fn eraser_press_enters_erasing_mode_and_publishes_first_erase() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Eraser;
    let mut state = DesignCanvasState::default();

    let action = canvas
        .handle_left_pressed(&mut state, iced::Point::new(30.0, 50.0), false)
        .expect("eraser press should publish");

    match published(action) {
        crate::app::Message::Design(DesignMessage::EraseBrushAt(point, radius)) => {
            assert_eq!(point, iced::Point::new(10.0, 15.0));
            assert_eq!(radius, 15.0);
        }
        message => panic!("unexpected message: {message:?}"),
    }
    assert!(state.brush_erasing);
    assert!(state.brush_erase_dirty);
}

#[test]
fn hand_press_starts_panning() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Hand;
    let mut state = DesignCanvasState::default();

    assert!(canvas.handle_left_pressed(&mut state, iced::Point::new(5.0, 6.0), false).is_some());
    assert!(state.is_panning);
    assert_eq!(state.last_cursor_pos, Some(iced::Point::new(5.0, 6.0)));
}

#[test]
fn text_tool_creates_text_element_at_world_position() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Text;
    let mut state = DesignCanvasState::default();

    let action = canvas
        .handle_left_pressed(&mut state, iced::Point::new(30.0, 60.0), false)
        .expect("text press should create element");

    match published(action) {
        crate::app::Message::Design(DesignMessage::CreateElement {
            element,
            parent_id,
            start_editing,
        }) => {
            assert_eq!(element.kind, "text");
            assert_eq!(element.x, 10.0);
            assert_eq!(element.y, 20.0);
            assert!(parent_id.is_none());
            assert!(start_editing);
        }
        message => panic!("unexpected message: {message:?}"),
    }
}

#[test]
fn icon_tool_uses_toolbar_icon_metadata() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Icon;
    let mut state = DesignCanvasState::default();

    let action = canvas
        .handle_left_pressed(&mut state, iced::Point::new(30.0, 60.0), false)
        .expect("icon press should create element");

    match published(action) {
        crate::app::Message::Design(DesignMessage::CreateElement {
            element,
            start_editing,
            ..
        }) => {
            assert_eq!(element.kind, "icon_font");
            assert_eq!(element.icon_font_family.as_deref(), Some("phosphor"));
            assert_eq!(element.icon_font_name.as_deref(), Some("arrow-up"));
            assert_eq!(element.name.as_deref(), Some("Arrow Up"));
            assert!(!start_editing);
        }
        message => panic!("unexpected message: {message:?}"),
    }
}

#[test]
fn rectangle_tool_starts_drag_preview() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::Rectangle;
    let mut state = DesignCanvasState::default();

    assert!(canvas.handle_left_pressed(&mut state, iced::Point::new(30.0, 60.0), false).is_some());
    assert_eq!(state.tool_preview_start, Some(iced::Point::new(30.0, 60.0)));
    assert_eq!(state.tool_preview_current, Some(iced::Point::new(30.0, 60.0)));
}

#[test]
fn move_tool_selects_hit_element_and_prepares_move() {
    let doc = DesignDoc { children: vec![rect("a", 10.0, 20.0, 30.0, 30.0)], ..Default::default() };
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let mut state = DesignCanvasState::default();

    let action = canvas
        .handle_left_pressed(&mut state, iced::Point::new(35.0, 65.0), false)
        .expect("move click should select hit element");

    match published(action) {
        crate::app::Message::Design(DesignMessage::ElementSelected(id)) => assert_eq!(id, "a"),
        message => panic!("unexpected message: {message:?}"),
    }
    assert!(state.moving_elements.as_ref().is_some_and(|(items, _, moved)| {
        items == &vec![("a".to_string(), iced::Point::new(10.0, 20.0))] && !*moved
    }));
}

#[test]
fn move_tool_blank_space_starts_selection_box() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let canvas = canvas(&doc, &cache, &selected_ids);
    let mut state = DesignCanvasState::default();

    assert!(
        canvas.handle_left_pressed(&mut state, iced::Point::new(100.0, 120.0), false).is_some()
    );
    assert_eq!(state.selection_box_start, Some(iced::Point::new(100.0, 120.0)));
}

#[test]
fn non_move_blank_space_submits_edit() {
    let doc = DesignDoc::default();
    let cache = iced::widget::canvas::Cache::new();
    let selected_ids = HashSet::new();
    let mut canvas = canvas(&doc, &cache, &selected_ids);
    canvas.active_tool = DesignTool::ImportImage;
    let mut state = DesignCanvasState::default();

    let action = canvas
        .handle_left_pressed(&mut state, iced::Point::new(100.0, 120.0), false)
        .expect("non-move blank click should submit edit");

    match published(action) {
        crate::app::Message::Design(DesignMessage::EditSubmit) => {}
        message => panic!("unexpected message: {message:?}"),
    }
}
