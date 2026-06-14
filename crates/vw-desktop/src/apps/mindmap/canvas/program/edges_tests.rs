use super::ThemeView;
use super::edges::{edge_color, edge_endpoints, edge_path, edge_stroke, fishbone_meta};
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::canvas::layout::{EdgeLayout, Layout, NodeLayout, layout_node_rect};
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, MindMapCanvasTool, MindMapDiagramType,
    MindMapDoodleStroke, MindMapLayoutFormat, OrgChartLayoutFormat, TimelineLayoutFormat,
    TreeLayoutFormat,
};
use iced::widget::canvas::{Cache, LineCap, Style};
use iced::{Color, Point, Size, Vector};
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
            doc: MindNode { text: "root".into(), children: Vec::new() },
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

    fn canvas(&self) -> super::super::MindMapCanvas<'_> {
        super::super::MindMapCanvas {
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

fn sample_layout() -> Layout {
    Layout {
        nodes: vec![
            NodeLayout {
                path: vec![],
                text: "root".into(),
                pos: Point::new(0.0, 0.0),
                size: Size::new(100.0, 40.0),
            },
            NodeLayout {
                path: vec![0],
                text: "child".into(),
                pos: Point::new(220.0, 80.0),
                size: Size::new(80.0, 30.0),
            },
            NodeLayout {
                path: vec![0, 0],
                text: "leaf".into(),
                pos: Point::new(360.0, 120.0),
                size: Size::new(70.0, 28.0),
            },
        ],
        edges: vec![
            EdgeLayout { from: vec![], to: vec![0] },
            EdgeLayout { from: vec![0], to: vec![0, 0] },
        ],
    }
}

fn theme(line_color: Option<u32>) -> ThemeView<'static> {
    ThemeView {
        background_color: 0xffffffff,
        root_fill: 0x111111ff,
        root_text: 0xffffffff,
        branch_fills: &[0xff0000ff, 0x00ff00ff],
        branch_text: 0xffffffff,
        leaf_fill: 0xeeeeeeff,
        leaf_text: 0x111111ff,
        line_color,
        is_dark: false,
    }
}

#[test]
fn fishbone_meta_only_exists_for_fishbone_diagrams() {
    let fixture = Fixture::new();
    let mut canvas = fixture.canvas();
    let layout = sample_layout();

    assert!(fishbone_meta(&canvas, &layout).is_none());

    canvas.diagram_type = MindMapDiagramType::Fishbone;
    let (_, root_size, root_rect, dir) = fishbone_meta(&canvas, &layout).unwrap();
    assert_eq!(root_size, Size::new(100.0, 40.0));
    assert_eq!(root_rect, layout_node_rect(&layout.nodes[0]));
    assert_eq!(dir, -1.0);

    canvas.fishbone_layout_format = FishboneLayoutFormat::HeadLeft;
    assert_eq!(fishbone_meta(&canvas, &layout).unwrap().3, 1.0);

    assert!(fishbone_meta(&canvas, &Layout { nodes: Vec::new(), edges: Vec::new() }).is_none());
}

#[test]
fn edge_color_prefers_explicit_then_theme_then_palette_then_default_stroke() {
    let mut fixture = Fixture::new();
    fixture.edge_colors.insert(vec![0], 0x12345678);
    let canvas = fixture.canvas();

    assert_eq!(
        edge_color(&canvas, &theme(Some(0xaabbccdd)), &[0], Color::BLACK),
        Color::from_rgba8(0x12, 0x34, 0x56, 0x78 as f32 / 255.0)
    );
    assert_eq!(
        edge_color(&canvas, &theme(Some(0xaabbccdd)), &[1], Color::BLACK),
        Color::from_rgba8(0xaa, 0xbb, 0xcc, 0xdd as f32 / 255.0)
    );
    assert_eq!(
        edge_color(&canvas, &theme(None), &[], Color::from_rgb(0.2, 0.3, 0.4)),
        Color::from_rgb(0.2, 0.3, 0.4)
    );
    assert_eq!(
        edge_color(&canvas, &theme(None), &[1], Color::BLACK),
        Color::from_rgba8(0, 255, 0, 1.0)
    );
}

#[test]
fn edge_stroke_applies_dash_patterns_and_round_caps() {
    let solid = edge_stroke(EdgeStyle::Solid, Color::WHITE, 3.0, 1.0);
    assert_eq!(solid.width, 3.0);
    assert_eq!(solid.line_dash.segments, &[] as &[f32]);
    assert_eq!(solid.style, Style::Solid(Color::WHITE));

    let dashed = edge_stroke(EdgeStyle::Dashed, Color::BLACK, 2.0, 1.0);
    assert_eq!(dashed.line_dash.segments, &[12.0, 12.0]);

    let dotted = edge_stroke(EdgeStyle::Dotted, Color::BLACK, 2.0, 1.0);
    assert_eq!(dotted.line_dash.segments, &[3.5, 10.5]);
    assert!(matches!(dotted.line_cap, LineCap::Round));
}

#[test]
fn normal_edge_endpoints_attach_to_nearest_horizontal_sides() {
    let fixture = Fixture::new();
    let canvas = fixture.canvas();
    let layout = sample_layout();
    let a = &layout.nodes[0];
    let b = &layout.nodes[1];
    let edge = &layout.edges[0];

    let (start, end) =
        edge_endpoints(&canvas, edge, a, b, layout_node_rect(a), layout_node_rect(b), None);
    assert_eq!(start, Point::new(50.0, 0.0));
    assert_eq!(end, Point::new(180.0, 80.0));

    let mut left_child = b.clone();
    left_child.pos.x = -220.0;
    let (start, end) = edge_endpoints(
        &canvas,
        edge,
        a,
        &left_child,
        layout_node_rect(a),
        layout_node_rect(&left_child),
        None,
    );
    assert_eq!(start, Point::new(-50.0, 0.0));
    assert_eq!(end, Point::new(-180.0, 80.0));
}

#[test]
fn edge_endpoints_cover_org_timeline_tree_and_fishbone_branches() {
    let fixture = Fixture::new();
    let mut canvas = fixture.canvas();
    let layout = sample_layout();
    let a = &layout.nodes[0];
    let b = &layout.nodes[1];
    let c = &layout.nodes[2];

    canvas.diagram_type = MindMapDiagramType::OrgChart;
    let (start, end) = edge_endpoints(
        &canvas,
        &layout.edges[0],
        a,
        b,
        layout_node_rect(a),
        layout_node_rect(b),
        None,
    );
    assert_eq!(start, Point::new(0.0, 20.0));
    assert_eq!(end, Point::new(220.0, 65.0));

    canvas.diagram_type = MindMapDiagramType::Timeline;
    let (start, end) = edge_endpoints(
        &canvas,
        &layout.edges[1],
        b,
        c,
        layout_node_rect(b),
        layout_node_rect(c),
        None,
    );
    assert_eq!(start, Point::new(220.0, 95.0));
    assert_eq!(end, Point::new(325.0, 120.0));

    canvas.diagram_type = MindMapDiagramType::Tree;
    let (start, end) = edge_endpoints(
        &canvas,
        &layout.edges[0],
        a,
        b,
        layout_node_rect(a),
        layout_node_rect(b),
        None,
    );
    assert_eq!(start, Point::new(0.0, 20.0));
    assert_eq!(end, Point::new(220.0, 65.0));

    canvas.diagram_type = MindMapDiagramType::Fishbone;
    let meta = fishbone_meta(&canvas, &layout).unwrap();
    let (start, end) = edge_endpoints(
        &canvas,
        &layout.edges[0],
        a,
        b,
        layout_node_rect(a),
        layout_node_rect(b),
        Some(&meta),
    );
    assert_eq!(start.y, 0.0);
    assert_eq!(end, Point::new(260.0, 80.0));

    let (start, end) = edge_endpoints(
        &canvas,
        &layout.edges[1],
        b,
        c,
        layout_node_rect(b),
        layout_node_rect(c),
        Some(&meta),
    );
    assert_eq!(start.y, 120.0);
    assert_eq!(end, Point::new(395.0, 120.0));
}

#[test]
fn edge_path_builds_all_diagram_variants_without_panicking() {
    let fixture = Fixture::new();
    let mut canvas = fixture.canvas();
    let edge = EdgeLayout { from: vec![0], to: vec![0, 0] };
    let start = Point::new(0.0, 0.0);
    let end = Point::new(100.0, 80.0);

    let _ = edge_path(&canvas, &edge, start, end, false);
    let _ = edge_path(&canvas, &edge, start, end, true);

    canvas.diagram_type = MindMapDiagramType::Timeline;
    let _ = edge_path(&canvas, &edge, start, end, false);

    canvas.diagram_type = MindMapDiagramType::Tree;
    canvas.tree_layout_format = TreeLayoutFormat::SymmetricSplit;
    let _ = edge_path(&canvas, &edge, start, end, false);
    canvas.tree_layout_format = TreeLayoutFormat::LeftAligned;
    let _ = edge_path(&canvas, &edge, start, end, false);
    canvas.tree_layout_format = TreeLayoutFormat::RightAligned;
    let _ = edge_path(&canvas, &edge, start, end, false);
    canvas.tree_layout_format = TreeLayoutFormat::FanDown;
    let _ = edge_path(&canvas, &edge, start, end, false);

    canvas.diagram_type = MindMapDiagramType::OrgChart;
    canvas.org_chart_layout_format = OrgChartLayoutFormat::TopDown;
    let _ = edge_path(&canvas, &edge, start, end, false);
    canvas.org_chart_layout_format = OrgChartLayoutFormat::LeftRight;
    let _ = edge_path(&canvas, &edge, start, end, false);
}
