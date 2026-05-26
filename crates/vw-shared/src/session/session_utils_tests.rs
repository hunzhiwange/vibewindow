#[test]
fn generated_slug_has_expected_shape() {
    let slug = super::create_slug();

    assert_eq!(slug.len(), 8);
    assert!(slug.bytes().all(|b| b.is_ascii_alphanumeric()));
}

#[test]
fn default_title_matches_only_generated_titles() {
    assert!(super::is_default_title(
        "New session - 2026-05-24T12:34:56.789Z"
    ));
    assert!(super::is_default_title(
        "Child session - 2026-05-24T12:34:56.789Z"
    ));
    assert!(!super::is_default_title("New session - draft"));
    assert!(!super::is_default_title(
        "New session - 2026-05-24T12:34:56Z"
    ));
}
