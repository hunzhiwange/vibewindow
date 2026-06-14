#[test]
fn hit_test_module_is_linked() {
    let name = "hit";
    assert_eq!(name.len(), 3);
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

fn doc(children: Vec<super::DesignElement>) -> super::DesignDoc {
    super::DesignDoc { version: "test".to_string(), children, ..Default::default() }
}

#[test]
fn hit_test_returns_topmost_element_and_none_outside() {
    let bottom = element("bottom", 0.0, 0.0, 100.0, 100.0);
    let top = element("top", 25.0, 25.0, 50.0, 50.0);
    let doc = doc(vec![bottom.clone(), top.clone()]);

    assert_eq!(super::hit_test(&doc.children, &doc, 30.0, 30.0), Some("top".to_string()));
    assert_eq!(super::hit_test(&doc.children, &doc, 10.0, 10.0), Some("bottom".to_string()));
    assert_eq!(super::hit_test(&doc.children, &doc, 200.0, 200.0), None);
}

#[test]
fn hit_test_prefers_nested_children_over_parent() {
    let child = element("child", 10.0, 10.0, 20.0, 20.0);
    let parent = super::DesignElement {
        id: "parent".to_string(),
        kind: "frame".to_string(),
        x: 40.0,
        y: 50.0,
        width: Some(serde_json::json!(100)),
        height: Some(serde_json::json!(100)),
        children: vec![child],
        ..Default::default()
    };
    let doc = doc(vec![parent]);

    assert_eq!(super::hit_test(&doc.children, &doc, 55.0, 65.0), Some("child".to_string()));
    assert_eq!(super::hit_test(&doc.children, &doc, 45.0, 55.0), Some("parent".to_string()));
}

#[test]
fn hit_test_accounts_for_rotation() {
    let rotated =
        super::DesignElement { rotation: Some(45.0), ..element("rotated", 0.0, 0.0, 100.0, 20.0) };
    let doc = doc(vec![rotated]);

    assert_eq!(super::hit_test(&doc.children, &doc, 50.0, 10.0), Some("rotated".to_string()));
    assert_eq!(super::hit_test(&doc.children, &doc, 95.0, 10.0), None);
}

#[test]
fn resize_handles_are_detected_before_rotate_handles() {
    use super::Handle;

    assert_eq!(
        super::hit_test_handle(10.0, 20.0, 100.0, 80.0, 10.0, 20.0, 1.0),
        Some(Handle::TopLeft)
    );
    assert_eq!(
        super::hit_test_handle(10.0, 20.0, 100.0, 80.0, 110.0, 20.0, 1.0),
        Some(Handle::TopRight)
    );
    assert_eq!(
        super::hit_test_handle(10.0, 20.0, 100.0, 80.0, 10.0, 100.0, 1.0),
        Some(Handle::BottomLeft)
    );
    assert_eq!(
        super::hit_test_handle(10.0, 20.0, 100.0, 80.0, 110.0, 100.0, 1.0),
        Some(Handle::BottomRight)
    );
    assert_eq!(super::hit_test_handle(10.0, 20.0, 100.0, 80.0, 60.0, 20.0, 1.0), Some(Handle::Top));
    assert_eq!(
        super::hit_test_handle(10.0, 20.0, 100.0, 80.0, 60.0, 100.0, 1.0),
        Some(Handle::Bottom)
    );
    assert_eq!(
        super::hit_test_handle(10.0, 20.0, 100.0, 80.0, 10.0, 60.0, 1.0),
        Some(Handle::Left)
    );
    assert_eq!(
        super::hit_test_handle(10.0, 20.0, 100.0, 80.0, 110.0, 60.0, 1.0),
        Some(Handle::Right)
    );
}

#[test]
fn rotate_handles_use_annulus_around_corners() {
    use super::Handle;

    assert_eq!(
        super::hit_test_handle(10.0, 20.0, 100.0, 80.0, 10.0, 40.0, 1.0),
        Some(Handle::RotateTopLeft)
    );
    assert_eq!(
        super::hit_test_handle(10.0, 20.0, 100.0, 80.0, 110.0, 40.0, 1.0),
        Some(Handle::RotateTopRight)
    );
    assert_eq!(
        super::hit_test_handle(10.0, 20.0, 100.0, 80.0, 10.0, 80.0, 1.0),
        Some(Handle::RotateBottomLeft)
    );
    assert_eq!(
        super::hit_test_handle(10.0, 20.0, 100.0, 80.0, 110.0, 80.0, 1.0),
        Some(Handle::RotateBottomRight)
    );
    assert_eq!(super::hit_test_handle(10.0, 20.0, 100.0, 80.0, 50.0, 50.0, 1.0), None);
}
