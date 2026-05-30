#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("execution_tests"));
}

#[test]
fn append_task_log_stream_updates_subtask_execution_status() {
    let mut task = crate::app::task::Task::new(1);
    let mut first = crate::app::task::SubTask::new("first".to_string());
    first.id = "SUB-first".to_string();
    let mut second = crate::app::task::SubTask::new("second".to_string());
    second.id = "SUB-second".to_string();
    task.subtasks = vec![first, second];

    let started = crate::app::task::TaskLogStream::SubTaskStarted {
        subtask_id: "SUB-first".to_string(),
        content: "first".to_string(),
    };
    assert!(super::append_task_log_stream(&mut task, &started));
    assert_eq!(task.subtasks[0].status, crate::app::task::SubTaskStatus::Running);
    assert!(!task.subtasks[0].completed);

    let completed =
        crate::app::task::TaskLogStream::SubTaskCompleted { subtask_id: "SUB-first".to_string() };
    assert!(super::append_task_log_stream(&mut task, &completed));
    assert_eq!(task.subtasks[0].status, crate::app::task::SubTaskStatus::Completed);
    assert!(task.subtasks[0].completed);

    let failed = crate::app::task::TaskLogStream::SubTaskFailed {
        subtask_id: "SUB-second".to_string(),
        error: "boom".to_string(),
    };
    assert!(super::append_task_log_stream(&mut task, &failed));
    assert_eq!(task.subtasks[1].status, crate::app::task::SubTaskStatus::Failed);
    assert!(!task.subtasks[1].completed);
}
