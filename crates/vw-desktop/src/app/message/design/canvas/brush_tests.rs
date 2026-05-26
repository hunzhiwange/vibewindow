#[test]
fn extract_hex_token_returns_first_complete_hex_color() {
    assert_eq!(super::extract_hex_token("color #12ABef and #000000"), Some("#12ABef".to_string()));
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
    let points = super::parse_brush_points("M 0,0 L 10 -5 L 20,5").unwrap();
    assert_eq!(points.len(), 3);
    assert_eq!(points[1], iced::Point::new(10.0, -5.0));
}

#[test]
fn parse_brush_points_rejects_single_point_or_invalid_numbers() {
    assert_eq!(super::parse_brush_points("M 0 0"), None);
    assert_eq!(super::parse_brush_points("M 0 0 L nope 1"), None);
}

#[test]
fn split_brush_segments_removes_points_inside_radius() {
    let points = [
        iced::Point::new(0.0, 0.0),
        iced::Point::new(1.0, 0.0),
        iced::Point::new(3.0, 0.0),
        iced::Point::new(5.0, 0.0),
        iced::Point::new(6.0, 0.0),
    ];
    let segments = super::split_brush_segments(&points, iced::Point::new(3.0, 0.0), 0.5);
    assert_eq!(
        segments,
        vec![
            vec![iced::Point::new(0.0, 0.0), iced::Point::new(1.0, 0.0)],
            vec![iced::Point::new(5.0, 0.0), iced::Point::new(6.0, 0.0)]
        ]
    );
}

#[test]
fn brush_path_detection_requires_path_kind_and_brush_class() {
    let element = crate::app::views::design::models::DesignElement {
        kind: "path".to_string(),
        class: Some("foo vw-brush-stroke bar".to_string()),
        ..Default::default()
    };
    assert!(super::is_brush_path(&element));

    let non_brush = crate::app::views::design::models::DesignElement {
        kind: "rect".to_string(),
        class: Some("vw-brush-stroke".to_string()),
        ..Default::default()
    };
    assert!(!super::is_brush_path(&non_brush));
}
