#[test]
fn runtime_defaults_match_native_docker_and_wasm_profiles() {
    let runtime = super::RuntimeConfig::default();
    assert_eq!(runtime.kind, "native");
    assert_eq!(runtime.docker.image, "alpine:3.20");
    assert_eq!(runtime.docker.network, "none");
    assert_eq!(runtime.docker.memory_limit_mb, Some(512));
    assert_eq!(runtime.wasm.tools_dir, "tools/wasm");
    assert_eq!(runtime.wasm.fuel_limit, 1_000_000);
    assert_eq!(
        runtime.wasm.security.capability_escalation_mode,
        super::WasmCapabilityEscalationMode::Deny
    );
}

#[test]
fn runtime_deserializes_reasoning_and_wasm_security_values() {
    let parsed: super::RuntimeConfig = serde_json::from_value(serde_json::json!({
        "reasoning_enabled": true,
        "reasoning_level": "high",
        "wasm": {
            "security": {
                "capability_escalation_mode": "clamp"
            }
        }
    }))
    .unwrap();

    assert_eq!(parsed.reasoning_enabled, Some(true));
    assert_eq!(parsed.reasoning_level.as_deref(), Some("high"));
    assert_eq!(
        parsed.wasm.security.capability_escalation_mode,
        super::WasmCapabilityEscalationMode::Clamp
    );
}
