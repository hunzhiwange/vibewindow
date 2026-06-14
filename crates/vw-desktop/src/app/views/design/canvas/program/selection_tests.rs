#[test]
fn empty_element_list_has_no_intersections() {
    let doc = crate::app::views::design::models::DesignDoc::default();
    let ids = super::find_intersecting_ids(
        &[],
        &doc,
        iced::Vector::new(0.0, 0.0),
        1.0,
        iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(10.0, 10.0)),
    );
    assert!(ids.is_empty());
}

fn element(id: &str, kind: &str, x: f32, y: f32, width: f32, height: f32) -> super::DesignElement {
    super::DesignElement {
        id: id.to_string(),
        kind: kind.to_string(),
        x,
        y,
        width: Some(serde_json::json!(width)),
        height: Some(serde_json::json!(height)),
        ..Default::default()
    }
}

#[test]
fn intersecting_ids_include_nested_children_but_skip_root_frames() {
    let nested = element("nested", "rect", 10.0, 10.0, 20.0, 20.0);
    let frame = super::DesignElement {
        children: vec![nested],
        ..element("frame", "frame", 0.0, 0.0, 100.0, 100.0)
    };
    let root_rect = element("root_rect", "rect", 120.0, 0.0, 40.0, 40.0);
    let doc = crate::app::views::design::models::DesignDoc {
        version: "test".to_string(),
        children: vec![frame, root_rect],
        ..Default::default()
    };

    let ids = super::find_intersecting_ids(
        &doc.children,
        &doc,
        iced::Vector::new(0.0, 0.0),
        1.0,
        iced::Rectangle::new(iced::Point::new(5.0, 5.0), iced::Size::new(135.0, 30.0)),
    );

    assert_eq!(ids, vec!["nested".to_string(), "root_rect".to_string()]);
}

#[test]
fn selection_uses_screen_pan_and_zoom() {
    let rect = element("rect", "rect", 10.0, 10.0, 10.0, 10.0);
    let doc = crate::app::views::design::models::DesignDoc {
        version: "test".to_string(),
        children: vec![rect],
        ..Default::default()
    };

    let ids = super::find_intersecting_ids(
        &doc.children,
        &doc,
        iced::Vector::new(100.0, 50.0),
        2.0,
        iced::Rectangle::new(iced::Point::new(119.0, 69.0), iced::Size::new(2.0, 2.0)),
    );

    assert_eq!(ids, vec!["rect".to_string()]);
}
