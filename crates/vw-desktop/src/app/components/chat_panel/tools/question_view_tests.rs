use super::question_view::{
    derived_summary, parse_answers, parse_questions, question_request_targets_message,
    tool_question_view,
};
use crate::app::{App, Message};
use serde_json::json;
use std::collections::BTreeMap;
use vw_shared::question::{Info, Request, ToolMeta};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn parse_questions_accepts_json_array() {
    let questions =
        parse_questions(r#"[{"header":"Confirm","question":"Continue?","options":[]}]"#);

    assert_eq!(questions.len(), 1);
}

#[test]
fn parse_questions_accepts_wrapped_questions_and_rejects_plain_text() {
    let questions =
        parse_questions(r#"{"questions":[{"header":"Confirm","question":"Ship?","options":[]}]}"#);

    assert_eq!(questions.len(), 1);
    assert_eq!(questions[0].header, "Confirm");
    assert!(parse_questions("Ship?").is_empty());
    assert!(parse_questions("{bad json").is_empty());
}

#[test]
fn parse_answers_reads_structured_and_output_answers() {
    let structured = parse_answers(&json!({"data":{"answers":{"q1":"yes"}}}), "");
    let fallback = parse_answers(&json!({}), r#"[["yes","__custom__:later"]]"#);

    assert_eq!(structured.get("q1").map(String::as_str), Some("yes"));
    assert_eq!(fallback.get("0").map(String::as_str), Some("yes / later"));
}

#[test]
fn question_request_targets_message_requires_matching_message_id() {
    assert!(!question_request_targets_message(None, Some("m1")));

    let request_without_tool = Request {
        id: "req".to_string(),
        session_id: "s".to_string(),
        questions: Vec::new(),
        tool: None,
    };
    assert!(question_request_targets_message(Some(&request_without_tool), None));

    let request_with_tool = Request {
        id: "req".to_string(),
        session_id: "s".to_string(),
        questions: Vec::new(),
        tool: Some(ToolMeta { message_id: "m1".to_string(), call_id: "c1".to_string() }),
    };
    assert!(question_request_targets_message(Some(&request_with_tool), Some("m1")));
    assert!(!question_request_targets_message(Some(&request_with_tool), Some("m2")));
    assert!(!question_request_targets_message(Some(&request_with_tool), None));
}

#[test]
fn derived_summary_prefers_explicit_summary_and_counts_states() {
    let app = test_app();
    let questions = vec![
        Info {
            header: "A".to_string(),
            question: "One?".to_string(),
            options: Vec::new(),
            multiple: None,
            custom: None,
        },
        Info {
            header: "B".to_string(),
            question: "Two?".to_string(),
            options: Vec::new(),
            multiple: None,
            custom: None,
        },
    ];
    let mut answers = BTreeMap::new();

    assert_eq!(derived_summary(&app, &questions, &answers, false, " explicit "), "explicit");
    assert_eq!(derived_summary(&app, &questions, &answers, true, ""), "等待 2 个问题的回答");
    assert_eq!(derived_summary(&app, &questions[..1], &answers, false, ""), "1 个问题");

    answers.insert("One?".to_string(), "yes".to_string());
    assert_eq!(derived_summary(&app, &questions, &answers, false, ""), "已回答 1 个问题");
    answers.insert("Two?".to_string(), "no".to_string());
    assert_eq!(derived_summary(&app, &questions, &answers, false, ""), "已回答 2 个问题");
}

#[test]
fn tool_question_view_rejects_invalid_or_empty_inputs() {
    let app = test_app();

    assert!(tool_question_view(&app, 0, 0, "tool read\n{}").is_none());
    assert!(tool_question_view(&app, 0, 0, "tool question\nnot json").is_none());
    assert!(
        tool_question_view(
            &app,
            0,
            0,
            r#"tool question
{"status":"completed","input":"{}"}"#
        )
        .is_none()
    );
}

#[test]
fn tool_question_view_builds_completed_running_and_error_cards() {
    let mut app = test_app();
    app.chat_message_ids = vec![Some("m1".to_string())];
    app.question_modal_request = Some(Request {
        id: "req".to_string(),
        session_id: "s".to_string(),
        questions: Vec::new(),
        tool: Some(ToolMeta { message_id: "m1".to_string(), call_id: "c1".to_string() }),
    });

    let completed = tool_question_view(&app, 0, 1, r#"tool question
{"status":"completed","input":"{\"questions\":[{\"header\":\"Confirm\",\"question\":\"Ship?\",\"options\":[]}]}","output":"[[\"yes\"]]"}"#)
    .expect("completed question view");
    keep_element(completed);

    let running = tool_question_view(&app, 0, 2, r#"tool question
{"status":"running","input":"{\"questions\":[{\"header\":\"\",\"question\":\"Continue?\",\"options\":[]}]}"}"#)
    .expect("running question view");
    keep_element(running);

    let error = tool_question_view(&app, 0, 3, r#"tool question
{"status":"error","summary":"failed","input":"{\"questions\":[{\"header\":\"Confirm\",\"question\":\"Ship?\",\"options\":[]}]}"}"#)
    .expect("error question view");
    keep_element(error);
}
