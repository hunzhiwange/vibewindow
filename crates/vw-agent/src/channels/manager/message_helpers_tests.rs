use super::*;
use crate::app::agent::approval::ApprovalManager;
use crate::app::agent::config::AutonomyConfig;
use crate::app::agent::hooks::{HookHandler, HookResult};
use crate::app::agent::memory::{MemoryCategory, MemoryEntry};
use crate::app::agent::observability::NoopObserver;
use crate::app::agent::providers::Provider;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
struct RecordingMemory {
    stores: AtomicUsize,
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Memory for RecordingMemory {
    fn name(&self) -> &str {
        "recording"
    }

    async fn store(
        &self,
        _key: &str,
        _content: &str,
        _category: MemoryCategory,
        _session_id: Option<&str>,
    ) -> anyhow::Result<()> {
        self.stores.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn recall(
        &self,
        _query: &str,
        _limit: usize,
        _session_id: Option<&str>,
    ) -> anyhow::Result<Vec<MemoryEntry>> {
        Ok(Vec::new())
    }

    async fn get(&self, _key: &str) -> anyhow::Result<Option<MemoryEntry>> {
        Ok(None)
    }

    async fn list(
        &self,
        _category: Option<&MemoryCategory>,
        _session_id: Option<&str>,
    ) -> anyhow::Result<Vec<MemoryEntry>> {
        Ok(Vec::new())
    }

    async fn forget(&self, _key: &str) -> anyhow::Result<bool> {
        Ok(false)
    }

    async fn count(&self) -> anyhow::Result<usize> {
        Ok(0)
    }

    async fn health_check(&self) -> bool {
        true
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

struct MutatingHook;

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl HookHandler for MutatingHook {
    fn name(&self) -> &str {
        "mutating"
    }

    async fn on_message_received(
        &self,
        mut message: traits::ChannelMessage,
    ) -> HookResult<traits::ChannelMessage> {
        message.content.push_str(" patched");
        HookResult::Continue(message)
    }
}

struct CancelHook;

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl HookHandler for CancelHook {
    fn name(&self) -> &str {
        "cancel"
    }

    async fn on_message_received(
        &self,
        _message: traits::ChannelMessage,
    ) -> HookResult<traits::ChannelMessage> {
        HookResult::Cancel("drop".to_string())
    }
}

fn channel_message(content: &str) -> traits::ChannelMessage {
    traits::ChannelMessage {
        id: "msg-1".to_string(),
        sender: "sender".to_string(),
        reply_target: "recipient".to_string(),
        content: content.to_string(),
        channel: "test".to_string(),
        timestamp: 1,
        thread_ts: Some("thread".to_string()),
    }
}

fn test_context(
    memory: Arc<dyn Memory>,
    hooks: Option<Arc<crate::app::agent::hooks::HookRunner>>,
) -> ChannelRuntimeContext {
    let provider: Arc<dyn Provider> = Arc::new(StaticProvider);
    ChannelRuntimeContext {
        channels_by_name: Arc::new(HashMap::new()),
        provider: Arc::clone(&provider),
        default_provider: Arc::new("test-provider".to_string()),
        memory,
        tools_registry: Arc::new(Vec::new()),
        observer: Arc::new(NoopObserver),
        system_prompt: Arc::new("system".to_string()),
        model: Arc::new("model".to_string()),
        temperature: 0.0,
        auto_save_memory: true,
        max_tool_iterations: 1,
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
        hooks,
        non_cli_excluded_tools: Arc::new(Mutex::new(Vec::new())),
        query_classification: crate::app::agent::config::QueryClassificationConfig::default(),
        model_routes: Vec::new(),
        approval_manager: Arc::new(ApprovalManager::from_config(&AutonomyConfig::default())),
    }
}

#[tokio::test]
async fn apply_inbound_hooks_returns_original_message_without_hooks() {
    let ctx = test_context(Arc::new(RecordingMemory::default()), None);
    let msg = channel_message("hello");

    let processed = apply_inbound_hooks(&ctx, msg.clone()).await.expect("message should continue");

    assert_eq!(processed.content, msg.content);
}

#[tokio::test]
async fn apply_inbound_hooks_applies_modified_message() {
    let mut runner = crate::app::agent::hooks::HookRunner::new();
    runner.register(Box::new(MutatingHook));
    let ctx = test_context(Arc::new(RecordingMemory::default()), Some(Arc::new(runner)));

    let processed = apply_inbound_hooks(&ctx, channel_message("hello")).await.unwrap();

    assert_eq!(processed.content, "hello patched");
}

#[tokio::test]
async fn apply_inbound_hooks_returns_none_when_hook_cancels() {
    let mut runner = crate::app::agent::hooks::HookRunner::new();
    runner.register(Box::new(CancelHook));
    let ctx = test_context(Arc::new(RecordingMemory::default()), Some(Arc::new(runner)));

    assert!(apply_inbound_hooks(&ctx, channel_message("hello")).await.is_none());
}

#[tokio::test]
async fn apply_semantic_guard_skips_commands_and_missing_runtime_config() {
    let ctx = test_context(Arc::new(RecordingMemory::default()), None);

    assert_eq!(apply_semantic_guard(&ctx, &channel_message(" /help"), None).await, Some(false));
    assert_eq!(
        apply_semantic_guard(&ctx, &channel_message("normal message"), None).await,
        Some(false)
    );
}

#[tokio::test]
async fn maybe_auto_save_message_respects_enabled_flag_and_minimum_length() {
    let memory = Arc::new(RecordingMemory::default());
    let ctx = test_context(Arc::clone(&memory) as Arc<dyn Memory>, None);

    maybe_auto_save_message(&ctx, &channel_message("short")).await;
    maybe_auto_save_message(&ctx, &channel_message("this message is long enough")).await;

    assert_eq!(memory.stores.load(Ordering::SeqCst), 1);
}

#[test]
fn has_prior_history_reports_only_non_empty_history() {
    let ctx = test_context(Arc::new(RecordingMemory::default()), None);

    assert!(!has_prior_history(&ctx, "sender"));
    ctx.conversation_histories
        .lock()
        .unwrap()
        .insert("sender".to_string(), vec![ChatMessage::user("hello")]);

    assert!(has_prior_history(&ctx, "sender"));
}

#[tokio::test]
async fn load_prior_turns_normalizes_cached_turns() {
    let ctx = test_context(Arc::new(RecordingMemory::default()), None);
    ctx.conversation_histories
        .lock()
        .unwrap()
        .insert("sender".to_string(), vec![ChatMessage::user("one"), ChatMessage::user("two")]);

    let turns = load_prior_turns(&ctx, "sender", "ignored", true).await;

    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].content, "one\n\ntwo");
}
