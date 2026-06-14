#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("tests"));
}

#[test]
fn expand_direction_variants_are_distinct() {
    assert_ne!(super::ExpandDirection::Up, super::ExpandDirection::Down);
    assert_eq!(super::ExpandDirection::All, super::ExpandDirection::All);
}
