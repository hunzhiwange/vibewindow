use super::{Status, status_git};
use std::path::Path;
use std::process::Command;

fn git(worktree: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(worktree)
        .status()
        .expect("git command should start");
    assert!(status.success(), "git {:?} failed with {status}", args);
}

fn write(path: impl AsRef<Path>, content: &str) {
    std::fs::write(path, content).expect("file should be written");
}

#[test]
fn status_git_returns_empty_for_non_git_directory() {
    let temp = tempfile::tempdir().expect("temp dir");

    assert!(status_git(temp.path()).is_empty());
}

#[test]
fn status_git_collects_modified_added_and_deleted_files() {
    let temp = tempfile::tempdir().expect("temp dir");
    git(temp.path(), &["init"]);
    git(temp.path(), &["config", "user.email", "unit@example.com"]);
    git(temp.path(), &["config", "user.name", "Unit Test"]);
    git(temp.path(), &["config", "commit.gpgsign", "false"]);
    write(temp.path().join("tracked.txt"), "one\ntwo\n");
    write(temp.path().join("deleted.txt"), "gone\n");
    git(temp.path(), &["add", "."]);
    git(temp.path(), &["commit", "-m", "initial"]);

    write(temp.path().join("tracked.txt"), "one\ntwo\nthree\n");
    std::fs::remove_file(temp.path().join("deleted.txt")).expect("tracked file should delete");
    write(temp.path().join("new.txt"), "alpha\nbeta\n");

    let changed = status_git(temp.path());

    let tracked = changed
        .iter()
        .find(|info| info.path == "tracked.txt" && info.status == Status::Modified)
        .expect("modified tracked file should be reported");
    assert_eq!(tracked.added, 1);
    assert_eq!(tracked.removed, 0);

    let added = changed
        .iter()
        .find(|info| info.path == "new.txt" && info.status == Status::Added)
        .expect("untracked file should be reported");
    assert_eq!(added.added, 2);
    assert_eq!(added.removed, 0);

    assert!(
        changed.iter().any(|info| info.path == "deleted.txt" && info.status == Status::Deleted)
    );
}

#[cfg(unix)]
#[test]
fn status_git_counts_unreadable_untracked_path_as_zero_lines() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("temp dir");
    git(temp.path(), &["init"]);
    symlink("missing-target", temp.path().join("dangling")).expect("dangling symlink");

    let changed = status_git(temp.path());

    let added_path = changed
        .iter()
        .find(|info| info.path == "dangling" && info.status == Status::Added)
        .expect("untracked dangling symlink should be reported by git ls-files");
    assert_eq!(added_path.added, 0);
    assert_eq!(added_path.removed, 0);
}
