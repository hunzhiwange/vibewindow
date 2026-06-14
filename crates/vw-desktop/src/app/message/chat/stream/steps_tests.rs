use crate::app::models::{ChatSession, ChatSessionStep, TokenUsage};

fn empty_session() -> ChatSession {
    ChatSession {
        id: "session-1".to_string(),
        title: "Session".to_string(),
        messages: Vec::new(),
        message_ids: Vec::new(),
        calls: Vec::new(),
        steps: Vec::new(),
        created_ms: 1,
        updated_ms: 1,
    }
}

fn usage(input_tokens: i64, output_tokens: i64) -> TokenUsage {
    TokenUsage { input_tokens, output_tokens, cached_tokens: 3, reasoning_tokens: 4 }
}

#[test]
fn upsert_step_start_inserts_steps_sorted_and_updates_session_time() {
    let mut session = empty_session();

    super::upsert_step_start(&mut session, 2, 20, Some("model-b".to_string()), None);
    super::upsert_step_start(
        &mut session,
        1,
        10,
        Some("model-a".to_string()),
        Some("start.snap".to_string()),
    );

    assert_eq!(session.steps.iter().map(|step| step.index).collect::<Vec<_>>(), vec![1, 2]);
    assert_eq!(session.steps[0].started_ms, 10);
    assert_eq!(session.steps[0].start_snapshot_path.as_deref(), Some("start.snap"));
    assert_eq!(session.steps[0].model.as_deref(), Some("model-a"));
    assert_eq!(session.updated_ms, 20);
}

#[test]
fn upsert_step_start_preserves_existing_optional_fields_when_none() {
    let mut session = empty_session();
    session.steps.push(ChatSessionStep {
        index: 7,
        started_ms: 1,
        finished_ms: None,
        start_snapshot_path: Some("old.snap".to_string()),
        finish_snapshot_path: None,
        usage: TokenUsage::default(),
        cost_usd: None,
        finish_reason: None,
        model: Some("old-model".to_string()),
    });

    super::upsert_step_start(&mut session, 7, 30, None, None);

    let step = &session.steps[0];
    assert_eq!(step.started_ms, 30);
    assert_eq!(step.start_snapshot_path.as_deref(), Some("old.snap"));
    assert_eq!(step.model.as_deref(), Some("old-model"));
}

#[test]
fn upsert_step_finish_updates_existing_step_with_usage_and_reason() {
    let mut session = empty_session();
    super::upsert_step_start(&mut session, 1, 10, Some("draft-model".to_string()), None);

    super::upsert_step_finish(
        &mut session,
        1,
        40,
        usage(11, 22),
        Some("stop".to_string()),
        Some("final-model".to_string()),
    );

    let step = &session.steps[0];
    assert_eq!(step.finished_ms, Some(40));
    assert_eq!(step.usage.input_tokens, 11);
    assert_eq!(step.usage.output_tokens, 22);
    assert_eq!(step.finish_reason.as_deref(), Some("stop"));
    assert_eq!(step.model.as_deref(), Some("final-model"));
    assert_eq!(session.updated_ms, 40);
}

#[test]
fn upsert_step_finish_creates_missing_step_from_finish_time() {
    let mut session = empty_session();

    super::upsert_step_finish(&mut session, 5, 50, usage(1, 2), None, None);

    assert_eq!(session.steps.len(), 1);
    assert_eq!(session.steps[0].index, 5);
    assert_eq!(session.steps[0].started_ms, 50);
    assert_eq!(session.steps[0].finished_ms, Some(50));
}
