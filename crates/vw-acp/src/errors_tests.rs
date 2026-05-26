use std::io;

use super::errors::{AcpxErrorOptions, AcpxOperationalError};
use super::types::{OutputErrorAcpPayload, OutputErrorCode, OutputErrorOrigin};

#[test]
fn options_with_defaults_preserves_explicit_values() {
    let options = AcpxErrorOptions {
        output_code: Some(OutputErrorCode::Timeout),
        detail_code: Some("EXPLICIT".to_string()),
        origin: Some(OutputErrorOrigin::Queue),
        ..AcpxErrorOptions::default()
    }
    .with_defaults(OutputErrorCode::Runtime, "DEFAULT", OutputErrorOrigin::Acp);

    assert_eq!(options.output_code, Some(OutputErrorCode::Timeout));
    assert_eq!(options.detail_code.as_deref(), Some("EXPLICIT"));
    assert_eq!(options.origin, Some(OutputErrorOrigin::Queue));
}

#[test]
fn operational_error_exposes_output_fields_and_source() {
    let acp =
        OutputErrorAcpPayload { code: -32603, message: "adapter failed".to_string(), data: None };
    let error = AcpxOperationalError::new(
        "visible message",
        AcpxErrorOptions {
            source: Some(Box::new(io::Error::other("inner"))),
            output_code: Some(OutputErrorCode::Runtime),
            detail_code: Some("DETAIL".to_string()),
            origin: Some(OutputErrorOrigin::Acp),
            retryable: Some(true),
            acp: Some(acp.clone()),
            output_already_emitted: true,
        },
    );

    assert_eq!(error.message(), "visible message");
    assert_eq!(error.to_string(), "visible message");
    assert_eq!(error.output_code(), Some(OutputErrorCode::Runtime));
    assert_eq!(error.detail_code(), Some("DETAIL"));
    assert_eq!(error.origin(), Some(OutputErrorOrigin::Acp));
    assert_eq!(error.retryable(), Some(true));
    assert_eq!(error.acp(), Some(&acp));
    assert!(error.output_already_emitted());
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn output_error_params_require_output_code() {
    let error = AcpxOperationalError::new("message", AcpxErrorOptions::default());

    assert!(error.to_output_error_params().is_none());
}
