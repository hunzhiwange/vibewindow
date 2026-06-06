use super::*;

#[test]
fn tool_content_to_session_text_wraps_plain_output() {
    let text = tool_content_to_session_text("done");

    assert!(text.contains("[Tool results]"));
    assert!(text.contains("done"));
}

#[test]
fn split_stream_model_ref_handles_provider_and_model() {
    let model_ref = split_stream_model_ref(Some("openai/gpt-4.1"));

    assert_eq!(model_ref.provider_id, "openai");
    assert_eq!(model_ref.model_id, "gpt-4.1");
}

#[test]
fn normalize_tool_ids_drops_empty_items() {
    let ids = normalize_tool_ids(vec![" shell ".to_string(), " ".to_string()]);

    assert_eq!(ids, Some(vec!["shell".to_string()]));
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
