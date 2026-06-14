use super::task::{Task, create_task};

#[test]
fn task_new_uses_stable_bootstrap_defaults() {
    let task = Task::new(10);

    assert_eq!(task.id, "bootstrap-task");
    assert_eq!(task.priority, 10);
    assert_eq!(task.model, "auto");
    assert_eq!(task.agent.as_deref(), Some("main"));
    assert_eq!(task.acp_agent, None);
    assert!(task.prompt.is_empty());
}

#[test]
fn create_task_persists_task_pool_item() {
    let temp = tempfile::tempdir().expect("temp dir");
    let task = Task {
        id: "id-1".to_string(),
        priority: 999,
        model: "model-a".to_string(),
        agent: Some("main".to_string()),
        acp_agent: None,
        prompt: "do work".to_string(),
    };

    let created = create_task(temp.path().to_str().expect("utf-8 temp path"), task.clone())
        .expect("task creation should succeed");

    assert!(created.id.starts_with('T'));
    assert_eq!(created.model, task.model);
    assert_eq!(created.prompt, task.prompt);
}

#[test]
fn config_bridge_sets_and_loads_unique_field() {
    let key = format!("app_tests_bridge_{}", std::process::id());

    super::config::set_config_field(&key, serde_json::json!({"enabled": true}));
    let config = super::config::load_app_config();

    assert_eq!(config[&key]["enabled"], true);
}

#[test]
fn agent_compat_namespace_exposes_selected_runtime_modules() {
    let config = super::agent::config::AutonomyConfig::default();
    let manager = super::agent::approval::ApprovalManager::from_config(&config);

    assert_eq!(config.level, super::agent::security::AutonomyLevel::Supervised);
    assert!(manager.needs_approval("file_write"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn create_task_normalizes_blank_fields_and_allocates_sequences() {
    let temp = tempfile::tempdir().expect("temp dir");

    let first = create_task(
        temp.path().to_str().expect("utf-8 temp path"),
        Task {
            id: "ignored".to_string(),
            priority: 7,
            model: "   ".to_string(),
            agent: Some("   ".to_string()),
            acp_agent: Some(" acp-main ".to_string()),
            prompt: "first".to_string(),
        },
    )
    .expect("first task should be created");
    let second = create_task(
        temp.path().to_str().expect("utf-8 temp path"),
        Task {
            id: "ignored-2".to_string(),
            priority: 8,
            model: " model-b ".to_string(),
            agent: None,
            acp_agent: Some(" ".to_string()),
            prompt: "second".to_string(),
        },
    )
    .expect("second task should be created");

    assert!(first.id.ends_with(".0001"));
    assert!(second.id.ends_with(".0002"));
    assert_eq!(first.model, "auto");
    assert_eq!(first.agent.as_deref(), Some("main"));
    assert_eq!(first.acp_agent.as_deref(), Some("acp-main"));
    assert_eq!(second.model, "model-b");
    assert_eq!(second.agent.as_deref(), Some("main"));
    assert_eq!(second.acp_agent, None);

    let conn = rusqlite::Connection::open(
        temp.path().join(".vibewindow").join("tasks").join("_index.sqlite3"),
    )
    .expect("task db should open");
    let (priority, model, agent, acp_agent, prompt, status_key, order_no): (
        i64,
        String,
        Option<String>,
        Option<String>,
        String,
        String,
        i64,
    ) = conn
        .query_row(
            "SELECT priority, model, agent, acp_agent, prompt, status_key, order_no
             FROM tasks WHERE id = ?1",
            [&first.id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .expect("created task should be persisted");

    assert_eq!(priority, 7);
    assert_eq!(model, "auto");
    assert_eq!(agent.as_deref(), Some("main"));
    assert_eq!(acp_agent.as_deref(), Some("acp-main"));
    assert_eq!(prompt, "first");
    assert_eq!(status_key, "pool");
    assert_eq!(order_no, 0);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn create_task_returns_io_error_when_project_path_is_file() {
    let temp = tempfile::tempdir().expect("temp dir");
    let project_file = temp.path().join("project-file");
    std::fs::write(&project_file, "").expect("project file should be written");

    let err = create_task(project_file.to_str().expect("utf-8 temp path"), Task::new(1))
        .expect_err("task creation should fail when project path is a file");

    assert!(
        matches!(err.kind(), std::io::ErrorKind::AlreadyExists | std::io::ErrorKind::NotADirectory),
        "unexpected error kind: {:?}",
        err.kind()
    );
}
