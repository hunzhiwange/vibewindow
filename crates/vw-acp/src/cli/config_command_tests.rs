use std::collections::HashMap;

use serde_json::Value;

use crate::cli::flags::GlobalFlags;
use crate::{
    AuthPolicy, InitGlobalConfigFileResult, NonInteractivePermissionPolicy, OutputFormat,
    PermissionMode, ResolvedAcpxConfig,
};

use super::*;

fn config() -> ResolvedAcpxConfig {
    ResolvedAcpxConfig {
        default_agent: "codex".to_string(),
        default_permissions: PermissionMode::ApproveReads,
        non_interactive_permissions: NonInteractivePermissionPolicy::Deny,
        auth_policy: AuthPolicy::Skip,
        ttl_ms: 30_000,
        timeout_ms: Some(1_000),
        queue_max_depth: 8,
        format: OutputFormat::Text,
        agents: HashMap::new(),
        auth: HashMap::new(),
        disable_exec: false,
        mcp_servers: Vec::new(),
        global_path: "/tmp/global.json".to_string(),
        project_path: "/tmp/project.json".to_string(),
        has_global_config: true,
        has_project_config: false,
    }
}

fn flags(format: OutputFormat) -> GlobalFlags {
    GlobalFlags {
        agent: None,
        cwd: "/tmp/repo".to_string(),
        auth_policy: Some(AuthPolicy::Skip),
        non_interactive_permissions: NonInteractivePermissionPolicy::Deny,
        json_strict: false,
        suppress_reads: false,
        timeout: None,
        ttl: 30_000,
        verbose: false,
        format,
        model: None,
        allowed_tools: None,
        max_turns: None,
        prompt_retries: None,
        approve_all: false,
        approve_reads: false,
        deny_all: false,
    }
}

#[test]
fn config_show_payload_includes_paths_and_loaded_flags() {
    let payload = config_show_payload(&config());
    let value = serde_json::to_value(payload).expect("payload json");

    assert_eq!(value["defaultAgent"], "codex");
    assert_eq!(value["paths"]["global"], "/tmp/global.json");
    assert_eq!(value["loaded"]["global"].as_bool(), Some(true));
    assert_eq!(value["loaded"]["project"].as_bool(), Some(false));
}

#[test]
fn config_init_payload_preserves_path_and_created_flag() {
    let payload = config_init_payload(InitGlobalConfigFileResult {
        path: "/tmp/config.json".to_string(),
        created: true,
    });

    assert_eq!(payload.path, "/tmp/config.json");
    assert!(payload.created);
}

#[test]
fn write_config_show_emits_json_for_json_format() {
    let mut output = Vec::new();

    write_config_show(&mut output, &flags(OutputFormat::Json), &config()).expect("write config");
    let value: Value = serde_json::from_slice(&output).expect("json output");

    assert_eq!(value["paths"]["project"], "/tmp/project.json");
    assert_eq!(value["ttl"], 30);
}

#[test]
fn write_config_show_emits_pretty_json_for_text_format() {
    let mut output = Vec::new();

    write_config_show(&mut output, &flags(OutputFormat::Text), &config()).expect("write config");
    let text = String::from_utf8(output).expect("utf8 output");

    assert!(text.contains("\"defaultAgent\": \"codex\""));
    assert!(text.ends_with('\n'));
}

#[test]
fn write_config_init_result_emits_json_for_json_format() {
    let mut output = Vec::new();
    let payload = config_init_payload(InitGlobalConfigFileResult {
        path: "/tmp/config.json".to_string(),
        created: true,
    });

    write_config_init_result(&mut output, &flags(OutputFormat::Json), &payload)
        .expect("write config init");
    let value: Value = serde_json::from_slice(&output).expect("json output");

    assert_eq!(value["path"], "/tmp/config.json");
    assert_eq!(value["created"].as_bool(), Some(true));
}

#[test]
fn write_config_init_result_emits_path_only_for_quiet_format() {
    let mut output = Vec::new();
    let payload = config_init_payload(InitGlobalConfigFileResult {
        path: "/tmp/config.json".to_string(),
        created: true,
    });

    write_config_init_result(&mut output, &flags(OutputFormat::Quiet), &payload)
        .expect("write config init");

    assert_eq!(String::from_utf8(output).expect("utf8 output"), "/tmp/config.json\n");
}

#[test]
fn write_config_init_result_reports_created_config_for_text_format() {
    let mut output = Vec::new();
    let payload = config_init_payload(InitGlobalConfigFileResult {
        path: "/tmp/config.json".to_string(),
        created: true,
    });

    write_config_init_result(&mut output, &flags(OutputFormat::Text), &payload)
        .expect("write config init");

    assert_eq!(String::from_utf8(output).expect("utf8 output"), "Created /tmp/config.json\n");
}

#[test]
fn write_config_init_result_reports_existing_config_for_text_format() {
    let mut output = Vec::new();
    let payload = config_init_payload(InitGlobalConfigFileResult {
        path: "/tmp/config.json".to_string(),
        created: false,
    });

    write_config_init_result(&mut output, &flags(OutputFormat::Text), &payload)
        .expect("write config init");

    assert_eq!(
        String::from_utf8(output).expect("utf8 output"),
        "Config already exists: /tmp/config.json\n"
    );
}

#[test]
fn config_command_variants_are_distinct() {
    assert_eq!(ConfigCommand::Show, ConfigCommand::Show);
    assert_ne!(ConfigCommand::Show, ConfigCommand::Init);
}
