use super::layout_for_canvas;
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, MindMapCanvasTool, MindMapDiagramType,
    MindMapDoodleStroke, MindMapLayoutFormat, OrgChartLayoutFormat, TimelineLayoutFormat,
    TreeLayoutFormat,
};
use iced::widget::canvas::Cache;
use iced::{Point, Vector};
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
}

impl Fixture {
    fn new() -> Self {
        Self {
            doc: MindNode {
                text: "root".into(),
                children: vec![
                    MindNode {
                        text: "A".into(),
                        children: vec![MindNode { text: "A1".into(), children: vec![] }],
                    },
                    MindNode { text: "B".into(), children: vec![] },
                ],
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
        }
    }

    fn canvas(&self, diagram_type: MindMapDiagramType) -> super::super::MindMapCanvas<'_> {
        super::super::MindMapCanvas {
            doc: &self.doc,
            cache: &self.cache,
            pan: Vector::ZERO,
            zoom: 1.0,
            selected_path: None,
            node_positions: &self.node_positions,
            diagram_type,
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
            canvas_tool: MindMapCanvasTool::Select,
            doodle_rgba: 0x000000ff,
            doodle_width_px: 2.0,
            doodles: &self.doodles,
            ui_blocked_rects: Vec::new(),
            theme_group: "classic",
            theme_variant: 0,
            custom_themes: &self.custom_themes,
            theme_panel_open: false,
        }
    }
}

#[test]
fn layout_for_canvas_uses_canvas_layout_inputs() {
    let mut fixture = Fixture::new();
    fixture.collapsed_paths.insert(vec![0]);
    fixture.node_priorities.insert(vec![1], 10);
    fixture.node_urls.insert(vec![1], "https://example.test".into());
    let canvas = fixture.canvas(MindMapDiagramType::MindMap);

    let layout = layout_for_canvas(&canvas);

    assert!(layout.nodes.iter().any(|node| node.path == vec![0]));
    assert!(!layout.nodes.iter().any(|node| node.path == vec![0, 0]));
    assert!(layout.nodes.iter().any(|node| node.path == vec![1]));
    assert!(!layout.edges.iter().any(|edge| edge.to == vec![0, 0]));
}

#[test]
fn layout_for_canvas_dispatches_each_diagram_type() {
    let fixture = Fixture::new();

    for diagram_type in [
        MindMapDiagramType::MindMap,
        MindMapDiagramType::OrgChart,
        MindMapDiagramType::Fishbone,
        MindMapDiagramType::Timeline,
        MindMapDiagramType::Tree,
        MindMapDiagramType::Bracket,
    ] {
        let canvas = fixture.canvas(diagram_type);
        let layout = layout_for_canvas(&canvas);

        assert!(layout.nodes.iter().any(|node| node.path.is_empty()));
        assert!(layout.nodes.len() >= 3);
    }
}
