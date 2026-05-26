use super::search::{fuzzy_score, is_hidden};

#[test]
fn fuzzy_score_prefers_substring_matches_and_filters_hidden_when_query_is_hidden() {
    assert_eq!(fuzzy_score("lib", "src/lib.rs"), 6);
    assert!(is_hidden("src/.secret/file"));
    assert_eq!(fuzzy_score(".secret", "src/visible.rs"), i64::MAX);
}
