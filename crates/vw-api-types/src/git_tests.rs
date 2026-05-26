use crate::git::{GitDiffRequest, GitDiffScope, GitFileStatus, GitStatusDto};
use serde_json::json;

#[test]
fn git_dtos_use_stable_wire_names_and_empty_lists() {
    assert_eq!(
        serde_json::to_value(GitFileStatus::TypeChanged).expect("serialize"),
        json!("type_changed")
    );

    let status: GitStatusDto =
        serde_json::from_value(json!({ "branch": "main" })).expect("valid status");
    assert!(status.staged.is_empty());
    assert!(status.unstaged.is_empty());
    assert!(status.untracked.is_empty());

    let diff: GitDiffRequest = serde_json::from_value(json!({
        "project_id": "p1",
        "scope": "working_tree"
    }))
    .expect("valid diff request");
    assert_eq!(diff.scope, GitDiffScope::WorkingTree);
    assert_eq!(diff.worktree_id, None);
}
