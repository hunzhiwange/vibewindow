use crate::app::views::design::models::{DesignElement, Stroke};
use iced::Point;

#[test]
fn extract_hex_token_returns_first_complete_hex_color() {
    assert_eq!(super::extract_hex_token("color #12ABef and #000000"), Some("#12ABef".to_string()));
    assert_eq!(
        super::extract_hex_token("alpha #11223344 then #000000"),
        Some("#11223344".to_string())
    );
    assert_eq!(super::extract_hex_token("no complete #123 token"), None);
}

#[test]
fn with_alpha_replaces_or_appends_alpha_component() {
    assert_eq!(super::with_alpha("#112233", 0x80), "#11223380");
    assert_eq!(super::with_alpha("#11223344", 0xFF), "#112233FF");
    assert_eq!(super::with_alpha("bad", 0x7F), "#0000007F");
}

#[test]
fn parse_brush_points_handles_commands_commas_and_negative_numbers() {
    let points = super::parse_brush_points("M 0,0 L 10 -5 l 20,5").unwrap();
    assert_eq!(points.len(), 3);
    assert_eq!(points[1], Point::new(10.0, -5.0));
}

#[test]
fn parse_brush_points_rejects_single_point_or_invalid_numbers() {
    assert_eq!(super::parse_brush_points("M 0 0"), None);
    assert_eq!(super::parse_brush_points("M 0 0 L nope 1"), None);
}

#[test]
fn split_brush_segments_removes_points_inside_radius() {
    let points = [
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        Point::new(3.0, 0.0),
        Point::new(5.0, 0.0),
        Point::new(6.0, 0.0),
    ];
    let segments = super::split_brush_segments(&points, Point::new(3.0, 0.0), 0.5);
    assert_eq!(
        segments,
        vec![
            vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0)],
            vec![Point::new(5.0, 0.0), Point::new(6.0, 0.0)]
        ]
    );
}

#[test]
fn brush_path_detection_requires_path_kind_and_brush_class() {
    let element = DesignElement {
        kind: "path".to_string(),
        class: Some("foo vw-brush-stroke bar".to_string()),
        ..Default::default()
    };
    assert!(super::is_brush_path(&element));

    let non_brush = DesignElement {
        kind: "rect".to_string(),
        class: Some("vw-brush-stroke".to_string()),
        ..Default::default()
    };
    assert!(!super::is_brush_path(&non_brush));
}

#[test]
fn brush_color_and_width_parse_defaults_and_clamps() {
    let default_element = DesignElement::default();
    assert_eq!(super::parse_brush_color(&default_element), "#111827");
    assert_eq!(super::parse_brush_width(&default_element), 3.0);

    let element = DesignElement {
        stroke: Some(Stroke {
            align: None,
            thickness: Some(serde_json::json!("99")),
            fill: Some("#ABCDEF".to_string()),
        }),
        ..Default::default()
    };

    assert_eq!(super::parse_brush_color(&element), "#ABCDEF");
    assert_eq!(super::parse_brush_width(&element), 18.0);

    let thin_element = DesignElement {
        stroke: Some(Stroke { align: None, thickness: Some(serde_json::json!(0)), fill: None }),
        ..Default::default()
    };
    assert_eq!(super::parse_brush_width(&thin_element), 1.0);
}

#[test]
fn erase_brush_nodes_keeps_untouched_paths() {
    let mut children = vec![DesignElement {
        id: "brush".to_string(),
        kind: "path".to_string(),
        class: Some("vw-brush-stroke".to_string()),
        geometry: Some("M 0 0 L 10 0".to_string()),
        stroke: Some(Stroke {
            align: None,
            thickness: Some(serde_json::json!(2.0)),
            fill: Some("#123456".to_string()),
        }),
        ..Default::default()
    }];

    let changed =
        super::erase_brush_nodes(&mut children, Point::ORIGIN, Point::new(50.0, 50.0), 2.0);

    assert!(!changed);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id, "brush");
}

#[test]
fn erase_brush_nodes_removes_fully_erased_path() {
    let mut children = vec![DesignElement {
        id: "brush".to_string(),
        kind: "path".to_string(),
        class: Some("vw-brush-stroke".to_string()),
        geometry: Some("M 0 0 L 1 0".to_string()),
        ..Default::default()
    }];

    let changed = super::erase_brush_nodes(&mut children, Point::ORIGIN, Point::new(0.5, 0.0), 5.0);

    assert!(changed);
    assert!(children.is_empty());
}

#[test]
fn erase_brush_nodes_splits_path_and_preserves_metadata() {
    let mut children = vec![DesignElement {
        id: "brush".to_string(),
        kind: "path".to_string(),
        class: Some("vw-brush-stroke extra".to_string()),
        geometry: Some("M 0 0 L 1 0 L 5 0 L 9 0 L 10 0".to_string()),
        group_id: 7,
        name: Some("Stroke".to_string()),
        visible: Some(true),
        enabled: Some(true),
        opacity: Some(0.5),
        stroke: Some(Stroke {
            align: None,
            thickness: Some(serde_json::json!("4")),
            fill: Some("#654321".to_string()),
        }),
        ..Default::default()
    }];

    let changed = super::erase_brush_nodes(&mut children, Point::ORIGIN, Point::new(5.0, 0.0), 1.0);

    assert!(changed);
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].id, "brush");
    assert_eq!(children[0].group_id, 7);
    assert_eq!(children[0].name.as_deref(), Some("Stroke"));
    assert_eq!(children[1].group_id, 7);
    assert_eq!(super::parse_brush_color(&children[0]), "#654321");
    assert_eq!(super::parse_brush_width(&children[0]), 4.0);
}

#[test]
fn erase_brush_nodes_recurses_through_children_with_offsets() {
    let mut children = vec![DesignElement {
        id: "frame".to_string(),
        x: 10.0,
        y: 20.0,
        children: vec![DesignElement {
            id: "brush".to_string(),
            kind: "path".to_string(),
            class: Some("vw-brush-stroke".to_string()),
            geometry: Some("M 0 0 L 2 0".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    }];

    let changed =
        super::erase_brush_nodes(&mut children, Point::ORIGIN, Point::new(11.0, 20.0), 5.0);

    assert!(changed);
    assert!(children[0].children.is_empty());
}
