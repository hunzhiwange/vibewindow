#![allow(unused_must_use)]
use super::handlers::{
    handle_canvas_context_menu_action, handle_canvas_context_menu_close,
    handle_canvas_context_menu_open, handle_create_element, handle_fit_to_element,
    handle_select_toolbar_icon, handle_tool_selected, handle_update_context_border,
    handle_update_context_fill, handle_update_context_shape, handle_zoom_fit,
};
use crate::app::message::design::{CanvasContextMenuAction, DesignMessage};
use crate::app::views::design::models::{DesignDoc, DesignElement, DesignTool};
use crate::app::views::design::state::{ContextPopoverType, DesignState};
use iced::{Point, Vector};

fn element(id: &str, x: f32, y: f32, width: f32, height: f32) -> DesignElement {
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

fn state_with_elements(elements: Vec<DesignElement>) -> DesignState {
    let mut doc = DesignDoc::default();
    doc.children = elements;
    DesignState::new(doc)
}

#[test]
fn tool_selected_updates_active_tool_and_brush_popover() {
    let mut state = state_with_elements(vec![]);
    state.context_popover = Some(ContextPopoverType::ToolbarShape);

    handle_tool_selected(&mut state, DesignTool::Pen);
    assert_eq!(state.active_tool, DesignTool::Pen);
    assert_eq!(state.context_popover, Some(ContextPopoverType::ToolbarBrush));

    handle_tool_selected(&mut state, DesignTool::Move);
    assert_eq!(state.context_popover, None);
}

#[test]
fn create_element_selects_element_and_sets_group() {
    let mut state = state_with_elements(vec![DesignElement {
        id: "parent".to_string(),
        group_id: 9,
        ..Default::default()
    }]);
    state.active_group_id = 3;

    handle_create_element(
        &mut state,
        DesignElement {
            id: "child".to_string(),
            content: Some("hello".to_string()),
            ..Default::default()
        },
        Some("parent".to_string()),
        true,
    );

    let child = state.doc.find_element("child").unwrap();
    assert_eq!(child.group_id, 9);
    assert_eq!(state.selected_element_id.as_deref(), Some("child"));
    assert!(state.selected_element_ids.contains("child"));
    assert_eq!(state.editing_id.as_deref(), Some("child"));
    assert_eq!(state.editing_content, "hello");
}

#[test]
fn create_element_resets_one_shot_shape_tool_without_editing() {
    let mut state = state_with_elements(vec![]);
    state.active_tool = DesignTool::Rectangle;
    state.editing_id = Some("old".to_string());
    state.editing_content = "old".to_string();

    handle_create_element(
        &mut state,
        DesignElement { id: "rect".to_string(), ..Default::default() },
        Some("missing".to_string()),
        false,
    );

    assert_eq!(state.active_tool, DesignTool::Move);
    assert_eq!(state.editing_id, None);
    assert!(state.editing_content.is_empty());
    assert!(state.doc.find_element("rect").is_some());
}

#[test]
fn zoom_fit_and_fit_to_element_update_view_transform() {
    let mut state = state_with_elements(vec![element("rect", 0.0, 0.0, 100.0, 50.0)]);

    handle_zoom_fit(&mut state, (1000.0, 700.0));
    assert!(state.zoom > 1.0);
    assert!(!state.show_zoom_menu);

    handle_fit_to_element(&mut state, "rect", (500.0, 400.0));
    assert_eq!(state.zoom, 2.0);
    assert_eq!(state.pan, Vector::new(150.0, 150.0));
}

#[test]
fn canvas_context_menu_open_selects_hit_or_clears_selection() {
    let mut state = state_with_elements(vec![element("rect", 0.0, 0.0, 10.0, 10.0)]);

    handle_canvas_context_menu_open(&mut state, Point::new(1.0, 2.0), Some("rect".to_string()));
    assert_eq!(state.canvas_context_menu_anchor, Some(Point::new(1.0, 2.0)));
    assert_eq!(state.selected_element_id.as_deref(), Some("rect"));
    assert!(state.selected_element_ids.contains("rect"));

    handle_canvas_context_menu_open(&mut state, Point::new(3.0, 4.0), None);
    assert_eq!(state.selected_element_id, None);
    assert!(state.selected_element_ids.is_empty());

    state.paste_anchor = Some(Point::new(9.0, 9.0));
    handle_canvas_context_menu_close(&mut state);
    assert_eq!(state.canvas_context_menu_anchor, None);
    assert_eq!(state.paste_anchor, None);
}

#[test]
fn canvas_context_menu_actions_clear_anchor_and_set_paste_anchor() {
    let mut state = state_with_elements(vec![]);
    state.canvas_context_menu_anchor = Some(Point::new(7.0, 8.0));
    state.selected_element_id = Some("rect".to_string());

    let _task = handle_canvas_context_menu_action(&mut state, CanvasContextMenuAction::Paste);
    assert_eq!(state.canvas_context_menu_anchor, None);
    assert_eq!(state.paste_anchor, Some(Point::new(7.0, 8.0)));

    for action in [
        CanvasContextMenuAction::Cut,
        CanvasContextMenuAction::Copy,
        CanvasContextMenuAction::Delete,
        CanvasContextMenuAction::MoveUp,
        CanvasContextMenuAction::MoveDown,
    ] {
        let _task = handle_canvas_context_menu_action(&mut state, action);
        assert_eq!(state.canvas_context_menu_anchor, None);
    }
}

#[test]
fn toolbar_icon_selection_updates_icon_state() {
    let mut state = state_with_elements(vec![]);
    state.icon_filter_query = "search".to_string();

    handle_select_toolbar_icon(&mut state, "lucide".to_string(), "star".to_string());

    assert_eq!(state.toolbar_icon_family, "lucide");
    assert_eq!(state.toolbar_icon_family_tab, "lucide");
    assert_eq!(state.toolbar_icon_name, "star");
    assert!(state.icon_filter_query.is_empty());
    assert_eq!(state.active_tool, DesignTool::Icon);
}

#[test]
fn context_shape_fill_and_border_build_property_update_tasks() {
    let mut state = state_with_elements(vec![DesignElement {
        id: "rect".to_string(),
        fill: Some(serde_json::json!("[{\"color\":\"#123456\"}]")),
        ..element("rect", 0.0, 0.0, 10.0, 10.0)
    }]);
    state.selected_element_id = Some("rect".to_string());
    state.context_popover = Some(ContextPopoverType::ToolbarShape);
    state.context_shape_group_hover = Some("basic".to_string());

    let _shape_task = handle_update_context_shape(&mut state, "ellipse".to_string());
    assert_eq!(state.context_popover, None);
    assert_eq!(state.context_shape_group_hover, None);

    for fill_type in ["color", "transparent", "none", "#ABCDEF", "unknown"] {
        let _task = handle_update_context_fill(&mut state, fill_type.to_string());
    }

    for border_type in ["solid|#111111", "dashed", "none", "#222222", "unknown"] {
        let _task = handle_update_context_border(&mut state, border_type.to_string());
    }
}

#[test]
fn context_updates_noop_without_selection() {
    let mut state = state_with_elements(vec![]);

    let _shape_task = handle_update_context_shape(&mut state, "ellipse".to_string());
    let _fill_task = handle_update_context_fill(&mut state, "color".to_string());
    let _border_task = handle_update_context_border(&mut state, "solid".to_string());

    assert_eq!(state.selected_element_id, None);
    assert!(!matches!(DesignMessage::ToggleZoomMenu, DesignMessage::Pan(_)));
}
