use super::interaction::mouse_interaction;
use super::{DragMode, HoverButtonKind, MindMapCanvasState};
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::canvas::layout::compute_layout_for_diagram;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, MindMapCanvasTool, MindMapDiagramType,
    MindMapDoodleStroke, MindMapLayoutFormat, OrgChartLayoutFormat, TimelineLayoutFormat,
    TreeLayoutFormat,
};
use iced::widget::canvas::Cache;
use iced::{Point, Rectangle, Size, Vector, mouse};
use std::collections::{HashMap, HashSet};

struct Fixture {
    doc: MindNode,
    cache: Cache,
    node_positions: HashMap<Vec<usize>, Point>,
    node_fills: HashMap<Vec<usize>, u32>,
    node_text_colors: HashMap<Vec<usize>, u32>,
    node_border_colors: HashMap<Vec<usize>, u32>,
    node_border_styles: HashMap<Vec<usize>, EdgeStyle>,
    node_priorities: HashMap<Vec<usize>, u8>,
    node_urls: HashMap<Vec<usize>, String>,
    collapsed_paths: HashSet<Vec<usize>>,
    edge_styles: HashMap<Vec<usize>, EdgeStyle>,
    edge_colors: HashMap<Vec<usize>, u32>,
    doodles: Vec<MindMapDoodleStroke>,
    custom_themes: Vec<crate::apps::mindmap::canvas::theme::MindMapCustomTheme>,
    tool: MindMapCanvasTool,
    ui_blocked_rects: Vec<Rectangle>,
}

impl Fixture {
    fn new() -> Self {
        Self {
            doc: MindNode {
                text: "root".into(),
                children: vec![MindNode {
                    text: "child".into(),
                    children: vec![MindNode { text: "leaf".into(), children: vec![] }],
                }],
            },
            cache: Cache::new(),
            node_positions: HashMap::new(),
            node_fills: HashMap::new(),
            node_text_colors: HashMap::new(),
            node_border_colors: HashMap::new(),
            node_border_styles: HashMap::new(),
            node_priorities: HashMap::new(),
            node_urls: HashMap::new(),
            collapsed_paths: HashSet::new(),
            edge_styles: HashMap::new(),
            edge_colors: HashMap::new(),
            doodles: Vec::new(),
            custom_themes: Vec::new(),
            tool: MindMapCanvasTool::Select,
            ui_blocked_rects: Vec::new(),
        }
    }

    fn canvas(&self) -> super::MindMapCanvas<'_> {
        super::MindMapCanvas {
            doc: &self.doc,
            cache: &self.cache,
            pan: Vector::ZERO,
            zoom: 1.0,
            selected_path: None,
            node_positions: &self.node_positions,
            diagram_type: MindMapDiagramType::MindMap,
            layout_format: MindMapLayoutFormat::RightAligned,
            org_chart_layout_format: OrgChartLayoutFormat::TopDown,
            fishbone_layout_format: FishboneLayoutFormat::HeadRight,
            timeline_layout_format: TimelineLayoutFormat::UpDown,
            bracket_layout_format: BracketLayoutFormat::BraceRight,
            tree_layout_format: TreeLayoutFormat::FanDown,
            node_fills: &self.node_fills,
            node_text_colors: &self.node_text_colors,
            node_border_colors: &self.node_border_colors,
            node_border_style: EdgeStyle::Solid,
            node_border_styles: &self.node_border_styles,
            node_priorities: &self.node_priorities,
            node_urls: &self.node_urls,
            collapsed_paths: &self.collapsed_paths,
            background: None,
            follow_theme_background: false,
            edge_style: EdgeStyle::Solid,
            edge_styles: &self.edge_styles,
            edge_colors: &self.edge_colors,
            canvas_tool: self.tool,
            doodle_rgba: 0x000000ff,
            doodle_width_px: 2.0,
            doodles: &self.doodles,
            ui_blocked_rects: self.ui_blocked_rects.clone(),
            theme_group: "classic",
            theme_variant: 0,
            custom_themes: &self.custom_themes,
            theme_panel_open: false,
        }
    }
}

fn bounds() -> Rectangle {
    Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0))
}

#[test]
fn mouse_interaction_respects_blocked_ui_and_missing_cursor() {
    let fixture = Fixture::new();
    let canvas = fixture.canvas();
    let state = MindMapCanvasState::default();

    let _ = mouse_interaction(
        &canvas,
        &state,
        bounds(),
        mouse::Cursor::Available(Point::new(400.0, 20.0)),
    );
    let _ = mouse_interaction(&canvas, &state, bounds(), mouse::Cursor::Unavailable);
}

#[test]
fn mouse_interaction_reflects_active_drag_modes() {
    let fixture = Fixture::new();
    let canvas = fixture.canvas();
    let mut state = MindMapCanvasState::default();

    state.drag_mode = DragMode::Pan;
    assert_eq!(
        mouse_interaction(
            &canvas,
            &state,
            bounds(),
            mouse::Cursor::Available(Point::new(200.0, 200.0))
        ),
        mouse::Interaction::Grabbing
    );

    state.drag_mode = DragMode::Node(vec![0]);
    assert_eq!(
        mouse_interaction(
            &canvas,
            &state,
            bounds(),
            mouse::Cursor::Available(Point::new(200.0, 200.0))
        ),
        mouse::Interaction::Grabbing
    );

    state.drag_mode = DragMode::DoodlePen;
    assert_eq!(
        mouse_interaction(
            &canvas,
            &state,
            bounds(),
            mouse::Cursor::Available(Point::new(200.0, 200.0))
        ),
        mouse::Interaction::Crosshair
    );

    state.drag_mode = DragMode::DoodleErase;
    assert_eq!(
        mouse_interaction(
            &canvas,
            &state,
            bounds(),
            mouse::Cursor::Available(Point::new(200.0, 200.0))
        ),
        mouse::Interaction::Crosshair
    );
}

#[test]
fn mouse_interaction_uses_tool_defaults() {
    for (tool, expected) in [
        (MindMapCanvasTool::Pen, mouse::Interaction::Crosshair),
        (MindMapCanvasTool::Eraser, mouse::Interaction::Crosshair),
        (MindMapCanvasTool::Pan, mouse::Interaction::Idle),
    ] {
        let mut fixture = Fixture::new();
        fixture.tool = tool;
        let canvas = fixture.canvas();
        let state = MindMapCanvasState::default();

        assert_eq!(
            mouse_interaction(
                &canvas,
                &state,
                bounds(),
                mouse::Cursor::Available(Point::new(200.0, 200.0))
            ),
            expected
        );
    }
}

#[test]
fn mouse_interaction_points_at_url_toggle_badge_and_add_buttons() {
    let mut fixture = Fixture::new();
    fixture.node_urls.insert(vec![0], "https://example.test".into());
    fixture.collapsed_paths.insert(vec![0]);
    let canvas = fixture.canvas();
    let layout = compute_layout_for_diagram(
        canvas.doc,
        canvas.node_positions,
        canvas.node_priorities,
        canvas.node_urls,
        canvas.collapsed_paths,
        canvas.diagram_type,
        canvas.layout_format,
        canvas.org_chart_layout_format,
        canvas.fishbone_layout_format,
        canvas.timeline_layout_format,
        canvas.bracket_layout_format,
        canvas.tree_layout_format,
    );
    let child = layout.nodes.iter().find(|node| node.path == vec![0]).unwrap();
    let rect = canvas.node_screen_rect(child);
    let r = (8.0 * canvas.zoom).clamp(4.0, 12.0);
    let pad = (8.0 * canvas.zoom).clamp(4.0, 10.0);
    let url_center = Point::new(rect.x + rect.width - pad - r, rect.y + rect.height / 2.0);
    let state = MindMapCanvasState::default();
    let _ = mouse_interaction(&canvas, &state, bounds(), mouse::Cursor::Available(url_center));

    let (_, toggle_center, toggle_r) = canvas
        .node_button_specs(&[0], rect)
        .into_iter()
        .find(|(kind, _, _)| *kind == HoverButtonKind::ToggleCollapse)
        .unwrap();
    assert_eq!(
        mouse_interaction(&canvas, &state, bounds(), mouse::Cursor::Available(toggle_center)),
        mouse::Interaction::Pointer
    );

    let badge = canvas.collapsed_count_badge_rect(&[0], toggle_center, toggle_r, 1);
    let badge_center = Point::new(badge.x + badge.width / 2.0, badge.y + badge.height / 2.0);
    assert_eq!(
        mouse_interaction(&canvas, &state, bounds(), mouse::Cursor::Available(badge_center)),
        mouse::Interaction::Pointer
    );

    fixture.collapsed_paths.clear();
    let canvas = fixture.canvas();
    let layout = compute_layout_for_diagram(
        canvas.doc,
        canvas.node_positions,
        canvas.node_priorities,
        canvas.node_urls,
        canvas.collapsed_paths,
        canvas.diagram_type,
        canvas.layout_format,
        canvas.org_chart_layout_format,
        canvas.fishbone_layout_format,
        canvas.timeline_layout_format,
        canvas.bracket_layout_format,
        canvas.tree_layout_format,
    );
    let child = layout.nodes.iter().find(|node| node.path == vec![0]).unwrap();
    let rect = canvas.node_screen_rect(child);
    let mut state = MindMapCanvasState::default();
    state.hovered_node = Some(vec![0]);
    let add_center = canvas
        .node_button_specs(&[0], rect)
        .into_iter()
        .find(|(kind, _, _)| *kind == HoverButtonKind::AddChild)
        .unwrap()
        .1;
    let _ = mouse_interaction(&canvas, &state, bounds(), mouse::Cursor::Available(add_center));
}
