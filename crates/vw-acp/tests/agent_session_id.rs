//! 代理会话 ID 规范化与提取测试。
//!
//! 用例锁定空白过滤和字段优先级，确保 ACP 与本地会话 ID 映射不会
//! 因输入差异产生空字符串或错误回退。

use serde_json::json;
use vw_acp::{AGENT_SESSION_ID_META_KEYS, extract_agent_session_id, normalize_agent_session_id};

/// 验证代理会话 ID 会被修剪，并拒绝空白字符串。
#[test]
fn normalize_agent_session_id_trims_and_validates_strings() {
    assert_eq!(
        normalize_agent_session_id(&json!("  session-123  ")),
        Some("session-123".to_string())
    );
    assert_eq!(normalize_agent_session_id(&json!("   ")), None);
    assert_eq!(normalize_agent_session_id(&json!(123)), None);
}

/// 验证提取逻辑优先使用 agentSessionId，其次才回退到 sessionId。
#[test]
fn extract_agent_session_id_prefers_agent_session_id_then_session_id() {
    let preferred = json!({
        "agentSessionId": " agent-session ",
        "sessionId": "fallback-session"
    });
    let fallback = json!({
        "sessionId": " fallback-session "
    });
    let invalid = json!({
        "sessionId": ""
    });

    assert_eq!(extract_agent_session_id(&preferred), Some("agent-session".to_string()));
    assert_eq!(extract_agent_session_id(&fallback), Some("fallback-session".to_string()));
    assert_eq!(extract_agent_session_id(&invalid), None);
    assert_eq!(AGENT_SESSION_ID_META_KEYS, &["agentSessionId", "sessionId"]);
}
