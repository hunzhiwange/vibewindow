use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    AuthPolicy, EXIT_CODE_ERROR, EXIT_CODE_PERMISSION_DENIED, NonInteractivePermissionPolicy,
    OutputFormat, OutputPolicy, PermissionMode, PermissionStats, ResolvedAcpxConfig,
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
fn command_argv_skips_binary_and_script_launcher() {
    assert_eq!(command_argv(&argv(&["vwacp", "status"])), &argv(&["status"])[..]);
    assert_eq!(command_argv(&argv(&["node", "dist/cli.js", "status"])), &argv(&["status"])[..]);
    assert!(command_argv(&argv(&["vwacp"])).is_empty());
}

#[test]
fn bootstrap_plan_detects_queue_owner_cwd_and_flags() {
    let plan = build_cli_bootstrap_plan(
        &argv(&[
            "vwacp",
            "--cwd",
            "sub/../repo",
            "--json-strict",
            "--suppress-reads",
            "--skill=rust",
        ]),
        "/work",
    );

    assert_eq!(plan.perf_capture_role, PerfCaptureRole::Cli);
    assert_eq!(plan.initial_cwd, PathBuf::from("/work/repo"));
    assert!(plan.requested_json_strict);
    assert!(plan.suppress_reads);
    assert!(plan.should_handle_skillflag);
}

#[test]
fn requested_output_format_stops_at_argument_separator() {
    assert_eq!(
        detect_requested_output_format(
            &argv(&["--format", "json", "--", "--format", "quiet"]),
            OutputFormat::Text
        ),
        OutputFormat::Json
    );
    assert!(!detect_json_strict(&argv(&["--", "--json-strict"])));
}

#[test]
fn output_policy_applies_json_strict_and_suppress_reads() {
    let policy = resolve_requested_output_policy(OutputFormat::Json, true, true);

    assert_eq!(
        policy,
        OutputPolicy {
            format: OutputFormat::Json,
            json_strict: true,
            suppress_reads: true,
            suppress_non_json_stderr: true,
            queue_error_already_emitted: true,
            suppress_sdk_console_errors: true,
        }
    );
}

#[test]
fn permission_exit_code_only_changes_when_all_requests_denied_or_cancelled() {
    assert_eq!(
        apply_permission_exit_code(
            EXIT_CODE_ERROR,
            &PermissionStats { requested: 1, approved: 0, denied: 1, cancelled: 0 }
        ),
        EXIT_CODE_PERMISSION_DENIED
    );
    assert_eq!(
        apply_permission_exit_code(
            EXIT_CODE_ERROR,
            &PermissionStats { requested: 1, approved: 1, denied: 1, cancelled: 0 }
        ),
        EXIT_CODE_ERROR
    );
}

#[test]
fn runtime_plan_respects_cli_format_over_config() {
    let plan = build_cli_runtime_plan(&argv(&["--format=json", "status"]), &config());

    assert_eq!(plan.requested_output_format, OutputFormat::Json);
    assert!(plan.public_cli_plan.dynamic_agent_command.is_none());
}
