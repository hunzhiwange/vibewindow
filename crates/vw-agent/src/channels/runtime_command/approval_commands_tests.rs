use super::*;
use crate::app::agent::approval::{ApprovalManager, ApprovalResponse};
use crate::app::agent::config::{AutonomyConfig, ReliabilityConfig};
use crate::app::agent::memory::NoneMemory;
use crate::app::agent::observability::NoopObserver;
use crate::app::agent::providers::{ChatMessage, Provider, ProviderRuntimeOptions};
use crate::app::agent::tools::{Tool, ToolResult};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

struct StaticProvider;

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Provider for StaticProvider {
    async fn chat_with_system(
        &self,
        _system_prompt: Option<&str>,
        message: &str,
        _model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        Ok(message.to_string())
    }
}

struct NamedTool(&'static str);

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Tool for NamedTool {
    fn name(&self) -> &str {
        self.0
    }

    fn description(&self) -> &str {
        "unit test tool"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({"type": "object"})
    }

    async fn execute(&self, _args: Value) -> anyhow::Result<ToolResult> {
        Ok(ToolResult { success: true, output: "ok".to_string(), error: None })
    }
}

fn approval_config(auto_approve: &[&str]) -> AutonomyConfig {
    let mut config = AutonomyConfig::default();
    config.auto_approve = auto_approve.iter().map(|tool| (*tool).to_string()).collect();
    config.always_ask = Vec::new();
    config
}

fn boxed_tools(names: &[&'static str]) -> Arc<Vec<Box<dyn Tool>>> {
    Arc::new(names.iter().map(|name| Box::new(NamedTool(*name)) as Box<dyn Tool>).collect())
}

fn context(
    workspace_dir: &Path,
    tool_names: &[&'static str],
    auto_approve: &[&str],
    excluded_tools: &[&str],
) -> ChannelRuntimeContext {
    ChannelRuntimeContext {
        channels_by_name: Arc::new(HashMap::new()),
        provider: Arc::new(StaticProvider),
        default_provider: Arc::new("default-provider".to_string()),
        memory: Arc::new(NoneMemory::new()),
        tools_registry: boxed_tools(tool_names),
        observer: Arc::new(NoopObserver),
        system_prompt: Arc::new("system".to_string()),
        model: Arc::new("default-model".to_string()),
        temperature: 0.2,
        auto_save_memory: false,
        max_tool_iterations: 4,
        min_relevance_score: 0.0,
        conversation_histories: Arc::new(Mutex::new(HashMap::<String, Vec<ChatMessage>>::new())),
        provider_cache: Arc::new(Mutex::new(HashMap::new())),
        route_overrides: Arc::new(Mutex::new(HashMap::new())),
        api_key: None,
        api_url: None,
        reliability: Arc::new(ReliabilityConfig::default()),
        provider_runtime_options: ProviderRuntimeOptions::default(),
        workspace_dir: Arc::new(PathBuf::from(workspace_dir)),
        message_timeout_secs: 30,
        interrupt_on_new_message: false,
        multimodal: crate::app::agent::config::MultimodalConfig::default(),
        hooks: None,
        non_cli_excluded_tools: Arc::new(Mutex::new(
            excluded_tools.iter().map(|tool| (*tool).to_string()).collect(),
        )),
        query_classification: crate::app::agent::config::QueryClassificationConfig::default(),
        model_routes: Vec::new(),
        approval_manager: Arc::new(ApprovalManager::from_config(&approval_config(auto_approve))),
    }
}

fn create_pending(ctx: &ChannelRuntimeContext, tool_name: &str) -> String {
    ctx.approval_manager
        .create_non_cli_pending_request(
            tool_name,
            "sender",
            "telegram",
            "chat-1",
            Some("reason".to_string()),
            Value::Null,
            None,
            None,
        )
        .request_id
}

#[test]
fn available_tools_preview_sorts_and_limits_to_twelve_tools() {
    let temp = tempfile::tempdir().unwrap();
    let ctx = context(
        temp.path(),
        &["m", "a", "z", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"],
        &[],
        &[],
    );

    assert_eq!(available_tools_preview(&ctx), "a, b, c, d, e, f, g, h, i, j, k, m");
}

#[test]
fn request_all_tools_once_creates_scoped_pending_request() {
    let temp = tempfile::tempdir().unwrap();
    let ctx = context(temp.path(), &["shell"], &[], &[]);

    let response = handle_request_all_tools_once(&ctx, "sender", "telegram", "chat-1");
    let pending = ctx.approval_manager.list_non_cli_pending_requests(
        Some("sender"),
        Some("telegram"),
        Some("chat-1"),
    );

    assert!(response.contains("One-time all-tools approval request created."));
    assert!(response.contains("/approve-confirm"));
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].tool_name, APPROVAL_ALL_TOOLS_ONCE_TOKEN);
}

#[test]
fn request_tool_approval_handles_usage_unknown_already_approved_and_pending() {
    let temp = tempfile::tempdir().unwrap();
    let ctx = context(temp.path(), &["shell", "already"], &["already"], &[]);

    assert_eq!(
        handle_request_tool_approval(&ctx, "sender", "telegram", "chat-1", " ".to_string()),
        "Usage: `/approve-request <tool-name>`"
    );

    let unknown =
        handle_request_tool_approval(&ctx, "sender", "telegram", "chat-1", "missing".to_string());
    assert!(unknown.contains("Unknown tool `missing`"));
    assert!(unknown.contains("already, shell"));

    let approved =
        handle_request_tool_approval(&ctx, "sender", "telegram", "chat-1", "already".to_string());
    assert!(approved.contains("already approved"));

    let pending =
        handle_request_tool_approval(&ctx, "sender", "telegram", "chat-1", "shell".to_string());
    assert!(pending.contains("Approval request created."));
    assert!(pending.contains("Tool: `shell`"));
}

#[test]
fn approve_pending_request_handles_usage_not_found_mismatch_and_success() {
    let temp = tempfile::tempdir().unwrap();
    let ctx = context(temp.path(), &["shell"], &[], &[]);
    let request_id = create_pending(&ctx, "shell");

    assert_eq!(
        handle_approve_pending_request(&ctx, "sender", "telegram", "chat-1", " ".to_string()),
        "Usage: `/approve-allow <request-id>`"
    );
    assert!(
        handle_approve_pending_request(
            &ctx,
            "sender",
            "telegram",
            "chat-1",
            "missing".to_string(),
        )
        .contains("was not found")
    );
    assert!(
        handle_approve_pending_request(&ctx, "other", "telegram", "chat-1", request_id.clone(),)
            .contains("can only be approved by the same sender")
    );

    let response =
        handle_approve_pending_request(&ctx, "sender", "telegram", "chat-1", request_id.clone());
    assert!(response.contains("Approved pending request"));
    assert_eq!(
        ctx.approval_manager.take_non_cli_pending_resolution(&request_id),
        Some(ApprovalResponse::Yes)
    );
}

#[tokio::test]
async fn confirm_tool_approval_grants_all_tools_once_without_persistence() {
    let temp = tempfile::tempdir().unwrap();
    let ctx = context(temp.path(), &["shell"], &[], &[]);
    let request_id = create_pending(&ctx, APPROVAL_ALL_TOOLS_ONCE_TOKEN);

    let response =
        handle_confirm_tool_approval(&ctx, "sender", "telegram", "chat-1", request_id).await;

    assert!(response.contains("Approved one-time all-tools bypass"));
    assert!(response.contains("runtime-only"));
    assert_eq!(ctx.approval_manager.non_cli_allow_all_once_remaining(), 1);
}

#[tokio::test]
async fn confirm_tool_approval_grants_session_and_reports_excluded_tool() {
    let temp = tempfile::tempdir().unwrap();
    let ctx = context(temp.path(), &["shell"], &[], &["shell"]);
    let request_id = create_pending(&ctx, "shell");

    let response =
        handle_confirm_tool_approval(&ctx, "sender", "telegram", "chat-1", request_id).await;

    assert!(response.contains("Approved supervised execution for `shell`"));
    assert!(response.contains("No runtime config path was found"));
    assert!(response.contains("currently listed in `autonomy.non_cli_excluded_tools`"));
    assert!(ctx.approval_manager.is_non_cli_session_granted("shell"));
    assert!(!ctx.approval_manager.needs_approval("shell"));
}

#[tokio::test]
async fn confirm_tool_approval_reports_usage_and_pending_errors() {
    let temp = tempfile::tempdir().unwrap();
    let ctx = context(temp.path(), &["shell"], &[], &[]);
    let request_id = create_pending(&ctx, "shell");

    assert_eq!(
        handle_confirm_tool_approval(&ctx, "sender", "telegram", "chat-1", " ".to_string()).await,
        "Usage: `/approve-confirm <request-id>`"
    );
    assert!(
        handle_confirm_tool_approval(&ctx, "sender", "telegram", "chat-1", "missing".to_string())
            .await
            .contains("was not found")
    );
    assert!(
        handle_confirm_tool_approval(&ctx, "other", "telegram", "chat-1", request_id)
            .await
            .contains("can only be confirmed by the same sender")
    );
}

#[test]
fn deny_tool_approval_records_no_resolution_and_reports_errors() {
    let temp = tempfile::tempdir().unwrap();
    let ctx = context(temp.path(), &["shell"], &[], &[]);
    let request_id = create_pending(&ctx, "shell");

    assert_eq!(
        handle_deny_tool_approval(&ctx, "sender", "telegram", "chat-1", " ".to_string()),
        "Usage: `/approve-deny <request-id>`"
    );
    assert!(
        handle_deny_tool_approval(&ctx, "sender", "telegram", "chat-1", "missing".to_string())
            .contains("was not found")
    );
    assert!(
        handle_deny_tool_approval(&ctx, "other", "telegram", "chat-1", request_id.clone())
            .contains("can only be denied by the same sender")
    );

    let response =
        handle_deny_tool_approval(&ctx, "sender", "telegram", "chat-1", request_id.clone());
    assert!(response.contains("Denied pending approval request"));
    assert_eq!(
        ctx.approval_manager.take_non_cli_pending_resolution(&request_id),
        Some(ApprovalResponse::No)
    );
}

#[test]
fn list_pending_approvals_reports_empty_and_scoped_rows() {
    let temp = tempfile::tempdir().unwrap();
    let ctx = context(temp.path(), &["shell"], &[], &[]);

    assert_eq!(
        handle_list_pending_approvals(&ctx, "sender", "telegram", "chat-1"),
        "No pending approval requests for your current sender+chat/channel scope."
    );

    let request_id = create_pending(&ctx, "shell");
    let response = handle_list_pending_approvals(&ctx, "sender", "telegram", "chat-1");
    assert!(response.contains("Pending approval requests"));
    assert!(response.contains(&request_id));
    assert!(response.contains("tool=shell"));
    assert!(response.contains("reason=reason"));
}

#[tokio::test]
async fn approve_tool_handles_usage_unknown_grant_clear_pending_and_exclusion_note() {
    let temp = tempfile::tempdir().unwrap();
    let ctx = context(temp.path(), &["shell"], &[], &["shell"]);
    let request_id = create_pending(&ctx, "shell");

    assert_eq!(handle_approve_tool(&ctx, " ".to_string()).await, "Usage: `/approve <tool-name>`");
    assert!(handle_approve_tool(&ctx, "missing".to_string()).await.contains("Unknown tool"));

    let response = handle_approve_tool(&ctx, " shell ".to_string()).await;
    assert!(response.contains("Approved supervised execution for `shell`"));
    assert!(response.contains("No runtime config path was found"));
    assert!(response.contains("Runtime pending requests cleared: 1"));
    assert!(response.contains("currently listed in `autonomy.non_cli_excluded_tools`"));
    assert!(ctx.approval_manager.is_non_cli_session_granted("shell"));
    assert!(!ctx.approval_manager.has_non_cli_pending_request(&request_id));
}

#[tokio::test]
async fn unapprove_tool_handles_usage_and_revokes_runtime_session_without_config_path() {
    let temp = tempfile::tempdir().unwrap();
    let ctx = context(temp.path(), &["shell"], &[], &[]);
    ctx.approval_manager.grant_non_cli_session("shell");
    ctx.approval_manager.apply_persistent_runtime_grant("shell");
    create_pending(&ctx, "shell");

    assert_eq!(
        handle_unapprove_tool(&ctx, " ".to_string()).await,
        "Usage: `/unapprove <tool-name>`"
    );

    let response = handle_unapprove_tool(&ctx, "shell".to_string()).await;
    assert!(response.contains("Runtime config path was not found"));
    assert!(response.contains("Runtime session grant removed for `shell`: yes"));
    assert!(!ctx.approval_manager.is_non_cli_session_granted("shell"));
    assert!(!ctx.approval_manager.has_non_cli_pending_request("shell"));
}

#[tokio::test]
async fn list_approvals_reports_runtime_state_without_config_path() {
    let temp = tempfile::tempdir().unwrap();
    let ctx = context(temp.path(), &["shell"], &["file_read"], &["shell"]);
    create_pending(&ctx, "shell");
    ctx.approval_manager.grant_non_cli_allow_all_once();

    let response = handle_list_approvals(&ctx, "sender", "telegram", "chat-1").await;

    assert!(response.contains("Supervised non-CLI tool approvals:"));
    assert!(response.contains("Runtime auto_approve (effective): file_read"));
    assert!(response.contains("Runtime one-time all-tools bypass tokens: 1"));
    assert!(response.contains("Runtime non_cli_excluded_tools: shell"));
    assert!(response.contains("Pending approvals"));
    assert!(response.contains("Persisted config approvals: unavailable"));
}
