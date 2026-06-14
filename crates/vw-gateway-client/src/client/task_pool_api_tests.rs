use serde_json::json;
use vw_api_types::task::{
    TaskPoolScheduleRequest, TaskPoolScheduleSettingsDto, TaskPoolScheduleTaskDto, TaskPoolStatus,
};

use crate::client::test_support;

#[tokio::test]
async fn task_pool_schedule_posts_snapshot_and_reads_decision() {
    let server =
        test_support::server(vec![(200, json!({"promote_task_ids": ["task-a", "task-b"]}))]);
    let request = TaskPoolScheduleRequest {
        now_ms: 20_000,
        settings: TaskPoolScheduleSettingsDto {
            auto_execute: true,
            auto_promote_pool_tasks: true,
            max_concurrent: 2,
            auto_promote_delay_seconds: 30,
        },
        tasks: vec![TaskPoolScheduleTaskDto {
            id: "task-a".to_string(),
            status: TaskPoolStatus::Pool,
            priority: 3,
            order: 1,
            created_at_ms: 10_000,
            auto_promote_delay_ms: Some(5_000),
            deleted: false,
            archived: false,
        }],
    };

    let response = server.client().task_pool_schedule(&request).await.expect("schedule");

    assert_eq!(response.promote_task_ids, vec!["task-a".to_string(), "task-b".to_string()]);
    let recorded = server.take_request();
    assert_eq!(recorded.method, "POST");
    assert_eq!(recorded.path, "/v1/task-pool/schedule");
    assert_eq!(recorded.body["now_ms"], 20_000);
    assert_eq!(recorded.body["tasks"][0]["status"], "pool");
    server.join();
}
