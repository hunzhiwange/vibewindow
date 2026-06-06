#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("tests"));
}

#[test]
fn compact_sidebar_height_is_bounded() {
    let cases = [(200.0, 140.0), (600.0, 210.0), (1200.0, 300.0)];
    for (available_height, expected) in cases {
        let actual = super::compact_sidebar_height(available_height);
        assert!((actual - expected).abs() < 0.001);
    }
}
