use super::render;
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::{MindMapCanvasTool, MindMapTab};
use iced::{Point, Vector};

fn tab() -> MindMapTab {
    MindMapTab::new(
        "tab-1".to_string(),
        "Mind map".to_string(),
        None,
        MindNode {
            text: "Root".to_string(),
            children: vec![MindNode { text: "Child".to_string(), children: Vec::new() }],
        },
    )
}

#[test]
fn render_builds_active_view_for_default_tab() {
    let tab = tab();

    let _element = render(&tab);
}

#[test]
fn render_builds_active_view_with_open_overlays_and_canvas_state() {
    let mut tab = tab();
    tab.selected_path = Some(vec![0]);
    tab.last_click_screen = Some(Point::new(10.0, 20.0));
    tab.pan = Vector::new(20.0, 30.0);
    tab.zoom = 1.25;
    tab.show_zoom_menu = true;
    tab.show_export_menu = true;
    tab.show_action_menu = true;
    tab.show_priority_picker = true;
    tab.show_theme_panel = true;
    tab.canvas_tool = MindMapCanvasTool::Pan;

    let _element = render(&tab);
}
