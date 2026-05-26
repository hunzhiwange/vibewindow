//! 验证持久化 JSON 记录的 key 命名策略。
//!
//! 会被写入磁盘或跨版本读取的数据需要保持稳定的 `snake_case` 形状；这些测试
//! 同时保留 ACP/Zed 协议负载中允许混合大小写的例外路径，避免把外部协议字段
//! 误判为本地持久化模型违规。

use serde_json::json;
use vw_acp::{assert_persisted_key_policy, find_persisted_key_policy_violations};

/// 协议透传区域和明确跳过路径中的混合大小写 key 不应触发本地策略错误。
#[test]
fn persisted_key_policy_allows_zed_tags_and_skipped_paths() {
    let value = json!({
        "messages": {
            "Agent": {
                "content": {
                    "ToolUse": {
                        "input": {
                            "camelCaseAllowed": true
                        }
                    }
                },
                "tool_results": {
                    "tool-1": {
                        "result": {
                            "camelCaseAllowed": true
                        }
                    }
                }
            }
        },
        "request_token_usage": {
            "inputTokens": 12,
            "outputTokens": 8
        },
        "agent_capabilities": {
            "mixedCaseCapability": true
        },
        "vwacp": {
            "config_options": {
                "mixedCaseOption": true
            }
        }
    });

    assert!(find_persisted_key_policy_violations(&value).is_empty());
    assert!(assert_persisted_key_policy(&value).is_ok());
}

/// 本地持久化层出现非 `snake_case` key 时，应返回完整路径便于定位迁移风险。
#[test]
fn persisted_key_policy_reports_non_snake_case_paths() {
    let value = json!({
        "badKey": true,
        "messages": {
            "Agent": {
                "badField": true
            }
        }
    });

    assert_eq!(
        find_persisted_key_policy_violations(&value),
        vec!["badKey".to_string(), "messages.Agent.badField".to_string()]
    );
}

/// 断言型 API 应把所有违规路径合并到错误消息中，方便调用者一次性展示问题。
#[test]
fn assert_persisted_key_policy_returns_joined_error_message() {
    let error = assert_persisted_key_policy(&json!({
        "badKey": true,
        "nested_value": {
            "badField": true
        }
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "Persisted key policy violation (expected snake_case keys): badKey, nested_value.badField"
    );
}
