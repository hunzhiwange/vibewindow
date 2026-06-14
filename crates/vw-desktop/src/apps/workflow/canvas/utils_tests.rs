//! 工作流画布辅助函数测试，验证坐标、尺寸和布局计算的稳定性。

use super::*;
use serde_yaml::Value;

fn code_node(error_strategy: Option<&str>) -> WorkflowNode {
    let raw_node = match error_strategy {
        Some(strategy) => {
            serde_yaml::from_str::<Value>(&format!("data:\n  error_strategy: {strategy}\n"))
                .expect("node yaml should parse")
        }
        None => serde_yaml::from_str::<Value>("data: {}\n").expect("node yaml should parse"),
    };

    WorkflowNode {
        id: "code_1".to_string(),
        block_type: "code".to_string(),
        title: "初始化变量".to_string(),
        description: String::new(),
        position: Point::ORIGIN,
        size: Size::new(180.0, 84.0),
        parent_id: None,
        selected: false,
        source_side: WorkflowHandleSide::Right,
        target_side: WorkflowHandleSide::Left,
        source_handles: Vec::new(),
        target_handles: Vec::new(),
        z_index: 0.0,
        raw_node,
    }
}

fn basic_node(id: &str, block_type: &str, title: &str, description: &str) -> WorkflowNode {
    WorkflowNode {
        id: id.to_string(),
        block_type: block_type.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        position: Point::new(10.0, 20.0),
        size: Size::new(180.0, 84.0),
        parent_id: None,
        selected: false,
        source_side: WorkflowHandleSide::Right,
        target_side: WorkflowHandleSide::Left,
        source_handles: vec![WorkflowHandle {
            id: "source".to_string(),
            label: "输出".to_string(),
            kind: WorkflowHandleKind::Source,
        }],
        target_handles: vec![WorkflowHandle {
            id: "target".to_string(),
            label: "输入".to_string(),
            kind: WorkflowHandleKind::Target,
        }],
        z_index: 0.0,
        raw_node: Value::Null,
    }
}

fn edge(source: &str, target: &str, label: Option<&str>) -> WorkflowEdge {
    WorkflowEdge {
        id: format!("{source}-{target}"),
        source: source.to_string(),
        target: target.to_string(),
        source_handle: label.map(str::to_string),
        target_handle: Some("target".to_string()),
        source_type: "llm".to_string(),
        target_type: "answer".to_string(),
        selected: false,
        z_index: 0.0,
        raw_edge: Value::Null,
    }
}

#[test]
fn code_node_description_hides_when_error_strategy_is_none() {
    let document = WorkflowDocument::default();

    assert!(node_description_text(&document, &code_node(None)).is_empty());
    assert!(node_description_text(&document, &code_node(Some("none"))).is_empty());
}

#[test]
fn code_node_description_shows_error_strategy_summary() {
    let document = WorkflowDocument::default();

    assert_eq!(
        node_description_text(&document, &code_node(Some("default-value"))),
        "异常时 输出默认值"
    );
    assert_eq!(
        node_description_text(&document, &code_node(Some("fail-branch"))),
        "异常时 异常分支"
    );
}

#[test]
fn distance_to_segment_handles_degenerate_and_projected_segments() {
    assert_eq!(distance_to_segment(Point::new(3.0, 4.0), Point::ORIGIN, Point::ORIGIN), 5.0);
    assert_eq!(
        distance_to_segment(Point::new(5.0, 5.0), Point::ORIGIN, Point::new(10.0, 0.0)),
        5.0
    );
    assert_eq!(
        distance_to_segment(Point::new(-3.0, 4.0), Point::ORIGIN, Point::new(10.0, 0.0)),
        5.0
    );
}

#[test]
fn wrap_text_lines_trims_wraps_and_truncates() {
    assert_eq!(wrap_text_lines("   ", 8, 2), vec!["".to_string()]);
    assert_eq!(wrap_text_lines("abcdef", 3, 3), vec!["abcd".to_string(), "ef".to_string()]);
    assert_eq!(wrap_text_lines("一二三四五", 4, 1), vec!["一二…".to_string()]);
    assert_eq!(wrap_text_lines("one\n\ntwo", 10, 3), vec!["one".to_string(), "two".to_string()]);
}

#[test]
fn display_width_counts_wide_and_unknown_characters() {
    assert_eq!(display_width("abc"), 3);
    assert_eq!(display_width("你好"), 4);
}

#[test]
fn glyph_badge_and_line_height_cover_known_and_fallback_values() {
    assert_eq!(node_glyph("start"), "S");
    assert_eq!(node_glyph("end"), "E");
    assert_eq!(node_glyph("answer"), "A");
    assert_eq!(node_glyph("llm"), "AI");
    assert_eq!(node_glyph("if-else"), "IF");
    assert_eq!(node_glyph("code"), "C");
    assert_eq!(node_glyph("tool"), "T");
    assert_eq!(node_glyph("knowledge-retrieval"), "K");
    assert_eq!(node_glyph("question-classifier"), "Q");
    assert_eq!(node_glyph("http-request"), "H");
    assert_eq!(node_glyph("iteration"), "IT");
    assert_eq!(node_glyph("loop"), "L");
    assert_eq!(node_glyph("unknown"), "•");

    assert_eq!(start_variable_badge_text("number"), "#");
    assert_eq!(start_variable_badge_text("boolean"), "B");
    assert_eq!(start_variable_badge_text("file"), "F");
    assert_eq!(start_variable_badge_text("array[file]"), "F");
    assert_eq!(start_variable_badge_text("string"), "T");

    assert_eq!(line_block_height(0, 12.0, 16.0), 0.0);
    assert_eq!(line_block_height(3, 12.0, 16.0), 44.0);
}

#[test]
fn group_and_plain_node_description_text_follow_state() {
    let mut group = basic_node("group", "iteration", "Group", "");
    group.size = Size::new(220.0, 160.0);
    let child = WorkflowNode {
        parent_id: Some("group".to_string()),
        ..basic_node("child", "llm", "Child", "")
    };
    let document =
        WorkflowDocument { nodes: vec![group.clone(), child], ..WorkflowDocument::default() };

    assert_eq!(node_description_text(&document, &group), "包含 1 个子节点");

    let empty_group = basic_node("empty-group", "loop", "Loop", "");
    assert_eq!(
        node_description_text(&WorkflowDocument::default(), &empty_group),
        "容器节点，可拖动带走内部节点"
    );

    let described_group = basic_node("described", "loop", "Loop", "自定义描述");
    assert_eq!(node_description_text(&WorkflowDocument::default(), &described_group), "自定义描述");

    assert_eq!(
        node_description_text(
            &WorkflowDocument::default(),
            &basic_node("plain", "llm", "LLM", "节点描述")
        ),
        "节点描述"
    );
    assert!(
        node_description_text(&WorkflowDocument::default(), &basic_node("plain", "llm", "LLM", ""))
            .is_empty()
    );
}

#[test]
fn color_and_coordinate_helpers_clamp_and_transform() {
    let left = Color::from_rgba(0.0, 0.2, 0.4, 0.6);
    let right = Color::from_rgba(1.0, 0.8, 0.6, 0.4);

    assert_eq!(blend(left, right, -1.0), left);
    assert_eq!(blend(left, right, 2.0), right);
    assert_eq!(with_alpha(left, -1.0).a, 0.0);
    assert_eq!(with_alpha(left, 2.0).a, 1.0);
    assert_eq!(canvas_element_scale(0.1), 0.3);
    assert_eq!(canvas_element_scale(1.5), 1.5);

    let pan = Vector::new(10.0, -5.0);
    let screen = screen_from_world(Point::new(20.0, 10.0), pan, 2.0);
    assert_eq!(screen, Point::new(50.0, 15.0));
    assert_eq!(world_from_screen(screen, pan, 2.0), Point::new(20.0, 10.0));
    assert_eq!(world_from_screen(screen, pan, 0.0).x, 400000.0);

    let midpoint = cubic_bezier_point(
        Point::new(0.0, 0.0),
        Point::new(0.0, 10.0),
        Point::new(10.0, 10.0),
        Point::new(10.0, 0.0),
        0.5,
    );
    assert_eq!(midpoint, Point::new(5.0, 7.5));
}

#[test]
fn theme_darkness_uses_palette_background() {
    assert!(theme_is_dark(&iced::Theme::Dark));
    assert!(!theme_is_dark(&iced::Theme::Light));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn export_svg_handles_empty_documents() {
    let svg = export_svg(&WorkflowDocument::default());

    assert!(svg.contains("<svg"));
    assert!(svg.contains("width=\"960\""));
    assert!(svg.contains("rgba(248,250,253,1)"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn export_svg_renders_nodes_edges_labels_and_escapes_text() {
    let mut source = basic_node("a", "llm", "A <B>", "description & more");
    source.selected = true;
    let target = basic_node("b", "answer", "", "target");
    let document = WorkflowDocument {
        nodes: vec![source, WorkflowNode { position: Point::new(260.0, 20.0), ..target }],
        edges: vec![edge("a", "b", Some("true"))],
        ..WorkflowDocument::default()
    };

    let svg = export_svg(&document);

    assert!(svg.contains("A &lt;B&gt;"));
    assert!(svg.contains("description &amp; more"));
    assert!(svg.contains(">是<"));
    assert!(svg.contains("node-shadow"));
    assert!(svg.ends_with("</svg>"));
}
