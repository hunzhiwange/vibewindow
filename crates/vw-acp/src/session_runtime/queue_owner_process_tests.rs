//! 队列所有者进程参数与启动逻辑的单元测试。

use std::collections::HashMap;
use std::path::Path;

use super::queue_owner_process::resolve_queue_owner_spawn_args_with_override;
use crate::session_runtime::{
    QUEUE_OWNER_PROCESS_MARKER, QueueOwnerRuntimeSendOptions, build_queue_owner_arg_override,
    queue_owner_runtime_options_from_send, sanitize_queue_owner_exec_argv,
};
use crate::{AuthPolicy, NonInteractivePermissionPolicy, PermissionMode};

#[test]
fn sanitize_queue_owner_exec_argv_removes_test_and_inspect_flags() {
    let exec_argv = vec![
        "--conditions=dev".to_string(),
        "--test".to_string(),
        "--test-name-pattern".to_string(),
        "queue".to_string(),
        "--inspect".to_string(),
        "127.0.0.1:9229".to_string(),
        "--inspect-port=9333".to_string(),
        "--debug-port".to_string(),
        "9444".to_string(),
        "--loader=tsx".to_string(),
    ];

    let sanitized = sanitize_queue_owner_exec_argv(&exec_argv);

    assert_eq!(sanitized, vec!["--conditions=dev".to_string(), "--loader=tsx".to_string()]);
}

#[test]
fn build_queue_owner_arg_override_returns_none_without_exec_args() {
    let override_args = build_queue_owner_arg_override(Path::new("/tmp/vwacp"), &[]);

    assert_eq!(override_args, None);
}

#[test]
fn build_queue_owner_arg_override_includes_executable_and_marker() {
    let exec_argv = vec!["--conditions=dev".to_string(), "--inspect=127.0.0.1:9229".to_string()];

    let override_args =
        build_queue_owner_arg_override(Path::new("/tmp/vwacp"), &exec_argv).expect("override");

    assert_eq!(override_args, "[\"/tmp/vwacp\",\"--conditions=dev\",\"__queue-owner\"]");
}

#[test]
fn resolve_queue_owner_spawn_args_uses_override_when_present() {
    let args = resolve_queue_owner_spawn_args_with_override(
        Some(Path::new("/tmp/ignored")),
        Some("[\"/tmp/vwacp\",\"--conditions=dev\",\"__queue-owner\"]"),
    )
    .expect("args");

    assert_eq!(
        args,
        vec![
            "/tmp/vwacp".to_string(),
            "--conditions=dev".to_string(),
            QUEUE_OWNER_PROCESS_MARKER.to_string(),
        ]
    );
}

#[test]
fn resolve_queue_owner_spawn_args_rejects_invalid_override() {
    let error =
        resolve_queue_owner_spawn_args_with_override(Some(Path::new("/tmp/ignored")), Some("[]"))
            .expect_err("invalid override should fail");

    assert_eq!(error.to_string(), "vwacp self-spawn failed: invalid VWACP_QUEUE_OWNER_ARGS");
}

#[test]
fn resolve_queue_owner_spawn_args_uses_canonicalized_current_executable() {
    let args =
        resolve_queue_owner_spawn_args_with_override(Some(Path::new(".")), None).expect("args");

    assert_eq!(args.len(), 2);
    assert_eq!(args[1], QUEUE_OWNER_PROCESS_MARKER);
    assert!(Path::new(&args[0]).is_absolute());
}

#[test]
fn queue_owner_runtime_options_from_send_copies_all_fields() {
    let mut auth_credentials = HashMap::new();
    auth_credentials.insert("token".to_string(), "secret".to_string());
    let send_options = QueueOwnerRuntimeSendOptions {
        session_id: "session-1".to_string(),
        mcp_servers: None,
        permission_mode: PermissionMode::ApproveReads,
        non_interactive_permissions: Some(NonInteractivePermissionPolicy::Fail),
        auth_credentials: Some(auth_credentials.clone()),
        auth_policy: Some(AuthPolicy::Fail),
        suppress_sdk_console_errors: Some(true),
        verbose: Some(true),
        ttl_ms: Some(30_000),
        max_queue_depth: Some(8),
        prompt_retries: Some(3),
    };

    let runtime_options = queue_owner_runtime_options_from_send(&send_options);

    assert_eq!(runtime_options.session_id, "session-1");
    assert_eq!(runtime_options.mcp_servers, None);
    assert_eq!(runtime_options.permission_mode, PermissionMode::ApproveReads);
    assert_eq!(
        runtime_options.non_interactive_permissions,
        Some(NonInteractivePermissionPolicy::Fail)
    );
    assert_eq!(runtime_options.auth_credentials, Some(auth_credentials));
    assert_eq!(runtime_options.auth_policy, Some(AuthPolicy::Fail));
    assert_eq!(runtime_options.suppress_sdk_console_errors, Some(true));
    assert_eq!(runtime_options.verbose, Some(true));
    assert_eq!(runtime_options.ttl_ms, Some(30_000));
    assert_eq!(runtime_options.max_queue_depth, Some(8));
    assert_eq!(runtime_options.prompt_retries, Some(3));
}
