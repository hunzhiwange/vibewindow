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
