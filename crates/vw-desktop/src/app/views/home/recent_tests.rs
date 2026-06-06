#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("recent_tests"));
}

#[test]
fn project_name_from_path_trims_trailing_separator() {
    assert_eq!(
        super::project_name_from_path("/Users/Shared/work/dir/data/codes/vibe-window/"),
        "vibe-window"
    );
}

#[test]
fn project_name_from_path_keeps_last_segment_without_trailing_separator() {
    assert_eq!(super::project_name_from_path("/Users/Shared/work/dir/data/codes/rue"), "rue");
}
