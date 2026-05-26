//! 验证面向 CLI 输出的错误归一化规则。
//!
//! 这些测试确保错误 metadata、ACP JSON-RPC payload、重试判定和进程退出码之间
//! 的映射保持稳定，便于上层调用者可靠地区分运行时错误、认证错误和会话错误。

use serde_json::json;
use vw_acp::{
    NormalizeOutputErrorOptions, OutputErrorCode, OutputErrorOrigin,
    exit_code_for_output_error_code, is_acp_query_closed_before_response_error,
    is_retryable_prompt_error, normalize_output_error,
};

/// 验证错误对象自带的 metadata 优先于调用方提供的默认归一化选项。
#[test]
fn normalize_output_error_prefers_embedded_metadata() {
    let error = json!({
        "name": "TimeoutError",
        "message": "request timed out",
        "outputCode": "NO_SESSION",
        "detailCode": "FROM_META",
        "origin": "queue",
        "retryable": true,
        "acp": {
            "code": -32000,
            "message": "authentication required",
            "data": {
                "authRequired": true
            }
        }
    });

    let normalized = normalize_output_error(
        &error,
        NormalizeOutputErrorOptions {
            default_code: Some(OutputErrorCode::Runtime),
            detail_code: Some("FROM_OPTIONS".to_string()),
            origin: Some(OutputErrorOrigin::Runtime),
            retryable: Some(false),
            acp: None,
        },
    );

    assert_eq!(normalized.code, OutputErrorCode::NoSession);
    assert_eq!(normalized.message, "request timed out");
    assert_eq!(normalized.detail_code.as_deref(), Some("FROM_META"));
    assert_eq!(normalized.origin, Some(OutputErrorOrigin::Queue));
    assert_eq!(normalized.retryable, Some(true));
    assert_eq!(normalized.acp.as_ref().map(|payload| payload.code), Some(-32000));
}

/// 验证 ACP 认证 payload 会补充统一的 `AUTH_REQUIRED` 细分错误码。
#[test]
fn normalize_output_error_adds_auth_required_detail_from_acp_payload() {
    let error = json!({
        "message": "authorization required",
        "acp": {
            "code": -32000,
            "message": "authorization required",
            "data": {
                "methodId": "oauth"
            }
        }
    });

    let normalized = normalize_output_error(&error, NormalizeOutputErrorOptions::default());

    assert_eq!(normalized.code, OutputErrorCode::Runtime);
    assert_eq!(normalized.detail_code.as_deref(), Some("AUTH_REQUIRED"));
    assert_eq!(normalized.acp.as_ref().map(|payload| payload.code), Some(-32000));
}

/// 验证 ACP 查询提前关闭的特定错误文本能被识别出来，用于后续恢复或重试分支。
#[test]
fn query_closed_before_response_detection_matches_acp_details() {
    let error = json!({
        "error": {
            "code": -32603,
            "message": "internal error",
            "data": {
                "details": "Query closed before response received from agent"
            }
        }
    });

    assert!(is_acp_query_closed_before_response_error(&error));
}

/// 验证只有瞬时 ACP 失败会被视为可重试，参数错误、会话缺失和认证要求都不重试。
#[test]
fn retryable_prompt_error_only_accepts_transient_acp_failures() {
    let retryable_error = json!({
        "error": {
            "code": -32603,
            "message": "internal error"
        }
    });
    let invalid_params_error = json!({
        "error": {
            "code": -32602,
            "message": "invalid params"
        }
    });
    let no_session_error = json!({
        "error": {
            "code": -32002,
            "message": "Session \"abc-123\" not found"
        }
    });
    let auth_required_error = json!({
        "acp": {
            "code": -32000,
            "message": "token required",
            "data": {
                "methods": ["oauth"]
            }
        }
    });

    assert!(is_retryable_prompt_error(&retryable_error));
    assert!(!is_retryable_prompt_error(&invalid_params_error));
    assert!(!is_retryable_prompt_error(&no_session_error));
    assert!(!is_retryable_prompt_error(&auth_required_error));
}

/// 验证输出错误码到 CLI 退出码的映射，避免脚本集成依赖被无意破坏。
#[test]
fn exit_code_mapping_matches_output_code() {
    assert_eq!(exit_code_for_output_error_code(OutputErrorCode::Runtime), 1);
    assert_eq!(exit_code_for_output_error_code(OutputErrorCode::Usage), 2);
    assert_eq!(exit_code_for_output_error_code(OutputErrorCode::Timeout), 3);
    assert_eq!(exit_code_for_output_error_code(OutputErrorCode::NoSession), 4);
    assert_eq!(exit_code_for_output_error_code(OutputErrorCode::PermissionDenied), 5);
    assert_eq!(exit_code_for_output_error_code(OutputErrorCode::PermissionPromptUnavailable), 5);
}
