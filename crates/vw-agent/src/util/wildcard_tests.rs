use super::wildcard::{StructuredInput, all, all_structured, matches};

#[test]
fn wildcard_matches_strings_and_structured_inputs() {
    assert!(matches("hello world", "hello *"));
    assert!(matches("file.txt", "file.?xt"));
    assert_eq!(
        all("hello world", &[("hello*".to_string(), 1), ("hello world".to_string(), 2)]),
        Some(2)
    );
    let input = StructuredInput { head: "cmd".into(), tail: vec!["a".into(), "b".into()] };
    assert_eq!(all_structured(&input, &[("cmd a *".to_string(), 3)]), Some(3));
}
