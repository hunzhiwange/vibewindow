#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("worktree_admin_tests"));
}

#[test]
fn valid_branch_name_rejects_empty_and_head() {
    assert!(!super::valid_branch_name(""));
    assert!(!super::valid_branch_name("  "));
    assert!(!super::valid_branch_name("HEAD"));
    assert!(super::valid_branch_name("main"));
    assert!(super::valid_branch_name(" feature/task "));
}

#[test]
fn resolve_merge_target_prefers_task_branch_when_valid() {
    let mut task = crate::app::task::models::Task::new(1);
    task.merge_target_branch = Some(" release ".into());

    assert_eq!(
        super::resolve_merge_target_branch(&task, "/tmp/not-a-repo").as_deref(),
        Some("release")
    );

    task.merge_target_branch = Some("HEAD".into());
    assert_eq!(super::resolve_merge_target_branch(&task, "/tmp/not-a-repo"), None);
}

#[test]
fn claim_worktree_path_is_exclusive_until_released() {
    let path = format!(
        "/tmp/vibewindow-claim-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    );

    assert!(super::claim_worktree_path(&path));
    assert!(!super::claim_worktree_path(&path));
    super::release_claimed_worktree(&path);
    assert!(super::claim_worktree_path(&path));
    super::release_claimed_worktree(&path);
}

#[test]
fn claim_guard_releases_claim_on_drop() {
    let path = format!(
        "/tmp/vibewindow-guard-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    );

    assert!(super::claim_worktree_path(&path));
    {
        let _guard = super::WorktreeClaimGuard::new(Some(path.clone()));
        assert!(!super::claim_worktree_path(&path));
    }
    assert!(super::claim_worktree_path(&path));
    super::release_claimed_worktree(&path);
}

#[test]
fn assign_task_execution_worktree_uses_existing_selected_path_without_git_pool() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let selected = temp.path().join("selected");
    std::fs::create_dir_all(&selected).expect("selected dir");

    let mut task = crate::app::task::models::Task::new(1);
    task.id = "T20260612.0001".into();
    task.selected_worktree_path = Some(selected.to_string_lossy().to_string());
    task.merge_target_branch = Some("main".into());

    let assigned = super::assign_task_execution_worktree("/tmp/not-a-repo", &task, None)
        .expect("assigned selected path");

    let normalized = super::normalize_path(&selected.to_string_lossy());
    assert_eq!(assigned.as_deref(), Some(normalized.as_str()));
}

#[test]
fn resolve_task_execution_workspace_uses_selected_path_and_claims_it() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let selected = temp.path().join("selected");
    std::fs::create_dir_all(&selected).expect("selected dir");
    let selected_string = selected.to_string_lossy().to_string();
    let normalized = super::normalize_path(&selected_string);

    let mut task = crate::app::task::models::Task::new(1);
    task.id = "T20260612.0002".into();
    task.selected_worktree_path = Some(selected_string);
    task.merge_source_branch = Some("feature".into());

    let (workspace, guard) =
        super::resolve_task_execution_workspace(&task, "/tmp/not-a-repo", None).expect("workspace");

    assert_eq!(workspace.slot_id, None);
    assert_eq!(workspace.execution_path, normalized);
    assert_eq!(workspace.selected_worktree_path.as_deref(), Some(normalized.as_str()));
    assert_eq!(workspace.selected_worktree_branch.as_deref(), Some("feature"));
    assert!(!super::claim_worktree_path(&normalized));

    drop(guard);
    assert!(super::claim_worktree_path(&normalized));
    super::release_claimed_worktree(&normalized);
}
