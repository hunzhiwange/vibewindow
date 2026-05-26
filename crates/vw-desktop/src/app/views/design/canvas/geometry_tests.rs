#[test]
fn rotate_point_rotates_around_origin() {
    let rotated = super::rotate_point(1.0, 0.0, 0.0, 0.0, std::f32::consts::FRAC_PI_2);
    assert!(rotated.0.abs() < 0.0001);
    assert!((rotated.1 - 1.0).abs() < 0.0001);
}
