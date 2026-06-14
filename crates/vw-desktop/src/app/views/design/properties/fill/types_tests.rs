#[test]
fn task_1189_test_module_is_wired() {}

#[test]
fn fill_item_enabled_round_trips_all_variants() {
    let mut color = super::FillItem::Color("#ff0000".to_string());
    assert!(color.is_enabled());
    color.set_enabled(false);
    assert!(!color.is_enabled());
    assert!(matches!(
        color,
        super::FillItem::Object(super::FillObject::Solid { enabled: false, .. })
    ));

    let mut gradient = super::FillItem::Object(super::FillObject::Gradient(super::GradientFill {
        gradient_type: "linear".to_string(),
        enabled: true,
        rotation: 0.0,
        colors: vec![super::GradientStop { color: "#fff".to_string(), position: 0.0 }],
        center: None,
        size: None,
        size_h: None,
    }));
    gradient.set_enabled(false);
    assert!(!gradient.is_enabled());
}

#[test]
fn mesh_normalize_clamps_dimensions_and_preserves_valid_data() {
    let mut mesh = super::MeshFill {
        enabled: true,
        columns: 9,
        rows: 1,
        colors: vec!["#111111".to_string(), "#222222".to_string()],
        points: vec![vec![0.25], vec![0.75, 0.25]],
        handles: vec![vec![0.0; 8], vec![1.0, 1.0]],
        mirroring: Some("Y only".to_string()),
        outline: true,
        selected_point_index: Some(99),
    };

    mesh.normalize();

    assert_eq!((mesh.columns, mesh.rows), (6, 2));
    assert_eq!(mesh.colors.len(), 12);
    assert_eq!(mesh.points.len(), 12);
    assert_eq!(mesh.handles.len(), 12);
    assert_eq!(mesh.points[0], vec![0.25, 0.0]);
    assert_eq!(mesh.mirroring.as_deref(), Some("y"));
    assert_eq!(mesh.selected_point_index, None);
}

#[test]
fn mesh_default_handles_expand_edges_and_materialize_when_needed() {
    let mut mesh = super::MeshFill::new_random(3, 3);
    assert_eq!((mesh.columns, mesh.rows), (3, 3));
    assert_eq!(mesh.colors.len(), 9);
    assert!(mesh.colors.iter().all(|color| color.starts_with('#') && color.len() == 7));

    let center = mesh.effective_handles(4);
    assert_eq!(center.len(), 8);
    let invalid = mesh.effective_handles(99);
    assert_eq!(invalid, [0.0; 8]);

    let changed = mesh.materialize_effective_handles(0);
    assert!(changed);
    assert!(!mesh.materialize_effective_handles(99));
}

#[test]
fn serde_defaults_enable_objects_and_preserve_image_mode() {
    let solid: super::FillItem =
        serde_json::from_str(r##"{"type":"solid","color":"#abcdef"}"##).unwrap();
    assert!(solid.is_enabled());

    let image: super::FillObject =
        serde_json::from_str(r#"{"type":"image","url":"https://example.test/a.png"}"#).unwrap();
    match image {
        super::FillObject::Image(image) => {
            assert!(image.enabled);
            assert_eq!(image.mode, "");
            assert_eq!(image.url, "https://example.test/a.png");
        }
        other => panic!("unexpected fill object: {other:?}"),
    }
}
