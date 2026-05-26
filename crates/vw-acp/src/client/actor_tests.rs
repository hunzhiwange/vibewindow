use std::collections::HashMap;

use crate::{AcpAgentConfig, AgentLifecycleExit};

use super::super::{AcpClient, ChildExitSummary};

fn client(command: &str, args: &[&str]) -> AcpClient {
    AcpClient::new(
        "test-agent",
        AcpAgentConfig {
            command: command.to_string(),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
            env: HashMap::new(),
        },
    )
}

#[test]
fn command_family_detection_uses_basename_and_arguments() {
    assert!(client("/usr/local/bin/gemini", &["--experimental-acp"]).is_gemini_acp_command());
    assert!(!client("gemini", &["chat"]).is_gemini_acp_command());

    assert!(client("claude-agent-acp", &[]).is_claude_acp_command());
    assert!(
        client("npx", &["@agentclientprotocol/claude-agent-acp@^0.26.0"]).is_claude_acp_command()
    );
}

#[test]
fn store_reusable_session_replaces_and_clears_session_id() {
    let client = client("agent", &[]);

    client.store_reusable_session(Some("session-1".to_string()));
    assert_eq!(client.actor_state.lock().reusable_session_id, Some("session-1".to_string()));

    client.store_reusable_session(None);
    assert!(client.actor_state.lock().reusable_session_id.is_none());
}

#[test]
fn record_actor_start_and_exit_update_lifecycle_snapshot() {
    let client = client("agent", &[]);

    client.record_actor_start(Some(10));
    let started = client.get_agent_lifecycle_snapshot();

    assert_eq!(started.pid, Some(10));
    assert!(started.started_at.is_some());
    assert!(started.last_exit.is_none());

    client.record_actor_exit(
        ChildExitSummary { exit_code: Some(2), signal: Some("TERM".to_string()) },
        Some("test_reason"),
        true,
    );
    let exited = client.get_agent_lifecycle_snapshot();
    let exited_at = exited.last_exit.as_ref().and_then(|exit| exit.exited_at.clone());

    assert!(exited.pid.is_none());
    assert_eq!(
        exited.last_exit,
        Some(AgentLifecycleExit {
            exit_code: Some(2),
            signal: Some("TERM".to_string()),
            exited_at,
            reason: Some("test_reason".to_string()),
            unexpected_during_prompt: true,
        })
    );
}
