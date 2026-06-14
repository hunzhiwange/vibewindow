use super::*;
use crate::app::agent::config::{DelegateAgentConfig, MultimodalConfig};
use crate::app::agent::providers::ProviderRuntimeOptions;
use crate::app::agent::security::policy::AutonomyLevel;
use crate::app::agent::tools::subagent_registry::{SubAgentSession, SubAgentStatus};
use crate::app::agent::tools::traits::Tool;
use chrono::{Duration, Utc};
use serde_json::json;

fn build_tool_with_registry(
    registry: Arc<SubAgentRegistry>,
    mut security: SecurityPolicy,
) -> AgentTool {
    security.max_actions_per_hour = 10;
    let security = Arc::new(security);
    let agents = HashMap::from([("writer".to_string(), DelegateAgentConfig::default())]);
    let delegate_tool = Arc::new(DelegateTool::new(
        agents.clone(),
        Some("fallback-key".to_string()),
        Arc::clone(&security),
    ));
    let background_tool = Arc::new(SubAgentSpawnTool::new(
        agents.clone(),
        Some("fallback-key".to_string()),
        Arc::clone(&security),
        ProviderRuntimeOptions::default(),
        Arc::clone(&registry),
        Arc::new(Vec::new()),
        MultimodalConfig::default(),
    ));

    AgentTool::new(agents, delegate_tool, background_tool, registry, security)
}

fn build_tool() -> (AgentTool, Arc<SubAgentRegistry>) {
    let registry = Arc::new(SubAgentRegistry::new());
    (build_tool_with_registry(Arc::clone(&registry), SecurityPolicy::default()), registry)
}

fn insert_session(
    registry: &SubAgentRegistry,
    id: &str,
    status: SubAgentStatus,
    result: Option<ToolResult>,
) {
    let now = Utc::now();
    registry.insert(SubAgentSession {
        id: id.to_string(),
        agent_name: "writer".to_string(),
        title: Some("Draft".to_string()),
        task: "write a careful summary".to_string(),
        metadata: json!({"topic": "coverage"}),
        status,
        started_at: now - Duration::milliseconds(12),
        updated_at: now,
        completed_at: result.as_ref().map(|_| now),
        result,
        #[cfg(not(target_arch = "wasm32"))]
        handle: None,
    });
}

#[test]
fn truncate_output_preserves_short_text_and_marks_truncation() {
    assert_eq!(truncate_output("short", 10), "short");
    assert_eq!(truncate_output("abcdef", 3), "abc... (truncated)");
}

#[test]
fn truncate_output_respects_char_boundaries() {
    assert_eq!(truncate_output("你好世界", 2), "你好... (truncated)");
}

#[test]
fn action_resolution_accepts_aliases_and_legacy_inference() {
    let (tool, _) = build_tool();

    assert!(matches!(tool.resolve_action(&json!({"action": "run"})).unwrap(), AgentAction::Launch));
    assert!(matches!(tool.resolve_action(&json!({"action": "status"})).unwrap(), AgentAction::Get));
    assert!(matches!(tool.resolve_action(&json!({"action": "kill"})).unwrap(), AgentAction::Stop));
    assert!(matches!(
        tool.resolve_action(&json!({"status": "running"})).unwrap(),
        AgentAction::List
    ));
    assert!(matches!(tool.resolve_action(&json!({"session_id": "s1"})).unwrap(), AgentAction::Get));
    assert!(matches!(tool.resolve_action(&json!({})).unwrap(), AgentAction::Launch));

    let message_err = match tool.resolve_action(&json!({"action": "message"})) {
        Ok(_) => panic!("message action should fail"),
        Err(err) => err,
    };
    assert!(message_err.to_string().contains("not implemented"));

    let unknown_err = match tool.resolve_action(&json!({"action": "dance"})) {
        Ok(_) => panic!("unknown action should fail"),
        Err(err) => err,
    };
    assert!(unknown_err.to_string().contains("Unknown AgentTool action"));
}

#[test]
fn argument_helpers_trim_and_validate_aliases() {
    let (tool, _) = build_tool();
    let args = json!({
        "subagent_type": " writer ",
        "task": " draft ",
        "run_in_background": false,
        "status": "all"
    });

    assert_eq!(tool.agent_name_from_args(&args), Some("writer"));
    assert_eq!(tool.prompt_from_args(&args), Some("draft"));
    assert!(!tool.run_in_background(&args));
    assert_eq!(tool.normalized_status_filter(&args).unwrap(), None);
    assert_eq!(tool.session_id_from_args(&json!({"session_id": " s1 "})).unwrap(), "s1");

    let err = tool.session_id_from_args(&json!({"session_id": " "})).unwrap_err();
    assert!(err.to_string().contains("Missing 'session_id'"));

    let err = tool.normalized_status_filter(&json!({"status": "paused"})).unwrap_err();
    assert!(err.to_string().contains("Invalid status filter"));
}

#[test]
fn spec_and_schema_describe_configured_agents() {
    let (tool, _) = build_tool();

    assert_eq!(tool.name(), "AgentTool");
    assert!(tool.description().contains("AgentTool"));
    assert!(
        tool.parameters_schema()["properties"]["agent"]["description"]
            .as_str()
            .unwrap()
            .contains("writer")
    );

    let spec = tool.spec();
    assert_eq!(spec.id, "AgentTool");
    assert!(spec.aliases.contains(&"agent".to_string()));
    assert!(!spec.read_only);
    assert!(spec.strict);
}

#[tokio::test]
async fn list_get_and_stop_cover_registry_paths() {
    let (tool, registry) = build_tool();
    insert_session(
        &registry,
        "done",
        SubAgentStatus::Completed,
        Some(ToolResult { success: true, output: "x".repeat(520), error: None }),
    );
    insert_session(&registry, "running", SubAgentStatus::Running, None);

    let list = tool.execute(json!({"action": "list", "status": "completed"})).await.unwrap();
    assert!(list.success);
    assert!(list.output.contains("\"session_id\": \"done\""));
    assert!(!list.output.contains("\"session_id\": \"running\""));

    let get = tool.execute(json!({"action": "get", "session_id": "done"})).await.unwrap();
    assert!(get.success);
    let body: serde_json::Value = serde_json::from_str(&get.output).unwrap();
    assert_eq!(body["session_id"], "done");
    assert_eq!(body["result"]["success"], true);
    assert!(body["result"]["output"].as_str().unwrap().ends_with("... (truncated)"));

    let missing = tool.execute(json!({"action": "get", "session_id": "missing"})).await.unwrap();
    assert!(!missing.success);
    assert_eq!(missing.error.as_deref(), Some("Unknown session 'missing'"));

    let stopped = tool.execute(json!({"action": "stop", "session_id": "running"})).await.unwrap();
    assert!(stopped.success);
    let stopped_body: serde_json::Value = serde_json::from_str(&stopped.output).unwrap();
    assert_eq!(stopped_body["status"], "killed");

    let stopped_again =
        tool.execute(json!({"action": "stop", "session_id": "running"})).await.unwrap();
    assert!(!stopped_again.success);
    assert!(stopped_again.error.unwrap().contains("is not running"));
}

#[tokio::test]
async fn stop_reports_unknown_or_read_only_sessions() {
    let registry = Arc::new(SubAgentRegistry::new());
    insert_session(&registry, "running", SubAgentStatus::Running, None);

    let mut read_only = SecurityPolicy::default();
    read_only.autonomy = AutonomyLevel::ReadOnly;
    let tool = build_tool_with_registry(Arc::clone(&registry), read_only);

    let denied = tool.execute(json!({"action": "stop", "session_id": "running"})).await.unwrap();
    assert!(!denied.success);
    assert!(denied.error.unwrap().contains("read-only mode"));

    let (tool, _) = build_tool();
    let unknown = tool.execute(json!({"action": "stop", "session_id": "missing"})).await.unwrap();
    assert!(!unknown.success);
    assert_eq!(unknown.error.as_deref(), Some("Unknown session 'missing'"));
}
