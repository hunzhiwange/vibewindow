use super::{
    DragMode, ERASER_RADIUS_PX, HoverButtonKind, MindMapCanvasState, PEN_PANEL_GAP, PEN_PANEL_H,
    PEN_PANEL_W, TOOLBAR_H, TOOLBAR_MARGIN, TOOLBAR_W,
};
use iced::Point;

#[test]
fn canvas_state_default_starts_idle() {
    let state = MindMapCanvasState::default();

    assert!(matches!(state.drag_mode, DragMode::None));
    assert_eq!(state.last_cursor, None);
    assert_eq!(state.hovered_node, None);
    assert!(state.doodle_points_world.is_empty());
    assert_eq!(state.last_click_at, None);
    assert_eq!(state.last_click_node, None);
    assert_eq!(state.last_click_pos, None);
}

#[test]
fn drag_mode_variants_keep_their_payloads() {
    let modes = [
        DragMode::None,
        DragMode::Pan,
        DragMode::Node(vec![1, 2]),
        DragMode::DoodlePen,
        DragMode::DoodleErase,
    ];

    assert!(matches!(modes[0], DragMode::None));
    assert!(matches!(modes[1], DragMode::Pan));
    assert!(matches!(&modes[2], DragMode::Node(path) if path == &vec![1, 2]));
    assert!(matches!(modes[3], DragMode::DoodlePen));
    assert!(matches!(modes[4], DragMode::DoodleErase));
}

#[test]
fn hover_button_kinds_are_distinct() {
    let kinds =
        [HoverButtonKind::ToggleCollapse, HoverButtonKind::AddChild, HoverButtonKind::AddSibling];

    assert_eq!(
        std::mem::discriminant(&kinds[0]),
        std::mem::discriminant(&HoverButtonKind::ToggleCollapse)
    );
    assert_ne!(std::mem::discriminant(&kinds[0]), std::mem::discriminant(&kinds[1]));
    assert_ne!(std::mem::discriminant(&kinds[1]), std::mem::discriminant(&kinds[2]));
}

#[test]
fn ui_geometry_constants_match_expected_contract() {
    assert_eq!(ERASER_RADIUS_PX, 30.0);
    assert_eq!(TOOLBAR_W, 168.0);
    assert_eq!(TOOLBAR_H, 40.0);
    assert_eq!(TOOLBAR_MARGIN, 14.0);
    assert_eq!(PEN_PANEL_W, 460.0);
    assert_eq!(PEN_PANEL_H, 40.0);
    assert_eq!(PEN_PANEL_GAP, 8.0);
}

#[test]
fn mutable_state_fields_can_track_interaction_progress() {
    let mut state = MindMapCanvasState::default();
    state.drag_mode = DragMode::Node(vec![0, 1]);
    state.last_cursor = Some(Point::new(10.0, 20.0));
    state.hovered_node = Some(vec![0]);
    state.doodle_points_world.push(Point::new(1.0, 2.0));
    state.last_click_node = Some(vec![0, 1]);
    state.last_click_pos = Some(Point::new(3.0, 4.0));

    assert!(matches!(state.drag_mode, DragMode::Node(ref path) if path == &vec![0, 1]));
    assert_eq!(state.last_cursor, Some(Point::new(10.0, 20.0)));
    assert_eq!(state.hovered_node, Some(vec![0]));
    assert_eq!(state.doodle_points_world, vec![Point::new(1.0, 2.0)]);
    assert_eq!(state.last_click_node, Some(vec![0, 1]));
    assert_eq!(state.last_click_pos, Some(Point::new(3.0, 4.0)));
}
