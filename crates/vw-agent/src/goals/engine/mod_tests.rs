use super::*;

use tempfile::TempDir;

fn step(id: &str, status: StepStatus, attempts: u32) -> Step {
    Step { id: id.to_string(), description: format!("Step {id}"), status, result: None, attempts }
}

fn goal(id: &str, status: GoalStatus, priority: GoalPriority, steps: Vec<Step>) -> Goal {
    Goal {
        id: id.to_string(),
        description: format!("Goal {id}"),
        status,
        priority,
        created_at: String::new(),
        updated_at: String::new(),
        steps,
        context: String::new(),
        last_error: None,
    }
}

#[test]
fn new_targets_state_goals_json_under_workspace() {
    let tmp = TempDir::new().unwrap();
    let workspace = tmp.path();
    let engine = GoalEngine::new(workspace);

    assert_eq!(engine.state_path, workspace.join("state").join("goals.json"));
}

#[test]
fn goal_state_deserializes_missing_optional_fields_to_defaults() {
    let state: GoalState = serde_json::from_str(
        r#"{
            "goals": [{
                "id": "g1",
                "description": "missing fields",
                "steps": [{
                    "id": "s1",
                    "description": "also missing fields"
                }]
            }]
        }"#,
    )
    .unwrap();

    let goal = &state.goals[0];
    assert_eq!(goal.status, GoalStatus::Pending);
    assert_eq!(goal.priority, GoalPriority::Medium);
    assert!(goal.created_at.is_empty());
    assert!(goal.updated_at.is_empty());
    assert!(goal.context.is_empty());
    assert!(goal.last_error.is_none());

    let step = &goal.steps[0];
    assert_eq!(step.status, StepStatus::Pending);
    assert_eq!(step.attempts, 0);
    assert!(step.result.is_none());
}

#[test]
fn select_next_actionable_prefers_later_higher_priority_goal() {
    let state = GoalState {
        goals: vec![
            goal(
                "low",
                GoalStatus::InProgress,
                GoalPriority::Low,
                vec![step("a", StepStatus::Pending, 0)],
            ),
            goal(
                "critical",
                GoalStatus::InProgress,
                GoalPriority::Critical,
                vec![step("b", StepStatus::Pending, 0)],
            ),
            goal(
                "high",
                GoalStatus::InProgress,
                GoalPriority::High,
                vec![step("c", StepStatus::Pending, 0)],
            ),
        ],
    };

    assert_eq!(GoalEngine::select_next_actionable(&state), Some((1, 0)));
}

#[test]
fn select_next_actionable_skips_pending_steps_at_retry_limit() {
    let state = GoalState {
        goals: vec![goal(
            "g1",
            GoalStatus::InProgress,
            GoalPriority::Critical,
            vec![
                step("exhausted", StepStatus::Pending, GoalEngine::max_step_attempts()),
                step("fresh", StepStatus::Pending, GoalEngine::max_step_attempts() - 1),
            ],
        )],
    };

    assert_eq!(GoalEngine::select_next_actionable(&state), Some((0, 1)));
}

#[test]
fn find_stalled_goals_ignores_non_in_progress_statuses() {
    let state = GoalState {
        goals: vec![
            goal(
                "blocked",
                GoalStatus::Blocked,
                GoalPriority::Critical,
                vec![step("a", StepStatus::Pending, GoalEngine::max_step_attempts())],
            ),
            goal(
                "cancelled",
                GoalStatus::Cancelled,
                GoalPriority::Critical,
                vec![step("b", StepStatus::Completed, 0)],
            ),
            goal(
                "pending",
                GoalStatus::Pending,
                GoalPriority::Critical,
                vec![step("c", StepStatus::Pending, GoalEngine::max_step_attempts())],
            ),
        ],
    };

    assert!(GoalEngine::find_stalled_goals(&state).is_empty());
}

#[test]
fn build_reflection_prompt_marks_in_progress_step_as_pending() {
    let mut goal = goal(
        "g1",
        GoalStatus::InProgress,
        GoalPriority::Medium,
        vec![step("running", StepStatus::InProgress, 1)],
    );
    goal.steps[0].result = Some("still running".to_string());

    let prompt = GoalEngine::build_reflection_prompt(&goal);

    assert!(prompt.contains("[pending] Step running: still running"));
}

#[tokio::test]
async fn save_state_overwrites_existing_goal_file() {
    let tmp = TempDir::new().unwrap();
    let engine = GoalEngine::new(tmp.path());
    let first = GoalState {
        goals: vec![goal(
            "first",
            GoalStatus::InProgress,
            GoalPriority::Low,
            vec![step("a", StepStatus::Pending, 0)],
        )],
    };
    let second = GoalState {
        goals: vec![goal(
            "second",
            GoalStatus::Completed,
            GoalPriority::Critical,
            vec![step("done", StepStatus::Completed, 1)],
        )],
    };

    engine.save_state(&first).await.unwrap();
    engine.save_state(&second).await.unwrap();
    let loaded = engine.load_state().await.unwrap();

    assert_eq!(loaded.goals.len(), 1);
    assert_eq!(loaded.goals[0].id, "second");
    assert_eq!(loaded.goals[0].status, GoalStatus::Completed);
    assert_eq!(loaded.goals[0].priority, GoalPriority::Critical);
}
