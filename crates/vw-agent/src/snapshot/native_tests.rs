use super::*;
use std::fs;
use std::path::Path;
use std::process::Command;

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn ensure_gitdir_reports_whether_directory_was_created() {
    let dir = tempfile::tempdir().expect("temp dir");
    let git = dir.path().join("snapshot.git");

    assert!(ensure_gitdir(&git).expect("create gitdir"));
    assert!(git.is_dir());
    assert!(!ensure_gitdir(&git).expect("existing gitdir"));
}

#[cfg(not(target_arch = "wasm32"))]
fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(repo)
        .env("GIT_AUTHOR_NAME", "Snapshot Test")
        .env("GIT_AUTHOR_EMAIL", "snapshot@example.com")
        .env("GIT_COMMITTER_NAME", "Snapshot Test")
        .env("GIT_COMMITTER_EMAIL", "snapshot@example.com")
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn temp_git_worktree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init"]);
    git(dir.path(), &["config", "user.name", "Snapshot Test"]);
    git(dir.path(), &["config", "user.email", "snapshot@example.com"]);
    fs::write(dir.path().join("file.txt"), "one\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "init"]);
    dir
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn snapshot_operations_return_empty_outside_git_repo() {
    let dir = tempfile::tempdir().unwrap();

    cleanup(dir.path()).unwrap();
    assert_eq!(track(dir.path()).unwrap(), None);
    assert!(patch(dir.path(), "missing").unwrap().files.is_empty());
    assert_eq!(diff(dir.path(), "missing").unwrap(), "");
    assert!(diff_full(dir.path(), "from", "to").unwrap().is_empty());
    restore(dir.path(), "missing").unwrap();
    revert(dir.path(), &[Patch { hash: "missing".to_string(), files: vec![] }]).unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn track_diff_patch_restore_and_revert_roundtrip() {
    let dir = temp_git_worktree();
    let worktree = dir.path();
    let baseline = track(worktree).unwrap().expect("baseline snapshot");

    fs::write(worktree.join("file.txt"), "two\n").unwrap();
    fs::write(worktree.join("added.txt"), "added\n").unwrap();

    let changed = patch(worktree, &baseline).unwrap();
    assert_eq!(changed.hash, baseline);
    assert!(changed.files.iter().any(|file| file.ends_with("file.txt")));
    assert!(changed.files.iter().any(|file| file.ends_with("added.txt")));

    let diff_text = diff(worktree, &changed.hash).unwrap();
    assert!(diff_text.contains("-one"));
    assert!(diff_text.contains("+two"));
    assert!(diff_text.contains("added.txt"));

    let modified = track(worktree).unwrap().expect("modified snapshot");
    let full = diff_full(worktree, &changed.hash, &modified).unwrap();
    assert!(full.iter().any(|file| {
        file.file == "file.txt"
            && file.before == "one\n"
            && file.after == "two\n"
            && file.status == Some(DiffStatus::Modified)
    }));
    assert!(full.iter().any(|file| {
        file.file == "added.txt"
            && file.after == "added\n"
            && file.status == Some(DiffStatus::Added)
    }));

    revert(worktree, &[changed]).unwrap();
    assert_eq!(fs::read_to_string(worktree.join("file.txt")).unwrap(), "one\n");
    assert!(!worktree.join("added.txt").exists());

    fs::write(worktree.join("file.txt"), "three\n").unwrap();
    restore(worktree, &modified).unwrap();
    assert_eq!(fs::read_to_string(worktree.join("file.txt")).unwrap(), "two\n");
    assert_eq!(fs::read_to_string(worktree.join("added.txt")).unwrap(), "added\n");

    cleanup(worktree).unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn invalid_snapshot_hashes_return_empty_outputs_without_error() {
    let dir = temp_git_worktree();
    let worktree = dir.path();
    assert!(track(worktree).unwrap().is_some());

    assert!(patch(worktree, "not-a-valid-snapshot").unwrap().files.is_empty());
    assert_eq!(diff(worktree, "not-a-valid-snapshot").unwrap(), "");
    assert!(diff_full(worktree, "bad-from", "bad-to").unwrap().is_empty());
    restore(worktree, "not-a-valid-snapshot").unwrap();

    cleanup(worktree).unwrap();
}
