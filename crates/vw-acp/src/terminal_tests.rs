use super::*;

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
    let root =
        std::env::temp_dir().join(format!("vw-acp-terminal-tests-auto-{}", std::process::id()));
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
