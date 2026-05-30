#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("update_tests"));
}

#[test]
fn execution_tick_dispatches_merge_when_no_pending_task() {
    use crate::app::task::{Task, TaskStatus};

    let (mut app, _task) = crate::app::App::new();
    app.project_path = Some("/tmp/vibewindow-merge-dispatch-test".to_string());
    app.task_board_executor_running = true;
    app.task_board_settings.max_concurrent = 3;

    let mut task = Task::new(1);
    task.id = "T-merge-only".to_string();
    task.status = TaskStatus::PrSubmitted;
    task.merge_source_branch = Some("vw/task/T-merge-only".to_string());
    task.merge_target_branch = Some("main".to_string());
    app.task_board_tasks = vec![task];

    let _task = super::update(&mut app, super::TaskBoardMessage::ExecutionTick);

    let task = app
        .task_board_tasks
        .iter()
        .find(|task| task.id == "T-merge-only")
        .expect("merge task should remain in task board");
    assert!(app.task_board_executor.running_tasks.contains(&"T-merge-only".to_string()));
    assert!(task.logs.iter().any(|entry| entry.message == "开始执行合并任务"));
}

#[test]
fn loaded_tasks_preserve_running_subtask_state() {
    use crate::app::task::{SubTask, SubTaskStatus, Task, TaskStatus};

    let mut current = Task::new(1);
    current.id = "T-running".to_string();
    current.status = TaskStatus::Running;
    let mut current_subtask = SubTask::new("live".to_string());
    current_subtask.id = "SUB-live".to_string();
    current_subtask.start_execution();
    current.subtasks = vec![current_subtask];

    let mut loaded = current.clone();
    loaded.subtasks[0].status = SubTaskStatus::Pending;
    loaded.subtasks[0].completed = false;
    loaded.subtasks[0].execution_started_at_ms = None;

    let merged = super::merge_loaded_tasks_preserving_running_state(
        vec![loaded],
        &[current],
        &["T-running".to_string()],
    );

    assert_eq!(merged[0].subtasks[0].status, SubTaskStatus::Running);
    assert!(merged[0].subtasks[0].execution_started_at_ms.is_some());
}
