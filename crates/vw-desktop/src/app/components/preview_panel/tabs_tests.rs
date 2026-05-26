use super::tabs::all_languages;

#[test]
fn all_languages_keep_stable_order_and_labels() {
    let labels = all_languages().iter().map(ToString::to_string).collect::<Vec<_>>();

    assert_eq!(labels.first().map(String::as_str), Some("English"));
    assert!(labels.iter().any(|label| label == "简体中文"));
}
