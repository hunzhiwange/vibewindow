#[test]
fn observability_defaults_are_stable() {
    let config = super::ObservabilityConfig::default();
    assert_eq!(config.backend, "none");
    assert_eq!(config.runtime_trace_mode, "none");
    assert_eq!(config.runtime_trace_path, "state/runtime-trace.jsonl");
    assert_eq!(config.runtime_trace_max_entries, 200);
}

#[test]
fn observability_deserializes_optional_otlp_fields() {
    let parsed: super::ObservabilityConfig = serde_json::from_value(serde_json::json!({
        "backend": "otel",
        "otel_endpoint": "http://localhost:4318",
        "otel_service_name": "vw"
    }))
    .unwrap();

    assert_eq!(parsed.otel_endpoint.as_deref(), Some("http://localhost:4318"));
    assert_eq!(parsed.otel_service_name.as_deref(), Some("vw"));
}
