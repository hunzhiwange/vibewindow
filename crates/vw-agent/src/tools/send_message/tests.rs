//! Agent IPC 消息发送测试。
//!
//! 这些测试使用临时 IPC 数据库验证团队路由、团队删除后的失败路径以及空载荷/缺少
//! 目标的输入校验。

use super::super::agents_ipc::{AgentsInboxTool, IpcDb};
use super::super::*;
use super::*;
use crate::app::agent::config::AgentsIpcConfig;
use crate::app::agent::security::SecurityPolicy;
use serde_json::{Value, json};
use std::sync::Arc;
use tempfile::TempDir;

fn test_ipc_db(root: &TempDir, workspace_name: &str) -> Arc<IpcDb> {
    let workspace = root.path().join(workspace_name);
    std::fs::create_dir_all(&workspace).expect("workspace should be created");
    // 多个 agent 共享同一个临时数据库，但使用不同 workspace 生成不同 agent_id。
    let config = AgentsIpcConfig {
        enabled: true,
        db_path: root.path().join("agents.db").to_string_lossy().to_string(),
        staleness_secs: 300,
    };
    Arc::new(IpcDb::open(&workspace, &config).expect("ipc db should open"))
}

#[tokio::test]
async fn team_send_routes_messages_to_all_members() {
    let root = TempDir::new().expect("temp dir should be created");
    let db_a = test_ipc_db(&root, "agent-a");
    let db_b = test_ipc_db(&root, "agent-b");
    let db_c = test_ipc_db(&root, "agent-c");
    let security = Arc::new(SecurityPolicy::default());

    let team_create = TeamCreateTool::new(db_a.clone(), security.clone());
    let send_tool = SendMessageTool::new(db_a.clone(), security);

    let created = team_create
        .execute(json!({
            "id": "reviewers",
            "members": [
                db_b.agent_id(),
                " ",
                db_c.agent_id(),
                db_b.agent_id()
            ]
        }))
        .await
        .expect("team create should return result");
    assert!(created.success);
    let created_json: Value =
        serde_json::from_str(&created.output).expect("team create output should be json");
    let members = created_json["members"].as_array().expect("members should be returned as array");
    assert_eq!(members.len(), 2);
    assert!(members.iter().any(|member| member == db_b.agent_id()));
    assert!(members.iter().any(|member| member == db_c.agent_id()));

    let sent = send_tool
        .execute(json!({
            "team_id": "reviewers",
            "payload": "hello reviewers"
        }))
        .await
        .expect("team send should return result");
    assert!(sent.success);
    let sent_json: Value = serde_json::from_str(&sent.output).expect("send output should be json");
    assert_eq!(sent_json["count"], 2);

    let inbox_b = AgentsInboxTool::new(db_b);
    let inbox_b_result = inbox_b.execute(json!({})).await.expect("inbox should be readable");
    let inbox_b_messages: Vec<Value> =
        serde_json::from_str(&inbox_b_result.output).expect("inbox output should be json");
    assert_eq!(inbox_b_messages.len(), 1);
    assert_eq!(inbox_b_messages[0]["payload"], "hello reviewers");

    let inbox_c = AgentsInboxTool::new(db_c);
    let inbox_c_result = inbox_c.execute(json!({})).await.expect("inbox should be readable");
    let inbox_c_messages: Vec<Value> =
        serde_json::from_str(&inbox_c_result.output).expect("inbox output should be json");
    assert_eq!(inbox_c_messages.len(), 1);
    assert_eq!(inbox_c_messages[0]["payload"], "hello reviewers");
}

#[tokio::test]
async fn team_delete_removes_message_route() {
    let root = TempDir::new().expect("temp dir should be created");
    let db_a = test_ipc_db(&root, "agent-a");
    let db_b = test_ipc_db(&root, "agent-b");
    let security = Arc::new(SecurityPolicy::default());

    let team_create = TeamCreateTool::new(db_a.clone(), security.clone());
    let team_delete = TeamDeleteTool::new(db_a.clone(), security.clone());
    let send_tool = SendMessageTool::new(db_a, security);

    let created = team_create
        .execute(json!({
            "id": "reviewers",
            "members": [db_b.agent_id()]
        }))
        .await
        .expect("team create should return result");
    assert!(created.success);

    let deleted = team_delete
        .execute(json!({ "id": "reviewers" }))
        .await
        .expect("team delete should return result");
    assert!(deleted.success);
    let deleted_json: Value =
        serde_json::from_str(&deleted.output).expect("team delete output should be json");
    assert_eq!(deleted_json["deleted"], true);

    let error = send_tool
        .execute(json!({
            "team_id": "reviewers",
            "payload": "should fail"
        }))
        .await
        .expect_err("deleted team should not resolve");
    assert!(error.to_string().contains("unknown team"));
}

#[tokio::test]
async fn send_message_rejects_blank_payload_and_missing_target() {
    let root = TempDir::new().expect("temp dir should be created");
    let db_a = test_ipc_db(&root, "agent-a");
    let db_b = test_ipc_db(&root, "agent-b");
    let send_tool = SendMessageTool::new(db_a, Arc::new(SecurityPolicy::default()));

    let blank_payload = send_tool
        .execute(json!({
            "to_agent": db_b.agent_id(),
            "payload": "   "
        }))
        .await
        .expect_err("blank payload should be rejected");
    assert!(blank_payload.to_string().contains("payload must not be empty"));

    let missing_target = send_tool
        .execute(json!({
            "payload": "hello"
        }))
        .await
        .expect_err("missing route should be rejected");
    assert!(missing_target.to_string().contains("either 'to_agent' or 'team_id' is required"));
}
