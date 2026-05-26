use super::errors::CoordinationError;

#[test]
fn coordination_error_display_includes_context_key() {
    let err = CoordinationError::ContextVersionMismatch {
        key: "shared/key".to_string(),
        expected: 1,
        actual: 2,
    };

    assert!(err.to_string().contains("shared/key"));
}
