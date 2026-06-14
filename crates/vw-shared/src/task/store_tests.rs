use super::super::models::{
    SubTask, SubTaskStatus, Task, TaskBoardSettings, TaskExecutorBackend, TaskIndex, TaskLogEntry,
    TaskStatus,
};
use super::*;

fn project_path(dir: &tempfile::TempDir) -> String {
    dir.path().to_string_lossy().into_owned()
}

fn task_fixture(
    id: &str,
    status: TaskStatus,
    priority: u32,
    order: u32,
    created_at_ms: u64,
) -> Task {
    Task {
        id: id.to_string(),
        priority,
        assignee: "agent".to_string(),
        model: "model".to_string(),
        executor: TaskExecutorBackend::Codex,
        description: format!("description {id}"),
        prompt: format!("prompt {id}"),
        status,
        created_at_ms,
        updated_at_ms: created_at_ms + 1,
        logs: vec![TaskLogEntry {
            timestamp_ms: created_at_ms,
            status_from: None,
            status_to: None,
            message: "created".to_string(),
        }],
        order,
        deleted: false,
        archived: false,
        subtasks: vec![SubTask {
            id: format!("sub-{id}"),
            content: "subtask".to_string(),
            boundary: "boundary".to_string(),
            acceptance_criteria: vec!["done".to_string()],
            target_files: vec!["src/lib.rs".to_string()],
            created_at_ms,
            order: 0,
            completed: false,
            status: SubTaskStatus::Running,
            execution_started_at_ms: Some(created_at_ms),
            last_execution_duration_ms: None,
        }],
        auto_promote_delay_ms: Some(5_000),
        last_error: Some("previous error".to_string()),
        pause_reason: Some("waiting".to_string()),
        retry_count: 2,
        last_active_at_ms: created_at_ms + 2,
        execution_started_at_ms: Some(created_at_ms + 3),
        last_execution_duration_ms: Some(25),
        merge_source_branch: Some("feature/task".to_string()),
        merge_target_branch: Some("main".to_string()),
        selected_worktree_path: Some("/tmp/worktree".to_string()),
    }
}

fn index_with_task(task: &Task) -> TaskIndex {
    let mut index = TaskIndex::new();
    let status_key = task.status.to_string_key().to_string();
    index.tasks.insert(task.id.clone(), status_key.clone());
    index.order_by_status.entry(status_key).or_default().push(task.id.clone());
    index
}

#[test]
fn task_paths_are_under_project_vibewindow_tasks() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);

    assert_eq!(get_task_root_dir(&project), dir.path().join(".vibewindow/tasks"));
    assert_eq!(get_task_logs_dir(&project), dir.path().join(".vibewindow/tasks/logs"));
    assert_eq!(
        get_index_lock_file_path(&project),
        dir.path().join(".vibewindow/tasks/_index.lock")
    );
    assert_eq!(sanitize_project_path(&project), dir.path());
}

#[test]
fn ensure_task_dir_creates_nested_task_directory() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);

    ensure_task_dir(&project).expect("task dir should be created");

    assert!(dir.path().join(".vibewindow/tasks").is_dir());
}

#[test]
fn project_index_lock_is_reused_per_project() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);

    let first = get_project_index_lock(&project);
    let second = get_project_index_lock(&project);

    assert!(Arc::ptr_eq(&first, &second));
}

#[test]
fn with_index_lock_creates_lock_file_and_returns_closure_value() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);

    let value = with_index_lock(&project, || 42);

    assert_eq!(value, 42);
    assert!(get_index_lock_file_path(&project).is_file());
}

#[test]
fn max_sequence_for_date_ignores_other_dates_and_malformed_ids() {
    let mut index = TaskIndex::new();
    index.tasks.insert("T20260610.0007".to_string(), "pool".to_string());
    index.tasks.insert("T20260610.0012".to_string(), "pool".to_string());
    index.tasks.insert("T20260609.9999".to_string(), "pool".to_string());
    index.tasks.insert("T20260610.bad".to_string(), "pool".to_string());
    index.tasks.insert("not-a-task".to_string(), "pool".to_string());

    assert_eq!(max_sequence_for_date(&index, "20260610"), 12);
    assert_eq!(max_sequence_for_date(&index, "20260611"), 0);
}

#[test]
fn save_and_load_index_round_trips_meta_and_order() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let task_a = task_fixture("task-a", TaskStatus::Pending, 1, 1, 200);
    let task_b = task_fixture("task-b", TaskStatus::Pending, 1, 0, 100);
    save_task(&project, &task_a).expect("task a should save");
    save_task(&project, &task_b).expect("task b should save");
    let mut index = TaskIndex::new();
    index.last_task_date = Some("20260610".to_string());
    index.last_task_seq = 8;

    save_index(&project, &index).expect("index should save");
    let loaded = load_index(&project);

    assert_eq!(loaded.last_task_date.as_deref(), Some("20260610"));
    assert_eq!(loaded.last_task_seq, 8);
    assert_eq!(
        loaded.order_by_status.get("pending"),
        Some(&vec!["task-b".to_string(), "task-a".to_string()])
    );
    assert_eq!(loaded.tasks.get("task-a").map(String::as_str), Some("pending"));
}

#[test]
fn load_index_returns_empty_index_when_database_path_is_blocked_by_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    std::fs::create_dir_all(dir.path().join(".vibewindow/tasks")).expect("task dir");
    std::fs::write(dir.path().join(".vibewindow/tasks/_index.sqlite3"), "not a directory")
        .expect("blocking file");

    let index = load_index(&project);

    assert!(index.tasks.is_empty());
    assert!(index.order_by_status.values().all(Vec::is_empty));
}

#[test]
fn save_and_load_task_round_trips_all_persisted_fields() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let task = task_fixture("task-a", TaskStatus::Running, 3, 4, 100);

    save_task(&project, &task).expect("task should save");
    let loaded = load_task(&project, "task-a").expect("task should load");

    assert_eq!(loaded.id, task.id);
    assert_eq!(loaded.priority, 3);
    assert_eq!(loaded.assignee, "agent");
    assert_eq!(loaded.executor, TaskExecutorBackend::Codex);
    assert_eq!(loaded.status, TaskStatus::Running);
    assert_eq!(loaded.deleted, task.deleted);
    assert_eq!(loaded.archived, task.archived);
    assert_eq!(loaded.auto_promote_delay_ms, Some(5_000));
    assert_eq!(loaded.last_error.as_deref(), Some("previous error"));
    assert_eq!(loaded.pause_reason.as_deref(), Some("waiting"));
    assert_eq!(loaded.retry_count, 2);
    assert_eq!(loaded.execution_started_at_ms, Some(103));
    assert_eq!(loaded.last_execution_duration_ms, Some(25));
    assert_eq!(loaded.merge_source_branch.as_deref(), Some("feature/task"));
    assert_eq!(loaded.merge_target_branch.as_deref(), Some("main"));
    assert_eq!(loaded.selected_worktree_path.as_deref(), Some("/tmp/worktree"));
    assert_eq!(loaded.subtasks[0].status, SubTaskStatus::Running);
    assert_eq!(loaded.logs[0].message, "created");
}

#[test]
fn load_task_returns_none_when_task_is_missing_or_database_cannot_open() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    assert!(load_task(&project, "missing").is_none());

    let blocked_dir = tempfile::tempdir().expect("blocked temp dir");
    let blocked_project = project_path(&blocked_dir);
    std::fs::create_dir_all(blocked_dir.path().join(".vibewindow/tasks"))
        .expect("task dir should create");
    std::fs::write(blocked_dir.path().join(".vibewindow/tasks/_index.sqlite3"), "not a database")
        .expect("blocking file");
    assert!(load_task(&blocked_project, "missing").is_none());
}

#[test]
fn load_task_normalizes_completed_legacy_subtask_status() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let mut task = task_fixture("task-a", TaskStatus::Pool, 1, 0, 100);
    task.subtasks[0].completed = true;
    task.subtasks[0].status = SubTaskStatus::Pending;

    save_task(&project, &task).expect("task should save");
    let loaded = load_task(&project, "task-a").expect("task should load");

    assert_eq!(loaded.subtasks[0].status, SubTaskStatus::Completed);
}

#[test]
fn load_task_falls_back_unknown_executor_and_status() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let task = task_fixture("task-a", TaskStatus::Pool, 1, 0, 100);
    save_task(&project, &task).expect("task should save");
    let conn = open_index_connection(&project).expect("db should open");
    conn.execute(
        "UPDATE tasks SET executor_id = 'unknown', status_key = 'unknown' WHERE id = 'task-a'",
        [],
    )
    .expect("task row should update");

    let loaded = load_task(&project, "task-a").expect("task should load");

    assert_eq!(loaded.executor, TaskExecutorBackend::Internal);
    assert_eq!(loaded.status, TaskStatus::Pool);
}

#[test]
fn load_task_returns_none_for_invalid_subtask_json() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let task = task_fixture("task-a", TaskStatus::Pool, 1, 0, 100);
    save_task(&project, &task).expect("task should save");
    let conn = open_index_connection(&project).expect("db should open");
    conn.execute("UPDATE tasks SET subtasks_json = 'not-json' WHERE id = 'task-a'", [])
        .expect("task row should update");

    assert!(load_task(&project, "task-a").is_none());
}

#[test]
fn load_task_returns_none_for_invalid_logs_json() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let task = task_fixture("task-a", TaskStatus::Pool, 1, 0, 100);
    save_task(&project, &task).expect("task should save");
    let conn = open_index_connection(&project).expect("db should open");
    conn.execute("UPDATE tasks SET logs_json = 'not-json' WHERE id = 'task-a'", [])
        .expect("task row should update");

    assert!(load_task(&project, "task-a").is_none());
}

#[test]
fn create_task_assigns_sequential_id_order_and_updates_index() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let mut existing = TaskIndex::new();
    existing.last_task_date = Some("19000101".to_string());
    existing.last_task_seq = 99;
    existing.tasks.insert("T99991231.0005".to_string(), "pool".to_string());
    save_index(&project, &existing).expect("index should save");

    let first = create_task(&project, task_fixture("ignored", TaskStatus::Pool, 1, 9, 100))
        .expect("first task should create");
    let second = create_task(&project, task_fixture("ignored", TaskStatus::Pool, 2, 9, 200))
        .expect("second task should create");

    assert!(first.id.starts_with('T'));
    assert!(first.id.ends_with(".0001"));
    assert!(second.id.ends_with(".0002"));
    assert_eq!(first.order, 0);
    assert_eq!(second.order, 1);
    let index = load_index(&project);
    assert_eq!(index.tasks.get(&first.id).map(String::as_str), Some("pool"));
    assert_eq!(index.last_task_seq, 2);
    assert!(load_task(&project, &first.id).is_some());
}

#[test]
fn create_task_continues_sequence_from_existing_same_day_task_ids() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let seed = create_task(&project, task_fixture("ignored", TaskStatus::Pool, 1, 0, 100))
        .expect("seed task should create");
    let date = &seed.id[1..9];
    let mut index = load_index(&project);
    index.last_task_date = Some(date.to_string());
    index.last_task_seq = 1;
    let high_id = format!("T{date}.0009");
    let high_task = task_fixture(&high_id, TaskStatus::Pool, 1, 1, 200);
    save_task(&project, &high_task).expect("high task should save");
    index.tasks.insert(high_id.clone(), "pool".to_string());
    index.order_by_status.entry("pool".to_string()).or_default().push(high_id);
    save_index(&project, &index).expect("index should save");

    let task = create_task(&project, task_fixture("ignored", TaskStatus::Pool, 1, 0, 100))
        .expect("task should create");

    assert!(task.id.ends_with(".0010"));
}

#[test]
fn create_task_saturates_sequence_at_u32_max() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let seed = create_task(&project, task_fixture("ignored", TaskStatus::Pool, 1, 0, 100))
        .expect("seed task should create");
    let date = seed.id[1..9].to_string();
    let mut index = TaskIndex::new();
    index.last_task_date = Some(date);
    index.last_task_seq = u32::MAX;
    save_index(&project, &index).expect("index should save");

    let task = create_task(&project, task_fixture("ignored", TaskStatus::Pool, 1, 0, 100))
        .expect("task should create");

    assert!(task.id.ends_with(".4294967295"));
}

#[test]
fn update_task_status_moves_between_status_lists_and_logs_change() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let task = create_task(&project, task_fixture("ignored", TaskStatus::Pool, 1, 0, 100))
        .expect("task should create");

    let updated = update_task_status(&project, &task.id, TaskStatus::Pending)
        .expect("status update should succeed")
        .expect("task should exist");

    assert_eq!(updated.status, TaskStatus::Pending);
    assert_eq!(updated.order, 0);
    assert!(updated.last_error.is_none());
    assert!(updated.pause_reason.is_none());
    assert!(updated.logs.iter().any(|log| log.status_to == Some(TaskStatus::Pending)));
    let index = load_index(&project);
    assert!(!index.order_by_status.get("pool").is_some_and(|ids| ids.contains(&task.id)));
    assert!(index.order_by_status.get("pending").is_some_and(|ids| ids.contains(&task.id)));
}

#[test]
fn update_task_status_returns_existing_task_when_status_is_unchanged() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let task = create_task(&project, task_fixture("ignored", TaskStatus::Pool, 1, 0, 100))
        .expect("task should create");

    let updated = update_task_status(&project, &task.id, TaskStatus::Pool)
        .expect("status update should succeed")
        .expect("task should exist");

    assert_eq!(updated.status, TaskStatus::Pool);
    assert_eq!(updated.logs.len(), task.logs.len());
}

#[test]
fn update_task_status_returns_none_for_missing_task() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);

    let updated = update_task_status(&project, "missing", TaskStatus::Pending)
        .expect("missing update should not error");

    assert!(updated.is_none());
}

#[test]
fn update_task_status_appends_after_existing_tasks_in_new_status() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let pending = create_task(&project, task_fixture("ignored", TaskStatus::Pending, 1, 0, 100))
        .expect("pending task should create");
    let pool = create_task(&project, task_fixture("ignored", TaskStatus::Pool, 1, 0, 200))
        .expect("pool task should create");

    let updated = update_task_status(&project, &pool.id, TaskStatus::Pending)
        .expect("status update should succeed")
        .expect("task should exist");

    assert_eq!(updated.order, 1);
    let index = load_index(&project);
    assert_eq!(index.order_by_status.get("pending"), Some(&vec![pending.id, pool.id]));
}

#[test]
fn load_all_tasks_filters_deleted_tasks_and_ignores_missing_rows() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let visible = task_fixture("visible", TaskStatus::Pool, 1, 0, 100);
    let mut deleted = task_fixture("deleted", TaskStatus::Pool, 1, 1, 200);
    deleted.deleted = true;
    save_task(&project, &visible).expect("visible task should save");
    save_task(&project, &deleted).expect("deleted task should save");
    let mut index = index_with_task(&visible);
    index.tasks.insert(deleted.id.clone(), "pool".to_string());
    index.order_by_status.entry("pool".to_string()).or_default().push(deleted.id.clone());
    index.tasks.insert("missing".to_string(), "pool".to_string());
    index.order_by_status.entry("pool".to_string()).or_default().push("missing".to_string());
    save_index(&project, &index).expect("index should save");

    let tasks = load_all_tasks(&project);

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, "visible");
}

#[test]
#[should_panic(expected = "failed to create task directory before locking index")]
fn load_all_tasks_panics_when_task_directory_cannot_be_created_before_locking() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    std::fs::write(dir.path().join(".vibewindow"), "blocked").expect("blocking file");

    let _ = load_all_tasks(&project);
}

#[test]
fn load_tasks_by_status_initializes_all_statuses_and_sorts_visible_tasks() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let task_a = task_fixture("task-a", TaskStatus::Pending, 5, 0, 300);
    let task_b = task_fixture("task-b", TaskStatus::Pending, 1, 9, 400);
    let task_c = task_fixture("task-c", TaskStatus::Pending, 1, 2, 100);
    let mut task_d = task_fixture("task-d", TaskStatus::Pending, 1, 1, 50);
    task_d.deleted = true;
    for task in [&task_a, &task_b, &task_c, &task_d] {
        save_task(&project, task).expect("task should save");
    }
    let mut index = TaskIndex::new();
    for task in [&task_a, &task_b, &task_c, &task_d] {
        index.tasks.insert(task.id.clone(), task.status.to_string_key().to_string());
        index
            .order_by_status
            .entry(task.status.to_string_key().to_string())
            .or_default()
            .push(task.id.clone());
    }
    save_index(&project, &index).expect("index should save");

    let grouped = load_tasks_by_status(&project);

    assert!(TaskStatus::all().iter().all(|status| grouped.contains_key(status)));
    let pending_ids: Vec<_> =
        grouped[&TaskStatus::Pending].iter().map(|task| task.id.as_str()).collect();
    assert_eq!(pending_ids, vec!["task-c", "task-b", "task-a"]);
}

#[test]
fn delete_task_file_removes_task_and_index_entries() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let task = create_task(&project, task_fixture("ignored", TaskStatus::Pool, 1, 0, 100))
        .expect("task should create");

    delete_task_file(&project, &task.id).expect("task should delete");

    assert!(!task_file_exists(&project, &task.id));
    let index = load_index(&project);
    assert!(!index.tasks.contains_key(&task.id));
    assert!(!index.order_by_status.get("pool").is_some_and(|ids| ids.contains(&task.id)));
}

#[test]
fn delete_task_file_succeeds_for_missing_task() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);

    delete_task_file(&project, "missing").expect("missing delete should succeed");
}

#[test]
fn rebuild_index_from_task_files_preserves_sqlite_index() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let task = create_task(&project, task_fixture("ignored", TaskStatus::Pool, 1, 0, 100))
        .expect("task should create");

    let rebuilt = rebuild_index_from_task_files(&project).expect("index should rebuild");

    assert_eq!(rebuilt.tasks.get(&task.id).map(String::as_str), Some("pool"));
    assert_eq!(load_index(&project).tasks.get(&task.id).map(String::as_str), Some("pool"));
}

#[test]
fn task_file_exists_reflects_sqlite_task_presence() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let task = create_task(&project, task_fixture("ignored", TaskStatus::Pool, 1, 0, 100))
        .expect("task should create");

    assert!(task_file_exists(&project, &task.id));
    assert!(!task_file_exists(&project, "missing"));
}

#[test]
fn board_settings_load_default_when_missing_or_invalid() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);

    assert_eq!(load_task_board_settings(&project).max_concurrent, 3);

    std::fs::create_dir_all(dir.path().join(".vibewindow/tasks")).expect("task dir");
    std::fs::write(dir.path().join(".vibewindow/tasks/board_settings.json"), "not-json")
        .expect("invalid settings");
    assert_eq!(load_task_board_settings(&project).max_concurrent, 3);
}

#[test]
fn board_settings_save_and_load_round_trips_values() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    ensure_task_dir(&project).expect("task dir");
    let settings = TaskBoardSettings {
        auto_execute: false,
        code_review_enabled: true,
        max_concurrent: 7,
        default_priority: 100,
        auto_promote_pool_tasks: false,
        auto_promote_delay_seconds: 90,
        auto_refresh: false,
        refresh_interval_seconds: 12,
        scheduler_tick_interval_seconds: 3,
        auto_promote_tick_interval_seconds: 15,
        failed_retry_minutes: 45,
        running_timeout_minutes: 60,
        recycle_worktree_on_task_finish: true,
        pr_submitted_stall_timeout_seconds: 120,
    };

    save_task_board_settings(&project, &settings).expect("settings should save");
    let loaded = load_task_board_settings(&project);

    assert!(!loaded.auto_execute);
    assert!(loaded.code_review_enabled);
    assert_eq!(loaded.max_concurrent, 7);
    assert_eq!(loaded.pr_submitted_stall_timeout_seconds, 120);
}

#[test]
fn board_settings_save_errors_when_task_directory_is_missing() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);

    let result = save_task_board_settings(&project, &TaskBoardSettings::default());

    assert!(result.is_err());
}

#[test]
fn execution_and_code_review_artifacts_are_isolated_by_kind() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let task = create_task(&project, task_fixture("ignored", TaskStatus::Pool, 1, 0, 100))
        .expect("task should create");
    let execution = serde_json::json!({"ok": true, "items": [1, 2]});
    let review = serde_json::json!({"approved": false, "notes": "needs work"});

    save_task_execution_result(&project, &task.id, &execution).expect("execution should save");
    save_task_code_review_result(&project, &task.id, &review).expect("review should save");

    assert_eq!(
        load_task_execution_result(&project, &task.id).expect("execution should load"),
        Some(execution)
    );
    assert_eq!(
        load_task_code_review_result(&project, &task.id).expect("review should load"),
        Some(review)
    );
    assert_eq!(load_task_execution_result(&project, "missing").expect("missing should load"), None);
}

#[test]
fn artifact_sha_is_stable_for_equal_payloads() {
    let payload = serde_json::json!({"a": 1, "b": ["x", "y"]});

    let first = artifact_sha256_hex(&payload).expect("sha should compute");
    let second = artifact_sha256_hex(&payload).expect("sha should compute");

    assert_eq!(first, second);
    assert_eq!(first.len(), 64);
}

#[test]
fn load_artifact_returns_error_for_invalid_json_payload() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = project_path(&dir);
    let task = create_task(&project, task_fixture("ignored", TaskStatus::Pool, 1, 0, 100))
        .expect("task should create");
    save_task_execution_result(&project, &task.id, &serde_json::json!({"ok": true}))
        .expect("artifact should save");
    let conn = open_index_connection(&project).expect("db should open");
    conn.execute("UPDATE task_artifacts SET payload_json = 'not-json'", [])
        .expect("artifact should update");

    let result = load_task_execution_result(&project, &task.id);

    assert!(result.is_err());
}
