use super::ui::cursor_in_blocked_ui;
use super::DragMode;
use crate::apps::mindmap::state::MindMapCanvasTool;
use iced::{Point, Rectangle, Size};

#[test]
fn cursor_in_blocked_ui_ignores_missing_cursor_and_active_drag() {
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0));

    assert!(!cursor_in_blocked_ui(
        bounds,
        None,
        &DragMode::None,
        MindMapCanvasTool::Select,
        &[],
    ));
    assert!(!cursor_in_blocked_ui(
        bounds,
        Some(Point::new(400.0, 20.0)),
        &DragMode::Pan,
        MindMapCanvasTool::Select,
        &[],
    ));
}

#[test]
fn cursor_in_blocked_ui_detects_toolbar_and_snapped_custom_rect() {
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0));
    let custom = [Rectangle::new(Point::new(900.0, 900.0), Size::new(100.0, 100.0))];

    assert!(cursor_in_blocked_ui(
        bounds,
        Some(Point::new(400.0, 20.0)),
        &DragMode::None,
        MindMapCanvasTool::Select,
        &[],
    ));
    assert!(cursor_in_blocked_ui(
        bounds,
        Some(Point::new(720.0, 520.0)),
        &DragMode::None,
        MindMapCanvasTool::Select,
        &custom,
    ));
}
