use super::*;
use agent_client_protocol as acp;
use tokio::sync::mpsc;

fn unique_temp_dir(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("vw-acp-terminal-tests-{label}-{}", std::process::id()))
}

fn approve_all_manager(cwd: PathBuf) -> TerminalManager {
    TerminalManager::new(TerminalManagerOptions {
        cwd,
        permission_mode: PermissionMode::ApproveAll,
        ..TerminalManagerOptions::default()
    })
}

#[test]
fn trim_to_utf8_boundary_keeps_valid_suffix() {
    let text = "aé日";
    let trimmed = trim_to_utf8_boundary(text.as_bytes(), 4);

    assert_eq!(std::str::from_utf8(&trimmed).unwrap(), "日");
}

#[test]
fn to_command_line_renders_quoted_arguments() {
    let args = vec!["hello world".to_string(), "plain".to_string()];

    assert_eq!(to_command_line("echo", &args), r#"echo "hello world" "plain""#);
}

#[tokio::test]
async fn approve_reads_auto_approves_default_execute_inside_cwd() {
    let root = unique_temp_dir("auto");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let manager = TerminalManager::new(TerminalManagerOptions {
        cwd: root.clone(),
        permission_mode: PermissionMode::ApproveReads,
        non_interactive_permissions: Some(NonInteractivePermissionPolicy::Fail),
        ..TerminalManagerOptions::default()
    });

    let approved = manager
        .is_execute_approved(&root, "echo ok")
        .await
        .expect("workspace command should not prompt");

    assert!(approved);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn trim_to_utf8_boundary_handles_zero_limit() {
    assert!(trim_to_utf8_boundary("hello".as_bytes(), 0).is_empty());
}

#[test]
fn trim_to_utf8_boundary_keeps_short_buffer_unchanged() {
    assert_eq!(trim_to_utf8_boundary(b"ok", 8), b"ok");
}

#[test]
fn to_command_line_omits_empty_arguments() {
    assert_eq!(to_command_line("pwd", &[]), "pwd");
}

#[test]
fn terminal_cwd_uses_default_when_request_has_no_cwd() {
    let default_cwd = PathBuf::from("/tmp/vw-acp-terminal-default");
    let record = Map::new();

    let cwd = terminal_cwd(&record, &default_cwd).expect("default cwd should be accepted");

    assert_eq!(cwd, default_cwd);
}

#[test]
fn terminal_cwd_rejects_relative_request_cwd() {
    let mut record = Map::new();
    record.insert("cwd".to_string(), json!("relative"));

    let err = terminal_cwd(&record, Path::new("/tmp")).expect_err("relative cwd should fail");

    assert!(err.to_string().contains("cwd must be absolute"));
}

#[test]
fn normalize_absolute_path_removes_dot_and_parent_components() {
    assert_eq!(
        normalize_absolute_path(Path::new("/tmp/vw-acp/./child/../root")),
        PathBuf::from("/tmp/vw-acp/root")
    );
}

#[test]
fn is_within_root_accepts_root_and_descendant_only() {
    let root = Path::new("/tmp/vw-acp-root");

    assert!(is_within_root(root, Path::new("/tmp/vw-acp-root")));
    assert!(is_within_root(root, Path::new("/tmp/vw-acp-root/child")));
    assert!(!is_within_root(root, Path::new("/tmp/vw-acp-root-sibling")));
}

#[test]
fn env_overrides_ignores_malformed_entries() {
    let mut record = Map::new();
    record.insert(
        "env".to_string(),
        json!([
            { "name": "GOOD", "value": "one" },
            { "name": "MISSING_VALUE" },
            { "value": "missing-name" },
            "plain"
        ]),
    );

    let env = env_overrides(&record);

    assert_eq!(env.len(), 1);
    assert_eq!(env.get("GOOD").map(String::as_str), Some("one"));
}

#[test]
fn output_byte_limit_defaults_and_accepts_custom_values() {
    let empty = Map::new();
    let mut custom = Map::new();
    custom.insert("outputByteLimit".to_string(), json!(7));

    assert_eq!(output_byte_limit(&empty), DEFAULT_TERMINAL_OUTPUT_LIMIT_BYTES);
    assert_eq!(output_byte_limit(&custom), 7);
}

#[test]
fn request_helpers_extract_required_fields() {
    let record = request_record(&json!({ "terminalId": "term-test", "args": ["a", 9, "b"] }))
        .expect("json object should serialize as record");

    assert_eq!(required_string(&record, "terminalId").unwrap(), "term-test");
    assert_eq!(string_array(&record, "args"), vec!["a", "b"]);
    assert!(required_string(&record, "missing").is_err());
}

#[test]
fn terminal_response_helpers_round_trip_protocol_shapes() {
    let create = create_terminal_response("term-test".to_string());
    let output = terminal_output_response(
        "done".to_string(),
        true,
        Some(TerminalExitState { exit_code: Some(3), signal: Some("SIGTERM".to_string()) }),
    );
    let wait =
        wait_for_terminal_exit_response(TerminalExitState { exit_code: Some(4), signal: None });

    assert_eq!(terminal_id_from_response(&create).unwrap(), "term-test");
    assert_eq!(output.output, "done");
    assert!(output.truncated);
    assert_eq!(output.exit_status.as_ref().and_then(|status| status.exit_code), Some(3));
    assert_eq!(wait.exit_status.exit_code, Some(4));
}

#[test]
fn signal_name_maps_known_and_unknown_unix_signals() {
    #[cfg(unix)]
    {
        assert_eq!(signal_name(libc::SIGTERM).as_deref(), Some("SIGTERM"));
        assert_eq!(signal_name(9_999).as_deref(), Some("SIG9999"));
    }
}

#[tokio::test]
async fn managed_terminal_tracks_output_truncation_and_exit_once() {
    let (command_tx, _command_rx) = mpsc::unbounded_channel();
    let terminal = ManagedTerminal::new(5, command_tx);

    terminal.append_output("aé日".as_bytes());
    terminal.record_exit(exit_status_from_failed_wait());
    terminal.record_exit(exit_status_from_failed_wait());
    terminal.clear_output();
    let snapshot = terminal.snapshot();

    assert!(snapshot.truncated);
    assert!(snapshot.output.is_empty());
    assert_eq!(snapshot.exit_status.and_then(|status| status.exit_code), Some(1));
}

#[tokio::test]
async fn create_wait_output_and_release_terminal_successfully() {
    let root = unique_temp_dir("lifecycle");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");
    let manager = approve_all_manager(root.clone());

    let create_response = manager
        .create_terminal(
            &acp::CreateTerminalRequest::new("session-testcov-0072", "sh")
                .args(vec!["-c".to_string(), "printf \"$TESTCOV_VALUE\"".to_string()])
                .env(vec![acp::EnvVariable::new("TESTCOV_VALUE", "terminal-ok")])
                .cwd(root.clone()),
        )
        .await
        .expect("terminal should start");
    let terminal_id = create_response.terminal_id.clone();

    let wait_response = manager
        .wait_for_terminal_exit(&acp::WaitForTerminalExitRequest::new(
            "session-testcov-0072",
            terminal_id.clone(),
        ))
        .await
        .expect("terminal should exit");
    let output_response = manager
        .terminal_output(&acp::TerminalOutputRequest::new(
            "session-testcov-0072",
            terminal_id.clone(),
        ))
        .await
        .expect("terminal output should be available");
    manager
        .release_terminal(&acp::ReleaseTerminalRequest::new(
            "session-testcov-0072",
            terminal_id.clone(),
        ))
        .await
        .expect("release should succeed");
    let err = manager
        .terminal_output(&acp::TerminalOutputRequest::new("session-testcov-0072", terminal_id))
        .await
        .expect_err("released terminal should be unknown");

    assert_eq!(wait_response.exit_status.exit_code, Some(0));
    assert_eq!(output_response.output, "terminal-ok");
    assert!(!output_response.truncated);
    assert_eq!(output_response.exit_status.as_ref().and_then(|status| status.exit_code), Some(0));
    assert!(err.to_string().contains("Unknown terminal"));
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn terminal_output_respects_output_byte_limit() {
    let root = unique_temp_dir("limit");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");
    let manager = approve_all_manager(root.clone());
    let create_response = manager
        .create_terminal(
            &acp::CreateTerminalRequest::new("session-testcov-0072", "sh")
                .args(vec!["-c".to_string(), "printf abcdef".to_string()])
                .output_byte_limit(3)
                .cwd(root.clone()),
        )
        .await
        .expect("terminal should start");
    let terminal_id = create_response.terminal_id.clone();

    let _ = manager
        .wait_for_terminal_exit(&acp::WaitForTerminalExitRequest::new(
            "session-testcov-0072",
            terminal_id.clone(),
        ))
        .await
        .expect("terminal should exit");
    let output_response = manager
        .terminal_output(&acp::TerminalOutputRequest::new(
            "session-testcov-0072",
            terminal_id.clone(),
        ))
        .await
        .expect("terminal output should be available");
    manager
        .release_terminal(&acp::ReleaseTerminalRequest::new("session-testcov-0072", terminal_id))
        .await
        .expect("release should succeed");

    assert_eq!(output_response.output, "def");
    assert!(output_response.truncated);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn kill_terminal_stops_long_running_process() {
    let root = unique_temp_dir("kill");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");
    let manager = TerminalManager::new(TerminalManagerOptions {
        cwd: root.clone(),
        permission_mode: PermissionMode::ApproveAll,
        kill_grace_ms: Some(10),
        ..TerminalManagerOptions::default()
    });
    let create_response = manager
        .create_terminal(
            &acp::CreateTerminalRequest::new("session-testcov-0072", "sh")
                .args(vec!["-c".to_string(), "sleep 30".to_string()])
                .cwd(root.clone()),
        )
        .await
        .expect("terminal should start");
    let terminal_id = create_response.terminal_id.clone();

    manager
        .kill_terminal(&acp::KillTerminalRequest::new("session-testcov-0072", terminal_id.clone()))
        .await
        .expect("kill should succeed");
    let exit_response = manager
        .wait_for_terminal_exit(&acp::WaitForTerminalExitRequest::new(
            "session-testcov-0072",
            terminal_id.clone(),
        ))
        .await
        .expect("terminal should exit after kill");
    manager
        .release_terminal(&acp::ReleaseTerminalRequest::new("session-testcov-0072", terminal_id))
        .await
        .expect("release should succeed");

    assert!(
        exit_response.exit_status.exit_code.is_some() || exit_response.exit_status.signal.is_some()
    );
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn deny_all_rejects_terminal_create() {
    let root = unique_temp_dir("deny");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");
    let manager = TerminalManager::new(TerminalManagerOptions {
        cwd: root.clone(),
        permission_mode: PermissionMode::DenyAll,
        ..TerminalManagerOptions::default()
    });

    let err = manager
        .create_terminal(
            &acp::CreateTerminalRequest::new("session-testcov-0072", "sh")
                .args(vec!["-c".to_string(), "printf denied".to_string()])
                .cwd(root.clone()),
        )
        .await
        .expect_err("deny all should block create");

    assert!(err.to_string().contains("Permission denied"));
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn custom_confirm_execute_controls_approve_reads() {
    let root = unique_temp_dir("confirm");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");
    let manager = TerminalManager::new(TerminalManagerOptions {
        cwd: root.clone(),
        permission_mode: PermissionMode::ApproveReads,
        confirm_execute: Some(Arc::new(|command_line| {
            Box::pin(async move { Ok(command_line.contains("allowed")) })
        })),
        ..TerminalManagerOptions::default()
    });

    assert!(manager.is_execute_approved(&root, "echo allowed").await.expect("confirm should run"));
    assert!(!manager.is_execute_approved(&root, "echo denied").await.expect("confirm should run"));
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn terminal_manager_emits_running_and_completed_operations() {
    let root = unique_temp_dir("ops");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");
    let operations = Arc::new(Mutex::new(Vec::new()));
    let captured = operations.clone();
    let manager = TerminalManager::new(TerminalManagerOptions {
        cwd: root.clone(),
        permission_mode: PermissionMode::ApproveAll,
        on_operation: Some(Arc::new(move |operation| {
            captured.lock().push(operation);
        })),
        ..TerminalManagerOptions::default()
    });

    let create_response = manager
        .create_terminal(
            &acp::CreateTerminalRequest::new("session-testcov-0072", "sh")
                .args(vec!["-c".to_string(), "printf ops".to_string()])
                .cwd(root.clone()),
        )
        .await
        .expect("terminal should start");
    let terminal_id = create_response.terminal_id.clone();
    let _ = manager
        .wait_for_terminal_exit(&acp::WaitForTerminalExitRequest::new(
            "session-testcov-0072",
            terminal_id.clone(),
        ))
        .await
        .expect("terminal should exit");
    let _ = manager
        .terminal_output(&acp::TerminalOutputRequest::new(
            "session-testcov-0072",
            terminal_id.clone(),
        ))
        .await
        .expect("terminal output should be available");
    manager
        .release_terminal(&acp::ReleaseTerminalRequest::new("session-testcov-0072", terminal_id))
        .await
        .expect("release should succeed");

    let operations = operations.lock();
    assert!(operations.iter().any(|operation| {
        operation.method == ClientOperationMethod::TerminalCreate
            && operation.status == ClientOperationStatus::Running
    }));
    assert!(operations.iter().any(|operation| {
        operation.method == ClientOperationMethod::TerminalCreate
            && operation.status == ClientOperationStatus::Completed
            && operation
                .details
                .as_deref()
                .is_some_and(|details| details.contains("terminalId=term_"))
    }));
    assert!(operations.iter().any(|operation| {
        operation.method == ClientOperationMethod::TerminalOutput
            && operation.status == ClientOperationStatus::Completed
    }));
    let _ = std::fs::remove_dir_all(root);
}
