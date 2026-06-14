use super::*;
use crate::app::agent::runtime::NativeRuntime;
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::tools::traits::Tool;
use serde_json::json;
use std::sync::Arc;

#[test]
fn parse_caps_defaults_trims_hosts_and_rejects_bad_hosts() {
    let caps = WasmModuleTool::parse_caps(&json!({
        "read_workspace": true,
        "write_workspace": true,
        "fuel_override": 7,
        "memory_override_mb": 8,
        "allowed_hosts": [" api.test ", "", "cdn.test"]
    }))
    .unwrap();

    assert!(caps.read_workspace);
    assert!(caps.write_workspace);
    assert_eq!(caps.fuel_override, 7);
    assert_eq!(caps.memory_override_mb, 8);
    assert_eq!(caps.allowed_hosts, vec!["api.test", "cdn.test"]);
    assert!(WasmModuleTool::parse_caps(&json!({"allowed_hosts": "api.test"})).is_err());
    assert!(WasmModuleTool::parse_caps(&json!({"allowed_hosts": [1]})).is_err());
}

#[tokio::test]
async fn execute_validates_action_rate_limit_and_runtime_type() {
    let runtime = Arc::new(NativeRuntime::new());
    assert!(
        WasmModuleTool::new(Arc::new(SecurityPolicy::default()), runtime.clone())
            .execute(json!({}))
            .await
            .unwrap_err()
            .to_string()
            .contains("action")
    );

    let limited = WasmModuleTool::new(
        Arc::new(SecurityPolicy { max_actions_per_hour: 0, ..SecurityPolicy::default() }),
        runtime.clone(),
    )
    .execute(json!({"action": "list"}))
    .await
    .unwrap();
    assert!(!limited.success);
    assert!(limited.error.unwrap().contains("Rate limit"));

    let wrong_runtime = WasmModuleTool::new(Arc::new(SecurityPolicy::default()), runtime)
        .execute(json!({"action": "list"}))
        .await
        .unwrap();
    assert!(!wrong_runtime.success);
    assert!(wrong_runtime.error.unwrap().contains("runtime.kind"));
}

#[test]
fn schema_exposes_supported_actions() {
    let schema =
        WasmModuleTool::new(Arc::new(SecurityPolicy::default()), Arc::new(NativeRuntime::new()))
            .parameters_schema();
    assert_eq!(schema["required"], json!(["action"]));
    assert_eq!(schema["properties"]["action"]["enum"], json!(["list", "run"]));
}
