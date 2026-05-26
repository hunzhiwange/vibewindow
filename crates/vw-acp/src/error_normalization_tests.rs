use serde_json::json;

use super::error_normalization::*;
use super::types::{OutputErrorAcpPayload, OutputErrorCode, OutputErrorOrigin};

#[test]
fn metadata_overrides_default_code_and_origin() {
    let error = json!({
        "name": "TimeoutError",
        "message": "slow",
        "outputCode": "PERMISSION_DENIED",
        "detailCode": "POLICY",
        "origin": "queue",
        "retryable": false
    });

    let params = normalize_output_error(
        &error,
        NormalizeOutputErrorOptions {
            default_code: Some(OutputErrorCode::Runtime),
            origin: Some(OutputErrorOrigin::Acp),
            retryable: Some(true),
            ..NormalizeOutputErrorOptions::default()
        },
    );

    assert_eq!(params.code, OutputErrorCode::PermissionDenied);
    assert_eq!(params.detail_code.as_deref(), Some("POLICY"));
    assert_eq!(params.origin, Some(OutputErrorOrigin::Queue));
    assert_eq!(params.retryable, Some(false));
}

#[test]
fn auth_required_acp_payload_sets_detail_code() {
    let params = normalize_output_error(
        &json!({ "message": "authentication required" }),
        NormalizeOutputErrorOptions {
            acp: Some(OutputErrorAcpPayload {
                code: -32000,
                message: "auth required".to_string(),
                data: Some(json!({ "methodId": "token" })),
            }),
            ..NormalizeOutputErrorOptions::default()
        },
    );

    assert_eq!(params.detail_code.as_deref(), Some("AUTH_REQUIRED"));
    assert!(!is_retryable_prompt_error(&json!({
        "acp": { "code": -32000, "message": "auth required" }
    })));
}

#[test]
fn query_closed_detection_requires_internal_error_details() {
    assert!(is_acp_query_closed_before_response_error(&json!({
        "acp": {
            "code": -32603,
            "message": "internal error",
            "data": { "details": "Query closed before response received" }
        }
    })));
    assert!(!is_acp_query_closed_before_response_error(&json!({
        "acp": {
            "code": -32602,
            "message": "invalid params",
            "data": { "details": "Query closed before response received" }
        }
    })));
}
