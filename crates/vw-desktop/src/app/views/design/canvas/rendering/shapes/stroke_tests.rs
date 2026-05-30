#[test]
fn stroke_sides_report_uniformity_and_max_width() {
    let sides = super::StrokeSides::uniform(2.0);
    assert!(sides.any_positive());
    assert_eq!(sides.is_uniform(), Some(2.0));
    assert_eq!(sides.max(), 2.0);
}

#[test]
fn stroke_align_parses_known_values() {
    assert!(matches!(super::StrokeAlign::from_str(Some("inside")), super::StrokeAlign::Inside));
    assert!(matches!(super::StrokeAlign::from_str(Some("outside")), super::StrokeAlign::Outside));
    assert!(matches!(super::StrokeAlign::from_str(Some("center")), super::StrokeAlign::Center));
    assert!(matches!(super::StrokeAlign::from_str(Some("unknown")), super::StrokeAlign::Center));
}
