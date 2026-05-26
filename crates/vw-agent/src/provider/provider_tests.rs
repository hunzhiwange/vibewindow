use super::*;

#[test]
fn suggestions_prefer_close_candidates() {
    let suggestions = suggest("opnai", vec!["openai", "anthropic"]);
    assert!(suggestions.contains(&"openai".to_string()));
}

