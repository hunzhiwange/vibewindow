use super::*;
use crate::app::agent::approval::ApprovalManager;
use crate::app::agent::config::AutonomyConfig;
use crate::app::agent::memory::NoneMemory;
use crate::app::agent::observability::NoopObserver;
use crate::app::agent::providers::Provider;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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

fn channel_message(thread_ts: Option<&str>) -> traits::ChannelMessage {
    traits::ChannelMessage {
        id: "msg-1".to_string(),
        sender: "sender-1".to_string(),
        reply_target: "room-1".to_string(),
        content: "hello".to_string(),
        channel: "telegram".to_string(),
        timestamp: 1,
        thread_ts: thread_ts.map(str::to_string),
    }
}

fn test_context() -> ChannelRuntimeContext {
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
        hooks: None,
        non_cli_excluded_tools: Arc::new(Mutex::new(Vec::new())),
        query_classification: crate::app::agent::config::QueryClassificationConfig::default(),
        model_routes: Vec::new(),
        approval_manager: Arc::new(ApprovalManager::from_config(&AutonomyConfig::default())),
    }
}

/// 测试 `clear_sender_session_id_for_scope` 只移除目标范围键
///
/// 验证：
/// - 清除操作只影响指定的项目范围
/// - 其他项目范围的会话映射保持不变
#[test]
fn clear_sender_session_id_for_scope_removes_only_target_scope_key() {
    let project_scope_a = "scope-a";
    let project_scope_b = "scope-b";
    let sender_key = "telegram_user-1";
    let key_a = format!("{}::{}", project_scope_a, sender_key);
    let key_b = format!("{}::{}", project_scope_b, sender_key);

    // 准备测试数据
    let mut store = sender_session_store().lock().unwrap_or_else(|e| e.into_inner());
    store.insert(key_a.clone(), "ses_a".to_string());
    store.insert(key_b.clone(), "ses_b".to_string());
    drop(store);

    // 执行清除操作
    clear_sender_session_id_for_scope(project_scope_a, sender_key);

    // 验证结果
    let store = sender_session_store().lock().unwrap_or_else(|e| e.into_inner());
    assert!(!store.contains_key(&key_a));
    assert!(store.contains_key(&key_b));
    drop(store);
    sender_session_store().lock().unwrap_or_else(|e| e.into_inner()).remove(&key_b);
}

#[test]
fn channel_project_directory_uses_workspace_dir_without_override() {
    let workspace = tempfile::tempdir().expect("workspace temp dir");
    let mut ctx = test_context();
    ctx.workspace_dir = Arc::new(workspace.path().to_path_buf());

    channel_project_dir_override_store().lock().unwrap_or_else(|e| e.into_inner()).take();
    assert_eq!(channel_project_directory(&ctx), workspace.path().to_path_buf());
}

#[test]
fn initial_session_title_normalizes_truncates_and_falls_back_to_channel_sender() {
    let mut msg = channel_message(None);

    msg.content = "  hello\n   world\tfrom channel  ".to_string();
    assert_eq!(initial_session_title_for_message(&msg), "hello world from channel");

    msg.content = format!("{} tail", "a".repeat(51));
    assert_eq!(initial_session_title_for_message(&msg), format!("{}...", "a".repeat(50)));

    msg.content = " \n\t ".to_string();
    assert_eq!(initial_session_title_for_message(&msg), "telegram sender-1");
}

#[tokio::test]
async fn clear_sender_session_id_removes_mapping_for_resolved_project_scope() {
    let workspace = tempfile::tempdir().expect("workspace temp dir");
    let mut ctx = test_context();
    ctx.workspace_dir = Arc::new(workspace.path().to_path_buf());
    let msg = channel_message(Some("thread-9"));
    let sender_key = sender_session_key(&msg);
    let scope = resolve_channel_project_scope_id(&ctx).await;
    let key = format!("{scope}::{sender_key}");
    sender_session_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(key.clone(), "session-1".to_string());

    clear_sender_session_id(&ctx, &msg).await;

    assert!(!sender_session_store().lock().unwrap_or_else(|e| e.into_inner()).contains_key(&key));
}

#[tokio::test]
async fn resolve_or_create_sender_session_id_replaces_invalid_mapping_then_reuses_session() {
    let workspace = tempfile::tempdir().expect("workspace temp dir");
    let mut ctx = test_context();
    ctx.workspace_dir = Arc::new(workspace.path().to_path_buf());
    let mut msg = channel_message(None);
    msg.content = "first channel request".to_string();
    let sender_key = sender_session_key(&msg);
    let scope = resolve_channel_project_scope_id(&ctx).await;
    let key = format!("{scope}::{sender_key}");
    let missing_session_id = format!("missing-session-{}", std::process::id());
    sender_session_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(key.clone(), missing_session_id.clone());

    let created = resolve_or_create_sender_session_id(&ctx, &msg).await;
    let reused = resolve_or_create_sender_session_id(&ctx, &msg).await;

    assert_ne!(created, missing_session_id);
    assert_eq!(created, reused);
    assert_eq!(
        sender_session_store().lock().unwrap_or_else(|e| e.into_inner()).get(&key),
        Some(&created)
    );

    sender_session_store().lock().unwrap_or_else(|e| e.into_inner()).remove(&key);
}

#[test]
fn to_session_history_maps_known_roles_and_skips_unknown_roles() {
    let turns = vec![
        ChatMessage::system("system"),
        ChatMessage::user("user"),
        ChatMessage::assistant("assistant"),
        ChatMessage::tool("tool"),
        ChatMessage { role: "unknown".to_string(), content: "skip".to_string() },
    ];

    let history = to_session_history(&turns);

    assert_eq!(history.len(), 4);
    assert!(matches!(history[0].role, crate::session::ui_types::ChatRole::System));
    assert!(matches!(history[1].role, crate::session::ui_types::ChatRole::User));
    assert!(matches!(history[2].role, crate::session::ui_types::ChatRole::Assistant));
    assert!(matches!(history[3].role, crate::session::ui_types::ChatRole::Tool));
    assert!(history.iter().all(|message| message.think_timing.is_empty()));
}
