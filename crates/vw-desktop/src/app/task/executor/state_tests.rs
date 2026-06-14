#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("state_tests"));
}

#[test]
fn default_state_uses_expected_limits_and_empty_collections() {
    let state = super::TaskExecutorState::new();

    assert!(state.running_tasks.is_empty());
    assert_eq!(state.max_concurrent, 3);
    assert_eq!(state.simulation_delay_ms, 2000);
    assert!(state.log_receivers.is_empty());
    assert!(state.can_start_more());
}

#[test]
fn start_task_is_idempotent_and_can_start_more_respects_limit() {
    let mut state = super::TaskExecutorState::new();
    state.max_concurrent = 2;

    state.start_task("a");
    state.start_task("a");
    assert_eq!(state.running_tasks, vec!["a".to_string()]);
    assert!(state.is_running("a"));
    assert!(state.can_start_more());

    state.start_task("b");
    assert!(!state.can_start_more());
}

#[test]
fn finish_task_removes_running_entry_and_log_channels() {
    let mut state = super::TaskExecutorState::new();
    state.start_task("a");
    state.register_log_channel("a".to_string());

    assert!(state.get_log_sender("a").is_some());
    state.finish_task("a");

    assert!(!state.is_running("a"));
    assert!(state.get_log_sender("a").is_none());
    assert!(!state.log_receivers.contains_key("a"));
}

#[test]
fn poll_task_logs_drains_registered_channel() {
    let mut state = super::TaskExecutorState::new();
    state.register_log_channel("a".to_string());
    let sender = state.get_log_sender("a").expect("sender should exist");
    sender.send(super::TaskLogStream::Stdout("one".to_string())).expect("stdout should send");
    sender.send(super::TaskLogStream::Stderr("two".to_string())).expect("stderr should send");

    let logs = state.poll_task_logs("a");

    assert_eq!(logs.len(), 2);
    assert!(matches!(&logs[0], super::TaskLogStream::Stdout(value) if value == "one"));
    assert!(matches!(&logs[1], super::TaskLogStream::Stderr(value) if value == "two"));
    assert!(state.poll_task_logs("a").is_empty());
}

#[test]
fn poll_task_logs_for_unknown_task_is_empty() {
    let mut state = super::TaskExecutorState::new();

    assert!(state.poll_task_logs("missing").is_empty());
}

#[test]
fn get_all_running_logs_returns_only_tasks_with_logs() {
    let mut state = super::TaskExecutorState::new();
    state.start_task("a");
    state.start_task("b");
    state.register_log_channel("a".to_string());
    state.register_log_channel("b".to_string());
    state
        .get_log_sender("b")
        .expect("sender should exist")
        .send(super::TaskLogStream::ExitStatus { success: true, code: Some(0), signal: None })
        .expect("exit status should send");

    let logs = state.get_all_running_logs();

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].0, "b");
    assert!(matches!(
        logs[0].1[0],
        super::TaskLogStream::ExitStatus { success: true, code: Some(0), signal: None }
    ));
}

#[test]
fn task_log_stream_variants_keep_payloads() {
    let variants = vec![
        super::TaskLogStream::SubTaskStarted {
            subtask_id: "s1".to_string(),
            content: "do work".to_string(),
        },
        super::TaskLogStream::SubTaskCompleted { subtask_id: "s2".to_string() },
        super::TaskLogStream::SubTaskFailed {
            subtask_id: "s3".to_string(),
            error: "bad".to_string(),
        },
    ];

    assert!(matches!(
        &variants[0],
        super::TaskLogStream::SubTaskStarted { subtask_id, content }
            if subtask_id == "s1" && content == "do work"
    ));
    assert!(matches!(
        &variants[1],
        super::TaskLogStream::SubTaskCompleted { subtask_id } if subtask_id == "s2"
    ));
    assert!(matches!(
        &variants[2],
        super::TaskLogStream::SubTaskFailed { subtask_id, error }
            if subtask_id == "s3" && error == "bad"
    ));
}

#[test]
fn worktree_state_is_copy_and_comparable() {
    let state = super::WorktreeState::Idle;
    let copied = state;

    assert_eq!(copied, super::WorktreeState::Idle);
    assert_ne!(super::WorktreeState::Busy, super::WorktreeState::Dead);
}

#[test]
fn worktree_pool_structs_are_constructible_and_cloneable() {
    let slot = super::WorktreeSlot {
        id: "slot-1".to_string(),
        path: "/repo/.vibewindow/task-worktrees/slot-1".to_string(),
        base_branch: "main".to_string(),
        branch: "vw/task/1".to_string(),
        state: super::WorktreeState::Busy,
        leased_task_id: Some("task-1".to_string()),
        taint_reason: Some("dirty".to_string()),
    };
    let cloned_slot = slot.clone();
    assert_eq!(cloned_slot.id, "slot-1");
    assert_eq!(cloned_slot.base_branch, "main");

    let mut task_slots = std::collections::HashMap::new();
    task_slots.insert("task-1".to_string(), "slot-1".to_string());
    let mut merge_target_locks = std::collections::HashMap::new();
    merge_target_locks.insert("main".to_string(), "task-1".to_string());
    let pool = super::RepoWorktreePool {
        repo_root: "/repo".to_string(),
        base_branch: "main".to_string(),
        slots: vec![slot],
        task_slots,
        merge_target_locks,
        last_synced_at_ms: 42,
    };

    assert_eq!(pool.clone().slots.len(), 1);
    assert_eq!(pool.repo_root, "/repo");
    assert_eq!(pool.base_branch, "main");
    assert_eq!(pool.last_synced_at_ms, 42);
}

#[test]
fn public_worktree_snapshots_keep_counts_and_slots() {
    let slot = super::WorktreeSlotSnapshot {
        id: "slot-1".to_string(),
        path: "/slot".to_string(),
        branch: "vw/task/1".to_string(),
        state: super::WorktreeState::Tainted,
        leased_task_id: None,
        taint_reason: Some("conflict".to_string()),
    };
    let snapshot = super::WorktreePoolSnapshot {
        repo_root: "/repo".to_string(),
        pool_root: "/repo/.vibewindow/task-worktrees".to_string(),
        base_branch: "main".to_string(),
        idle_count: 1,
        busy_count: 2,
        tainted_count: 3,
        recycling_count: 4,
        dead_count: 5,
        merge_target_locks: vec![("main".to_string(), "task-1".to_string())],
        slots: vec![slot],
    };

    assert_eq!(snapshot.repo_root, "/repo");
    assert_eq!(snapshot.pool_root, "/repo/.vibewindow/task-worktrees");
    assert_eq!(snapshot.base_branch, "main");
    assert_eq!(snapshot.idle_count + snapshot.busy_count + snapshot.tainted_count, 6);
    assert_eq!(snapshot.recycling_count, 4);
    assert_eq!(snapshot.dead_count, 5);
    assert_eq!(snapshot.merge_target_locks[0].1, "task-1");
    assert_eq!(snapshot.slots[0].state, super::WorktreeState::Tainted);
    assert_eq!(snapshot.slots[0].id, "slot-1");
    assert_eq!(snapshot.slots[0].path, "/slot");
    assert_eq!(snapshot.slots[0].branch, "vw/task/1");
    assert_eq!(snapshot.slots[0].leased_task_id, None);
    assert_eq!(snapshot.slots[0].taint_reason.as_deref(), Some("conflict"));
}

#[test]
fn selected_workspace_and_worktree_entry_keep_values() {
    let entry = super::WorktreeEntry {
        path: "/repo/worktree".to_string(),
        branch: Some("feature".to_string()),
    };
    assert_eq!(entry.path, "/repo/worktree");
    assert_eq!(entry.branch.as_deref(), Some("feature"));

    let workspace = super::SelectedExecutionWorkspace {
        slot_id: Some("slot-1".to_string()),
        execution_path: "/exec".to_string(),
        selected_worktree_path: Some("/exec".to_string()),
        selected_worktree_branch: Some("feature".to_string()),
        merge_target_branch: Some("main".to_string()),
        project_path: "/repo".to_string(),
    };
    assert_eq!(workspace.slot_id.as_deref(), Some("slot-1"));
    assert_eq!(workspace.execution_path, "/exec");
    assert_eq!(workspace.selected_worktree_path.as_deref(), Some("/exec"));
    assert_eq!(workspace.selected_worktree_branch.as_deref(), Some("feature"));
    assert_eq!(workspace.merge_target_branch.as_deref(), Some("main"));
    assert_eq!(workspace.project_path, "/repo");
}

#[test]
fn global_state_maps_are_initializable() {
    let claimed = super::claimed_worktrees();
    let pools = super::worktree_pools();

    drop(claimed.lock().expect("claimed lock should open"));
    drop(pools.lock().expect("pools lock should open"));
}

#[test]
fn git_state_constants_are_stable() {
    assert_eq!(super::GIT_MAINTENANCE_COMMAND_TIMEOUT_SECS, 30);
    assert_eq!(super::GIT_SUMMARY_TAG, "__VW_GIT_SUMMARY__");
    assert_eq!(super::GIT_SOURCE_BRANCH_TAG, "__VW_GIT_SOURCE_BRANCH__");
    assert_eq!(super::GIT_TARGET_BRANCH_TAG, "__VW_GIT_TARGET_BRANCH__");
    assert_eq!(super::GIT_WORKTREE_PATH_TAG, "__VW_GIT_WORKTREE_PATH__");
    assert_eq!(super::WORKTREE_POOL_REFRESH_INTERVAL_MS, 3_000);
}
