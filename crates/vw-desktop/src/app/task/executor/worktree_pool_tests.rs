#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("worktree_pool_tests"));
}

fn unique_path(label: &str) -> String {
    format!(
        "/tmp/vibewindow-{label}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    )
}

fn clear_pool(repo_root: &str) {
    let mut pools = super::worktree_pools().lock().expect("worktree pool lock");
    pools.remove(repo_root);
}

#[test]
fn needs_maintenance_uses_cached_pool_before_git_lookup() {
    let repo_root = unique_path("cached-pool");

    {
        let mut pools = super::worktree_pools().lock().expect("worktree pool lock");
        pools.insert(
            repo_root.clone(),
            super::RepoWorktreePool {
                repo_root: repo_root.clone(),
                base_branch: "main".to_string(),
                slots: Vec::new(),
                task_slots: std::collections::HashMap::new(),
                merge_target_locks: std::collections::HashMap::new(),
                last_synced_at_ms: 0,
            },
        );
    }

    assert!(super::worktree_pool_needs_maintenance(&repo_root, 1));
    assert!(super::worktree_pool_needs_maintenance(&format!("{repo_root}/nested"), 1));

    clear_pool(&repo_root);
}

#[test]
fn parse_worktree_list_handles_branches_detached_heads_and_blank_records() {
    let output = "\
worktree /repo
HEAD abc
branch refs/heads/main

worktree /repo/.vibewindow/task-worktrees/slot-1
HEAD def
branch refs/heads/vw/task/one

worktree /repo/detached
HEAD 123
detached
";

    let entries = super::parse_worktree_list(output);

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].path, "/repo");
    assert_eq!(entries[0].branch.as_deref(), Some("main"));
    assert_eq!(entries[1].branch.as_deref(), Some("vw/task/one"));
    assert_eq!(entries[2].path, "/repo/detached");
    assert_eq!(entries[2].branch, None);
}

#[test]
fn sanitize_branch_token_replaces_unsafe_chars_and_trims_dashes() {
    assert_eq!(super::sanitize_branch_token(" task/ABC.123 "), "task-ABC.123");
    assert_eq!(super::sanitize_branch_token("///"), "");
    assert_eq!(super::sanitize_branch_token("a_b-c.d"), "a_b-c.d");
}

#[test]
fn pool_needs_maintenance_respects_idle_target_and_max_capacity() {
    let repo_root = unique_path("pool-needs");
    let pool = super::RepoWorktreePool {
        repo_root: repo_root.clone(),
        base_branch: "main".into(),
        slots: vec![
            super::WorktreeSlot {
                id: "idle".into(),
                path: format!("{repo_root}/idle"),
                base_branch: "main".into(),
                branch: "vw/task/idle".into(),
                state: super::WorktreeState::Idle,
                leased_task_id: None,
                taint_reason: None,
            },
            super::WorktreeSlot {
                id: "busy".into(),
                path: format!("{repo_root}/busy"),
                base_branch: "main".into(),
                branch: "vw/task/busy".into(),
                state: super::WorktreeState::Busy,
                leased_task_id: Some("task".into()),
                taint_reason: None,
            },
        ],
        task_slots: std::collections::HashMap::new(),
        merge_target_locks: std::collections::HashMap::new(),
        last_synced_at_ms: 0,
    };

    assert!(!super::pool_needs_maintenance(&pool, 1));
    assert!(super::pool_needs_maintenance(&pool, 2));

    let empty = super::RepoWorktreePool { slots: Vec::new(), ..pool };
    assert!(super::pool_needs_maintenance(&empty, 1));

    let full = super::RepoWorktreePool {
        slots: vec![
            super::WorktreeSlot {
                id: "a".into(),
                path: format!("{repo_root}/a"),
                base_branch: "main".into(),
                branch: "a".into(),
                state: super::WorktreeState::Busy,
                leased_task_id: Some("a".into()),
                taint_reason: None,
            },
            super::WorktreeSlot {
                id: "b".into(),
                path: format!("{repo_root}/b"),
                base_branch: "main".into(),
                branch: "b".into(),
                state: super::WorktreeState::Busy,
                leased_task_id: Some("b".into()),
                taint_reason: None,
            },
        ],
        ..empty
    };
    assert!(!super::pool_needs_maintenance(&full, 1));
}

#[test]
fn task_worktree_path_returns_assigned_slot_path_from_cached_pool() {
    let repo_root = super::normalized_repo_root(".").expect("test repository root");
    let slot_path = format!("{repo_root}/slot-1");
    {
        let mut pools = super::worktree_pools().lock().expect("worktree pool lock");
        let mut task_slots = std::collections::HashMap::new();
        task_slots.insert("task-1".into(), "slot-1".into());
        pools.insert(
            repo_root.clone(),
            super::RepoWorktreePool {
                repo_root: repo_root.clone(),
                base_branch: "main".into(),
                slots: vec![super::WorktreeSlot {
                    id: "slot-1".into(),
                    path: slot_path.clone(),
                    base_branch: "main".into(),
                    branch: "vw/task/task-1".into(),
                    state: super::WorktreeState::Busy,
                    leased_task_id: Some("task-1".into()),
                    taint_reason: None,
                }],
                task_slots,
                merge_target_locks: std::collections::HashMap::new(),
                last_synced_at_ms: super::now_ms(),
            },
        );
    }

    assert_eq!(super::task_worktree_path(&repo_root, "task-1"), Some(slot_path));
    assert_eq!(super::task_worktree_path(&repo_root, "missing"), None);
    clear_pool(&repo_root);
}

#[test]
fn merge_dispatch_uses_cached_target_locks() {
    let repo_root = super::normalized_repo_root(".").expect("test repository root");
    {
        let mut pools = super::worktree_pools().lock().expect("worktree pool lock");
        let mut locks = std::collections::HashMap::new();
        locks.insert("main".into(), "holder".into());
        pools.insert(
            repo_root.clone(),
            super::RepoWorktreePool {
                repo_root: repo_root.clone(),
                base_branch: "main".into(),
                slots: Vec::new(),
                task_slots: std::collections::HashMap::new(),
                merge_target_locks: locks,
                last_synced_at_ms: super::now_ms(),
            },
        );
    }

    let mut task = crate::app::task::models::Task::new(1);
    task.id = "holder".into();
    task.merge_target_branch = Some("main".into());
    assert!(super::can_dispatch_merge_task(&repo_root, &task));
    assert_eq!(super::task_merge_lock_holder(&repo_root, &task).as_deref(), Some("holder"));

    task.id = "other".into();
    assert!(!super::can_dispatch_merge_task(&repo_root, &task));
    clear_pool(&repo_root);
}

#[test]
fn worktree_pool_snapshot_counts_and_sorts_cached_slots_for_repo() {
    let repo_root = super::normalized_repo_root(".").expect("test repository root");
    {
        let mut pools = super::worktree_pools().lock().expect("worktree pool lock");
        let mut locks = std::collections::HashMap::new();
        locks.insert("z".into(), "task-z".into());
        locks.insert("a".into(), "task-a".into());
        pools.insert(
            repo_root.clone(),
            super::RepoWorktreePool {
                repo_root: repo_root.clone(),
                base_branch: "main".into(),
                slots: vec![
                    super::WorktreeSlot {
                        id: "b".into(),
                        path: format!("{repo_root}/b"),
                        base_branch: "main".into(),
                        branch: "branch-b".into(),
                        state: super::WorktreeState::Tainted,
                        leased_task_id: None,
                        taint_reason: Some("dirty".into()),
                    },
                    super::WorktreeSlot {
                        id: "a".into(),
                        path: format!("{repo_root}/a"),
                        base_branch: "main".into(),
                        branch: "branch-a".into(),
                        state: super::WorktreeState::Idle,
                        leased_task_id: None,
                        taint_reason: None,
                    },
                    super::WorktreeSlot {
                        id: "c".into(),
                        path: format!("{repo_root}/c"),
                        base_branch: "main".into(),
                        branch: "branch-c".into(),
                        state: super::WorktreeState::Dead,
                        leased_task_id: None,
                        taint_reason: Some("missing".into()),
                    },
                ],
                task_slots: std::collections::HashMap::new(),
                merge_target_locks: locks,
                last_synced_at_ms: super::now_ms(),
            },
        );
    }

    let snapshot = super::worktree_pool_snapshot(".").expect("snapshot");

    assert_eq!(snapshot.repo_root, repo_root);
    assert_eq!(snapshot.idle_count, 1);
    assert_eq!(snapshot.tainted_count, 1);
    assert_eq!(snapshot.dead_count, 1);
    assert_eq!(
        snapshot.merge_target_locks,
        vec![("a".into(), "task-a".into()), ("z".into(), "task-z".into())]
    );
    assert_eq!(
        snapshot.slots.iter().map(|slot| slot.id.as_str()).collect::<Vec<_>>(),
        vec!["a", "b", "c"]
    );
    clear_pool(&repo_root);
}
