use super::*;

fn todo(id: &str, content: &str, status: &str, priority: &str) -> Todo {
    Todo {
        id: id.to_string(),
        content: content.to_string(),
        status: status.to_string(),
        priority: priority.to_string(),
    }
}

#[test]
fn normalize_trims_deduplicates_and_allocates_ids() {
    let todos = normalize::normalize_todos(vec![
        todo("1", " task ", "pending", ""),
        todo("1", "task", "completed", "high"),
        todo("", "new", " pending ", " medium "),
    ]);

    assert_eq!(todos.len(), 2);
    assert_eq!(todos[0].content, "task");
    assert_eq!(todos[0].status, "completed");
    assert_eq!(todos[0].priority, "high");
    assert_eq!(todos[1].id, "2");
}
