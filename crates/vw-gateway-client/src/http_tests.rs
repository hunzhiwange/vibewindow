#[test]
fn git_command_success_path_is_quiet() {
    assert!(super::is_quiet_success_path("/v1/git/command"));
}

#[test]
fn non_git_command_success_path_is_not_quiet() {
    assert!(!super::is_quiet_success_path("/v1/git/commit"));
    assert!(!super::is_quiet_success_path("/v1/project/worktrees"));
}
