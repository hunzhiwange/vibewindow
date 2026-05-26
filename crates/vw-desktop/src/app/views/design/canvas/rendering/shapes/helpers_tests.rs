#[test]
fn brush_path_class_is_detected_from_class_string() {
    assert!(super::is_brush_path_class(Some("foo vw-brush-stroke bar")));
    assert!(!super::is_brush_path_class(Some("foo brush")));
    assert!(!super::is_brush_path_class(None));
}
