#[test]
fn rotate_point_rotates_around_origin() {
    let rotated = super::rotate_point(1.0, 0.0, 0.0, 0.0, std::f32::consts::FRAC_PI_2);
    assert!(rotated.0.abs() < 0.0001);
    assert!((rotated.1 - 1.0).abs() < 0.0001);
}

fn element(id: &str, x: f32, y: f32, width: f32, height: f32) -> super::DesignElement {
    super::DesignElement {
        id: id.to_string(),
        kind: "rect".to_string(),
        x,
        y,
        width: Some(serde_json::json!(width)),
        height: Some(serde_json::json!(height)),
        ..Default::default()
    }
}

#[test]
fn rotate_point_preserves_center_and_supports_arbitrary_origin() {
    let same = super::rotate_point(3.0, 4.0, 3.0, 4.0, 1.23);
    assert_eq!(same, (3.0, 4.0));

    let rotated = super::rotate_point(5.0, 4.0, 3.0, 4.0, std::f32::consts::PI);
    assert!((rotated.0 - 1.0).abs() < 0.0001);
    assert!((rotated.1 - 4.0).abs() < 0.0001);
}

#[test]
fn screen_bounds_applies_pan_zoom_and_nested_offsets() {
    let child = element("child", 5.0, 7.0, 20.0, 10.0);
    let parent = super::DesignElement {
        id: "parent".to_string(),
        kind: "frame".to_string(),
        x: 10.0,
        y: 20.0,
        width: Some(serde_json::json!(100)),
        height: Some(serde_json::json!(80)),
        children: vec![child],
        ..Default::default()
    };
    let doc = super::DesignDoc {
        version: "test".to_string(),
        children: vec![parent],
        ..Default::default()
    };

    let bounds =
        super::get_element_screen_bounds(&doc, "child", iced::Vector::new(100.0, 50.0), 2.0)
            .expect("child bounds");

    assert_eq!(bounds.x, 130.0);
    assert_eq!(bounds.y, 104.0);
    assert_eq!(bounds.width, 40.0);
    assert_eq!(bounds.height, 20.0);
}

#[test]
fn screen_bounds_returns_none_for_unknown_id() {
    let doc = super::DesignDoc {
        version: "test".to_string(),
        children: vec![element("known", 0.0, 0.0, 10.0, 10.0)],
        ..Default::default()
    };

    assert!(
        super::get_element_screen_bounds(&doc, "missing", iced::Vector::default(), 1.0).is_none()
    );
}
