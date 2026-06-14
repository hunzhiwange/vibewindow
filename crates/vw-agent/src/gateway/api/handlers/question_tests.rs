use super::*;
use axum::extract::Path;
use tokio::time::{Duration, sleep};

#[test]
fn router_builds_with_app_state() {
    let _: axum::Router<()> = router();
}

#[test]
fn question_reply_request_deserializes_answers() {
    let body: QuestionReplyRequest =
        serde_json::from_value(serde_json::json!({"answers": [["yes"]]})).expect("valid reply");

    assert_eq!(body.answers, vec![vec!["yes".to_string()]]);
}

#[tokio::test]
async fn question_list_and_reply_complete_pending_request() {
    let session_id = format!("session-question-handler-{}", uuid::Uuid::new_v4());
    let ask_task = tokio::spawn(question::ask(question::AskInput {
        session_id: session_id.clone(),
        questions: vec![question::Info {
            question: "Continue?".to_string(),
            header: "Confirm".to_string(),
            options: vec![question::OptionInfo {
                label: "Yes".to_string(),
                description: "Proceed".to_string(),
                preview: None,
            }],
            multiple: None,
            custom: None,
        }],
        tool: None,
    }));

    let request_id = loop {
        let Json(items) = question_list().await;
        if let Some(item) = items.iter().find(|item| item.session_id == session_id) {
            break item.id.clone();
        }
        sleep(Duration::from_millis(10)).await;
    };

    let Json(ok) = question_reply(
        Path(request_id),
        Json(QuestionReplyRequest { answers: vec![vec!["Yes".to_string()]] }),
    )
    .await
    .expect("reply should succeed");
    assert!(ok);

    let answers = ask_task.await.expect("ask task should join").expect("ask should resolve");
    assert_eq!(answers, vec![vec!["Yes".to_string()]]);
}

#[tokio::test]
async fn question_reject_unknown_request_is_idempotent() {
    let Json(ok) = question_reject(Path("missing-question".to_string()))
        .await
        .expect("unknown reject should still succeed");

    assert!(ok);
}
