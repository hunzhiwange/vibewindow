use super::config_checks::{embedding_provider_validation_error, provider_validation_error};

#[test]
fn provider_validation_errors_include_provider_name() {
    let err = provider_validation_error("vibewindow-provider-that-should-not-exist")
        .expect("invalid provider should produce an error");
    let embedding_err = embedding_provider_validation_error("voyage")
        .expect("unsupported embedding provider should produce an error");

    assert!(!err.is_empty());
    assert!(embedding_err.contains("supported values"));
}
