use super::doodles::doodle_stroke_width;

#[test]
fn doodle_stroke_width_clamps_to_visible_range() {
    assert_eq!(doodle_stroke_width(0.0), 1.0);
    assert_eq!(doodle_stroke_width(0.5), 1.0);
    assert_eq!(doodle_stroke_width(7.5), 7.5);
    assert_eq!(doodle_stroke_width(18.0), 18.0);
    assert_eq!(doodle_stroke_width(32.0), 18.0);
}
