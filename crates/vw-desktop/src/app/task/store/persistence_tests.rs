#[test]
fn persistence_test_module_is_linked() {
    let name = "persistence";
    assert_eq!(name.len(), 11);
}

fn project_path(temp: &tempfile::TempDir) -> String {
    temp.path().to_string_lossy().to_string()
}

fn task_with_id(
    id: &str,
    status: crate::app::task::models::TaskStatus,
) -> crate::app::task::models::Task {
    let mut task = crate::app::task::models::Task::new(3);
    task.id = id.to_string();
    task.status = status;
    task.order = 2;
    task.description = "description".into();
    task.prompt = "prompt".into();
    task.agent = None;
    task.acp_agent = Some("codex".into());
    task.auto_promote_delay_ms = Some(1_500);
    task.last_error = Some("old error".into());
    task.pause_reason = Some("paused".into());
    task.retry_count = 4;
    task.execution_started_at_ms = Some(10);
    task.last_execution_duration_ms = Some(20);
    task.merge_source_branch = Some("feature".into());
    task.merge_target_branch = Some("main".into());
    task.selected_worktree_path = Some("/tmp/worktree".into());
    task.subtasks = vec![crate::app::task::models::SubTask::new("subtask".into())];
    task
}

#[test]
fn max_sequence_for_date_ignores_malformed_ids() {
    let mut index = crate::app::task::models::TaskIndex::new();
    index.tasks.insert("T20260612.0001".into(), "pending".into());
    index.tasks.insert("T20260612.0100".into(), "pending".into());
    index.tasks.insert("T20260612.bad".into(), "pending".into());
    index.tasks.insert("T20260613.9999".into(), "pending".into());

    assert_eq!(super::max_sequence_for_date(&index, "20260612"), 100);
    assert_eq!(super::max_sequence_for_date(&index, "20260101"), 0);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn save_and_load_index_roundtrips_meta_and_orders() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = project_path(&temp);
    let mut index = crate::app::task::models::TaskIndex::new();
    index.last_task_date = Some("20260612".into());
    index.last_task_seq = 9;
    index.tasks.insert("T20260612.0009".into(), "pending".into());
    index.tasks.insert("T20260612.0010".into(), "running".into());
    index.order_by_status.insert("pending".into(), vec!["T20260612.0009".into()]);

    super::save_index(&project, &index).expect("save index");
    let loaded = super::load_index(&project);

    assert_eq!(loaded.last_task_date.as_deref(), Some("20260612"));
    assert_eq!(loaded.last_task_seq, 9);
    assert_eq!(loaded.tasks.get("T20260612.0009").map(String::as_str), Some("pending"));
    assert_eq!(loaded.tasks.get("T20260612.0010").map(String::as_str), Some("running"));
    assert_eq!(
        loaded.order_by_status.get("running").expect("running order"),
        &vec!["T20260612.0010".to_string()]
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn save_and_load_task_roundtrips_logs_and_fields() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = project_path(&temp);
    let mut task = task_with_id("T20260612.0001", crate::app::task::models::TaskStatus::Running);
    task.logs = vec![
        crate::app::task::models::TaskLogEntry::new_message("created".into()),
        crate::app::task::models::TaskLogEntry::new_status_change(
            crate::app::task::models::TaskStatus::Pending,
            crate::app::task::models::TaskStatus::Running,
        ),
    ];

    super::save_task(&project, &task).expect("save task");
    let loaded = super::load_task(&project, &task.id).expect("loaded task");

    assert_eq!(loaded.id, task.id);
    assert_eq!(loaded.status, crate::app::task::models::TaskStatus::Running);
    assert_eq!(loaded.agent.as_deref(), Some(crate::app::task::models::TASK_AGENT_MAIN));
    assert_eq!(loaded.acp_agent.as_deref(), Some("codex"));
    assert_eq!(loaded.auto_promote_delay_ms, Some(1_500));
    assert_eq!(loaded.last_error.as_deref(), Some("old error"));
    assert_eq!(loaded.pause_reason.as_deref(), Some("paused"));
    assert_eq!(loaded.retry_count, 4);
    assert_eq!(loaded.execution_started_at_ms, Some(10));
    assert_eq!(loaded.last_execution_duration_ms, Some(20));
    assert_eq!(loaded.merge_source_branch.as_deref(), Some("feature"));
    assert_eq!(loaded.merge_target_branch.as_deref(), Some("main"));
    assert_eq!(loaded.selected_worktree_path.as_deref(), Some("/tmp/worktree"));
    assert_eq!(loaded.logs.len(), 2);
    assert_eq!(loaded.logs[0].message, "created");
    assert_eq!(loaded.logs[1].status_from, Some(crate::app::task::models::TaskStatus::Pending));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn load_all_tasks_skips_deleted_and_delete_task_removes_index_entry() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = project_path(&temp);
    let live = task_with_id("T20260612.0001", crate::app::task::models::TaskStatus::Pending);
    let mut deleted = task_with_id("T20260612.0002", crate::app::task::models::TaskStatus::Pending);
    deleted.deleted = true;

    super::save_task(&project, &live).expect("save live");
    super::save_task(&project, &deleted).expect("save deleted");
    let mut index = crate::app::task::models::TaskIndex::new();
    index.tasks.insert(live.id.clone(), "pending".into());
    index.tasks.insert(deleted.id.clone(), "pending".into());
    index.order_by_status.insert("pending".into(), vec![live.id.clone(), deleted.id.clone()]);
    super::save_index(&project, &index).expect("save index");

    let all = super::load_all_tasks(&project);
    assert_eq!(all.iter().map(|task| task.id.as_str()).collect::<Vec<_>>(), vec![live.id.as_str()]);

    super::delete_task_file(&project, &live.id).expect("delete task");
    assert!(super::load_task(&project, &live.id).is_none());
    assert!(!super::load_index(&project).tasks.contains_key(&live.id));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn rebuild_index_from_task_files_preserves_sqlite_index() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = project_path(&temp);
    let mut index = crate::app::task::models::TaskIndex::new();
    index.last_task_date = Some("20260612".into());
    index.last_task_seq = 3;
    index.tasks.insert("T20260612.0003".into(), "completed".into());
    index.order_by_status.insert("completed".into(), vec!["T20260612.0003".into()]);

    super::save_index(&project, &index).expect("save index");
    let rebuilt = super::rebuild_index_from_task_files(&project).expect("rebuild index");

    assert_eq!(rebuilt.last_task_date.as_deref(), Some("20260612"));
    assert_eq!(rebuilt.last_task_seq, 3);
    assert_eq!(rebuilt.tasks.get("T20260612.0003").map(String::as_str), Some("completed"));
}
