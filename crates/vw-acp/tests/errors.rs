//! 验证 ACP 错误类型的展示文本、source 链和输出 metadata。
//!
//! 这些测试把错误类型当作对外契约检查：CLI、JSON formatter 和调用方会依赖
//! 固定的错误码、detail code、origin 与 retryable 语义。

use std::error::Error as _;
use std::io;

use serde_json::json;
use vw_acp::{
    AcpxErrorOptions, AgentDisconnectedError, AgentSpawnError, OutputErrorCode, OutputErrorOrigin,
    OutputErrorParams, PermissionPromptUnavailableError, SessionNotFoundError,
    SessionResumeRequiredError,
};

/// 验证会话不存在错误保留原始 session id，并且默认不携带输出错误码。
#[test]
fn session_not_found_error_keeps_session_id_and_message() {
    let error = SessionNotFoundError::new("session-123");

    assert_eq!(error.session_id, "session-123");
    assert_eq!(error.to_string(), "Session not found: session-123");
    assert!(error.output_code().is_none());
}

/// 验证 agent 启动失败时保留原始 IO 错误，方便调用方追踪底层原因。
#[test]
fn agent_spawn_error_wraps_original_source() {
    let error = AgentSpawnError::new("npx @agent", io::Error::other("spawn failed"));

    assert_eq!(error.agent_command, "npx @agent");
    assert_eq!(error.to_string(), "Failed to spawn agent command: npx @agent");
    assert_eq!(error.source().map(|source| source.to_string()), Some("spawn failed".to_string()));
}

/// 验证 agent 断连错误会生成稳定的默认输出 metadata，供 CLI formatter 使用。
#[test]
fn agent_disconnected_error_applies_default_output_metadata() {
    let error =
        AgentDisconnectedError::new("connection_close", None, None, AcpxErrorOptions::default());

    assert_eq!(
        error.to_string(),
        "ACP agent disconnected during request (connection_close, exit=null, signal=null)"
    );
    assert_eq!(error.output_code(), Some(OutputErrorCode::Runtime));
    assert_eq!(error.detail_code(), Some("AGENT_DISCONNECTED"));
    assert_eq!(error.origin(), Some(OutputErrorOrigin::Acp));

    assert_eq!(
        error.to_output_error_params(),
        Some(OutputErrorParams {
            code: OutputErrorCode::Runtime,
            detail_code: Some("AGENT_DISCONNECTED".to_string()),
            origin: Some(OutputErrorOrigin::Acp),
            message:
                "ACP agent disconnected during request (connection_close, exit=null, signal=null)"
                    .to_string(),
            retryable: None,
            acp: None,
            timestamp: None,
        })
    );
}

/// 验证需要恢复会话的错误默认可重试，并保留原始 ACP payload。
#[test]
fn session_resume_required_error_defaults_retryable_and_preserves_payload() {
    let error = SessionResumeRequiredError::new(
        "resume required",
        AcpxErrorOptions {
            acp: Some(vw_acp::OutputErrorAcpPayload {
                code: -32002,
                message: "session missing".to_string(),
                data: Some(json!({ "sessionId": "session-123" })),
            }),
            ..AcpxErrorOptions::default()
        },
    );

    assert_eq!(error.output_code(), Some(OutputErrorCode::Runtime));
    assert_eq!(error.detail_code(), Some("SESSION_RESUME_REQUIRED"));
    assert_eq!(error.origin(), Some(OutputErrorOrigin::Acp));
    assert_eq!(error.retryable(), Some(true));
    assert_eq!(error.acp().map(|payload| payload.code), Some(-32002));
}

/// 验证非交互模式权限提示不可用时，错误消息固定且不会被输出错误包装。
#[test]
fn permission_prompt_unavailable_error_has_fixed_message() {
    let error = PermissionPromptUnavailableError::new();

    assert_eq!(error.to_string(), "Permission prompt unavailable in non-interactive mode");
    assert!(error.to_output_error_params().is_none());
}
