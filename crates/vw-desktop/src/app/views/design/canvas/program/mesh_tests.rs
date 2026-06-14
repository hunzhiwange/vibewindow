#[test]
fn parse_fill_items_returns_empty_for_missing_or_invalid_fill() {
    assert!(super::parse_fill_items(&None).is_empty());
    assert!(super::parse_fill_items(&Some(serde_json::json!(true))).is_empty());
}

#[test]
fn cursor_to_uv_raw_clamps_to_rect() {
    let rect = iced::Rectangle::new(iced::Point::new(10.0, 20.0), iced::Size::new(100.0, 200.0));
    let uv = super::cursor_to_uv_raw(60.0, 120.0, rect);
    assert!((uv.0 - 0.5).abs() < 0.0001);
    assert!((uv.1 - 0.5).abs() < 0.0001);
}

fn mesh() -> crate::app::views::design::properties::fill::types::MeshFill {
    crate::app::views::design::properties::fill::types::MeshFill {
        enabled: true,
        columns: 2,
        rows: 2,
        colors: vec![
            "#000000".to_string(),
            "#111111".to_string(),
            "#222222".to_string(),
            "#333333".to_string(),
        ],
        points: vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]],
        handles: vec![
            vec![0.0, 0.0, 0.0, 0.0, 0.2, 0.0, 0.0, 0.2],
            vec![0.8, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.2],
            vec![0.0, 0.8, 0.0, 1.0, 0.2, 1.0, 0.0, 1.0],
            vec![0.8, 1.0, 1.0, 0.8, 1.0, 1.0, 1.0, 1.0],
        ],
        mirroring: Some("x".to_string()),
        outline: true,
        selected_point_index: None,
    }
}

#[test]
fn parse_fill_items_accepts_single_and_array_mesh_values() {
    let value = serde_json::json!({
        "type": "mesh_gradient",
        "enabled": true,
        "columns": 1,
        "rows": 1,
        "colors": [],
        "points": [],
        "handles": [],
        "selected_point_index": 99
    });
    let single = super::parse_fill_items(&Some(value.clone()));
    assert_eq!(single.len(), 1);

    let array = super::parse_fill_items(&Some(serde_json::json!([value, "#ffffff"])));
    assert_eq!(array.len(), 2);
}

#[test]
fn choose_mesh_fill_index_prefers_enabled_selected_then_first_enabled() {
    use crate::app::views::design::properties::fill::types::{FillItem, FillObject};

    let mut disabled = mesh();
    disabled.enabled = false;
    let enabled = mesh();
    let fills = vec![
        FillItem::Object(FillObject::Mesh(disabled)),
        FillItem::Color("#fff".to_string()),
        FillItem::Object(FillObject::Mesh(enabled)),
    ];

    assert_eq!(super::choose_mesh_fill_index(&fills, Some(0)), Some(2));
    assert_eq!(super::choose_mesh_fill_index(&fills, Some(2)), Some(2));
    assert_eq!(super::choose_mesh_fill_index(&fills, None), Some(2));
    assert_eq!(super::choose_mesh_fill_index(&fills[..2], None), None);
}

#[test]
fn cursor_to_uv_clamps_but_raw_allows_out_of_bounds_and_zero_size() {
    let rect = iced::Rectangle::new(iced::Point::new(10.0, 20.0), iced::Size::new(100.0, 200.0));
    assert_eq!(super::cursor_to_uv(-40.0, 500.0, rect), (0.0, 1.0));
    assert_eq!(
        super::cursor_to_uv_raw(
            50.0,
            50.0,
            iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(0.0, 0.0))
        ),
        (0.0, 0.0)
    );
}

#[test]
fn mesh_point_handles_returns_existing_or_point_defaults() {
    let mesh = mesh();
    assert_eq!(super::mesh_point_handles(&mesh, 0), [0.0, 0.0, 0.0, 0.0, 0.2, 0.0, 0.0, 0.2]);
    let mut missing = mesh.clone();
    missing.handles.clear();
    assert_eq!(super::mesh_point_handles(&missing, 1), [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    assert_eq!(super::mesh_point_handles(&missing, 99), [0.0; 8]);
}

#[test]
fn hit_test_mesh_detects_points_and_selected_handles() {
    use crate::app::views::design::canvas::types::MeshDragKind;

    let bounds = iced::Rectangle::new(iced::Point::new(10.0, 20.0), iced::Size::new(100.0, 100.0));
    let mut mesh = mesh();

    assert_eq!(super::hit_test_mesh(&mesh, bounds, 10.0, 20.0), Some((0, MeshDragKind::Point)));
    assert_eq!(super::hit_test_mesh(&mesh, bounds, 50.0, 50.0), None);

    mesh.selected_point_index = Some(0);
    assert_eq!(super::hit_test_mesh(&mesh, bounds, 30.0, 20.0), Some((0, MeshDragKind::Handle(2))));
}

#[test]
fn update_mesh_drag_moves_points_handles_and_rejects_invalid_indices() {
    use crate::app::views::design::canvas::types::{MeshDragKind, MeshDragState};

    let bounds = iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(100.0, 100.0));
    let mut mesh = mesh();
    let drag = MeshDragState {
        element_id: "e".to_string(),
        fill_index: 0,
        point_index: 0,
        kind: MeshDragKind::Point,
        has_moved: false,
        start_cursor_u: 0.0,
        start_cursor_v: 0.0,
        start_point_x: 0.0,
        start_point_y: 0.0,
        start_handles: super::mesh_point_handles(&mesh, 0),
    };

    assert!(super::update_mesh_drag(&mut mesh, &drag, bounds, 10.0, 20.0));
    assert!((mesh.points[0][0] - 0.1).abs() < 0.000001);
    assert!((mesh.points[0][1] - 0.2).abs() < 0.000001);
    assert!(!super::update_mesh_drag(&mut mesh, &drag, bounds, 10.0, 20.0));

    let handle_drag = MeshDragState { kind: MeshDragKind::Handle(2), ..drag };
    assert!(super::update_mesh_drag(&mut mesh, &handle_drag, bounds, 20.0, 0.0));
    assert!((mesh.handles[0][4] - 0.4).abs() < 0.000001);

    let invalid = MeshDragState { point_index: 99, ..handle_drag };
    assert!(!super::update_mesh_drag(&mut mesh, &invalid, bounds, 0.0, 0.0));
}

#[test]
fn mesh_curve_payload_reports_neighbor_paths_or_null() {
    use crate::app::views::design::canvas::types::MeshDragKind;

    let mesh = mesh();
    let payload = super::mesh_curve_change_payload(&mesh, 0, MeshDragKind::Point);
    assert_eq!(payload["pointIndex"], 0);
    assert_eq!(payload["kind"], "point");
    assert_eq!(payload["paths"].as_array().unwrap().len(), 2);

    let handle_payload = super::mesh_curve_change_payload(&mesh, 0, MeshDragKind::Handle(2));
    assert_eq!(handle_payload["paths"].as_array().unwrap().len(), 1);
    assert_eq!(handle_payload["paths"][0]["to"], 1);

    assert!(super::mesh_curve_change_payload(&mesh, 99, MeshDragKind::Point).is_null());
}
