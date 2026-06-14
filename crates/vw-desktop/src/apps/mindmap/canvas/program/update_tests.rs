use super::update::update;
use super::{DragMode, MindMapCanvasState};
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, MindMapCanvasTool, MindMapDiagramType,
    MindMapDoodleStroke, MindMapLayoutFormat, OrgChartLayoutFormat, TimelineLayoutFormat,
    TreeLayoutFormat,
};
use iced::widget::canvas::{Cache, Event};
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
    theme_panel_open: bool,
}

impl Fixture {
    fn new() -> Self {
        Self {
            doc: MindNode {
                text: "root".into(),
                children: vec![MindNode { text: "child".into(), children: Vec::new() }],
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
            theme_panel_open: false,
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
            doodle_rgba: 0x11223344,
            doodle_width_px: 6.0,
            doodles: &self.doodles,
            ui_blocked_rects: Vec::new(),
            theme_group: "classic",
            theme_variant: 0,
            custom_themes: &self.custom_themes,
            theme_panel_open: self.theme_panel_open,
        }
    }
}

fn bounds() -> Rectangle {
    Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0))
}

fn cursor(point: Point) -> mouse::Cursor {
    mouse::Cursor::Available(point)
}

#[test]
fn left_press_starts_pen_eraser_and_pan_tools() {
    for (tool, expected_mode) in [
        (MindMapCanvasTool::Pen, "pen"),
        (MindMapCanvasTool::Eraser, "eraser"),
        (MindMapCanvasTool::Pan, "pan"),
    ] {
        let mut fixture = Fixture::new();
        fixture.tool = tool;
        let canvas = fixture.canvas();
        let mut state = MindMapCanvasState::default();
        let action = update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            bounds(),
            cursor(Point::new(200.0, 200.0)),
        );

        assert!(action.is_some());
        match (expected_mode, state.drag_mode) {
            ("pen", DragMode::DoodlePen) => assert_eq!(state.doodle_points_world.len(), 1),
            ("eraser", DragMode::DoodleErase) => assert!(state.doodle_points_world.is_empty()),
            ("pan", DragMode::Pan) => {}
            other => panic!("unexpected drag mode: {other:?}"),
        }
    }
}

#[test]
fn left_press_selects_node_drag_or_background_pan() {
    let fixture = Fixture::new();
    let canvas = fixture.canvas();
    let mut state = MindMapCanvasState::default();

    let node_action = update(
        &canvas,
        &mut state,
        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        bounds(),
        cursor(Point::new(0.0, 0.0)),
    );
    assert!(node_action.is_some());
    assert!(matches!(state.drag_mode, DragMode::Node(ref path) if path.is_empty()));

    let bg_action = update(
        &canvas,
        &mut state,
        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        bounds(),
        cursor(Point::new(700.0, 500.0)),
    );
    assert!(bg_action.is_some());
    assert!(matches!(state.drag_mode, DragMode::Pan));
}

#[test]
fn theme_panel_and_blocked_ui_short_circuit_left_press() {
    let mut fixture = Fixture::new();
    fixture.theme_panel_open = true;
    let canvas = fixture.canvas();
    let mut state = MindMapCanvasState::default();

    assert!(
        update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            bounds(),
            cursor(Point::new(200.0, 200.0)),
        )
        .is_some()
    );

    let fixture = Fixture::new();
    let canvas = fixture.canvas();
    assert!(
        update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            bounds(),
            cursor(Point::new(400.0, 20.0)),
        )
        .is_none()
    );
}

#[test]
fn cursor_moved_updates_pan_node_pen_eraser_and_hover_modes() {
    let fixture = Fixture::new();
    let canvas = fixture.canvas();
    let mut state = MindMapCanvasState {
        drag_mode: DragMode::Pan,
        last_cursor: Some(Point::new(10.0, 10.0)),
        ..MindMapCanvasState::default()
    };
    assert!(
        update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::CursorMoved { position: Point::new(20.0, 25.0) }),
            bounds(),
            cursor(Point::new(20.0, 25.0)),
        )
        .is_some()
    );

    state.drag_mode = DragMode::Node(vec![0]);
    state.last_cursor = Some(Point::new(20.0, 25.0));
    assert!(
        update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::CursorMoved { position: Point::new(30.0, 35.0) }),
            bounds(),
            cursor(Point::new(30.0, 35.0)),
        )
        .is_some()
    );

    state.drag_mode = DragMode::DoodlePen;
    state.doodle_points_world = vec![Point::new(0.0, 0.0)];
    assert!(
        update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::CursorMoved { position: Point::new(40.0, 40.0) }),
            bounds(),
            cursor(Point::new(40.0, 40.0)),
        )
        .is_some()
    );
    assert!(state.doodle_points_world.len() > 1);

    state.drag_mode = DragMode::DoodleErase;
    assert!(
        update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::CursorMoved { position: Point::new(50.0, 50.0) }),
            bounds(),
            cursor(Point::new(50.0, 50.0)),
        )
        .is_some()
    );

    state.drag_mode = DragMode::None;
    state.hovered_node = None;
    assert!(
        update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::CursorMoved { position: Point::new(0.0, 0.0) }),
            bounds(),
            cursor(Point::new(0.0, 0.0)),
        )
        .is_some()
    );
    assert_eq!(state.hovered_node, Some(vec![]));
}

#[test]
fn left_release_finishes_drag_modes() {
    let fixture = Fixture::new();
    let canvas = fixture.canvas();

    let mut state = MindMapCanvasState {
        drag_mode: DragMode::DoodlePen,
        doodle_points_world: vec![Point::new(0.0, 0.0), Point::new(4.0, 4.0)],
        ..MindMapCanvasState::default()
    };
    assert!(
        update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            bounds(),
            cursor(Point::new(10.0, 10.0)),
        )
        .is_some()
    );
    assert!(matches!(state.drag_mode, DragMode::None));
    assert!(state.doodle_points_world.is_empty());

    state.drag_mode = DragMode::DoodleErase;
    assert!(
        update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            bounds(),
            cursor(Point::new(10.0, 10.0)),
        )
        .is_some()
    );

    state.drag_mode = DragMode::Pan;
    assert!(
        update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            bounds(),
            cursor(Point::new(10.0, 10.0)),
        )
        .is_some()
    );
}

#[test]
fn right_press_and_wheel_paths_publish_actions() {
    let fixture = Fixture::new();
    let canvas = fixture.canvas();
    let mut state = MindMapCanvasState::default();

    assert!(
        update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)),
            bounds(),
            cursor(Point::new(0.0, 0.0)),
        )
        .is_some()
    );

    assert!(
        update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)),
            bounds(),
            cursor(Point::new(700.0, 500.0)),
        )
        .is_some()
    );

    assert!(
        update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
            }),
            bounds(),
            cursor(Point::new(200.0, 200.0)),
        )
        .is_some()
    );

    assert!(
        update(
            &canvas,
            &mut state,
            &Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Pixels { x: 0.0, y: 0.0 },
            }),
            bounds(),
            cursor(Point::new(200.0, 200.0)),
        )
        .is_none()
    );
}
