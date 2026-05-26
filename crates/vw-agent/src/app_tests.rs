use super::task::{Task, create_task};

#[test]
fn task_new_uses_stable_bootstrap_defaults() {
    let task = Task::new(10);

    assert_eq!(task.id, "bootstrap-task");
    assert_eq!(task.model, "auto");
    assert!(task.prompt.is_empty());
}

#[test]
fn create_task_returns_the_supplied_task() {
    let task = Task {
        id: "id-1".to_string(),
        model: "model-a".to_string(),
        prompt: "do work".to_string(),
    };

    let created = create_task("/tmp/project", task.clone()).expect("task creation should succeed");

    assert_eq!(created.id, task.id);
    assert_eq!(created.model, task.model);
    assert_eq!(created.prompt, task.prompt);
}
