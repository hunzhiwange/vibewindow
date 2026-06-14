use super::*;
use git2::Repository;
use tempfile::TempDir;

#[test]
fn vcs_info_default_has_no_branch() {
    let info = Info { branch: None };
    assert!(info.branch.is_none());
}

#[test]
fn current_branch_returns_none_outside_git_repo() {
    let temp = TempDir::new().expect("tempdir should create");
    assert!(current_branch(&temp.path().to_string_lossy()).is_none());
}

#[test]
fn vcs_info_serializes_branch_and_event_type_is_stable() {
    let info = Info { branch: Some("main".to_string()) };
    let value = serde_json::to_value(&info).expect("info should serialize");
    assert_eq!(value["branch"], "main");
    assert_eq!(event::BRANCH_UPDATED.r#type, "vcs.branch.updated");
}

#[test]
fn current_branch_reads_checked_out_git_branch() {
    let temp = TempDir::new().expect("tempdir should create");
    let repo = Repository::init(temp.path()).expect("repo should init");
    let sig = git2::Signature::now("Vibe Window", "vw@example.com").expect("signature");
    let tree_id = {
        let mut index = repo.index().expect("index");
        index.write_tree().expect("tree")
    };
    let tree = repo.find_tree(tree_id).expect("tree should load");
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[]).expect("commit should write");

    let branch = current_branch(&temp.path().to_string_lossy()).expect("branch should read");
    assert!(!branch.is_empty());
    assert_ne!(branch, "HEAD");
}

#[tokio::test]
async fn init_returns_shared_info_with_initial_branch() {
    let temp = TempDir::new().expect("tempdir should create");
    let repo = Repository::init(temp.path()).expect("repo should init");
    let sig = git2::Signature::now("Vibe Window", "vw@example.com").expect("signature");
    let tree_id = {
        let mut index = repo.index().expect("index");
        index.write_tree().expect("tree")
    };
    let tree = repo.find_tree(tree_id).expect("tree should load");
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[]).expect("commit should write");

    let info = init(temp.path().to_path_buf()).await;
    let branch = info.lock().expect("info lock").branch.clone();
    assert!(branch.is_some());
}
