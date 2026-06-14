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

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn flush_gateway_output_lines_emits_complete_lines_and_forced_tail() {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut pending = "first\r\nsecond".to_string();

    super::flush_gateway_output_lines(&mut pending, Some(&tx), false);
    assert_eq!(pending, "second");
    super::flush_gateway_output_lines(&mut pending, Some(&tx), true);
    assert!(pending.is_empty());

    let logs = rx.try_iter().collect::<Vec<_>>();
    assert!(matches!(&logs[0], super::TaskLogStream::Stdout(value) if value == "first"));
    assert!(matches!(&logs[1], super::TaskLogStream::Stdout(value) if value == "second"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn flush_gateway_output_lines_skips_blank_lines() {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut pending = "\n  \nvalue\n   ".to_string();

    super::flush_gateway_output_lines(&mut pending, Some(&tx), true);

    let logs = rx.try_iter().collect::<Vec<_>>();
    assert_eq!(logs.len(), 1);
    assert!(matches!(&logs[0], super::TaskLogStream::Stdout(value) if value == "value"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn task_plan_subtask_fallback_uses_default_title_for_empty_prompt() {
    let fallback = super::TaskPlanSubTask::fallback("  ");

    assert_eq!(fallback.title, "完成原始需求");
    assert!(!fallback.boundary.is_empty());
    assert_eq!(fallback.acceptance_criteria.len(), 1);
    assert!(fallback.target_files.is_empty());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn task_plan_paths_use_project_plan_directory() {
    let root = super::task_plan_root_dir("/repo");
    let dir = super::task_plan_dir("/repo", "T1");
    let file = super::task_plan_file_path("/repo", "T1");

    assert!(root.ends_with(".vibewindow/tasks/plan"));
    assert!(dir.ends_with(".vibewindow/tasks/plan/T1"));
    assert!(file.ends_with(".vibewindow/tasks/plan/T1/plan.md"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn sanitize_task_plan_file_part_removes_unsafe_characters() {
    assert_eq!(super::sanitize_task_plan_file_part("SUB.1 abc/中文"), "SUB-1abc");
    assert_eq!(super::sanitize_task_plan_file_part("中文/ /"), "subtask");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn subtask_plan_file_name_uses_one_based_index_and_sanitized_id() {
    let mut subtask = crate::app::task::SubTask::new("Do work".to_string());
    subtask.id = "SUB.001/unsafe".to_string();

    assert_eq!(super::subtask_plan_file_name(0, &subtask), "001-SUB-001unsafe.md");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn format_task_plan_duration_handles_none_seconds_and_minutes() {
    assert_eq!(super::format_task_plan_duration(None), "-");
    assert_eq!(super::format_task_plan_duration(Some(999)), "0s");
    assert_eq!(super::format_task_plan_duration(Some(65_000)), "1m5s");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn subtask_status_label_covers_all_statuses() {
    use crate::app::task::SubTaskStatus;

    assert_eq!(super::subtask_status_label(SubTaskStatus::Pending), "pending");
    assert_eq!(super::subtask_status_label(SubTaskStatus::Running), "running");
    assert_eq!(super::subtask_status_label(SubTaskStatus::Completed), "completed");
    assert_eq!(super::subtask_status_label(SubTaskStatus::Failed), "failed");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn format_markdown_list_uses_empty_text_or_trimmed_items() {
    assert_eq!(super::format_markdown_list(&[], "none"), "- none\n");
    assert_eq!(
        super::format_markdown_list(&[" one ".to_string(), "two".to_string()], "none"),
        "- one\n- two\n"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn build_task_plan_markdown_lists_subtasks_and_metadata() {
    let mut task = crate::app::task::Task::new(1);
    task.id = "T1".to_string();
    task.description = "Test plan".to_string();
    task.prompt = "Original prompt".to_string();
    task.status = crate::app::task::TaskStatus::Planning;
    let mut subtask = crate::app::task::SubTask::new("First".to_string());
    subtask.id = "SUB-one".to_string();
    subtask.boundary = "Only first".to_string();
    subtask.target_files = vec!["src/lib.rs".to_string()];
    task.subtasks = vec![subtask];

    let markdown = super::build_task_plan_markdown(&task);

    assert!(markdown.contains("# T1"));
    assert!(markdown.contains("- 状态: 任务拆分"));
    assert!(markdown.contains("Original prompt"));
    assert!(markdown.contains("[First](001-SUB-one.md)"));
    assert!(markdown.contains("目标文件: src/lib.rs"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn build_subtask_plan_markdown_uses_defaults_for_empty_fields() {
    let mut task = crate::app::task::Task::new(1);
    task.id = "T1".to_string();
    task.prompt = "Prompt".to_string();
    let mut subtask = crate::app::task::SubTask::new("First".to_string());
    subtask.id = "SUB-one".to_string();

    let markdown = super::build_subtask_plan_markdown(&task, &subtask, 0, 1);

    assert!(markdown.contains("# 子任务 001: First"));
    assert!(markdown.contains("- 任务ID: T1"));
    assert!(markdown.contains("完成本子任务标题所描述的范围"));
    assert!(markdown.contains("- 未指定，由执行者按边界判断"));
    assert!(markdown.contains("- 完成后能用本子任务边界中的行为或检查方式验证"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn write_task_plan_files_removes_stale_markdown_files() {
    let temp = tempfile::TempDir::new().expect("temp project should be created");
    let project_path = temp.path().to_string_lossy().to_string();
    let mut task = crate::app::task::Task::new(1);
    task.id = "T-stale".to_string();
    task.prompt = "Prompt".to_string();
    let mut subtask = crate::app::task::SubTask::new("First".to_string());
    subtask.id = "SUB-one".to_string();
    task.subtasks = vec![subtask];
    let task_dir = temp.path().join(".vibewindow/tasks/plan/T-stale");
    std::fs::create_dir_all(&task_dir).expect("task dir should exist");
    std::fs::write(task_dir.join("999-stale.md"), "old").expect("stale md should be written");
    std::fs::write(task_dir.join("notes.txt"), "keep").expect("txt should be written");

    super::write_task_plan_files(&project_path, &task).expect("plan files should write");

    assert!(!task_dir.join("999-stale.md").exists());
    assert!(task_dir.join("notes.txt").exists());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn build_task_split_prompt_includes_task_identity_and_requirements() {
    let mut task = crate::app::task::Task::new(1);
    task.id = "T1".to_string();
    task.description = "Build tests".to_string();
    task.prompt = "Need coverage".to_string();

    let prompt = super::build_task_split_prompt(&task);

    assert!(prompt.contains("只输出 JSON"));
    assert!(prompt.contains("任务ID: T1"));
    assert!(prompt.contains("标题: Build tests"));
    assert!(prompt.contains("Need coverage"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn parse_string_list_accepts_arrays_and_bullet_text() {
    let array = serde_json::json!([" one ", "", 3, "two"]);
    assert_eq!(super::parse_string_list(Some(&array)), vec!["one".to_string(), "two".to_string()]);

    let text = serde_json::json!("- first\n2. second\n* third\n");
    assert_eq!(
        super::parse_string_list(Some(&text)),
        vec!["first".to_string(), "second".to_string(), "third".to_string()]
    );
    assert!(super::parse_string_list(None).is_empty());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn first_string_field_returns_first_non_empty_match() {
    let object = serde_json::json!({
        "title": " ",
        "name": " Name ",
        "other": "Other"
    })
    .as_object()
    .expect("object")
    .clone();

    assert_eq!(super::first_string_field(&object, &["title", "name"]), "Name");
    assert_eq!(super::first_string_field(&object, &["missing"]), "");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn parse_task_plan_item_accepts_string_and_alias_fields() {
    let string_item = serde_json::json!("Simple task");
    assert_eq!(
        super::parse_task_plan_item(&string_item).expect("string item").title,
        "Simple task"
    );

    let object = serde_json::json!({
        "name": "Implement",
        "scope": "Only executor",
        "acceptance": "- passes\n- documented",
        "modified_files": ["src/main.rs"]
    });
    let parsed = super::parse_task_plan_item(&object).expect("object item should parse");
    assert_eq!(parsed.title, "Implement");
    assert_eq!(parsed.boundary, "Only executor");
    assert_eq!(parsed.acceptance_criteria, vec!["passes".to_string(), "documented".to_string()]);
    assert_eq!(parsed.target_files, vec!["src/main.rs".to_string()]);

    assert!(super::parse_task_plan_item(&serde_json::json!({"scope": "missing title"})).is_none());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn parse_task_plan_items_requires_subtasks_array() {
    let value = serde_json::json!({"subtasks": ["A", {"title": "B"}]});
    let parsed = super::parse_task_plan_items(&value);

    assert_eq!(parsed.iter().map(|item| item.title.as_str()).collect::<Vec<_>>(), vec!["A", "B"]);
    assert!(super::parse_task_plan_items(&serde_json::json!({"items": []})).is_empty());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn collect_fenced_json_candidates_extracts_non_empty_fences() {
    let mut candidates = Vec::new();

    super::collect_fenced_json_candidates(
        "before\n```json\n{\"subtasks\":[\"A\"]}\n```\n```\n \n```",
        &mut candidates,
    );

    assert_eq!(candidates, vec!["{\"subtasks\":[\"A\"]}".to_string()]);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn collect_balanced_json_candidates_handles_nested_strings() {
    let mut candidates = Vec::new();

    super::collect_balanced_json_candidates(
        "noise {\"subtasks\":[{\"title\":\"A } still string\"}]} tail [1,{\"a\":2}]",
        &mut candidates,
    );

    assert!(candidates.iter().any(|candidate| candidate.contains("A } still string")));
    assert!(candidates.iter().any(|candidate| candidate == "[1,{\"a\":2}]"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn parse_task_split_output_accepts_fenced_json_and_arrays_inside_text() {
    let fenced = "```json\n{\"subtasks\":[{\"content\":\"Do it\",\"验收条件\":\"- done\",\"需要修改的文件\":\"src/lib.rs\"}]}\n```";
    let parsed = super::parse_task_split_output(fenced, "fallback");

    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].title, "Do it");
    assert_eq!(parsed[0].acceptance_criteria, vec!["done".to_string()]);
    assert_eq!(parsed[0].target_files, vec!["src/lib.rs".to_string()]);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn format_git_summary_and_normalize_gateway_path_are_stable() {
    assert_eq!(
        super::format_git_summary("ok", "skip", "done"),
        "Git动作摘要: add=ok; commit=skip; merge=done"
    );
    assert_eq!(super::normalize_gateway_path("C:\\repo\\worktree\\"), "C:/repo/worktree");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn read_plan_context_falls_back_to_generated_markdown() {
    let temp = tempfile::TempDir::new().expect("temp project should be created");
    let project_path = temp.path().to_string_lossy().to_string();
    let mut task = crate::app::task::Task::new(1);
    task.id = "T-fallback".to_string();
    task.prompt = "Prompt".to_string();
    let mut subtask = crate::app::task::SubTask::new("First".to_string());
    subtask.id = "SUB-one".to_string();
    task.subtasks = vec![subtask.clone()];

    let (plan, subtask_plan) = super::read_plan_context(&project_path, &task, &subtask, 0, 1);

    assert!(plan.contains("# T-fallback"));
    assert!(subtask_plan.contains("# 子任务 001: First"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn persist_subtask_status_updates_matching_subtask_and_writes_plan() {
    let temp = tempfile::TempDir::new().expect("temp project should be created");
    let project_path = temp.path().to_string_lossy().to_string();
    let mut task = crate::app::task::Task::new(1);
    task.id = "T-status".to_string();
    task.prompt = "Prompt".to_string();
    let mut subtask = crate::app::task::SubTask::new("First".to_string());
    subtask.id = "SUB-one".to_string();
    task.subtasks = vec![subtask];

    super::persist_subtask_status(
        &project_path,
        &mut task,
        "SUB-one",
        |subtask| {
            subtask.mark_completed();
        },
        None,
    );

    assert_eq!(task.subtasks[0].status, crate::app::task::SubTaskStatus::Completed);
    assert!(temp.path().join(".vibewindow/tasks/plan/T-status/plan.md").exists());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn build_subtask_execution_prompt_combines_contexts() {
    let mut task = crate::app::task::Task::new(1);
    task.prompt = "Original".to_string();

    let prompt = super::build_subtask_execution_prompt(&task, " plan body ", " subtask body ");

    assert!(prompt.contains("# plan.md\nplan body"));
    assert!(prompt.contains("# 子任务.md\nsubtask body"));
    assert!(prompt.contains("# 原始需求\nOriginal"));
}
