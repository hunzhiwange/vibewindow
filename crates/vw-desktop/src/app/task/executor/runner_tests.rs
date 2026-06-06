#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("runner_tests"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn parse_task_split_output_uses_final_structured_json() {
    let output = r#"
tool read
{"status":"completed","output":"noise"}
{"subtasks":[{"title":"Translate layout docs","boundary":"Only layout files","acceptance_criteria":["No Chinese text remains"],"target_files":["docs/tailwind/current/display.mdx"]}]}
"#;

    let parsed = super::parse_task_split_output(output, "fallback task");

    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].title, "Translate layout docs");
    assert_eq!(parsed[0].boundary, "Only layout files");
    assert_eq!(parsed[0].acceptance_criteria, vec!["No Chinese text remains".to_string()]);
    assert_eq!(parsed[0].target_files, vec!["docs/tailwind/current/display.mdx".to_string()]);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn parse_task_split_output_falls_back_to_single_task_without_json() {
    let output = "<think>Need to inspect files first.</think>\ntool read\nstatus completed";

    let parsed = super::parse_task_split_output(output, "Translate docs");

    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].title, "Translate docs");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn gateway_prompt_options_grants_full_access_for_internal_design_route() {
    let options = super::gateway_prompt_options("/tmp/project", false);

    assert_eq!(options.get("cwd").and_then(serde_json::Value::as_str), Some("/tmp/project"));
    assert_eq!(options.get("full_access").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(options.get("acp_test").and_then(serde_json::Value::as_bool), Some(false));
    assert!(!options.contains_key("acp_permission_mode"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn gateway_prompt_options_uses_approve_all_for_acp_agents() {
    let options = super::gateway_prompt_options("/tmp/project", true);

    assert_eq!(
        options.get("acp_permission_mode").and_then(serde_json::Value::as_str),
        Some("approve-all")
    );
    assert_eq!(
        options.get("acp_force_new_session").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        options.get("acp_history_strategy").and_then(serde_json::Value::as_str),
        Some("discard")
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn write_task_plan_files_uses_task_folder_with_subtask_files() {
    let temp = tempfile::TempDir::new().expect("temp project should be created");
    let project_path = temp.path().to_string_lossy().to_string();
    let mut task = crate::app::task::Task::new(1);
    task.id = "T20260529.0001".to_string();
    task.prompt = "Implement task pool split".to_string();

    let mut first = crate::app::task::SubTask::new("Create plan structure".to_string());
    first.id = "SUB-one".to_string();
    first.boundary = "Only plan file layout".to_string();
    first.acceptance_criteria = vec!["plan.md exists".to_string()];
    first.target_files = vec!["crates/vw-desktop/src/app/task/executor/runner.rs".to_string()];
    let mut second = crate::app::task::SubTask::new("Execute subtask request".to_string());
    second.id = "SUB-two".to_string();
    task.subtasks = vec![first, second];

    super::write_task_plan_files(&project_path, &task).expect("plan files should write");

    let plan_root = temp.path().join(".vibewindow/tasks/plan");
    let task_dir = plan_root.join("T20260529.0001");
    assert!(task_dir.join("plan.md").exists());
    assert!(task_dir.join("001-SUB-one.md").exists());
    assert!(task_dir.join("002-SUB-two.md").exists());
    assert!(!plan_root.join("plan.md").exists());
    assert!(!plan_root.join("T20260529.0001.md").exists());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn format_task_plan_started_at_uses_readable_seconds() {
    assert_eq!(super::format_task_plan_started_at(None), "-");
    assert_eq!(super::format_task_plan_started_at(Some(0)), "1970-01-01 00:00:00");
}
