use super::*;
use crate::app::agent::approval::ApprovalManager;
use crate::app::agent::config::AutonomyConfig;
use crate::app::agent::memory::NoneMemory;
use crate::app::agent::observability::NoopObserver;
use crate::app::agent::providers::Provider;

#[derive(Default)]
struct RecordingChannel {
    sent: tokio::sync::Mutex<Vec<String>>,
    finalized: tokio::sync::Mutex<Vec<String>>,
    cancelled: tokio::sync::Mutex<Vec<String>>,
    reactions: tokio::sync::Mutex<Vec<String>>,
    fail_finalize: bool,
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Channel for RecordingChannel {
    fn name(&self) -> &str {
        "test"
    }

    async fn send(&self, message: &SendMessage) -> anyhow::Result<()> {
        self.sent.lock().await.push(message.content.clone());
        Ok(())
    }

    async fn listen(
        &self,
        _tx: tokio::sync::mpsc::Sender<traits::ChannelMessage>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn finalize_draft(
        &self,
        _recipient: &str,
        message_id: &str,
        text: &str,
    ) -> anyhow::Result<()> {
        self.finalized.lock().await.push(format!("{message_id}:{text}"));
        if self.fail_finalize {
            anyhow::bail!("finalize failed");
        }
        Ok(())
    }

    async fn cancel_draft(&self, _recipient: &str, message_id: &str) -> anyhow::Result<()> {
        self.cancelled.lock().await.push(message_id.to_string());
        Ok(())
    }

    async fn add_reaction(
        &self,
        _recipient: &str,
        _message_id: &str,
        emoji: &str,
    ) -> anyhow::Result<()> {
        self.reactions.lock().await.push(format!("add:{emoji}"));
        Ok(())
    }

    async fn remove_reaction(
        &self,
        _recipient: &str,
        _message_id: &str,
        emoji: &str,
    ) -> anyhow::Result<()> {
        self.reactions.lock().await.push(format!("remove:{emoji}"));
        Ok(())
    }
}

struct StaticProvider;

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Provider for StaticProvider {
    async fn chat_with_system(
        &self,
        _system_prompt: Option<&str>,
        _message: &str,
        _model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        Ok("ok".to_string())
    }
}

fn message(channel: &str) -> traits::ChannelMessage {
    traits::ChannelMessage {
        id: "msg".to_string(),
        sender: "sender".to_string(),
        reply_target: "recipient".to_string(),
        content: "user turn".to_string(),
        channel: channel.to_string(),
        timestamp: 1,
        thread_ts: Some("thread".to_string()),
    }
}

fn route() -> ChannelRouteSelection {
    ChannelRouteSelection {
        provider: "provider".to_string(),
        model: "model".to_string(),
        task_mode_enabled: false,
    }
}

fn context(max_tool_iterations: usize) -> ChannelRuntimeContext {
    let provider: Arc<dyn Provider> = Arc::new(StaticProvider);
    ChannelRuntimeContext {
        channels_by_name: Arc::new(HashMap::new()),
        provider: Arc::clone(&provider),
        default_provider: Arc::new("provider".to_string()),
        memory: Arc::new(NoneMemory::new()),
        tools_registry: Arc::new(Vec::new()),
        observer: Arc::new(NoopObserver),
        system_prompt: Arc::new("system".to_string()),
        model: Arc::new("model".to_string()),
        temperature: 0.0,
        auto_save_memory: false,
        max_tool_iterations,
        min_relevance_score: 0.0,
        conversation_histories: Arc::new(Mutex::new(HashMap::new())),
        provider_cache: Arc::new(Mutex::new(HashMap::new())),
        route_overrides: Arc::new(Mutex::new(HashMap::new())),
        api_key: None,
        api_url: None,
        reliability: Arc::new(crate::app::agent::config::ReliabilityConfig::default()),
        provider_runtime_options: crate::app::agent::providers::ProviderRuntimeOptions::default(),
        workspace_dir: Arc::new(PathBuf::from(".")),
        message_timeout_secs: 30,
        interrupt_on_new_message: false,
        multimodal: crate::app::agent::config::MultimodalConfig::default(),
        hooks: None,
        non_cli_excluded_tools: Arc::new(Mutex::new(Vec::new())),
        query_classification: crate::app::agent::config::QueryClassificationConfig::default(),
        model_routes: Vec::new(),
        approval_manager: Arc::new(ApprovalManager::from_config(&AutonomyConfig::default())),
    }
}

#[tokio::test]
async fn completed_success_sends_response_and_appends_history() {
    let ctx = context(3);
    let channel = Arc::new(RecordingChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    handle_llm_result(
        &ctx,
        message("discord"),
        Some(&channel_dyn),
        &route(),
        LlmExecutionResult::Completed(Ok(Ok("assistant reply".to_string()))),
        "history",
        &[ChatMessage::system("system")],
        1,
        true,
        Instant::now(),
        None,
        "user turn",
        None,
        "\u{2705}",
        &crate::app::agent::security::CanaryGuard::new(false),
    )
    .await;

    assert_eq!(channel.sent.lock().await.as_slice(), ["assistant reply"]);
    let histories = ctx.conversation_histories.lock().unwrap();
    assert_eq!(histories["history"].last().unwrap().content, "assistant reply");
}

#[tokio::test]
async fn completed_success_finalizes_draft_and_falls_back_to_send_on_failure() {
    let ctx = context(1);
    let channel = Arc::new(RecordingChannel { fail_finalize: true, ..RecordingChannel::default() });
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    handle_llm_result(
        &ctx,
        message("discord"),
        Some(&channel_dyn),
        &route(),
        LlmExecutionResult::Completed(Ok(Ok("final text".to_string()))),
        "history",
        &[ChatMessage::system("system")],
        1,
        true,
        Instant::now(),
        Some("draft-1"),
        "user turn",
        None,
        "\u{2705}",
        &crate::app::agent::security::CanaryGuard::new(false),
    )
    .await;

    assert_eq!(channel.finalized.lock().await.as_slice(), ["draft-1:final text"]);
    assert_eq!(channel.sent.lock().await.as_slice(), ["final text"]);
}

#[tokio::test]
async fn cancelled_result_cancels_existing_draft() {
    let ctx = context(1);
    let channel = Arc::new(RecordingChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    handle_llm_result(
        &ctx,
        message("discord"),
        Some(&channel_dyn),
        &route(),
        LlmExecutionResult::Cancelled,
        "history",
        &[],
        0,
        true,
        Instant::now(),
        Some("draft-1"),
        "user turn",
        None,
        "\u{26A0}\u{FE0F}",
        &crate::app::agent::security::CanaryGuard::new(false),
    )
    .await;

    assert_eq!(channel.cancelled.lock().await.as_slice(), ["draft-1"]);
}

#[tokio::test]
async fn context_window_error_sends_retry_guidance() {
    let ctx = context(1);
    ctx.conversation_histories
        .lock()
        .unwrap()
        .insert("history".to_string(), vec![ChatMessage::user("old")]);
    let channel = Arc::new(RecordingChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    handle_llm_result(
        &ctx,
        message("discord"),
        Some(&channel_dyn),
        &route(),
        LlmExecutionResult::Completed(Ok(Err(anyhow::anyhow!("maximum context length reached")))),
        "history",
        &[],
        0,
        true,
        Instant::now(),
        None,
        "user turn",
        None,
        "\u{26A0}\u{FE0F}",
        &crate::app::agent::security::CanaryGuard::new(false),
    )
    .await;

    assert!(channel.sent.lock().await[0].contains("Context window exceeded"));
}

#[tokio::test]
async fn tool_iteration_limit_preserves_history_with_pause_marker() {
    let ctx = context(7);
    let channel = Arc::new(RecordingChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    handle_llm_result(
        &ctx,
        message("discord"),
        Some(&channel_dyn),
        &route(),
        LlmExecutionResult::Completed(Ok(Err(anyhow::anyhow!(
            "Agent exceeded maximum tool iterations (7)"
        )))),
        "history",
        &[],
        0,
        true,
        Instant::now(),
        None,
        "user turn",
        None,
        "\u{26A0}\u{FE0F}",
        &crate::app::agent::security::CanaryGuard::new(false),
    )
    .await;

    assert!(channel.sent.lock().await[0].contains("tool-iteration limit (7)"));
    let histories = ctx.conversation_histories.lock().unwrap();
    assert!(histories["history"].last().unwrap().content.contains("Task paused"));
}

#[tokio::test]
async fn generic_error_records_failure_marker_and_notifies_channel() {
    let ctx = context(1);
    let channel = Arc::new(RecordingChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    handle_llm_result(
        &ctx,
        message("discord"),
        Some(&channel_dyn),
        &route(),
        LlmExecutionResult::Completed(Ok(Err(anyhow::anyhow!("provider exploded")))),
        "history",
        &[],
        0,
        true,
        Instant::now(),
        None,
        "user turn",
        None,
        "\u{26A0}\u{FE0F}",
        &crate::app::agent::security::CanaryGuard::new(false),
    )
    .await;

    assert!(channel.sent.lock().await[0].contains("provider exploded"));
    let histories = ctx.conversation_histories.lock().unwrap();
    assert!(histories["history"].last().unwrap().content.contains("Task failed"));
}
