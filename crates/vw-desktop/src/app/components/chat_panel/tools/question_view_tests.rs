use super::question_view::{parse_answers, parse_questions, question_request_targets_message};
use serde_json::json;

#[test]
fn parse_questions_accepts_json_array() {
    let questions = parse_questions(r#"[{"id":"q1","question":"Continue?","kind":"text"}]"#);

    assert_eq!(questions.len(), 1);
}

#[test]
fn parse_answers_reads_structured_and_output_answers() {
    let structured = parse_answers(&json!({"answers":{"q1":"yes"}}), "");
    let fallback = parse_answers(&json!({}), r#"{"q2":"no"}"#);

    assert_eq!(structured.get("q1").map(String::as_str), Some("yes"));
    assert_eq!(fallback.get("q2").map(String::as_str), Some("no"));
}

#[test]
fn question_request_targets_message_requires_matching_message_id() {
    assert!(!question_request_targets_message(None, Some("m1")));
}
