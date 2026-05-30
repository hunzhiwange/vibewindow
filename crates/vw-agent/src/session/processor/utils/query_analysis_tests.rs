use super::should_try_auto_complete_todos;
use crate::app::agent::session::processor::types::ToolSessionState;

#[test]
fn auto_complete_requires_non_todo_tool_activity_and_substantial_answer() {
    let mut tool_state = ToolSessionState::default();
    let answer =
        "Work completed after checking the repository state and applying the requested updates.";

    assert!(!should_try_auto_complete_todos(answer, &tool_state));

    tool_state.non_todo_tool_runs = 1;
    assert!(should_try_auto_complete_todos(answer, &tool_state));
    assert!(!should_try_auto_complete_todos("short answer", &tool_state));
    assert!(!should_try_auto_complete_todos(
        "todo list still needs review after changes",
        &tool_state
    ));
}
