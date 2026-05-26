#[test]
fn task_724_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("glyph_buttons_tests.rs"));
}
