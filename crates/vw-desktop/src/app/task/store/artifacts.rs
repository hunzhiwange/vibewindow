//! 任务存储层的 artifacts.rs 子模块。
//!
//! 该模块负责任务索引、持久化或产物写入中的一部分能力。实现保持文件系统与 SQLite 路径清晰分离，让上层任务流程只依赖稳定的存储函数。

use std::fmt::Write as _;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(not(target_arch = "wasm32"))]
use rusqlite::params;
#[cfg(not(target_arch = "wasm32"))]
use sha2::{Digest, Sha256};

use crate::app::task::models::Task;

use super::paths::get_task_log_dir;
#[cfg(not(target_arch = "wasm32"))]
use super::paths::with_index_lock;
#[cfg(not(target_arch = "wasm32"))]
use super::persistence::{open_index_connection, sqlite_to_io_error};

#[cfg(not(target_arch = "wasm32"))]
const RAW_ARTIFACT_EXECUTION_RESULT: &str = "execution_result";
#[cfg(not(target_arch = "wasm32"))]
const RAW_ARTIFACT_CODE_REVIEW_RESULT: &str = "code_review_result";

fn stored_task_executor_name(task: &Task) -> String {
    task.acp_agent.clone().unwrap_or_else(|| "acp".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn checksum_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(not(target_arch = "wasm32"))]
fn build_task_execution_result_log_content(task: &Task, result: &Result<String, String>) -> String {
    let mut content = String::new();
    let now_ms = crate::app::time::now_ms() as u128;
    let _ = writeln!(content, "task_id={}", task.id);
    let _ = writeln!(content, "acp_agent={}", stored_task_executor_name(task));
    let _ = writeln!(content, "model={}", task.model);
    let _ = writeln!(content, "saved_at_ms={}", now_ms);
    let _ = writeln!(content);
    let _ = writeln!(content, "prompt:");
    let _ = writeln!(content, "{}", task.prompt);
    let _ = writeln!(content);
    match result {
        Ok(output) => {
            let _ = writeln!(content, "result=success");
            let _ = writeln!(content);
            let _ = writeln!(content, "output:");
            let _ = writeln!(content, "{}", output);
        }
        Err(error) => {
            let _ = writeln!(content, "result=error");
            let _ = writeln!(content);
            let _ = writeln!(content, "error:");
            let _ = writeln!(content, "{}", error);
        }
    }
    content
}

#[cfg(not(target_arch = "wasm32"))]
fn build_task_code_review_result_log_content(
    task: &Task,
    result: &Result<String, String>,
    full_system_prompt: Option<&str>,
) -> String {
    let mut content = String::new();
    let now_ms = crate::app::time::now_ms() as u128;
    let _ = writeln!(content, "task_id={}", task.id);
    let _ = writeln!(content, "acp_agent={}", stored_task_executor_name(task));
    let _ = writeln!(content, "model={}", task.model);
    let _ = writeln!(
        content,
        "source_branch={}",
        task.merge_source_branch.as_deref().unwrap_or_default()
    );
    let _ = writeln!(
        content,
        "target_branch={}",
        task.merge_target_branch.as_deref().unwrap_or_default()
    );
    let _ = writeln!(content, "saved_at_ms={}", now_ms);
    if let Some(system_prompt) = full_system_prompt {
        let _ = writeln!(content);
        let _ = writeln!(content, "review_system_prompt_full:");
        let _ = writeln!(content, "{}", system_prompt);
    }
    let _ = writeln!(content);
    match result {
        Ok(output) => {
            let _ = writeln!(content, "review_result=success");
            let _ = writeln!(content);
            let _ = writeln!(content, "review_output:");
            let _ = writeln!(content, "{}", output);
        }
        Err(error) => {
            let _ = writeln!(content, "review_result=error");
            let _ = writeln!(content);
            let _ = writeln!(content, "review_error:");
            let _ = writeln!(content, "{}", error);
        }
    }
    content
}

#[cfg(not(target_arch = "wasm32"))]
fn save_task_raw_artifact(
    project_path: &str,
    task: &Task,
    artifact_type: &str,
    content: &str,
    file_path: Option<&Path>,
    status: Option<&str>,
) -> io::Result<()> {
    with_index_lock(project_path, || {
        let conn = open_index_connection(project_path)?;
        conn.execute(
            "INSERT INTO task_raw_artifacts (
                task_id, artifact_type, created_at_ms, acp_agent, model, file_path,
                content_text, content_sha256, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                task.id,
                artifact_type,
                crate::app::time::now_ms() as i64,
                stored_task_executor_name(task),
                task.model,
                file_path.map(|path| path.to_string_lossy().to_string()),
                content,
                checksum_hex(content),
                status,
            ],
        )
        .map_err(sqlite_to_io_error)?;
        Ok(())
    })
}

/// 公开的 write_task_execution_result_log 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn write_task_execution_result_log(
    project_path: &str,
    task: &Task,
    result: &Result<String, String>,
) -> io::Result<PathBuf> {
    let dir = get_task_log_dir(project_path);
    std::fs::create_dir_all(&dir)?;
    let file_name = format!("[{}].log", task.id);
    let path = dir.join(file_name);

    #[cfg(not(target_arch = "wasm32"))]
    let content = build_task_execution_result_log_content(task, result);
    #[cfg(target_arch = "wasm32")]
    let content = {
        let mut content = String::new();
        let now_ms = crate::app::time::now_ms() as u128;
        let _ = writeln!(content, "task_id={}", task.id);
        let _ = writeln!(content, "acp_agent={}", stored_task_executor_name(task));
        let _ = writeln!(content, "model={}", task.model);
        let _ = writeln!(content, "saved_at_ms={}", now_ms);
        let _ = writeln!(content);
        let _ = writeln!(content, "prompt:");
        let _ = writeln!(content, "{}", task.prompt);
        let _ = writeln!(content);
        match result {
            Ok(output) => {
                let _ = writeln!(content, "result=success");
                let _ = writeln!(content);
                let _ = writeln!(content, "output:");
                let _ = writeln!(content, "{}", output);
            }
            Err(error) => {
                let _ = writeln!(content, "result=error");
                let _ = writeln!(content);
                let _ = writeln!(content, "error:");
                let _ = writeln!(content, "{}", error);
            }
        }
        content
    };

    std::fs::write(&path, content)?;
    Ok(path)
}

/// 公开的 write_task_code_review_result_log 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn write_task_code_review_result_log(
    project_path: &str,
    task: &Task,
    result: &Result<String, String>,
    full_system_prompt: Option<&str>,
) -> io::Result<PathBuf> {
    let dir = get_task_log_dir(project_path);
    std::fs::create_dir_all(&dir)?;
    let file_name = format!("[{}].review.log", task.id);
    let path = dir.join(file_name);

    #[cfg(not(target_arch = "wasm32"))]
    let content = build_task_code_review_result_log_content(task, result, full_system_prompt);
    #[cfg(target_arch = "wasm32")]
    let content = {
        let mut content = String::new();
        let now_ms = crate::app::time::now_ms() as u128;
        let _ = writeln!(content, "task_id={}", task.id);
        let _ = writeln!(content, "acp_agent={}", stored_task_executor_name(task));
        let _ = writeln!(content, "model={}", task.model);
        let _ = writeln!(
            content,
            "source_branch={}",
            task.merge_source_branch.as_deref().unwrap_or_default()
        );
        let _ = writeln!(
            content,
            "target_branch={}",
            task.merge_target_branch.as_deref().unwrap_or_default()
        );
        let _ = writeln!(content, "saved_at_ms={}", now_ms);
        if let Some(system_prompt) = full_system_prompt {
            let _ = writeln!(content);
            let _ = writeln!(content, "review_system_prompt_full:");
            let _ = writeln!(content, "{}", system_prompt);
        }
        let _ = writeln!(content);
        match result {
            Ok(output) => {
                let _ = writeln!(content, "review_result=success");
                let _ = writeln!(content);
                let _ = writeln!(content, "review_output:");
                let _ = writeln!(content, "{}", output);
            }
            Err(error) => {
                let _ = writeln!(content, "review_result=error");
                let _ = writeln!(content);
                let _ = writeln!(content, "review_error:");
                let _ = writeln!(content, "{}", error);
            }
        }
        content
    };

    std::fs::write(&path, content)?;
    Ok(path)
}

/// 公开的 save_task_execution_result_artifact 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub fn save_task_execution_result_artifact(
    project_path: &str,
    task: &Task,
    result: &Result<String, String>,
    file_path: Option<&Path>,
) -> io::Result<()> {
    let content = build_task_execution_result_log_content(task, result);
    let status = Some(if result.is_ok() { "success" } else { "error" });
    save_task_raw_artifact(
        project_path,
        task,
        RAW_ARTIFACT_EXECUTION_RESULT,
        &content,
        file_path,
        status,
    )
}

/// 公开的 save_task_execution_result_artifact 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(target_arch = "wasm32")]
pub fn save_task_execution_result_artifact(
    _project_path: &str,
    _task: &Task,
    _result: &Result<String, String>,
    _file_path: Option<&Path>,
) -> io::Result<()> {
    Ok(())
}

/// 公开的 save_task_code_review_result_artifact 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub fn save_task_code_review_result_artifact(
    project_path: &str,
    task: &Task,
    result: &Result<String, String>,
    full_system_prompt: Option<&str>,
    file_path: Option<&Path>,
) -> io::Result<()> {
    let content = build_task_code_review_result_log_content(task, result, full_system_prompt);
    let status = Some(if result.is_ok() { "success" } else { "error" });
    save_task_raw_artifact(
        project_path,
        task,
        RAW_ARTIFACT_CODE_REVIEW_RESULT,
        &content,
        file_path,
        status,
    )
}

/// 公开的 save_task_code_review_result_artifact 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(target_arch = "wasm32")]
pub fn save_task_code_review_result_artifact(
    _project_path: &str,
    _task: &Task,
    _result: &Result<String, String>,
    _full_system_prompt: Option<&str>,
    _file_path: Option<&Path>,
) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
#[path = "artifacts_tests.rs"]
mod artifacts_tests;
