use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::{
    AuthPolicy, NonInteractivePermissionPolicy, OutputFormat, PermissionMode, ResolvedAcpxConfig,
};

use super::*;

fn argv(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn config() -> ResolvedAcpxConfig {
    ResolvedAcpxConfig {
        default_agent: "codex".to_string(),
        default_permissions: PermissionMode::ApproveReads,
        non_interactive_permissions: NonInteractivePermissionPolicy::Deny,
        auth_policy: AuthPolicy::Skip,
        ttl_ms: 30_000,
        timeout_ms: None,
        queue_max_depth: 16,
        format: OutputFormat::Text,
        agents: HashMap::new(),
        auth: HashMap::new(),
        disable_exec: false,
        mcp_servers: Vec::new(),
        global_path: "/tmp/global.json".to_string(),
        project_path: "/tmp/project.json".to_string(),
        has_global_config: false,
        has_project_config: false,
    }
}

#[test]
fn detect_agent_token_skips_global_options() {
    let scan = detect_agent_token(&argv(&[
        "--cwd",
        "/tmp/repo",
        "--format=json",
        "--approve-reads",
        "custom-agent",
        "prompt",
    ]));

    assert_eq!(scan.token, Some("custom-agent".to_string()));
    assert!(!scan.has_agent_override);
}

#[test]
fn detect_agent_token_records_agent_override_without_dynamic_command() {
    let scan = detect_agent_token(&argv(&["--agent", "./server", "prompt"]));

    assert_eq!(scan.token, Some("prompt".to_string()));
    assert!(scan.has_agent_override);
    assert!(resolve_dynamic_agent_command(&scan, &[], &HashSet::new()).is_none());
}

#[test]
fn resolve_dynamic_agent_command_ignores_top_level_verbs() {
    let verbs = HashSet::from(["status".to_string()]);

    assert_eq!(
        resolve_dynamic_agent_command(
            &AgentTokenScan { token: Some("my-agent".to_string()), has_agent_override: false },
            &[],
            &verbs,
        ),
        Some("my-agent".to_string())
    );
    assert_eq!(
        resolve_dynamic_agent_command(
            &AgentTokenScan { token: Some("status".to_string()), has_agent_override: false },
            &[],
            &verbs,
        ),
        None
    );
}

#[test]
fn root_prompt_action_shows_help_or_errors_for_empty_tty_prompt() {
    assert_eq!(
        resolve_root_prompt_action(&[], true, false).expect("help action"),
        RootPromptAction::ShowHelp
    );
    assert!(resolve_root_prompt_action(&[], true, true).is_err());
    assert_eq!(
        resolve_root_prompt_action(&["hello".to_string()], true, true).expect("prompt action"),
        RootPromptAction::HandlePrompt
    );
}

#[test]
fn configure_public_cli_invokes_registration_callbacks_in_order() {
    let calls = RefCell::new(Vec::new());
    let mut program = ();
    let plan = configure_public_cli(ConfigurePublicCliOptions {
        program: &mut program,
        argv: &argv(&["new-agent", "hello"]),
        config: &config(),
        requested_json_strict: true,
        top_level_verbs: &HashSet::from(["status".to_string()]),
        register_agent_command: |_, name, _| calls.borrow_mut().push(format!("agent:{name}")),
        register_default_commands: |_, _| calls.borrow_mut().push("defaults".to_string()),
        register_root_prompt: |_, strict| calls.borrow_mut().push(format!("root:{strict}")),
        add_help_text: |_, text| {
            calls.borrow_mut().push(format!("help:{}", text.contains("Examples:")))
        },
    });
    let calls = calls.into_inner();

    assert_eq!(plan.dynamic_agent_command, Some("new-agent".to_string()));
    assert!(calls.iter().any(|call| call == "agent:new-agent"));
    assert_eq!(calls[calls.len() - 2], "root:true");
    assert_eq!(calls[calls.len() - 1], "help:true");
}
