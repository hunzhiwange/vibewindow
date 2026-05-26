use super::custom::default_custom_themes;

#[test]
fn default_custom_themes_are_named_by_color_data() {
    let themes = default_custom_themes();

    assert!(!themes.is_empty());
    assert!(themes.iter().all(|theme| !theme.branch_fills.is_empty()));
}
