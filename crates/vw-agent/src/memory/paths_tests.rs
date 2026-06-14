use super::*;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn project_data_dir_is_user_scoped() {
    let workspace = TempDir::new().unwrap();
    let path = project_data_dir(workspace.path()).unwrap();
    let home = directories::UserDirs::new().unwrap().home_dir().to_path_buf();

    assert!(path.starts_with(vw_config_types::paths::home_config_dir(home).join("worktree")));
    assert!(!path.starts_with(workspace.path()));
    assert!(path.components().any(|component| component.as_os_str() == "projects"));
}

#[test]
fn project_and_workspace_data_dirs_use_different_scopes() {
    let workspace = TempDir::new().unwrap();

    let project = project_data_dir(workspace.path()).unwrap();
    let worktree = workspace_data_dir(workspace.path()).unwrap();

    assert!(project.components().any(|component| component.as_os_str() == "projects"));
    assert!(worktree.components().any(|component| component.as_os_str() == "workspaces"));
    assert_ne!(project, worktree);
}

#[test]
fn best_effort_uses_normal_project_dir_when_home_is_available() {
    let workspace = TempDir::new().unwrap();

    assert_eq!(
        project_data_dir_best_effort(workspace.path()),
        project_data_dir(workspace.path()).unwrap()
    );
}

#[test]
fn canonical_path_keeps_absolute_missing_paths_without_fs_errors() {
    let workspace = TempDir::new().unwrap();
    let missing = workspace.path().join("missing").join("child");

    assert_eq!(canonical_path(&missing), missing);
}

#[test]
fn canonical_path_resolves_existing_relative_paths() {
    let expected = std::fs::canonicalize(std::env::current_dir().unwrap()).unwrap();

    assert_eq!(canonical_path(Path::new(".")), expected);
}

#[test]
fn git_common_dir_returns_none_outside_git_repositories() {
    let workspace = TempDir::new().unwrap();

    assert!(git_common_dir(workspace.path()).is_none());
}

#[test]
fn git_common_dir_returns_absolute_common_dir_inside_repo() {
    if !git_is_available() {
        return;
    }

    let workspace = TempDir::new().unwrap();
    assert!(
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(workspace.path())
            .output()
            .unwrap()
            .status
            .success()
    );

    let common_dir = git_common_dir(workspace.path()).unwrap();

    assert!(common_dir.is_absolute());
    assert_eq!(common_dir.file_name().and_then(|name| name.to_str()), Some(".git"));
}

#[test]
fn project_state_id_uses_git_common_dir_when_available() {
    if !git_is_available() {
        return;
    }

    let workspace = TempDir::new().unwrap();
    assert!(
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(workspace.path())
            .output()
            .unwrap()
            .status
            .success()
    );

    let common_dir = git_common_dir(workspace.path()).unwrap();

    assert_eq!(project_state_id(workspace.path()), scoped_id(&common_dir));
}

#[test]
fn workspace_state_id_uses_workspace_path_not_git_common_dir() {
    if !git_is_available() {
        return;
    }

    let workspace = TempDir::new().unwrap();
    assert!(
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(workspace.path())
            .output()
            .unwrap()
            .status
            .success()
    );

    let canonical_workspace = canonical_path(workspace.path());

    assert_eq!(workspace_state_id(workspace.path()), scoped_id(&canonical_workspace));
    assert_ne!(workspace_state_id(workspace.path()), project_state_id(workspace.path()));
}

#[test]
fn scoped_id_is_deterministic_and_keeps_only_a_short_hash_suffix() {
    let workspace = Path::new("/tmp/My Workspace!");
    let id = scoped_id(workspace);

    assert_eq!(id, scoped_id(workspace));
    assert!(id.starts_with("my-workspace-"));
    assert_eq!(id.rsplit_once('-').unwrap().1.len(), 16);
}

#[test]
fn safe_name_replaces_unsafe_chars_and_falls_back_for_empty_names() {
    assert_eq!(safe_name(" Project_01 "), "project_01");
    assert_eq!(safe_name("a.b/c"), "a-b-c");
    assert_eq!(safe_name("..."), "workspace");
}

fn git_is_available() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
