use super::*;

#[test]
fn tool_content_to_session_text_wraps_plain_output() {
    let text = tool_content_to_session_text("done");

    assert!(text.contains("[Tool results]"));
    assert!(text.contains("done"));
}

#[test]
fn tool_content_to_session_text_extracts_json_tool_result_and_id() {
    let text = tool_content_to_session_text(
        r#"{"tool_call_id":"call-1","content":"line one\nline two","ignored":true}"#,
    );

    assert!(text.contains(r#"<tool_result id="call-1">"#));
    assert!(text.contains("line one\nline two"));
    assert!(!text.contains("ignored"));
}

#[test]
fn tool_content_to_session_text_falls_back_to_original_json_without_string_content() {
    let input = r#"{"tool_call_id":"call-1","content":{"ok":true}}"#;
    let text = tool_content_to_session_text(input);

    assert!(text.contains(r#"<tool_result id="call-1">"#));
    assert!(text.contains(input));
}

#[test]
fn split_stream_model_ref_handles_provider_and_model() {
    let model_ref = split_stream_model_ref(Some("openai/gpt-4.1"));

    assert_eq!(model_ref.provider_id, "openai");
    assert_eq!(model_ref.model_id, "gpt-4.1");
}

#[test]
fn split_stream_model_ref_trims_empty_and_provider_only_values() {
    let empty = split_stream_model_ref(Some("  "));
    let provider_only = split_stream_model_ref(Some("anthropic"));

    assert_eq!(empty.provider_id, "");
    assert_eq!(empty.model_id, "");
    assert_eq!(provider_only.provider_id, "");
    assert_eq!(provider_only.model_id, "anthropic");
}

#[test]
fn normalize_tool_ids_drops_empty_items() {
    let ids = normalize_tool_ids(vec![" shell ".to_string(), " ".to_string()]);

    assert_eq!(ids, Some(vec!["shell".to_string()]));
}

#[test]
fn normalize_tool_ids_deduplicates_and_returns_none_when_empty() {
    let ids = normalize_tool_ids(vec![
        " shell ".to_string(),
        "shell".to_string(),
        " file_read ".to_string(),
    ]);
    let empty = normalize_tool_ids(vec![" ".to_string(), "".to_string()]);

    assert_eq!(ids, Some(vec!["shell".to_string(), "file_read".to_string()]));
    assert_eq!(empty, None);
}

#[test]
fn merge_gateway_request_options_forwards_allowed_tools_to_acp() {
    let options = merge_gateway_request_options(
        None,
        None,
        Some(vec!["file_read".to_string()]),
        None,
        None,
        None,
        Some("codex".to_string()),
        None,
    );

    assert_eq!(options.get("allowed_tools").and_then(|value| value.as_array()).unwrap().len(), 1);
    assert_eq!(
        options.get("acp_allowed_tools").and_then(|value| value.as_array()).unwrap().len(),
        1
    );
}

#[test]
fn merge_gateway_request_options_preserves_explicit_options_over_agent_defaults() {
    let options = merge_gateway_request_options(
        Some(serde_json::json!({
            "chat_system_prompt": "explicit system",
            "temperature": 0.8,
            "top_p": 0.9
        })),
        Some("agent-a".to_string()),
        Some(vec![" shell ".to_string()]),
        Some("agent system".to_string()),
        Some(0.1),
        Some(0.2),
        None,
        Some(vec!["file_read".to_string()]),
    );

    assert_eq!(options.get("agent").and_then(Value::as_str), Some("agent-a"));
    assert_eq!(options.get("chat_system_prompt").and_then(Value::as_str), Some("explicit system"));
    assert_eq!(options.get("temperature").and_then(Value::as_f64), Some(0.8));
    assert_eq!(options.get("top_p").and_then(Value::as_f64), Some(0.9));
    assert_eq!(
        options
            .get("allowed_tools")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(Value::as_str),
        Some(" shell ")
    );
    assert_eq!(
        options
            .get("acp_allowed_tools")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(Value::as_str),
        Some("file_read")
    );
    assert_eq!(options.get("acp_test").and_then(Value::as_bool), Some(true));
}

#[test]
fn merge_gateway_request_options_replaces_non_object_options_with_object() {
    let options = merge_gateway_request_options(
        Some(serde_json::json!("not an object")),
        None,
        None,
        Some("system".to_string()),
        Some(0.3),
        Some(0.7),
        None,
        None,
    );

    assert_eq!(options.get("chat_system_prompt").and_then(Value::as_str), Some("system"));
    assert_eq!(options.get("temperature").and_then(Value::as_f64), Some(0.3));
    assert_eq!(options.get("top_p").and_then(Value::as_f64), Some(0.7));
}

#[test]
fn stream_part_builders_fill_expected_message_boundaries() {
    let base = stream_part_base("ses-1", "msg-assistant", "prt-base".to_string());
    let step_start = build_stream_step_start_part(
        "ses-1",
        "msg-assistant",
        "prt-start".to_string(),
        Some("snapshot-before".to_string()),
    );
    let step_finish = build_stream_step_finish_part(
        "ses-1",
        "msg-assistant",
        "prt-finish".to_string(),
        "snapshot-after".to_string(),
    );
    let patch = build_stream_patch_part(
        "ses-1",
        "msg-assistant",
        "prt-patch".to_string(),
        "hash".to_string(),
        vec!["src/main.rs".to_string()],
    );

    assert_eq!(base.session_id, "ses-1");
    assert_eq!(base.message_id, "msg-assistant");

    match step_start {
        agent_session::message::Part::StepStart(part) => {
            assert_eq!(part.base.id, "prt-start");
            assert_eq!(part.snapshot.as_deref(), Some("snapshot-before"));
        }
        _ => panic!("expected step start part"),
    }

    match step_finish {
        agent_session::message::Part::StepFinish(part) => {
            assert_eq!(part.base.id, "prt-finish");
            assert_eq!(part.reason, "tool_round");
            assert_eq!(part.snapshot.as_deref(), Some("snapshot-after"));
            assert_eq!(part.tokens.total, Some(0));
        }
        _ => panic!("expected step finish part"),
    }

    match patch {
        agent_session::message::Part::Patch(part) => {
            assert_eq!(part.base.id, "prt-patch");
            assert_eq!(part.hash, "hash");
            assert_eq!(part.files, vec!["src/main.rs".to_string()]);
        }
        _ => panic!("expected patch part"),
    }
}

#[test]
fn generated_stream_parts_and_preallocated_ids_are_non_empty() {
    let start = new_stream_step_start_part("ses-1", "msg-a", None).expect("start part");
    let finish =
        new_stream_step_finish_part("ses-1", "msg-a", "snap".to_string()).expect("finish part");
    let patch =
        new_stream_patch_part("ses-1", "msg-a", "hash".to_string(), vec!["file".to_string()])
            .expect("patch part");
    let ids = preallocate_stream_turn_message_ids().expect("preallocated ids");

    assert!(matches!(start, agent_session::message::Part::StepStart(_)));
    assert!(matches!(finish, agent_session::message::Part::StepFinish(_)));
    assert!(matches!(patch, agent_session::message::Part::Patch(_)));
    assert!(!ids.assistant_id.is_empty());
    assert!(!ids.user_id.is_empty());
    assert_ne!(ids.assistant_id, ids.user_id);
}

#[test]
fn token_info_from_usage_maps_gateway_fields() {
    let usage = ui_models::TokenUsage {
        input_tokens: 3,
        output_tokens: 5,
        cached_tokens: 7,
        reasoning_tokens: 11,
    };
    let tokens = token_info_from_usage(&usage);

    assert_eq!(tokens.total, Some(15));
    assert_eq!(tokens.input, 3);
    assert_eq!(tokens.output, 5);
    assert_eq!(tokens.reasoning, 11);
    assert_eq!(tokens.cache.read, 7);
    assert_eq!(tokens.cache.write, 0);
}

#[tokio::test]
async fn resolve_delegate_request_overrides_without_agent_returns_explicit_values() {
    let overrides = resolve_delegate_request_overrides(
        Some("  ".to_string()),
        Some(vec![" shell ".to_string(), "shell".to_string(), " ".to_string()]),
        Some("provider/model".to_string()),
    )
    .await
    .expect("overrides");

    assert_eq!(overrides.agent, None);
    assert_eq!(overrides.model.as_deref(), Some("provider/model"));
    assert_eq!(overrides.allowed_tools, Some(vec!["shell".to_string()]));
    assert_eq!(overrides.chat_system_prompt, None);
    assert_eq!(overrides.temperature, None);
    assert_eq!(overrides.top_p, None);
}

#[tokio::test]
async fn resolve_delegate_request_overrides_rejects_unknown_agent() {
    let error = resolve_delegate_request_overrides(
        Some("agent-that-does-not-exist-for-stream-tests".to_string()),
        None,
        None,
    )
    .await
    .expect_err("unknown agent");

    assert_eq!(error.status, axum::http::StatusCode::BAD_REQUEST);
    assert!(error.to_string().contains("unknown agent:"));
}
