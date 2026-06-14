use std::path::{Path, PathBuf};

use crate::tools::shell::ast::ParsedCommand;
use crate::tools::shell::path::{PathCheckResult, check_path_constraints, extract_redirect_paths};

#[test]
fn extract_redirect_paths_skips_fd_duplicates_heredocs_and_dev_null() {
    let cmd = crate::tools::shell::ast::parse_command("echo hi > out.txt 2>&1 <<EOF >/dev/null");

    assert_eq!(extract_redirect_paths(&cmd), vec![PathBuf::from("out.txt")]);
}

#[test]
fn fallback_candidate_paths_use_shell_words_tokens() {
    let cmd = ParsedCommand::Fallback {
        raw: "cat README.md".into(),
        tokens: vec!["cat".into(), "README.md".into()],
    };

    assert_eq!(super::extract_candidate_paths(&cmd), vec!["README.md"]);
}

#[test]
fn tilde_user_paths_are_blocked_before_resolution() {
    let cmd = crate::tools::shell::ast::parse_command("cat ~alice/.ssh/config");
    let workspace = PathBuf::from("/workspace");

    assert_eq!(
        check_path_constraints(&cmd, &workspace, &[]),
        PathCheckResult::Blocked {
            path: PathBuf::from("~alice/.ssh/config"),
            reason: "tilde-user paths are not allowed".into(),
        }
    );
}

#[test]
fn resolve_path_expands_workspace_relative_segments() {
    assert_eq!(
        super::resolve_path("./src/../Cargo.toml", Path::new("/workspace/project")),
        Some(PathBuf::from("/workspace/project/Cargo.toml"))
    );
}

#[test]
fn allows_all_paths_only_when_root_is_in_allowlist() {
    assert!(super::allows_all_paths(&[PathBuf::from("/")]));
    assert!(!super::allows_all_paths(&[PathBuf::from("/workspace")]));
}

#[test]
fn home_ssh_directory_is_considered_dangerous() {
    assert!(super::is_dangerous_path(Path::new("/home/alice/.ssh/id_rsa")));
}
