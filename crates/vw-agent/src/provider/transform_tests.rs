use super::*;
use serde_json::json;

#[test]
fn remap_provider_options_moves_matching_key() {
    let remapped = remap_provider_options(
        &json!({"openai": {"temperature": 0.2}, "keep": true}),
        "openai",
        "azure",
    );
    assert_eq!(remapped["azure"]["temperature"], json!(0.2));
    assert!(remapped.get("openai").is_none());
}

#[test]
fn provider_options_normalizes_adapter_aliases() {
    let options =
        provider_options("openai", "openai-compatible", json!({"openai": {"maxTokens": 12}}));
    assert_eq!(options["openai"]["maxTokens"], json!(12));
}

#[test]
fn normalize_messages_filters_anthropic_empty_content_parts() {
    let messages = vec![
        json!({"role": "user", "content": ""}),
        json!({"role": "assistant", "content": [
            {"type": "text", "text": ""},
            {"type": "reasoning", "text": ""},
            {"type": "text", "text": "kept"},
            {"type": "image", "data": "x"}
        ]}),
        json!({"role": "user", "content": [{"type": "text", "text": ""}]}),
    ];

    let normalized = normalize_messages(messages, " google-vertex/anthropic ", "claude-3");

    assert_eq!(normalized.len(), 1);
    let content = normalized[0]["content"].as_array().unwrap();
    assert_eq!(content.len(), 2);
    assert_eq!(content[0]["text"], "kept");
}

#[test]
fn normalize_messages_sanitizes_claude_tool_call_ids() {
    let messages = vec![
        json!({"role": "assistant", "content": [
            {"type": "tool-call", "toolCallId": "call.1/needs space"},
            {"type": "tool-result", "toolCallId": "ok-id_2"}
        ]}),
        json!({"role": "user", "content": [
            {"type": "tool-call", "toolCallId": "unchanged / user"}
        ]}),
    ];

    let normalized = normalize_messages(messages, "openai", "claude-sonnet");

    assert_eq!(normalized[0]["content"][0]["toolCallId"], "call_1_needs_space");
    assert_eq!(normalized[0]["content"][1]["toolCallId"], "ok-id_2");
    assert_eq!(normalized[1]["content"][0]["toolCallId"], "unchanged / user");
}

#[test]
fn apply_provider_options_key_remap_updates_message_and_parts() {
    let messages = vec![json!({
        "role": "user",
        "content": [
            {"type": "text", "text": "hi", "providerOptions": {"openai": {"foo": true}}}
        ],
        "providerOptions": {"openai": {"temperature": 0.1}, "keep": 1}
    })];

    let remapped = apply_provider_options_key_remap(messages, "openai", "anthropic");

    assert_eq!(remapped[0]["providerOptions"]["anthropic"]["temperature"], json!(0.1));
    assert_eq!(remapped[0]["providerOptions"]["keep"], json!(1));
    assert_eq!(remapped[0]["content"][0]["providerOptions"]["anthropic"]["foo"], json!(true));
}

#[test]
fn provider_option_remap_skips_unknown_same_and_azure_adapters() {
    let messages = vec![json!({"providerOptions": {"openai": {"x": 1}}})];
    assert_eq!(apply_provider_options_key_remap(messages.clone(), "openai", "unknown"), messages);
    assert_eq!(
        apply_provider_options_key_remap(messages.clone(), "openai", "openai-compatible"),
        messages
    );
    assert_eq!(apply_provider_options_key_remap(messages.clone(), "openai", "azure"), messages);
}

#[test]
fn max_output_tokens_respects_global_cap_and_zero_model_limit() {
    assert_eq!(max_output_tokens(0), OUTPUT_TOKEN_MAX_DEFAULT);
    assert_eq!(max_output_tokens(12), 12);
    assert_eq!(max_output_tokens(OUTPUT_TOKEN_MAX_DEFAULT + 1), OUTPUT_TOKEN_MAX_DEFAULT);
}
