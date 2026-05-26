use super::shared::with_selected_option;

#[test]
fn with_selected_option_keeps_existing_selection_once() {
    let options = with_selected_option(vec!["alpha".to_string(), "beta".to_string()], "beta");

    assert_eq!(options, vec!["alpha".to_string(), "beta".to_string()]);
}

#[test]
fn with_selected_option_appends_missing_selection() {
    let options = with_selected_option(vec!["alpha".to_string()], "beta");

    assert_eq!(options, vec!["alpha".to_string(), "beta".to_string()]);
}
