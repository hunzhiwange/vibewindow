use super::super::layout::{compute_layout_for_diagram, layout_node_rect};
use super::{HoverButtonKind, MindMapCanvas};
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, MindMapCanvasTool, MindMapDiagramType,
    MindMapDoodleStroke, MindMapLayoutFormat, OrgChartLayoutFormat, TimelineLayoutFormat,
    TreeLayoutFormat,
};
use iced::widget::canvas::Cache;
use iced::{Point, Rectangle, Size, Vector};
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
                        text: "left".into(),
                        children: vec![MindNode {
                            text: "leaf".into(),
                            children: vec![MindNode { text: "deep".into(), children: vec![] }],
                        }],
                    },
                    MindNode { text: "right".into(), children: vec![] },
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

    fn canvas(&self) -> MindMapCanvas<'_> {
        MindMapCanvas {
            doc: &self.doc,
            cache: &self.cache,
            pan: Vector::new(10.0, 20.0),
            zoom: 2.0,
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
            doodle_rgba: 0x11223344,
            doodle_width_px: 4.0,
            doodles: &self.doodles,
            ui_blocked_rects: Vec::new(),
            theme_group: "classic",
            theme_variant: 0,
            custom_themes: &self.custom_themes,
            theme_panel_open: false,
        }
    }

    fn layout(&self, canvas: &MindMapCanvas<'_>) -> super::super::layout::Layout {
        compute_layout_for_diagram(
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
        )
    }
}

#[test]
fn node_screen_rect_applies_pan_and_zoom() {
    let fixture = Fixture::new();
    let canvas = fixture.canvas();
    let layout = fixture.layout(&canvas);
    let root = layout.nodes.iter().find(|node| node.path.is_empty()).unwrap();
    let world = layout_node_rect(root);

    let rect = canvas.node_screen_rect(root);

    assert_eq!(rect.x, world.x * 2.0 + 10.0);
    assert_eq!(rect.y, world.y * 2.0 + 20.0);
    assert_eq!(rect.width, world.width * 2.0);
    assert_eq!(rect.height, world.height * 2.0);
}

#[test]
fn button_specs_include_expected_buttons_for_root_child_and_collapsed_node() {
    let mut fixture = Fixture::new();
    let canvas = fixture.canvas();
    let root_rect = Rectangle::new(Point::new(100.0, 100.0), Size::new(80.0, 40.0));

    let root_kinds: Vec<_> =
        canvas.node_button_specs(&[], root_rect).into_iter().map(|(kind, _, _)| kind).collect();
    assert_eq!(root_kinds, vec![HoverButtonKind::ToggleCollapse, HoverButtonKind::AddChild]);

    let child_kinds: Vec<_> =
        canvas.node_button_specs(&[0], root_rect).into_iter().map(|(kind, _, _)| kind).collect();
    assert_eq!(
        child_kinds,
        vec![
            HoverButtonKind::ToggleCollapse,
            HoverButtonKind::AddSibling,
            HoverButtonKind::AddChild,
        ]
    );

    fixture.collapsed_paths.insert(vec![0]);
    let canvas = fixture.canvas();
    let collapsed_kinds: Vec<_> =
        canvas.node_button_specs(&[0], root_rect).into_iter().map(|(kind, _, _)| kind).collect();
    assert_eq!(collapsed_kinds, vec![HoverButtonKind::ToggleCollapse]);
}

#[test]
fn button_group_and_hover_rect_expand_for_collapsed_count_badge() {
    let mut fixture = Fixture::new();
    fixture.collapsed_paths.insert(vec![0]);
    let canvas = fixture.canvas();
    let node_rect = Rectangle::new(Point::new(100.0, 100.0), Size::new(80.0, 40.0));
    let (_, toggle_center, r) = canvas
        .node_button_specs(&[0], node_rect)
        .into_iter()
        .find(|(kind, _, _)| *kind == HoverButtonKind::ToggleCollapse)
        .unwrap();

    let badge = canvas.collapsed_count_badge_rect(&[0], toggle_center, r, 2);
    let group = canvas.node_button_group_rect(&[0], node_rect).unwrap();
    let hover = canvas.node_hover_rect(&[0], node_rect);

    assert!(group.contains(Point::new(badge.x + badge.width / 2.0, badge.y + badge.height / 2.0)));
    assert!(hover.contains(Point::new(badge.x + badge.width / 2.0, badge.y + badge.height / 2.0)));
}

#[test]
fn left_aligned_buttons_and_badges_are_placed_to_the_left() {
    let mut fixture = Fixture::new();
    fixture.collapsed_paths.insert(vec![0]);
    let mut canvas = fixture.canvas();
    canvas.layout_format = MindMapLayoutFormat::LeftAligned;
    let rect = Rectangle::new(Point::new(100.0, 100.0), Size::new(80.0, 40.0));
    let (_, center, r) = canvas.node_button_specs(&[0], rect).remove(0);

    let badge = canvas.collapsed_count_badge_rect(&[0], center, r, 12);

    assert!(center.x < rect.x);
    assert!(badge.x + badge.width < center.x);
}

#[test]
fn bidirectional_and_bracket_formats_choose_button_side() {
    let fixture = Fixture::new();
    let mut canvas = fixture.canvas();
    let rect = Rectangle::new(Point::new(100.0, 100.0), Size::new(80.0, 40.0));

    canvas.layout_format = MindMapLayoutFormat::Bidirectional;
    assert!(canvas.node_button_specs(&[], rect)[0].1.x > rect.x + rect.width);
    assert!(canvas.node_button_specs(&[0], rect)[0].1.x > rect.x + rect.width);
    assert!(canvas.node_button_specs(&[1], rect)[0].1.x < rect.x);

    canvas.diagram_type = MindMapDiagramType::Bracket;
    canvas.layout_format = MindMapLayoutFormat::LeftAligned;
    assert!(canvas.node_button_specs(&[0], rect)[0].1.x < rect.x);

    canvas.diagram_type = MindMapDiagramType::OrgChart;
    assert!(canvas.node_button_specs(&[0], rect)[0].1.x > rect.x + rect.width);
}

#[test]
fn hit_node_buttons_returns_messages_for_each_button_kind() {
    let fixture = Fixture::new();
    let canvas = fixture.canvas();
    let layout = fixture.layout(&canvas);
    let node = layout.nodes.iter().find(|node| node.path == vec![0]).unwrap();
    let rect = canvas.node_screen_rect(node);

    for (kind, center, _) in canvas.node_button_specs(&[0], rect) {
        let msg = canvas.hit_node_buttons(&layout, center).unwrap();
        match (kind, msg) {
            (HoverButtonKind::ToggleCollapse, MindMapMessage::ToggleCollapseAt(path)) => {
                assert_eq!(path, vec![0])
            }
            (HoverButtonKind::AddSibling, MindMapMessage::AddSiblingAt(path)) => {
                assert_eq!(path, vec![0])
            }
            (HoverButtonKind::AddChild, MindMapMessage::AddChildAt(path)) => {
                assert_eq!(path, vec![0])
            }
            other => panic!("unexpected button hit result: {other:?}"),
        }
    }

    assert!(canvas.hit_node_buttons(&layout, Point::new(-10_000.0, -10_000.0)).is_none());
}

#[test]
fn hovered_node_path_uses_expanded_hover_rects() {
    let mut fixture = Fixture::new();
    fixture.collapsed_paths.insert(vec![0]);
    let canvas = fixture.canvas();
    let layout = fixture.layout(&canvas);
    let node = layout.nodes.iter().find(|node| node.path == vec![0]).unwrap();
    let rect = canvas.node_screen_rect(node);
    let hover_rect = canvas.node_hover_rect(&[0], rect);

    assert_eq!(
        canvas.hovered_node_path(&layout, Point::new(hover_rect.x + 1.0, hover_rect.y + 1.0)),
        Some(vec![0])
    );
    assert!(canvas.hovered_node_path(&layout, Point::new(-1_000.0, -1_000.0)).is_none());
}

#[test]
fn descendant_count_counts_all_nested_children() {
    let fixture = Fixture::new();
    assert_eq!(super::hit_test::descendant_count(&fixture.doc), 4);
    assert_eq!(super::hit_test::descendant_count(&fixture.doc.children[0]), 2);
}
