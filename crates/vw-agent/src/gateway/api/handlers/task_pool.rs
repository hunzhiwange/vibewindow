//! 任务池调度路由模块。

use axum::Json;
use axum::Router;
use axum::routing::post;
use vw_api_types::task::{TaskPoolScheduleRequest, TaskPoolScheduleResponse, TaskPoolStatus};

use crate::app::agent::gateway::ApiError;

pub(crate) fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new().route("/task-pool/schedule", post(task_pool_schedule_v1))
}

async fn task_pool_schedule_v1(
    Json(body): Json<TaskPoolScheduleRequest>,
) -> Result<Json<TaskPoolScheduleResponse>, ApiError> {
    Ok(Json(schedule_task_pool(&body)))
}

fn schedule_task_pool(request: &TaskPoolScheduleRequest) -> TaskPoolScheduleResponse {
    let settings = &request.settings;
    if !settings.auto_execute || !settings.auto_promote_pool_tasks {
        return TaskPoolScheduleResponse { promote_task_ids: Vec::new() };
    }

    let max_concurrent = settings.max_concurrent.max(1);
    let max_pending = max_concurrent.saturating_mul(2);
    let pending_count = request
        .tasks
        .iter()
        .filter(|task| {
            !task.deleted
                && !task.archived
                && matches!(task.status, TaskPoolStatus::Pending | TaskPoolStatus::Planning)
        })
        .count() as u32;
    if pending_count >= max_pending {
        return TaskPoolScheduleResponse { promote_task_ids: Vec::new() };
    }

    let default_delay_ms = settings.auto_promote_delay_seconds.saturating_mul(1000);
    let mut candidates: Vec<(u32, u32, String)> = request
        .tasks
        .iter()
        .filter(|task| !task.deleted && !task.archived && task.status == TaskPoolStatus::Pool)
        .filter(|task| {
            let delay_ms = task.auto_promote_delay_ms.unwrap_or(default_delay_ms);
            task.created_at_ms.saturating_add(delay_ms) <= request.now_ms
        })
        .map(|task| (task.priority, task.order, task.id.clone()))
        .collect();
    candidates.sort_by_key(|(priority, order, _)| (*priority, *order));

    let max_promote = max_pending.saturating_sub(pending_count) as usize;
    let promote_task_ids =
        candidates.into_iter().take(max_promote).map(|(_, _, task_id)| task_id).collect();

    TaskPoolScheduleResponse { promote_task_ids }
}

#[cfg(test)]
#[path = "task_pool_tests.rs"]
mod task_pool_tests;
