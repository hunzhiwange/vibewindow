#[test]
fn operations_test_module_is_linked() {
    let name = "operations";
    assert_eq!(name.len(), 10);
}

fn project_path(temp: &tempfile::TempDir) -> String {
    temp.path().to_string_lossy().to_string()
}

fn draft_task(
    priority: u32,
    status: crate::app::task::models::TaskStatus,
) -> crate::app::task::models::Task {
    let mut task = crate::app::task::models::Task::new(priority);
    task.status = status;
    task.prompt = format!("prompt {priority}");
    task.description = format!("description {priority}");
    task
}

#[test]
fn create_task_assigns_dated_id_order_and_index_entry() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = project_path(&temp);

    let first =
        super::create_task(&project, draft_task(2, crate::app::task::models::TaskStatus::Pending))
            .expect("create first");
    let second =
        super::create_task(&project, draft_task(1, crate::app::task::models::TaskStatus::Pending))
            .expect("create second");

    assert!(first.id.starts_with('T'));
    assert!(first.id.contains('.'));
    assert_eq!(first.order, 0);
    assert_eq!(second.order, 1);

    let index = super::load_index(&project);
    assert_eq!(index.tasks.get(&first.id).map(String::as_str), Some("pending"));
    assert_eq!(
        index.order_by_status.get("pending").expect("pending order"),
        &vec![first.id.clone(), second.id.clone()]
    );
}

#[test]
fn load_tasks_by_status_sorts_and_filters_deleted_tasks() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = project_path(&temp);
    let low_priority =
        super::create_task(&project, draft_task(10, crate::app::task::models::TaskStatus::Pending))
            .expect("create low");
    let high_priority =
        super::create_task(&project, draft_task(1, crate::app::task::models::TaskStatus::Pending))
            .expect("create high");
    let mut deleted =
        super::create_task(&project, draft_task(0, crate::app::task::models::TaskStatus::Pending))
            .expect("create deleted");
    deleted.deleted = true;
    super::update_task(&project, &deleted).expect("mark deleted");

    let grouped = super::load_tasks_by_status(&project);
    let pending =
        grouped.get(&crate::app::task::models::TaskStatus::Pending).expect("pending group");

    assert_eq!(
        pending.iter().map(|task| task.id.as_str()).collect::<Vec<_>>(),
        vec![high_priority.id.as_str(), low_priority.id.as_str()]
    );
    assert!(grouped.contains_key(&crate::app::task::models::TaskStatus::Completed));
}

#[test]
fn update_task_status_moves_task_between_status_orders() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = project_path(&temp);
    let task =
        super::create_task(&project, draft_task(5, crate::app::task::models::TaskStatus::Pending))
            .expect("create");

    let unchanged = super::update_task_status(
        &project,
        &task.id,
        crate::app::task::models::TaskStatus::Pending,
    )
    .expect("unchanged")
    .expect("task");
    assert_eq!(unchanged.status, crate::app::task::models::TaskStatus::Pending);

    let moved = super::update_task_status(
        &project,
        &task.id,
        crate::app::task::models::TaskStatus::Running,
    )
    .expect("move")
    .expect("task");
    assert_eq!(moved.status, crate::app::task::models::TaskStatus::Running);
    assert_eq!(moved.order, 0);
    assert!(
        super::load_index(&project).order_by_status.get("pending").expect("pending").is_empty()
    );

    let missing = super::update_task_status(
        &project,
        "missing",
        crate::app::task::models::TaskStatus::Completed,
    )
    .expect("missing");
    assert!(missing.is_none());
}

#[test]
fn soft_delete_archive_and_archive_completed_update_flags_and_logs() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = project_path(&temp);
    let task = super::create_task(
        &project,
        draft_task(5, crate::app::task::models::TaskStatus::Completed),
    )
    .expect("create completed");
    let deleted =
        super::create_task(&project, draft_task(6, crate::app::task::models::TaskStatus::Pending))
            .expect("create deleted");

    let deleted =
        super::soft_delete_task(&project, &deleted.id).expect("soft delete").expect("deleted task");
    assert!(deleted.deleted);
    assert!(deleted.logs.iter().any(|log| log.message == "任务已删除"));

    let archived = super::archive_completed_tasks(&project).expect("archive completed");
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].id, task.id);
    assert!(archived[0].archived);
    assert_eq!(archived[0].status, crate::app::task::models::TaskStatus::Archived);

    assert!(super::archive_task(&project, "missing").expect("missing archive").is_none());
    assert!(super::soft_delete_task(&project, "missing").expect("missing delete").is_none());
}

#[test]
fn reorder_tasks_in_status_rewrites_order_for_existing_tasks() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = project_path(&temp);
    let first =
        super::create_task(&project, draft_task(5, crate::app::task::models::TaskStatus::Pending))
            .expect("create first");
    let second =
        super::create_task(&project, draft_task(5, crate::app::task::models::TaskStatus::Pending))
            .expect("create second");

    super::reorder_tasks_in_status(
        &project,
        crate::app::task::models::TaskStatus::Pending,
        vec![second.id.clone(), first.id.clone(), "missing".into()],
    )
    .expect("reorder");

    let first_loaded = super::load_task(&project, &first.id).expect("first");
    let second_loaded = super::load_task(&project, &second.id).expect("second");
    assert_eq!(second_loaded.order, 0);
    assert_eq!(first_loaded.order, 1);
    assert_eq!(
        super::load_index(&project).order_by_status.get("pending").expect("pending order"),
        &vec![second.id, first.id, "missing".into()]
    );
}
