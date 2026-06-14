use crate::apps::mindmap::model::default_doc;
use crate::apps::mindmap::state::{MindMapCanvasTool, MindMapTab};

fn tab_with_tool(tool: MindMapCanvasTool) -> MindMapTab {
    let mut tab = MindMapTab::new("tab".to_string(), "Toolbar".to_string(), None, default_doc());
    tab.canvas_tool = tool;
    tab
}

#[test]
fn tool_toolbar_builds_all_active_tool_states() {
    for tool in [
        MindMapCanvasTool::Pan,
        MindMapCanvasTool::Select,
        MindMapCanvasTool::Pen,
        MindMapCanvasTool::Eraser,
    ] {
        let tab = tab_with_tool(tool);
        let toolbar = super::tool_toolbar(&tab, 180.0, 44.0);

        std::hint::black_box(toolbar);
    }
}

#[test]
fn tool_toolbar_accepts_compact_dimensions() {
    let tab = tab_with_tool(MindMapCanvasTool::Select);
    let toolbar = super::tool_toolbar(&tab, 1.0, 1.0);

    std::hint::black_box(toolbar);
}
