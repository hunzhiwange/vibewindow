use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use aisdk::Error as AiError;
use aisdk::core::capabilities::ToolCallSupport;
use aisdk::core::language_model::{
    LanguageModel, LanguageModelOptions, LanguageModelResponse,
    LanguageModelResponseContentType as AiContent, LanguageModelStreamChunk as AiStreamChunk,
    LanguageModelStreamChunkType, ReasoningEffort, Usage,
};
use aisdk::core::tools::{ToolCallInfo, ToolDetails, ToolResultInfo};
use aisdk::core::{AssistantMessage, Message};
use async_trait::async_trait;
use futures_util::Stream;
use serde_json::{Value, json};

use crate::app::agent::session::llm::types::{Error, StreamEvent};
use crate::app::agent::session::message::AssistantError;
use crate::app::agent::tools;

use super::{
    aisdk_stream_chunk_error, do_stream_request_aisdk_with_model, parse_reasoning_effort_label,
    reasoning_request_config, uses_dashscope_reasoning_body,
};

type ChunkResult = aisdk::Result<Vec<AiStreamChunk>>;
type BoxedProviderStream = Pin<Box<dyn Stream<Item = ChunkResult> + Send>>;

#[derive(Clone, Debug)]
struct FakeModel {
    open_error: Option<AiError>,
    items: Vec<ChunkResult>,
    captured_options: Arc<Mutex<Option<LanguageModelOptions>>>,
}

impl FakeModel {
    fn with_items(items: Vec<ChunkResult>) -> (Self, Arc<Mutex<Option<LanguageModelOptions>>>) {
        let captured_options = Arc::new(Mutex::new(None));
        (
            Self { open_error: None, items, captured_options: captured_options.clone() },
            captured_options,
        )
    }

    fn with_open_error(error: AiError) -> Self {
        Self {
            open_error: Some(error),
            items: Vec::new(),
            captured_options: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl LanguageModel for FakeModel {
    fn name(&self) -> String {
        "fake".to_string()
    }

    async fn generate_text(
        &mut self,
        _options: LanguageModelOptions,
    ) -> aisdk::Result<LanguageModelResponse> {
        Ok(LanguageModelResponse::new(""))
    }

    async fn stream_text(
        &mut self,
        options: LanguageModelOptions,
    ) -> aisdk::Result<BoxedProviderStream> {
        *self.captured_options.lock().expect("capture options") = Some(options);

        if let Some(error) = self.open_error.clone() {
            return Err(error);
        }

        Ok(Box::pin(futures_util::stream::iter(self.items.clone())))
    }
}

impl ToolCallSupport for FakeModel {}

fn tool_call(id: &str, name: &str, input: Value) -> ToolCallInfo {
    let mut info = ToolCallInfo::new(name);
    info.id(id);
    info.input(input);
    info
}

fn done(content: AiContent, usage: Option<Usage>) -> AiStreamChunk {
    AiStreamChunk::Done(AssistantMessage::new(content, usage))
}

async fn run_fake_stream(
    provider_id: &str,
    request_url: &str,
    model: FakeModel,
    merged_options: &Value,
    abort: Option<&tokio::sync::watch::Receiver<bool>>,
) -> (Result<(), Error>, Vec<StreamEvent>) {
    let mut events = Vec::new();
    let tools = HashMap::new();

    let result = do_stream_request_aisdk_with_model(
        provider_id.to_string(),
        model,
        request_url.to_string(),
        true,
        vec![Message::User("hello".into())],
        &tools,
        None,
        None,
        None,
        merged_options,
        0,
        abort,
        &mut |event| events.push(event),
    )
    .await;

    (result, events)
}

#[test]
fn stream_chunk_error_builds_non_retryable_api_error_metadata() {
    let error = aisdk_stream_chunk_error("failed", "bad chunk".to_string());

    match error {
        AssistantError::APIError {
            message,
            status_code,
            is_retryable,
            response_body,
            metadata,
            ..
        } => {
            assert_eq!(message, "bad chunk");
            assert_eq!(status_code, None);
            assert!(!is_retryable);
            assert_eq!(response_body.as_deref(), Some("bad chunk"));
            let metadata = metadata.expect("metadata");
            assert_eq!(metadata.get("source").map(String::as_str), Some("aisdk"));
            assert_eq!(metadata.get("raw_error").map(String::as_str), Some("bad chunk"));
            assert_eq!(metadata.get("stream_failure_kind").map(String::as_str), Some("failed"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn reasoning_helpers_normalize_labels_and_dashscope_routing() {
    assert_eq!(parse_reasoning_effort_label(" LOW "), Some("low"));
    assert_eq!(parse_reasoning_effort_label("medium"), Some("medium"));
    assert_eq!(parse_reasoning_effort_label("High"), Some("high"));
    assert_eq!(parse_reasoning_effort_label("max"), None);

    assert!(uses_dashscope_reasoning_body("alibaba-cn", "https://example.com"));
    assert!(uses_dashscope_reasoning_body(
        "openai",
        "https://dashscope-us.aliyuncs.com/compatible-mode/v1/chat/completions"
    ));
    assert!(uses_dashscope_reasoning_body(
        "openai",
        "https://dashscope-intl.aliyuncs.com/compatible-mode/v1/chat/completions"
    ));
    assert!(!uses_dashscope_reasoning_body("openai", "https://api.openai.com/v1/chat/completions"));
}

#[test]
fn reasoning_request_config_splits_dashscope_body_from_top_level_effort() {
    let missing = json!({});
    let invalid = json!({ "reasoning_effort": "extreme" });
    let dashscope = json!({ "reasoning_effort": "high" });
    let openai = json!({ "reasoning_effort": "medium" });

    assert_eq!(
        reasoning_request_config(
            "openai",
            "https://api.openai.com/v1/chat/completions",
            missing.as_object().unwrap()
        ),
        (None, None)
    );
    assert_eq!(
        reasoning_request_config(
            "openai",
            "https://api.openai.com/v1/chat/completions",
            invalid.as_object().unwrap()
        ),
        (None, None)
    );

    let (effort, body) = reasoning_request_config(
        "alibaba-cn",
        "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions",
        dashscope.as_object().unwrap(),
    );
    assert_eq!(effort, None);
    assert_eq!(body.and_then(|map| map.get("enable_thinking").cloned()), Some(json!(true)));

    let (effort, body) = reasoning_request_config(
        "openai",
        "https://api.openai.com/v1/chat/completions",
        openai.as_object().unwrap(),
    );
    assert_eq!(effort, Some("medium"));
    assert_eq!(body, None);
}

#[tokio::test]
async fn stream_sends_deltas_tool_calls_usage_and_clamped_request_options() {
    let available = tool_call("call-1", "lookup", json!({ "q": "rust" }));
    let duplicate_done = available.clone();
    let (model, captured) = FakeModel::with_items(vec![Ok(vec![
        AiStreamChunk::Delta(LanguageModelStreamChunkType::TextStart),
        AiStreamChunk::Delta(LanguageModelStreamChunkType::TextDelta("hello".to_string())),
        AiStreamChunk::Delta(LanguageModelStreamChunkType::TextEnd),
        AiStreamChunk::Delta(LanguageModelStreamChunkType::ReasoningStart),
        AiStreamChunk::Delta(LanguageModelStreamChunkType::ReasoningDelta("why".to_string())),
        AiStreamChunk::Delta(LanguageModelStreamChunkType::ReasoningEnd),
        AiStreamChunk::Delta(LanguageModelStreamChunkType::ToolCallStart(ToolDetails {
            id: "call-1".to_string(),
            name: "lookup".to_string(),
        })),
        AiStreamChunk::Delta(LanguageModelStreamChunkType::ToolCallDelta {
            id: "call-1".to_string(),
            delta: "{}".to_string(),
        }),
        AiStreamChunk::Delta(LanguageModelStreamChunkType::ToolCallAvailable(available)),
        AiStreamChunk::Delta(LanguageModelStreamChunkType::ToolCallEnd(ToolResultInfo::new(
            "lookup",
        ))),
        done(
            AiContent::Text("ignored because text delta was already sent".to_string()),
            Some(Usage {
                input_tokens: Some(11),
                output_tokens: Some(22),
                cached_tokens: Some(3),
                reasoning_tokens: Some(4),
            }),
        ),
        done(AiContent::ToolCall(duplicate_done), None),
    ])]);

    let tools = HashMap::from([(
        "lookup".to_string(),
        tools::ToolSpec::new(
            "lookup",
            "Lookup docs",
            json!({
                "type": "object",
                "properties": {
                    "q": { "type": "string" }
                }
            }),
        ),
    )]);
    let mut events = Vec::new();
    let options = json!({
        "presence_penalty": 0.25,
        "frequency_penalty": 0.5,
        "seed": u64::MAX,
        "top_k": u64::MAX,
        "reasoning_effort": "high",
        "stop_sequences": [" END ", ""]
    });

    let result = do_stream_request_aisdk_with_model(
        "openai".to_string(),
        model,
        "https://api.openai.com/v1/chat/completions".to_string(),
        true,
        vec![Message::User("hello".into())],
        &tools,
        Some(2.0),
        Some(-1.0),
        Some(u64::MAX),
        &options,
        u64::MAX,
        None,
        &mut |event| events.push(event),
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(events.len(), 4);
    assert!(matches!(&events[0], StreamEvent::Delta(text) if text == "hello"));
    assert!(matches!(&events[1], StreamEvent::ReasoningDelta(text) if text == "why"));
    assert!(matches!(
        &events[2],
        StreamEvent::ToolCalls(calls)
            if calls.len() == 1
                && calls[0].id == "call-1"
                && calls[0].name == "lookup"
                && calls[0].arguments == "{\"q\":\"rust\"}"
    ));
    assert!(matches!(
        &events[3],
        StreamEvent::Done { finish_reason, usage }
            if finish_reason.as_deref() == Some("tool_calls")
                && usage.input_tokens == 11
                && usage.output_tokens == 22
                && usage.cached_tokens == 3
                && usage.reasoning_tokens == 4
    ));

    let captured =
        captured.lock().expect("capture lock").clone().expect("stream options should be captured");
    assert_eq!(captured.temperature, Some(100));
    assert_eq!(captured.top_p, Some(0));
    assert_eq!(captured.max_output_tokens, Some(u32::MAX));
    assert_eq!(captured.max_retries, Some(u32::MAX));
    assert_eq!(captured.presence_penalty, Some(0.25));
    assert_eq!(captured.frequency_penalty, Some(0.5));
    assert_eq!(captured.seed, Some(u32::MAX));
    assert_eq!(captured.top_k, Some(u32::MAX));
    assert_eq!(captured.stop_sequences, Some(vec!["END".to_string()]));
    assert!(matches!(captured.reasoning_effort, Some(ReasoningEffort::High)));
    assert!(captured.body.is_none());
    assert_eq!(captured.messages().len(), 1);
    assert!(format!("{captured:?}").contains("lookup"));
}

#[tokio::test]
async fn dashscope_reasoning_option_is_sent_in_extra_body() {
    let (model, captured) = FakeModel::with_items(vec![]);
    let mut events = Vec::new();

    let result = do_stream_request_aisdk_with_model(
        "alibaba-cn".to_string(),
        model,
        "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions".to_string(),
        false,
        vec![Message::User("hello".into())],
        &HashMap::new(),
        None,
        None,
        None,
        &json!({ "reasoning_effort": "low" }),
        0,
        None,
        &mut |event| events.push(event),
    )
    .await;

    assert!(result.is_ok());
    assert!(matches!(
        events.as_slice(),
        [StreamEvent::Done { finish_reason, .. }] if finish_reason.as_deref() == Some("stop")
    ));

    let captured =
        captured.lock().expect("capture lock").clone().expect("stream options should be captured");
    assert!(captured.reasoning_effort.is_none());
    assert_eq!(
        captured.body.as_ref().and_then(|body| body.get("enable_thinking")),
        Some(&json!(true))
    );
}

#[tokio::test]
async fn done_content_emits_when_no_prior_delta_and_ignores_other_done_content() {
    let (model, _) = FakeModel::with_items(vec![Ok(vec![
        done(AiContent::Text("final text".to_string()), None),
        done(
            AiContent::Reasoning { content: "because".to_string(), extensions: Default::default() },
            None,
        ),
        done(AiContent::NotSupported("nope".to_string()), None),
    ])]);

    let (result, events) = run_fake_stream(
        "openai",
        "https://api.openai.com/v1/chat/completions",
        model,
        &Value::Null,
        None,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(events.len(), 3);
    assert!(matches!(&events[0], StreamEvent::Delta(text) if text == "final text"));
    assert!(matches!(&events[1], StreamEvent::ReasoningDelta(text) if text == "because"));
    assert!(matches!(
        &events[2],
        StreamEvent::Done { finish_reason, usage }
            if finish_reason.as_deref() == Some("stop")
                && usage.input_tokens == 0
                && usage.output_tokens == 0
    ));
}

#[tokio::test]
async fn stream_open_failure_maps_aisdk_error() {
    let model = FakeModel::with_open_error(AiError::MissingField("api_key".to_string()));

    let (result, events) = run_fake_stream(
        "openai",
        "https://api.openai.com/v1/chat/completions",
        model,
        &json!({}),
        None,
    )
    .await;

    assert!(events.is_empty());
    assert!(matches!(
        result,
        Err(Error::Api(AssistantError::ProviderAuthError { provider_id, message }))
            if provider_id == "openai" && message == "api_key"
    ));
}

#[tokio::test]
async fn stream_read_failure_maps_aisdk_api_error() {
    let (model, _) = FakeModel::with_items(vec![Err(AiError::ApiError {
        details: "read failed".to_string(),
        status_code: Some(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
    })]);

    let (result, events) = run_fake_stream(
        "openai",
        "https://api.openai.com/v1/chat/completions",
        model,
        &json!({}),
        None,
    )
    .await;

    assert!(events.is_empty());
    assert!(matches!(
        result,
        Err(Error::Api(AssistantError::APIError {
            message,
            status_code: Some(500),
            is_retryable: true,
            ..
        })) if message == "read failed"
    ));
}

#[tokio::test]
async fn stream_chunk_failure_variants_return_api_errors() {
    for (chunk, expected_kind, expected_message) in [
        (LanguageModelStreamChunkType::Failed("failed".to_string()), "failed", "failed"),
        (
            LanguageModelStreamChunkType::Incomplete("incomplete".to_string()),
            "incomplete",
            "incomplete",
        ),
        (
            LanguageModelStreamChunkType::NotSupported("unsupported".to_string()),
            "not_supported",
            "unsupported",
        ),
    ] {
        let (model, _) = FakeModel::with_items(vec![Ok(vec![AiStreamChunk::Delta(chunk.clone())])]);

        let (result, events) = run_fake_stream(
            "openai",
            "https://api.openai.com/v1/chat/completions",
            model,
            &json!({}),
            None,
        )
        .await;

        assert!(events.is_empty());
        match result {
            Err(Error::Api(AssistantError::APIError { message, metadata, .. })) => {
                assert_eq!(message, expected_message);
                assert_eq!(
                    metadata
                        .as_ref()
                        .and_then(|map| map.get("stream_failure_kind"))
                        .map(String::as_str),
                    Some(expected_kind)
                );
            }
            other => panic!("unexpected result: {other:?}"),
        }
    }
}

#[tokio::test]
async fn stream_returns_aborted_when_abort_flag_is_already_set() {
    let (model, _) = FakeModel::with_items(vec![Ok(vec![AiStreamChunk::Delta(
        LanguageModelStreamChunkType::TextDelta("never emitted".to_string()),
    )])]);
    let (_tx, rx) = tokio::sync::watch::channel(true);

    let (result, events) = run_fake_stream(
        "openai",
        "https://api.openai.com/v1/chat/completions",
        model,
        &json!({}),
        Some(&rx),
    )
    .await;

    assert!(matches!(result, Err(Error::Aborted)));
    assert!(events.is_empty());
}
