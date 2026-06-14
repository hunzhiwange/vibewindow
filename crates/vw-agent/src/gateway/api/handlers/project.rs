//! 项目管理相关的 HTTP 路由模块

use axum::Json;
use axum::Router;
use axum::extract::{Path, Query};
use axum::routing::{get, post};
use base64::Engine;
use vw_api_types::common::{OperationAck, PaginatedResponse, TimestampMs};
use vw_api_types::project::{
    GetProjectResponse, ListProjectChangeRecordsRequest, ListProjectChangeRecordsResponse,
    ListProjectsRequest, ProjectChangeRecordDto, ProjectDto, ProjectGitStateDto, ProjectStatus,
    ResolveProjectRequest, ResolveProjectResponse, UpdateProjectRequest,
};
use vw_api_types::worktree::{
    CreateWorktreeRequest, CreateWorktreeResponse, DeleteWorktreeRequest, GetWorktreeResponse,
    ListWorktreesResponse, ResetMode, ResetWorktreeRequest, WorktreeDto, WorktreeStatus,
};

use crate::app::agent::gateway::ApiError;
use crate::app::agent::gateway::instance::with_instance;
use crate::app::agent::project;
use crate::app::agent::project::instance;
use crate::app::agent::{file, storage};
use crate::worktree;

pub(crate) fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/projects", get(project_list_v1))
        .route("/projects/change-records", get(project_change_records_v1))
        .route("/projects/resolve", post(project_resolve_v1))
        .route("/projects/{project_id}", get(project_get_v1).patch(project_update_v1))
        .route(
            "/projects/{project_id}/worktrees",
            get(project_worktrees_v1).post(project_worktree_create_v1),
        )
        .route("/worktrees/{worktree_id}", get(worktree_get_v1).delete(worktree_delete_v1))
        .route("/worktrees/{worktree_id}/reset", post(worktree_reset_v1))
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

fn timestamp_ms(value: u64) -> TimestampMs {
    TimestampMs(value.min(i64::MAX as u64) as i64)
}

fn git_stdout(directory: &str, args: &[&str]) -> Option<String> {
    let output = crate::app::agent::shell::git_std_command()
        .current_dir(directory)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn collect_project_change_records(directory: &str) -> Vec<ProjectChangeRecordDto> {
    let Ok(repo) = git2::Repository::open(directory) else {
        return Vec::new();
    };
    let Ok(head) = repo.head() else {
        return Vec::new();
    };
    let Ok(tree) = head.peel_to_tree() else {
        return Vec::new();
    };

    let mut opts = git2::DiffOptions::new();
    opts.include_untracked(true);
    let Ok(diff) = repo.diff_tree_to_workdir_with_index(Some(&tree), Some(&mut opts)) else {
        return Vec::new();
    };

    let mut items = Vec::<ProjectChangeRecordDto>::new();
    let mut current_path: Option<String> = None;

    let _ = diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .and_then(|value| value.to_str())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "unknown".to_string());

        if current_path.as_ref() != Some(&path) {
            current_path = Some(path.clone());
            items.push(ProjectChangeRecordDto { path, patch: String::new() });
        }

        if let Ok(content) = std::str::from_utf8(line.content())
            && let Some(item) = items.last_mut()
        {
            item.patch.push_str(content);
        }
        true
    });

    items
}

fn map_project(info: &project::Info) -> ProjectDto {
    let directory = if info.worktree.trim().is_empty() {
        info.sandboxes.first().cloned().unwrap_or_default()
    } else {
        info.worktree.clone()
    };
    let is_repo = matches!(info.vcs, Some(project::Vcs::Git));
    let git = ProjectGitStateDto {
        is_repo,
        has_uncommitted_changes: is_repo && !file::status_git(&directory).is_empty(),
        ahead: None,
        behind: None,
    };
    let display_name = info
        .name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::path::Path::new(&directory)
                .file_name()
                .and_then(|value| value.to_str())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| directory.clone());
    let session_count = Some(
        crate::app::agent::session::ui_store::load_agent_sessions_scoped(Some(&directory)).len()
            as u32,
    );

    ProjectDto {
        id: info.id.as_str().into(),
        name: display_name,
        directory: directory.clone(),
        display_path: directory.clone(),
        status: ProjectStatus::Ready,
        created_at_ms: timestamp_ms(info.time.created),
        updated_at_ms: timestamp_ms(info.time.updated),
        default_branch: None,
        current_branch: git_stdout(&directory, &["rev-parse", "--abbrev-ref", "HEAD"]),
        git,
        active_worktree_id: None,
        session_count,
    }
}

fn worktree_id_from_directory(directory: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(directory.as_bytes())
}

fn directory_from_worktree_id(worktree_id: &str) -> Result<String, ApiError> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(worktree_id)
        .map_err(|_| ApiError::not_found("worktree not found"))?;
    String::from_utf8(bytes).map_err(|_| ApiError::not_found("worktree not found"))
}

fn map_worktree(project_id: &str, info: &project::Info, directory: String) -> WorktreeDto {
    let name = std::path::Path::new(&directory)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "worktree".to_string());
    WorktreeDto {
        id: worktree_id_from_directory(&directory).as_str().into(),
        project_id: project_id.into(),
        name,
        branch: git_stdout(&directory, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default(),
        directory,
        status: WorktreeStatus::Ready,
        created_at_ms: timestamp_ms(info.time.created),
        updated_at_ms: timestamp_ms(info.time.updated),
    }
}

async fn load_project(project_id: &str) -> Result<project::Info, ApiError> {
    let mut info = storage::read::<project::Info>(&["project", project_id])
        .await
        .map_err(|_| ApiError::not_found("project not found"))?;
    info.sandboxes.retain(|path| std::path::Path::new(path).is_dir());
    Ok(info)
}

async fn project_by_directory(directory: &str) -> Result<project::Info, ApiError> {
    let target = normalize_path(directory);
    let projects = project::list().await.map_err(|e| ApiError::bad_request(e.to_string()))?;
    if let Some(info) = projects.into_iter().find(|info| {
        normalize_path(&info.worktree) == target
            || info.sandboxes.iter().any(|sandbox| normalize_path(sandbox) == target)
    }) {
        return Ok(info);
    }
    with_instance(directory.to_string(), move || {
        Box::pin(async move {
            instance::project().ok_or_else(|| ApiError::not_found("project not found"))
        })
    })
    .await
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

async fn project_list_v1(
    Query(query): Query<ListProjectsRequest>,
) -> Result<Json<PaginatedResponse<ProjectDto>>, ApiError> {
    let mut items = project::list()
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?
        .into_iter()
        .map(|info| map_project(&info))
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .updated_at_ms
            .0
            .cmp(&left.updated_at_ms.0)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.directory.cmp(&right.directory))
            .then_with(|| left.id.0.cmp(&right.id.0))
    });

    if let Some(search) = query
        .query
        .as_ref()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
    {
        items.retain(|item| {
            item.name.to_lowercase().contains(&search)
                || item.directory.to_lowercase().contains(&search)
        });
    }
    if let Some(status) = query.status.as_ref() {
        items.retain(|item| &item.status == status);
    }

    let start_index = query
        .cursor
        .as_ref()
        .and_then(|cursor| {
            items.iter().position(|item| item.id.0 == *cursor).map(|index| index + 1)
        })
        .unwrap_or(0);
    let limit = query.limit.unwrap_or(50).clamp(1, 200) as usize;
    let sliced = items.into_iter().skip(start_index).collect::<Vec<_>>();
    let next_cursor = (sliced.len() > limit).then(|| sliced[limit - 1].id.0.clone());
    let items = sliced.into_iter().take(limit).collect::<Vec<_>>();
    Ok(Json(PaginatedResponse { items, next_cursor }))
}

async fn project_resolve_v1(
    Json(body): Json<ResolveProjectRequest>,
) -> Result<Json<ResolveProjectResponse>, ApiError> {
    let directory = body.directory.clone();
    let info = with_instance(directory, move || {
        Box::pin(async move {
            instance::project().ok_or_else(|| ApiError::not_found("project not found"))
        })
    })
    .await?;
    Ok(Json(ResolveProjectResponse { project: map_project(&info) }))
}

async fn project_change_records_v1(
    Query(query): Query<ListProjectChangeRecordsRequest>,
) -> Result<Json<ListProjectChangeRecordsResponse>, ApiError> {
    let directory = query
        .directory
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApiError::bad_request("directory is required"))?
        .to_string();

    let _ = project_by_directory(&directory).await?;
    let items = tokio::task::spawn_blocking(move || collect_project_change_records(&directory))
        .await
        .map_err(|err| ApiError::internal(format!("project change records task failed: {err}")))?;

    Ok(Json(ListProjectChangeRecordsResponse { items }))
}

async fn project_get_v1(
    Path(project_id): Path<String>,
) -> Result<Json<GetProjectResponse>, ApiError> {
    let info = load_project(&project_id).await?;
    Ok(Json(GetProjectResponse { project: map_project(&info) }))
}

async fn project_update_v1(
    Path(project_id): Path<String>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<GetProjectResponse>, ApiError> {
    let updated = project::update(project::UpdateInput {
        project_id,
        name: body.name.map(Some),
        icon: body.icon.map(|dto| project::IconUpdate {
            url: None,
            override_icon: Some(dto.override_icon),
            color: Some(dto.color),
        }),
        commands: body.commands.map(|dto| project::CommandsUpdate { start: Some(dto.start) }),
    })
    .await
    .map_err(|e| ApiError::bad_request(e.to_string()))?;
    Ok(Json(GetProjectResponse { project: map_project(&updated) }))
}

async fn project_worktrees_v1(
    Path(project_id): Path<String>,
) -> Result<Json<ListWorktreesResponse>, ApiError> {
    let info = load_project(&project_id).await?;
    let items = worktree_list_for_project(&info)
        .await?
        .into_iter()
        .map(|directory| map_worktree(&project_id, &info, directory))
        .collect();
    Ok(Json(ListWorktreesResponse { items }))
}

async fn project_worktree_create_v1(
    Path(project_id): Path<String>,
    Json(body): Json<CreateWorktreeRequest>,
) -> Result<Json<CreateWorktreeResponse>, ApiError> {
    let info = load_project(&project_id).await?;
    let context_directory = project_context_directory(&info)?;
    let created: worktree::Info = with_instance(context_directory, move || {
        Box::pin(async move {
            let result: Result<worktree::Info, worktree::Error> =
                worktree::create(Some(worktree::CreateInput {
                    name: Some(body.name),
                    start_command: None,
                }))
                .await;
            result.map_err(|e: worktree::Error| ApiError::bad_request(e.to_string()))
        })
    })
    .await?;
    Ok(Json(CreateWorktreeResponse {
        worktree: map_worktree(&project_id, &info, created.directory),
    }))
}

async fn worktree_get_v1(
    Path(worktree_id): Path<String>,
) -> Result<Json<GetWorktreeResponse>, ApiError> {
    let directory = directory_from_worktree_id(&worktree_id)?;
    let project = project_by_directory(&directory).await?;
    Ok(Json(GetWorktreeResponse { worktree: map_worktree(&project.id, &project, directory) }))
}

async fn worktree_delete_v1(
    Path(worktree_id): Path<String>,
    Json(body): Json<DeleteWorktreeRequest>,
) -> Result<Json<OperationAck>, ApiError> {
    let directory = directory_from_worktree_id(&worktree_id)?;
    let project = project_by_directory(&directory).await?;
    let context_directory = project_context_directory(&project)?;
    with_instance::<()>(context_directory, move || {
        Box::pin(async move {
            let result: Result<bool, worktree::Error> = worktree::remove(worktree::RemoveInput {
                directory: directory.clone(),
                force: body.force,
            })
            .await;
            result.map_err(|e: worktree::Error| ApiError::bad_request(e.to_string()))?;
            Ok::<(), ApiError>(())
        })
    })
    .await?;
    Ok(Json(OperationAck { ok: true, message: None }))
}

async fn worktree_reset_v1(
    Path(worktree_id): Path<String>,
    Json(body): Json<ResetWorktreeRequest>,
) -> Result<Json<OperationAck>, ApiError> {
    let directory = directory_from_worktree_id(&worktree_id)?;
    let project = project_by_directory(&directory).await?;
    let context_directory = project_context_directory(&project)?;
    let base_ref = match body.mode {
        ResetMode::Soft | ResetMode::Mixed | ResetMode::Hard => body.target_ref,
    };
    with_instance::<()>(context_directory, move || {
        Box::pin(async move {
            let result: Result<bool, worktree::Error> =
                worktree::reset(worktree::ResetInput { directory: directory.clone(), base_ref })
                    .await;
            result.map_err(|e: worktree::Error| ApiError::bad_request(e.to_string()))?;
            Ok::<(), ApiError>(())
        })
    })
    .await?;
    Ok(Json(OperationAck { ok: true, message: None }))
}

#[cfg(test)]
#[path = "project_tests.rs"]
mod project_tests;
