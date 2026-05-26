//! Git 提交网关路由模块

use std::io::Write;
use std::path::{Component, Path};

use axum::Json;
use axum::Router;
use axum::routing::post;
use base64::Engine;
use similar::TextDiff;
use tempfile::NamedTempFile;
use vw_api_types::git::{GitCommitDto, GitCommitRequest, GitCommitResponse};

use crate::app::agent::gateway::ApiError;
use crate::app::agent::gateway::instance::with_instance;
use crate::app::agent::project;
use crate::app::agent::shell::git_std_command;
use crate::app::agent::storage;
use crate::worktree;

const DIFF_CONTEXT: usize = 3;

pub(crate) fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new().route("/git/commit", post(git_commit_v1))
}

async fn git_commit_v1(
    Json(body): Json<GitCommitRequest>,
) -> Result<Json<GitCommitResponse>, ApiError> {
    let message = body.message.trim().to_string();
    if message.is_empty() {
        return Err(ApiError::bad_request("commit message is required"));
    }

    let directory = resolve_git_directory(&body.project_id.0, body.worktree_id.as_ref()).await?;
    let request = GitCommitRequest { message, ..body };
    let response =
        tokio::task::spawn_blocking(move || commit_selected_blocking(&directory, &request))
            .await
            .map_err(|err| ApiError::internal(format!("git commit task failed: {err}")))?
            .map_err(ApiError::bad_request)?;

    Ok(Json(response))
}

async fn resolve_git_directory(
    project_id: &str,
    worktree_id: Option<&vw_api_types::id::WorktreeId>,
) -> Result<String, ApiError> {
    let info = load_project(project_id).await?;
    let project_directory = project_context_directory(&info)?;

    let Some(worktree_id) = worktree_id else {
        return Ok(project_directory);
    };

    let requested_directory = directory_from_worktree_id(&worktree_id.0)?;
    let allowed_directories = worktree_list_for_project(&info).await?;
    let requested_norm = normalize_path(&requested_directory);
    if allowed_directories.iter().any(|path| normalize_path(path) == requested_norm) {
        return Ok(requested_directory);
    }

    Err(ApiError::not_found("worktree not found"))
}

async fn load_project(project_id: &str) -> Result<project::Info, ApiError> {
    let mut info = storage::read::<project::Info>(&["project", project_id])
        .await
        .map_err(|_| ApiError::not_found("project not found"))?;
    info.sandboxes.retain(|path| std::path::Path::new(path).is_dir());
    Ok(info)
}

fn project_context_directory(info: &project::Info) -> Result<String, ApiError> {
    if !info.worktree.trim().is_empty() {
        return Ok(info.worktree.clone());
    }
    info.sandboxes
        .iter()
        .find(|path| !path.trim().is_empty())
        .cloned()
        .ok_or_else(|| ApiError::bad_request("project directory missing"))
}

async fn worktree_list_for_project(info: &project::Info) -> Result<Vec<String>, ApiError> {
    if !matches!(info.vcs, Some(project::Vcs::Git)) {
        return Ok(Vec::new());
    }

    let context_directory = project_context_directory(info)?;
    let primary = normalize_path(&context_directory);
    let mut directories: Vec<String> = with_instance(context_directory, move || {
        Box::pin(async move {
            let result: Result<Vec<String>, worktree::Error> = worktree::list_directories().await;
            result.map_err(|e: worktree::Error| ApiError::bad_request(e.to_string()))
        })
    })
    .await?;
    directories.sort();
    directories.dedup_by(|left, right| normalize_path(left) == normalize_path(right));
    if !directories.iter().any(|path| normalize_path(path) == primary) {
        directories.insert(0, info.worktree.clone());
    }
    Ok(directories)
}

fn directory_from_worktree_id(worktree_id: &str) -> Result<String, ApiError> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(worktree_id)
        .map_err(|_| ApiError::not_found("worktree not found"))?;
    String::from_utf8(bytes).map_err(|_| ApiError::not_found("worktree not found"))
}

fn normalize_path(value: &str) -> String {
    let normalized = value.replace('\\', "/");
    normalized
        .trim_end_matches('/')
        .trim_start_matches('/')
        .strip_prefix("./")
        .unwrap_or_else(|| normalized.trim_end_matches('/').trim_start_matches('/'))
        .to_string()
}

fn commit_selected_blocking(
    directory: &str,
    request: &GitCommitRequest,
) -> Result<GitCommitResponse, String> {
    for path in &request.selected_files {
        validate_repo_relative_path(path)?;
        git_stage_file(directory, path)?;
    }
    for selection in &request.selected_hunks {
        validate_repo_relative_path(&selection.path)?;
        git_stage_hunk(directory, &selection.path, selection.index)?;
    }
    for selection in &request.selected_lines {
        validate_repo_relative_path(&selection.path)?;
        git_stage_line_insert(directory, &selection.path, selection.line)?;
    }
    for selection in &request.selected_old_lines {
        validate_repo_relative_path(&selection.path)?;
        git_stage_line_delete(directory, &selection.path, selection.line)?;
    }

    git_commit_with_message_file(directory, &request.message)?;
    let sha = run_git(directory, &["rev-parse", "HEAD"])?;

    Ok(GitCommitResponse {
        ok: true,
        commit: GitCommitDto { sha: sha.trim().to_string(), message: request.message.clone() },
    })
}

fn validate_repo_relative_path(path: &str) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("git selection path is empty".to_string());
    }

    let candidate = Path::new(trimmed);
    if candidate.is_absolute() {
        return Err(format!("git selection path must be relative: {trimmed}"));
    }
    if candidate.components().any(|component| {
        matches!(component, Component::ParentDir | Component::Prefix(_) | Component::RootDir)
    }) {
        return Err(format!("git selection path escapes repository: {trimmed}"));
    }

    Ok(())
}

fn git_stage_file(directory: &str, file: &str) -> Result<(), String> {
    run_git(directory, &["add", "-A", "--", file]).map(|_| ())
}

fn git_commit_with_message_file(directory: &str, message: &str) -> Result<(), String> {
    let mut temp = NamedTempFile::new().map_err(|err| err.to_string())?;
    temp.write_all(message.as_bytes()).map_err(|err| err.to_string())?;
    temp.flush().map_err(|err| err.to_string())?;
    let path = temp.path().to_string_lossy().to_string();
    run_git_owned(directory, vec!["commit".to_string(), "-F".to_string(), path]).map(|_| ())
}

fn git_stage_hunk(directory: &str, file: &str, idx: usize) -> Result<(), String> {
    let (old_content, new_content) = get_file_content_pair(directory, file)
        .ok_or_else(|| "Failed to read file content".to_string())?;

    let diff = TextDiff::from_lines(&old_content, &new_content);
    let hunks: Vec<String> = diff
        .unified_diff()
        .context_radius(DIFF_CONTEXT)
        .iter_hunks()
        .map(|hunk| hunk.to_string())
        .collect();

    if idx >= hunks.len() {
        return Err("bad hunk index".to_string());
    }

    let mut patch = build_patch_prefix(file, &old_content, &new_content);
    patch.push_str(&hunks[idx]);
    apply_cached_patch(directory, &patch)
}

fn git_stage_line_insert(directory: &str, file_path: &str, new_idx: usize) -> Result<(), String> {
    let (old_content, new_content) = get_file_content_pair(directory, file_path)
        .ok_or_else(|| "Failed to read file".to_string())?;
    let new_lines: Vec<&str> = new_content.lines().collect();
    let diff = TextDiff::from_lines(&old_content, &new_content);

    let mut last_old_end = 0usize;
    let mut target_old_pos = None::<usize>;

    for group in diff.grouped_ops(DIFF_CONTEXT) {
        for op in group {
            match op {
                similar::DiffOp::Equal { old_index, new_index, len } => {
                    last_old_end = last_old_end.max(old_index + len);
                    if new_index <= new_idx && new_idx < new_index + len {
                        target_old_pos = Some(old_index + (new_idx - new_index));
                    }
                }
                similar::DiffOp::Delete { old_index, old_len, .. } => {
                    last_old_end = last_old_end.max(old_index + old_len);
                }
                similar::DiffOp::Insert { new_index, new_len, .. } => {
                    if new_index <= new_idx && new_idx < new_index + new_len {
                        target_old_pos = Some(last_old_end);
                    }
                }
                similar::DiffOp::Replace { old_index, old_len, new_index, new_len } => {
                    last_old_end = last_old_end.max(old_index + old_len);
                    if new_index <= new_idx && new_idx < new_index + new_len {
                        target_old_pos = Some(old_index);
                    }
                }
            }
        }
    }

    let old_pos = target_old_pos.ok_or_else(|| "Cannot locate insertion anchor".to_string())?;
    let line = new_lines.get(new_idx).copied().unwrap_or("");

    let mut patch = build_patch_prefix(file_path, &old_content, &new_content);
    patch.push_str(&format!("@@ -{},{} +{},{} @@\n", old_pos + 1, 0, new_idx + 1, 1));
    patch.push_str(&format!("+{}\n", line));
    apply_cached_patch(directory, &patch)
}

fn git_stage_line_delete(directory: &str, file_path: &str, old_idx: usize) -> Result<(), String> {
    let (old_content, new_content) = get_file_content_pair(directory, file_path)
        .ok_or_else(|| "Failed to read file".to_string())?;
    let old_lines: Vec<&str> = old_content.lines().collect();
    if old_idx >= old_lines.len() {
        return Err("Old line index out of bounds".to_string());
    }

    let diff = TextDiff::from_lines(&old_content, &new_content);
    let mut last_new_end = 0usize;
    let mut target_new_pos = None::<usize>;

    for group in diff.grouped_ops(DIFF_CONTEXT) {
        for op in group {
            match op {
                similar::DiffOp::Equal { old_index, new_index, len } => {
                    if old_index <= old_idx && old_idx < old_index + len {
                        target_new_pos = Some(new_index + (old_idx - old_index));
                    }
                    last_new_end = last_new_end.max(new_index + len);
                }
                similar::DiffOp::Delete { old_index, old_len, new_index } => {
                    if old_index <= old_idx && old_idx < old_index + old_len {
                        target_new_pos = Some(new_index);
                    }
                    last_new_end = last_new_end.max(new_index);
                }
                similar::DiffOp::Insert { new_index, new_len, .. } => {
                    last_new_end = last_new_end.max(new_index + new_len);
                }
                similar::DiffOp::Replace { old_index, old_len, new_index, new_len } => {
                    if old_index <= old_idx && old_idx < old_index + old_len {
                        target_new_pos = Some(new_index);
                    }
                    last_new_end = last_new_end.max(new_index + new_len);
                }
            }
        }
    }

    let new_pos = target_new_pos.unwrap_or(last_new_end);
    let mut patch = build_patch_prefix(file_path, &old_content, &new_content);
    patch.push_str(&format!("@@ -{},{} +{},{} @@\n", old_idx + 1, 1, new_pos + 1, 0));
    patch.push_str(&format!("-{}\n", old_lines[old_idx]));
    apply_cached_patch(directory, &patch)
}

fn build_patch_prefix(file_path: &str, old_content: &str, new_content: &str) -> String {
    let mut patch = String::new();
    patch.push_str(&format!("diff --git a/{} b/{}\n", file_path, file_path));

    let is_new_file = old_content.is_empty() && !new_content.is_empty();
    let is_deleted_file = !old_content.is_empty() && new_content.is_empty();

    if is_new_file {
        patch.push_str("new file mode 100644\n");
        patch.push_str("index 0000000..0000000\n");
        patch.push_str("--- /dev/null\n");
        patch.push_str(&format!("+++ b/{}\n", file_path));
    } else if is_deleted_file {
        patch.push_str("deleted file mode 100644\n");
        patch.push_str("index 0000000..0000000\n");
        patch.push_str(&format!("--- a/{}\n", file_path));
        patch.push_str("+++ /dev/null\n");
    } else {
        patch.push_str("index 0000000..0000000 100644\n");
        patch.push_str(&format!("--- a/{}\n", file_path));
        patch.push_str(&format!("+++ b/{}\n", file_path));
    }

    patch
}

fn apply_cached_patch(directory: &str, patch: &str) -> Result<(), String> {
    let mut temp = NamedTempFile::new().map_err(|err| err.to_string())?;
    temp.write_all(patch.as_bytes()).map_err(|err| err.to_string())?;
    temp.flush().map_err(|err| err.to_string())?;
    let path = temp.path().to_string_lossy().to_string();
    run_git_owned(directory, vec!["apply".to_string(), "--cached".to_string(), path]).map(|_| ())
}

fn get_file_content_pair(directory: &str, file_path: &str) -> Option<(String, String)> {
    let repo = git2::Repository::open(directory).ok()?;
    let old_content = repo
        .head()
        .ok()
        .and_then(|head| head.peel_to_tree().ok())
        .and_then(|tree| tree.get_path(Path::new(file_path)).ok())
        .and_then(|entry| entry.to_object(&repo).ok())
        .and_then(|obj| {
            obj.as_blob().map(|blob| String::from_utf8_lossy(blob.content()).to_string())
        })
        .unwrap_or_default();

    let new_content =
        std::fs::read_to_string(Path::new(directory).join(file_path)).unwrap_or_default();
    Some((old_content, new_content))
}

fn run_git(directory: &str, args: &[&str]) -> Result<String, String> {
    run_git_owned(directory, args.iter().map(|value| (*value).to_string()).collect())
}

fn run_git_owned(directory: &str, args: Vec<String>) -> Result<String, String> {
    let output = git_std_command()
        .current_dir(directory)
        .args(&args)
        .output()
        .map_err(|err| err.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n").replace('\r', "\n");
    let stderr = String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n").replace('\r', "\n");
    if output.status.success() {
        return Ok(stdout);
    }

    let detail = if stderr.trim().is_empty() {
        stdout.trim().to_string()
    } else {
        stderr.trim().to_string()
    };
    if detail.is_empty() {
        return Err(format!("git command failed: {}", args.join(" ")));
    }
    Err(detail)
}

#[cfg(test)]
#[path = "git_tests.rs"]
mod git_tests;
