use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HooksConfig {
    /// 是否启用生命周期 hook 执行。
    ///
    /// hook 与主运行时在同一进程内执行，并共享相同权限。
    /// 已启用的 hook 处理器应保持最小职责并易于审计。
    pub enabled: bool,
    /// 内置 hook 的开关配置。
    #[serde(default)]
    pub builtin: BuiltinHooksConfig,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self { enabled: true, builtin: BuiltinHooksConfig::default() }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct BuiltinHooksConfig {
    /// 是否启用 command-logger hook，用于记录工具调用以便审计。
    pub command_logger: bool,
}
#[cfg(test)]
#[path = "hooks_tests.rs"]
mod hooks_tests;
