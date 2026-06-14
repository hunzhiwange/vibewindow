use super::*;
use crate::queue_owner_turn_controller::QueueOwnerActiveSessionController;
use crate::types::{
    AcpAgentConfig, PermissionStats, PromptResult, SessionResumePolicy, SessionStrategy,
};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct TestRuntimeError(&'static str);

#[test]
fn map_finish_reason_maps_known_values_and_defaults() {
    assert_eq!(map_finish_reason(Some("length")), StopReason::MaxTokens);
    assert_eq!(map_finish_reason(Some("max_turn_requests")), StopReason::MaxTurnRequests);
    assert_eq!(map_finish_reason(Some("refusal")), StopReason::Refusal);
    assert_eq!(map_finish_reason(Some("cancelled")), StopReason::Cancelled);
    assert_eq!(map_finish_reason(Some("other")), StopReason::EndTurn);
    assert_eq!(map_finish_reason(None), StopReason::EndTurn);
}

#[test]
fn resolve_agent_config_prefers_structured_config() {
    let mut env = HashMap::new();
    env.insert("KEY".to_string(), "value".to_string());
    let structured = AcpAgentConfig {
        command: "structured-agent".to_string(),
        args: vec!["--fast".to_string()],
        env,
    };

    let resolved = resolve_agent_config("string-agent --ignored", Some(structured.clone()));

    assert_eq!(resolved, structured);
}

#[test]
fn resolve_agent_config_parses_command_when_config_missing() {
    let resolved = resolve_agent_config(r#"agent "two words" --flag"#, None);

    assert_eq!(resolved.command, "agent");
    assert_eq!(resolved.args, vec!["two words", "--flag"]);
    assert!(resolved.env.is_empty());
}

#[test]
fn prompt_request_uses_display_text_and_default_resume_strategy() {
    let prompt = crate::prompt_content::text_prompt("hello");

    let request = prompt_request_for_session(
        "session-1".to_string(),
        ".",
        &prompt,
        Some(SessionResumePolicy::AllowNew),
    );

    assert_eq!(request.prompt, "hello");
    assert!(request.cwd.is_absolute());
    assert_eq!(request.session_strategy, SessionStrategy::ResumeLoadOrNew("session-1".to_string()));
}

#[test]
fn prompt_request_same_session_only_uses_resume_or_load() {
    let prompt = crate::prompt_content::text_prompt("resume only");

    let request = prompt_request_for_session(
        "session-2".to_string(),
        "/tmp/project",
        &prompt,
        Some(SessionResumePolicy::SameSessionOnly),
    );

    assert_eq!(request.cwd, std::path::PathBuf::from("/tmp/project"));
    assert_eq!(request.session_strategy, SessionStrategy::ResumeOrLoad("session-2".to_string()));
}

#[test]
fn should_retry_prompt_requires_runtime_output_code() {
    let timeout = SessionRuntimeError::from_source(crate::session_runtime_helpers::TimeoutError {
        timeout_ms: 10,
    });

    assert!(!should_retry_prompt(&timeout));
}

#[test]
fn should_retry_prompt_accepts_retryable_acp_internal_error_messages() {
    let internal = SessionRuntimeError::from_source(TestRuntimeError("agent returned -32603"));
    let parse = SessionRuntimeError::from_source(TestRuntimeError("agent returned -32700"));

    assert!(should_retry_prompt(&internal));
    assert!(should_retry_prompt(&parse));
}

#[test]
fn should_retry_prompt_rejects_plain_runtime_errors() {
    let error = SessionRuntimeError::from_source(TestRuntimeError("plain runtime failure"));

    assert!(!should_retry_prompt(&error));
}

#[test]
fn to_prompt_result_preserves_stop_reason_and_session_id() {
    let client = AcpClient::new(
        "agent".to_string(),
        AcpAgentConfig { command: "agent".to_string(), args: Vec::new(), env: HashMap::new() },
    );

    let result = to_prompt_result(StopReason::Cancelled, "session-3".to_string(), &client);

    assert_eq!(
        result,
        RunPromptResult {
            stop_reason: StopReason::Cancelled,
            session_id: "session-3".to_string(),
            permission_stats: PermissionStats::default(),
        }
    );
}

#[test]
fn prompt_result_finish_reason_maps_to_runtime_stop_reason() {
    let result = PromptResult {
        session_id: "agent-session".to_string(),
        deltas: vec!["a".to_string(), "b".to_string()],
        finish_reason: Some("length".to_string()),
        usage: None,
    };

    assert_eq!(map_finish_reason(result.finish_reason.as_deref()), StopReason::MaxTokens);
}

#[tokio::test]
async fn noop_active_session_controller_denies_active_cancel() {
    let controller = NoopActiveSessionController;

    assert!(!controller.has_active_prompt());
    assert!(!controller.request_cancel_active_prompt().await.unwrap());
    assert!(controller.set_session_mode("default".to_string()).await.is_ok());
    assert!(controller.set_session_model("model".to_string()).await.is_ok());
    assert!(
        controller.set_session_config_option("id".to_string(), "value".to_string()).await.is_err()
    );
}
