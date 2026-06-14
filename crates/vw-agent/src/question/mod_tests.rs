use super::*;
use serde_json::json;
use tokio::time::{Duration, sleep};

#[test]
fn extra_builds_json_map_for_empty_and_multiple_pairs() {
    let empty = extra([]);
    assert!(empty.is_empty());

    let map = extra([("request_id", json!("q1")), ("count", json!(2))]);
    assert_eq!(map.get("request_id"), Some(&json!("q1")));
    assert_eq!(map.get("count"), Some(&json!(2)));
}

fn question(label: &str) -> Info {
    Info {
        question: format!("Pick {label}?"),
        header: "Choice".to_string(),
        options: vec![OptionInfo {
            label: label.to_string(),
            description: "A choice".to_string(),
            preview: None,
        }],
        multiple: Some(false),
        custom: Some(true),
    }
}

async fn pending_request(session_id: &str) -> Request {
    loop {
        if let Some(req) = list().into_iter().find(|r| r.session_id == session_id) {
            break req;
        }
        sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn ask_lists_pending_and_reply_returns_answers() {
    let session_id = format!("session-{}", uuid::Uuid::new_v4());
    let handle = tokio::spawn({
        let session_id = session_id.clone();
        async move {
            ask(AskInput {
                session_id,
                questions: vec![question("A")],
                tool: Some(ToolMeta { message_id: "m1".to_string(), call_id: "c1".to_string() }),
            })
            .await
        }
    });

    let request = pending_request(&session_id).await;
    assert_eq!(request.questions.len(), 1);
    assert!(request.tool.is_some());

    reply(ReplyInput {
        request_id: request.id.clone(),
        answers: vec![vec!["A".to_string(), "custom".to_string()]],
    });
    reply(ReplyInput { request_id: request.id, answers: vec![vec!["late".to_string()]] });

    let answers = handle.await.expect("task should join").expect("ask should resolve");
    assert_eq!(answers, vec![vec!["A".to_string(), "custom".to_string()]]);
    assert!(!list().iter().any(|r| r.session_id == session_id));
}

#[tokio::test]
async fn ask_accepts_empty_questions_and_empty_answers() {
    let session_id = format!("session-{}", uuid::Uuid::new_v4());
    let handle = tokio::spawn({
        let session_id = session_id.clone();
        async move { ask(AskInput { session_id, questions: vec![], tool: None }).await }
    });

    let request = pending_request(&session_id).await;
    assert!(request.questions.is_empty());
    assert!(request.tool.is_none());

    reply(ReplyInput { request_id: request.id, answers: vec![] });

    let answers = handle.await.expect("task should join").expect("ask should resolve");
    assert!(answers.is_empty());
}

#[tokio::test]
async fn reject_resolves_ask_with_rejected_error_and_unknowns_are_ignored() {
    reply(ReplyInput { request_id: "missing".to_string(), answers: vec![] });
    reject("missing");

    let session_id = format!("session-{}", uuid::Uuid::new_v4());
    let handle = tokio::spawn({
        let session_id = session_id.clone();
        async move { ask(AskInput { session_id, questions: vec![question("B")], tool: None }).await }
    });

    let request = pending_request(&session_id).await;

    reject(request.id.clone());
    reject(request.id);

    let err = handle.await.expect("task should join").expect_err("ask should reject");
    assert!(matches!(err, Error::Rejected(_)));
    assert!(err.to_string().contains("dismissed"));
    assert!(!list().iter().any(|r| r.session_id == session_id));
}

#[tokio::test]
async fn ask_returns_rejected_when_pending_sender_is_dropped() {
    let session_id = format!("session-{}", uuid::Uuid::new_v4());
    let handle = tokio::spawn({
        let session_id = session_id.clone();
        async move { ask(AskInput { session_id, questions: vec![question("C")], tool: None }).await }
    });

    let request = pending_request(&session_id).await;
    {
        let mut lock = STATE.lock().unwrap_or_else(|e| e.into_inner());
        lock.pending.remove(&request.id);
    }

    let err = handle.await.expect("task should join").expect_err("ask should reject");
    assert!(matches!(err, Error::Rejected(_)));
}

#[test]
fn poisoned_state_lock_recovers_for_public_operations() {
    let _ = std::thread::spawn(|| {
        let _guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
        panic!("poison question state");
    })
    .join();

    assert!(list().is_empty());
    reply(ReplyInput { request_id: "poison-missing".to_string(), answers: vec![] });
    reject("poison-missing");
}

#[test]
fn question_error_display_conversion_and_events_are_stable() {
    let rejected = Error::Rejected(RejectedError);
    assert_eq!(rejected.to_string(), "The user dismissed this question");

    let id_error = id::Error::Random("random failed".to_string());
    let err = Error::from(id_error);
    assert!(matches!(err, Error::Id(_)));
    assert_eq!(err.to_string(), "random failed");

    assert_eq!(event::ASKED.r#type, "question.asked");
    assert_eq!(event::REPLIED.r#type, "question.replied");
    assert_eq!(event::REJECTED.r#type, "question.rejected");
}
