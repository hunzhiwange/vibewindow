//! WasmModule 工具测试。
//!
//! 覆盖工具名称、模块列表过滤、运行参数校验和运行时类型保护，避免 wasm 工具在非 wasm
//! 运行时或非法模块名下暴露额外能力。

use super::super::*;
use crate::app::agent::config::WasmRuntimeConfig;
use crate::app::agent::runtime::{NativeRuntime, WasmRuntime};
use crate::app::agent::security::{AutonomyLevel, SecurityPolicy};
use serde_json::json;

fn test_security(workspace_dir: std::path::PathBuf) -> Arc<SecurityPolicy> {
    Arc::new(SecurityPolicy {
        autonomy: AutonomyLevel::Full,
        workspace_dir,
        ..SecurityPolicy::default()
    })
}

#[test]
fn wasm_module_tool_name() {
    let dir = tempfile::tempdir().unwrap();
    let security = test_security(dir.path().to_path_buf());
    let runtime: Arc<dyn RuntimeAdapter> = Arc::new(WasmRuntime::new(WasmRuntimeConfig::default()));
    let tool = WasmModuleTool::new(security, runtime);
    assert_eq!(tool.name(), "wasm_module");
}

#[tokio::test]
async fn list_action_returns_modules() {
    let dir = tempfile::tempdir().unwrap();
    let tools_dir = dir.path().join("tools/wasm");
    std::fs::create_dir_all(&tools_dir).unwrap();
    std::fs::write(tools_dir.join("alpha.wasm"), b"\0asm").unwrap();
    std::fs::write(tools_dir.join("beta.wasm"), b"\0asm").unwrap();
    std::fs::write(tools_dir.join("bad$name.wasm"), b"\0asm").unwrap();

    let security = test_security(dir.path().to_path_buf());
    let runtime: Arc<dyn RuntimeAdapter> = Arc::new(WasmRuntime::new(WasmRuntimeConfig::default()));
    let tool = WasmModuleTool::new(security, runtime);

    let result = tool.execute(json!({"action": "list"})).await.unwrap();
    assert!(result.success);
    assert!(result.output.contains("alpha"));
    assert!(result.output.contains("beta"));
    assert!(!result.output.contains("bad$name"));
}

#[tokio::test]
async fn run_action_requires_module() {
    let dir = tempfile::tempdir().unwrap();
    let security = test_security(dir.path().to_path_buf());
    let runtime: Arc<dyn RuntimeAdapter> = Arc::new(WasmRuntime::new(WasmRuntimeConfig::default()));
    let tool = WasmModuleTool::new(security, runtime);

    let result = tool.execute(json!({"action": "run"})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("module"));
}

#[tokio::test]
async fn run_action_errors_without_runtime_wasm_feature() {
    if WasmRuntime::is_available() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let tools_dir = dir.path().join("tools/wasm");
    std::fs::create_dir_all(&tools_dir).unwrap();
    std::fs::write(tools_dir.join("hello.wasm"), b"\0asm\x01\0\0\0").unwrap();

    let security = test_security(dir.path().to_path_buf());
    let runtime: Arc<dyn RuntimeAdapter> = Arc::new(WasmRuntime::new(WasmRuntimeConfig::default()));
    let tool = WasmModuleTool::new(security, runtime);

    let result = tool.execute(json!({"action": "run", "module": "hello"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.unwrap_or_default().contains("not available"));
}

#[tokio::test]
async fn tool_rejects_non_wasm_runtime() {
    let dir = tempfile::tempdir().unwrap();
    let security = test_security(dir.path().to_path_buf());
    let runtime: Arc<dyn RuntimeAdapter> = Arc::new(NativeRuntime::new());
    let tool = WasmModuleTool::new(security, runtime);

    let result = tool.execute(json!({"action": "list"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.unwrap_or_default().contains("runtime.kind = \"wasm\""));
}
