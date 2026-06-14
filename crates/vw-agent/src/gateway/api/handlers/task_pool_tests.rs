use super::*;
use axum::Json;
use vw_api_types::task::{TaskPoolScheduleSettingsDto, TaskPoolScheduleTaskDto};

fn task(id: &str, status: TaskPoolStatus, priority: u32, order: u32) -> TaskPoolScheduleTaskDto {
    TaskPoolScheduleTaskDto {
        id: id.to_string(),
        status,
        priority,
        order,
        created_at_ms: 1_000,
        auto_promote_delay_ms: None,
        deleted: false,
        archived: false,
    }
}

fn request(tasks: Vec<TaskPoolScheduleTaskDto>) -> TaskPoolScheduleRequest {
    TaskPoolScheduleRequest {
        now_ms: 31_000,
        settings: TaskPoolScheduleSettingsDto {
            auto_execute: true,
            auto_promote_pool_tasks: true,
            max_concurrent: 1,
            auto_promote_delay_seconds: 30,
        },
        tasks,
    }
}

#[test]
fn schedule_task_pool_promotes_ready_tasks_by_priority() {
    let response = schedule_task_pool(&request(vec![
        task("slow", TaskPoolStatus::Pool, 9, 0),
        task("fast", TaskPoolStatus::Pool, 1, 0),
    ]));

    assert_eq!(response.promote_task_ids, vec!["fast", "slow"]);
}

#[test]
fn schedule_task_pool_respects_pending_capacity() {
    let response = schedule_task_pool(&request(vec![
        task("pending-a", TaskPoolStatus::Pending, 1, 0),
        task("pending-b", TaskPoolStatus::Pending, 2, 0),
        task("pool", TaskPoolStatus::Pool, 1, 0),
    ]));

    assert!(response.promote_task_ids.is_empty());
}

#[test]
fn schedule_task_pool_requires_both_auto_flags() {
    let mut request = request(vec![task("pool", TaskPoolStatus::Pool, 1, 0)]);
    request.settings.auto_execute = false;
    assert!(schedule_task_pool(&request).promote_task_ids.is_empty());

    request.settings.auto_execute = true;
    request.settings.auto_promote_pool_tasks = false;
    assert!(schedule_task_pool(&request).promote_task_ids.is_empty());
}

#[test]
fn schedule_task_pool_filters_ineligible_and_waiting_tasks() {
    let mut deleted = task("deleted", TaskPoolStatus::Pool, 1, 0);
    deleted.deleted = true;
    let mut archived = task("archived", TaskPoolStatus::Pool, 1, 1);
    archived.archived = true;
    let mut waiting = task("waiting", TaskPoolStatus::Pool, 1, 2);
    waiting.auto_promote_delay_ms = Some(60_000);

    let response = schedule_task_pool(&request(vec![
        deleted,
        archived,
        waiting,
        task("ready", TaskPoolStatus::Pool, 1, 3),
        task("done", TaskPoolStatus::Completed, 1, 4),
    ]));

    assert_eq!(response.promote_task_ids, vec!["ready"]);
}

#[test]
fn schedule_task_pool_treats_zero_concurrency_as_one_and_orders_by_priority_then_order() {
    let mut request = request(vec![
        task("third", TaskPoolStatus::Pool, 2, 0),
        task("second", TaskPoolStatus::Pool, 1, 9),
        task("first", TaskPoolStatus::Pool, 1, 1),
    ]);
    request.settings.max_concurrent = 0;

    let response = schedule_task_pool(&request);

    assert_eq!(response.promote_task_ids, vec!["first", "second"]);
}

#[test]
fn schedule_task_pool_counts_planning_as_pending_capacity() {
    let response = schedule_task_pool(&request(vec![
        task("planning-a", TaskPoolStatus::Planning, 1, 0),
        task("pending-b", TaskPoolStatus::Pending, 1, 1),
        task("pool", TaskPoolStatus::Pool, 1, 2),
    ]));

    assert!(response.promote_task_ids.is_empty());
}

#[tokio::test]
async fn task_pool_schedule_handler_returns_json_response() {
    let Json(response) =
        task_pool_schedule_v1(Json(request(vec![task("pool", TaskPoolStatus::Pool, 1, 0)])))
            .await
            .expect("handler should schedule");

    assert_eq!(response.promote_task_ids, vec!["pool"]);
}
