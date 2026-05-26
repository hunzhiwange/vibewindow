//! sandbox executor 的后端探测与命令包装测试。
//!
//! 测试不假设本机一定安装 sandbox 后端，只验证探测不会 panic，并根据后端实际状态检查
//! 构建结果，避免 CI 平台差异造成误报。

use std::path::PathBuf;

use crate::app::agent::runtime::{NativeRuntime, RuntimeAdapter};
use crate::tools::shell::sandbox::SandboxConfig;

use super::SandboxExecutor;

#[test]
fn sandbox_backend_detection_does_not_panic() {
    let _ = SandboxExecutor::backend_available();
}

#[test]
fn sandbox_executor_builds_runtime_command() {
    let executor = SandboxExecutor::new(SandboxConfig::for_workspace(PathBuf::from(".")));
    let runtime: Box<dyn RuntimeAdapter> = Box::new(NativeRuntime::new());

    let result =
        executor.build_command(runtime.as_ref(), "echo hello", PathBuf::from(".").as_path());

    if SandboxExecutor::backend_available() {
        assert!(result.is_ok());
    } else {
        assert!(result.is_err());
    }
}
