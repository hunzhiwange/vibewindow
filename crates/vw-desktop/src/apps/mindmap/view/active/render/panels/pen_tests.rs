use crate::apps::mindmap::model::default_doc;
use crate::apps::mindmap::state::{MindMapCanvasTool, MindMapTab};

fn base_tab() -> MindMapTab {
    MindMapTab::new("tab".to_string(), "Pen".to_string(), None, default_doc())
}

#[test]
fn pen_panel_returns_empty_container_for_non_pen_tools() {
    for tool in [MindMapCanvasTool::Pan, MindMapCanvasTool::Select, MindMapCanvasTool::Eraser] {
        let mut tab = base_tab();
        tab.canvas_tool = tool;

        let panel = super::pen_panel(&tab, 320.0, 44.0);
        std::hint::black_box(panel);
    }
}

#[test]
fn pen_panel_builds_palette_and_slider_for_pen_tool() {
    let mut tab = base_tab();
    tab.canvas_tool = MindMapCanvasTool::Pen;

    for (rgba, width) in [(0xFFFFFFFF, 0.25), (0x111827FF, 3.0), (0xA855F7FF, 42.0)] {
        tab.doodle_rgba = rgba;
        tab.doodle_width_px = width;

        let panel = super::pen_panel(&tab, 360.0, 44.0);
        std::hint::black_box(panel);
    }
}
