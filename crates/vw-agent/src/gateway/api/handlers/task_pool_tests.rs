use super::*;
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
