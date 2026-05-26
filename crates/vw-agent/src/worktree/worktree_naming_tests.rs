use super::naming::{random_name, slug};

#[test]
fn slug_normalizes_user_visible_names() {
    assert_eq!(slug("  Feature/Test 42! "), "feature-test-42");
    assert_eq!(slug("___"), "");
}

#[test]
fn random_name_uses_expected_shape() {
    let name = random_name();
    assert_eq!(name.split('-').count(), 2);
    assert!(name.chars().all(|ch| ch.is_ascii_lowercase() || ch == '-'));
}
