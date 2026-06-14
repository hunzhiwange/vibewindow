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
fn toggle_board_without_project_keeps_current_screen() {
    let (mut app, _task) = crate::app::App::new();
    app.project_path = None;
    app.screen = crate::app::Screen::Home;

    let _task = super::update(&mut app, super::TaskBoardMessage::ToggleBoard);

    assert_eq!(app.screen, crate::app::Screen::Home);
    assert!(!app.show_task_board);
}

#[test]
fn close_board_without_project_returns_home() {
    let (mut app, _task) = crate::app::App::new();
    app.project_path = None;
    app.screen = crate::app::Screen::TaskBoard;
    app.show_task_board = true;

    let _task = super::update(&mut app, super::TaskBoardMessage::CloseBoard);

    assert_eq!(app.screen, crate::app::Screen::Home);
    assert!(!app.show_task_board);
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

#[test]
fn loaded_tasks_does_not_rebootstrap_running_executor() {
    let (mut app, _task) = crate::app::App::new();
    app.task_board_settings.auto_execute = true;
    app.task_board_executor_running = true;
    app.task_board_next_auto_promote_tick_at_ms = 12_345;

    let _task = super::update(&mut app, super::TaskBoardMessage::TasksLoaded(Vec::new()));

    assert!(app.task_board_executor_running);
    assert_eq!(app.task_board_next_auto_promote_tick_at_ms, 12_345);
}

#[test]
fn stop_execution_disables_auto_scheduling() {
    let (mut app, _task) = crate::app::App::new();
    app.task_board_settings.auto_execute = true;
    app.task_board_settings.auto_promote_pool_tasks = true;
    app.task_board_executor_running = true;

    let _task = super::update(&mut app, super::TaskBoardMessage::StopExecution);

    assert!(!app.task_board_executor_running);
    assert!(!app.task_board_settings.auto_execute);
    assert!(!app.task_board_settings.auto_promote_pool_tasks);
}

#[test]
fn local_pool_scheduler_promotes_ready_pool_tasks_by_priority() {
    use crate::app::task::{Task, TaskStatus};

    let (mut app, _task) = crate::app::App::new();
    app.task_board_settings.auto_execute = true;
    app.task_board_settings.auto_promote_pool_tasks = true;
    app.task_board_settings.max_concurrent = 1;
    app.task_board_settings.auto_promote_delay_seconds = 30;

    let mut slow = Task::new(1);
    slow.id = "slow".to_string();
    slow.status = TaskStatus::Pool;
    slow.priority = 9;
    slow.order = 0;
    slow.created_at_ms = 1_000;

    let mut fast = Task::new(2);
    fast.id = "fast".to_string();
    fast.status = TaskStatus::Pool;
    fast.priority = 1;
    fast.order = 0;
    fast.created_at_ms = 1_000;

    app.task_board_tasks = vec![slow, fast];

    let promote_task_ids = super::local_pool_tasks_to_promote(&app, 31_000);

    assert_eq!(promote_task_ids, vec!["fast", "slow"]);
}

#[test]
fn local_pool_scheduler_respects_flags_and_pending_capacity() {
    use crate::app::task::{Task, TaskStatus};

    let (mut app, _task) = crate::app::App::new();
    app.task_board_settings.auto_execute = true;
    app.task_board_settings.auto_promote_pool_tasks = true;
    app.task_board_settings.max_concurrent = 1;
    app.task_board_settings.auto_promote_delay_seconds = 30;

    let mut pool = Task::new(1);
    pool.id = "pool".to_string();
    pool.status = TaskStatus::Pool;
    pool.created_at_ms = 1_000;

    let mut planning = Task::new(2);
    planning.id = "planning".to_string();
    planning.status = TaskStatus::Planning;

    let mut pending = Task::new(3);
    pending.id = "pending".to_string();
    pending.status = TaskStatus::Pending;

    app.task_board_tasks = vec![pool, planning, pending];

    assert!(super::local_pool_tasks_to_promote(&app, 31_000).is_empty());

    app.task_board_tasks.retain(|task| task.status != TaskStatus::Pending);
    app.task_board_settings.auto_promote_pool_tasks = false;

    assert!(super::local_pool_tasks_to_promote(&app, 31_000).is_empty());
}
