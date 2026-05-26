use super::*;
use crate::OutputErrorCode;

#[derive(Debug, thiserror::Error)]
#[error("outer: {0}")]
struct OuterError(#[source] InterruptedError);

#[derive(Debug, thiserror::Error)]
#[error("plain failure")]
struct PlainError;

#[test]
fn from_source_marks_interrupted_errors_in_source_chain() {
    let error = SessionRuntimeError::from_source(OuterError(InterruptedError));

    assert!(error.is_interrupted());
    assert_eq!(error.to_string(), "outer: Interrupted");
    assert!(error.source().is_some());
}

#[test]
fn output_already_emitted_is_configurable() {
    let error = SessionRuntimeError::from_source(PlainError).with_output_already_emitted(true);

    assert!(error.output_already_emitted());
}

#[test]
fn normalize_runtime_error_maps_timeout_to_retryable_timeout_output() {
    let output = normalize_runtime_error(
        &TimeoutError { timeout_ms: 25 },
        Some("detail"),
        Some(OutputErrorOrigin::Runtime),
    );

    assert_eq!(output.code, OutputErrorCode::Timeout);
    assert_eq!(output.detail_code.as_deref(), Some("detail"));
    assert_eq!(output.retryable, Some(true));
    assert_eq!(output.origin, Some(OutputErrorOrigin::Runtime));
}

#[test]
fn output_params_uses_runtime_fallback_for_plain_errors() {
    let output = SessionRuntimeError::from_source(PlainError).output_params();

    assert_eq!(output.code, OutputErrorCode::Runtime);
    assert_eq!(output.origin, Some(OutputErrorOrigin::Runtime));
    assert_eq!(output.message, "plain failure");
}
