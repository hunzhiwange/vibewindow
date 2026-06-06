#[cfg(not(target_arch = "wasm32"))]
use super::ops::{get_diff_file_metas_for_repo_path, load_diff_content_for_repo_path};

#[test]
fn task_721_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("ops_tests.rs"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn diff_content_uses_head_baseline_when_index_also_changed() {
    let temp = tempfile::tempdir().expect("temp repo");
    let repo = git2::Repository::init(temp.path()).expect("init repo");
    let file_path = temp.path().join("note.md");
    std::fs::write(&file_path, "alpha\nbase\nomega\n").expect("write head file");

    commit_file(&repo, "note.md", "initial");

    std::fs::write(&file_path, "alpha\nstaged\nomega\n").expect("write index file");
    stage_path(&repo, "note.md");

    std::fs::write(&file_path, "alpha\nworktree\nomega\n").expect("write worktree file");

    let metas = get_diff_file_metas_for_repo_path(temp.path().to_str().expect("utf8 path"));
    let meta = metas.iter().find(|meta| meta.path == "note.md").expect("changed file");
    let (old_content, new_content) =
        load_diff_content_for_repo_path(temp.path().to_str().expect("utf8 path"), meta);

    assert_eq!(old_content, "alpha\nbase\nomega\n");
    assert_eq!(new_content, "alpha\nworktree\nomega\n");
}

#[cfg(not(target_arch = "wasm32"))]
fn commit_file(repo: &git2::Repository, path: &str, message: &str) {
    stage_path(repo, path);
    let tree_oid = repo.index().expect("index").write_tree().expect("write tree");
    let tree = repo.find_tree(tree_oid).expect("tree");
    let signature = git2::Signature::now("VibeWindow Test", "test@example.com").expect("signature");
    repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[]).expect("commit");
}

#[cfg(not(target_arch = "wasm32"))]
fn stage_path(repo: &git2::Repository, path: &str) {
    let mut index = repo.index().expect("index");
    index.add_path(std::path::Path::new(path)).expect("add path");
    index.write().expect("write index");
}
