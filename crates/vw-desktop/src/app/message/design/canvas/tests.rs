#![allow(unused_must_use)]
use crate::app::App;
use crate::app::message::design::DesignMessage;
use crate::app::views::design::models::{DesignDoc, DesignElement, DesignTool};
use crate::app::views::design::state::{ContextPopoverType, DesignState};
use iced::{Point, Vector};

fn new_app_with_design_state(state: DesignState) -> App {
    let mut app = App::new().0;
    let tab_id = "design".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, state);
    app
}

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

#[test]
fn update_noops_without_active_design_state() {
    let mut app = App::new().0;

    super::update(&mut app, DesignMessage::ZoomIn);

    assert!(app.active_design_state().is_none());
}

#[test]
fn pan_zoom_and_zoom_buttons_update_state() {
    let mut app = new_app_with_design_state(DesignState::new(DesignDoc::default()));

    super::update(&mut app, DesignMessage::Pan(Vector::new(10.0, 20.0)));
    assert_eq!(app.active_design_state().unwrap().pan, Vector::new(10.0, 20.0));

    super::update(&mut app, DesignMessage::Zoom(2.0, Some(Point::new(20.0, 20.0))));
    assert_eq!(app.active_design_state().unwrap().zoom, 2.0);

    super::update(&mut app, DesignMessage::ZoomOut);
    assert!(app.active_design_state().unwrap().zoom < 2.0);

    super::update(&mut app, DesignMessage::ZoomSet(99.0));
    assert_eq!(app.active_design_state().unwrap().zoom, 10.0);

    super::update(&mut app, DesignMessage::ZoomPresetSelected("30%".to_string()));
    assert_eq!(app.active_design_state().unwrap().zoom, 0.3);

    super::update(&mut app, DesignMessage::ZoomPresetSelected("125%".to_string()));
    assert_eq!(app.active_design_state().unwrap().zoom, 1.25);

    super::update(&mut app, DesignMessage::ZoomPresetSelected("bad".to_string()));
    assert_eq!(app.active_design_state().unwrap().zoom, 1.25);
}

#[test]
fn canvas_update_handles_tools_brush_settings_and_popovers() {
    let mut app = new_app_with_design_state(DesignState::new(DesignDoc::default()));

    super::update(&mut app, DesignMessage::ToolSelected(DesignTool::Pen));
    assert_eq!(app.active_design_state().unwrap().active_tool, DesignTool::Pen);
    assert_eq!(
        app.active_design_state().unwrap().context_popover,
        Some(ContextPopoverType::ToolbarBrush)
    );

    super::update(&mut app, DesignMessage::SetBrushColor("fill #123456AA".to_string()));
    super::update(&mut app, DesignMessage::SetBrushWidth(99.0));
    assert_eq!(app.active_design_state().unwrap().brush_color_hex, "#123456AA");
    assert_eq!(app.active_design_state().unwrap().brush_width_px, 18.0);

    super::update(
        &mut app,
        DesignMessage::ToggleContextPopover(Some(ContextPopoverType::ToolbarShape)),
    );
    assert_eq!(
        app.active_design_state().unwrap().context_popover,
        Some(ContextPopoverType::ToolbarShape)
    );
    super::update(
        &mut app,
        DesignMessage::ToggleContextPopover(Some(ContextPopoverType::ToolbarShape)),
    );
    assert_eq!(app.active_design_state().unwrap().context_popover, None);
}

#[test]
fn canvas_update_creates_reparents_and_erases_brushes() {
    let mut doc = DesignDoc::default();
    doc.children.push(DesignElement {
        id: "brush".to_string(),
        kind: "path".to_string(),
        class: Some("vw-brush-stroke".to_string()),
        geometry: Some("M 0 0 L 1 0".to_string()),
        ..Default::default()
    });
    let mut app = new_app_with_design_state(DesignState::new(doc));

    super::update(&mut app, DesignMessage::EraseBrushAt(Point::new(0.0, 0.0), 0.0));
    assert!(app.active_design_state().unwrap().doc.find_element("brush").is_some());

    super::update(&mut app, DesignMessage::EraseBrushAt(Point::new(0.0, 0.0), 5.0));
    assert!(app.active_design_state().unwrap().doc.find_element("brush").is_none());

    super::update(
        &mut app,
        DesignMessage::CreateElement {
            element: element("rect", 0.0, 0.0, 10.0, 10.0),
            parent_id: None,
            start_editing: false,
        },
    );
    assert!(app.active_design_state().unwrap().doc.find_element("rect").is_some());

    super::update(&mut app, DesignMessage::ReparentElements(vec!["missing".to_string()], None));
}

#[test]
fn canvas_update_handles_context_menu_and_toolbar_icon_state() {
    let mut doc = DesignDoc::default();
    doc.children.push(element("rect", 0.0, 0.0, 10.0, 10.0));
    let mut app = new_app_with_design_state(DesignState::new(doc));

    super::update(
        &mut app,
        DesignMessage::CanvasContextMenuOpen(Point::new(3.0, 4.0), Some("rect".to_string())),
    );
    assert_eq!(
        app.active_design_state().unwrap().canvas_context_menu_anchor,
        Some(Point::new(3.0, 4.0))
    );

    super::update(&mut app, DesignMessage::CanvasContextMenuClose);
    assert_eq!(app.active_design_state().unwrap().canvas_context_menu_anchor, None);

    super::update(&mut app, DesignMessage::SetIconFilter("star".to_string()));
    super::update(&mut app, DesignMessage::SetToolbarIconFamilyTab("lucide".to_string()));
    super::update(
        &mut app,
        DesignMessage::SelectToolbarIcon { family: "lucide".to_string(), name: "star".to_string() },
    );
    assert_eq!(app.active_design_state().unwrap().toolbar_icon_name, "star");
    assert_eq!(app.active_design_state().unwrap().active_tool, DesignTool::Icon);
}
