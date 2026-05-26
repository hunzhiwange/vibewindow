//! shell sandbox 决策逻辑测试。
//!
//! 覆盖全局禁用、上下文覆盖和包装命令排除，确保是否启用沙箱的原因保持可解释。

use std::path::PathBuf;

use crate::tools::shell::ast::parse_command;

use super::{SandboxConfig, SandboxReason, should_use_sandbox};

#[test]
fn sandbox_disabled_globally_short_circuits() {
    let mut config = SandboxConfig::for_workspace(PathBuf::from("/workspace"));
    config.enabled = false;

    let decision = should_use_sandbox(&parse_command("cat README.md"), &config);

    assert!(!decision.use_sandbox);
    assert_eq!(decision.reason, SandboxReason::DisabledGlobally);
}

#[test]
fn sandbox_override_disables_execution() {
    let mut config = SandboxConfig::for_workspace(PathBuf::from("/workspace"));
    config.allow_override = true;
    config.override_enabled = true;

    let decision = should_use_sandbox(&parse_command("cat README.md"), &config);

    assert!(!decision.use_sandbox);
    assert_eq!(decision.reason, SandboxReason::DisabledByOverride);
}

#[test]
fn sandbox_excluded_command_matches_wrapped_shell_command() {
    let mut config = SandboxConfig::for_workspace(PathBuf::from("/workspace"));
    config.excluded_commands = vec!["rg".into()];

    let decision =
        should_use_sandbox(&parse_command("env FOO=bar bash -lc 'rg needle src'"), &config);

    if super::executor::SandboxExecutor::backend_available() {
        assert!(!decision.use_sandbox);
        assert_eq!(decision.reason, SandboxReason::DisabledForCommand);
    } else {
        assert!(!decision.use_sandbox);
        assert_eq!(decision.reason, SandboxReason::BackendUnavailable);
    }
}
