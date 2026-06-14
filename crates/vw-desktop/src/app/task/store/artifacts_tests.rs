#[cfg(not(target_arch = "wasm32"))]
#[test]
fn checksum_hex_is_stable_sha256_hex() {
    assert_eq!(
        super::checksum_hex("abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

fn sample_task() -> crate::app::task::models::Task {
    let mut task = crate::app::task::models::Task::new(5);
    task.id = "T20260612.0001".into();
    task.model = "gpt-test".into();
    task.acp_agent = Some("codex".into());
    task.prompt = "implement feature".into();
    task.merge_source_branch = Some("feature/task".into());
    task.merge_target_branch = Some("main".into());
    task
}

#[test]
fn execution_result_log_writes_success_and_error_content() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = temp.path().to_string_lossy().to_string();
    let task = sample_task();

    let success_path = super::write_task_execution_result_log(&project, &task, &Ok("done".into()))
        .expect("success log");
    let success = std::fs::read_to_string(&success_path).expect("success content");
    assert!(success_path.ends_with("[T20260612.0001].log"));
    assert!(success.contains("task_id=T20260612.0001"));
    assert!(success.contains("acp_agent=codex"));
    assert!(success.contains("result=success"));
    assert!(success.contains("output:\ndone"));

    let error_path = super::write_task_execution_result_log(&project, &task, &Err("boom".into()))
        .expect("error log");
    let error = std::fs::read_to_string(error_path).expect("error content");
    assert!(error.contains("result=error"));
    assert!(error.contains("error:\nboom"));
}

#[test]
fn code_review_log_includes_branches_and_optional_system_prompt() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = temp.path().to_string_lossy().to_string();
    let task = sample_task();

    let path = super::write_task_code_review_result_log(
        &project,
        &task,
        &Ok("looks good".into()),
        Some("full system prompt"),
    )
    .expect("review log");

    let content = std::fs::read_to_string(path).expect("review content");
    assert!(content.contains("source_branch=feature/task"));
    assert!(content.contains("target_branch=main"));
    assert!(content.contains("review_system_prompt_full:\nfull system prompt"));
    assert!(content.contains("review_result=success"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn raw_artifact_saves_execution_result_metadata_to_sqlite() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = temp.path().to_string_lossy().to_string();
    let task = sample_task();
    let file_path = temp.path().join("result.txt");

    super::save_task_execution_result_artifact(
        &project,
        &task,
        &Err("failed output".into()),
        Some(&file_path),
    )
    .expect("artifact save");

    let conn = super::open_index_connection(&project).expect("sqlite connection");
    let row = conn
        .query_row(
            "SELECT artifact_type, acp_agent, model, file_path, content_text, content_sha256, status
             FROM task_raw_artifacts
             WHERE task_id = ?1",
            rusqlite::params![task.id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            },
        )
        .expect("artifact row");

    assert_eq!(row.0, "execution_result");
    assert_eq!(row.1, "codex");
    assert_eq!(row.2, "gpt-test");
    assert_eq!(row.3, file_path.to_string_lossy().to_string());
    assert!(row.4.contains("result=error"));
    assert_eq!(row.5, super::checksum_hex(&row.4));
    assert_eq!(row.6, "error");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn raw_artifact_saves_review_result_with_default_agent() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = temp.path().to_string_lossy().to_string();
    let mut task = sample_task();
    task.id = "T20260612.0002".into();
    task.acp_agent = None;

    super::save_task_code_review_result_artifact(
        &project,
        &task,
        &Ok("approved".into()),
        Some("system"),
        None,
    )
    .expect("review artifact save");

    let conn = super::open_index_connection(&project).expect("sqlite connection");
    let (artifact_type, acp_agent, status): (String, String, String) = conn
        .query_row(
            "SELECT artifact_type, acp_agent, status FROM task_raw_artifacts WHERE task_id = ?1",
            rusqlite::params![task.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("artifact row");

    assert_eq!(artifact_type, "code_review_result");
    assert_eq!(acp_agent, "acp");
    assert_eq!(status, "success");
}
