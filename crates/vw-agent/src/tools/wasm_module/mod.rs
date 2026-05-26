//! WASM 模块工具
//!
//! 本模块提供列出和执行沙箱化 WASM 模块的功能。该工具仅在 WASM 运行时环境中可用。
//!
//! # 功能概述
//!
//! - 列出可用的 WASM 模块：扫描 `runtime.wasm.tools_dir` 目录下的 `.wasm` 文件
//! - 执行 WASM 模块：在严格的沙箱限制下运行指定的 WASM 模块
//!
//! # 安全性
//!
//! 所有 WASM 模块的执行都受到安全策略的约束，包括：
//! - 速率限制：防止过度使用资源
//! - 能力限制：文件系统访问、网络访问等需显式授权
//! - 资源限制：燃料（fuel）和内存使用上限

use super::traits::{Tool, ToolResult};
use crate::app::agent::runtime::{RuntimeAdapter, WasmCapabilities, WasmRuntime};
use crate::app::agent::security::SecurityPolicy;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

/// WASM 模块工具
///
/// 提供列出和执行沙箱化 WASM 模块的能力。该工具与运行时适配器紧密集成，
/// 仅在配置为 WASM 运行时模式时才可用。
///
/// # 示例
///
/// ```ignore
/// use std::sync::Arc;
/// use crate::app::agent::tools::WasmModuleTool;
/// use crate::app::agent::runtime::WasmRuntime;
/// use crate::app::agent::security::SecurityPolicy;
///
/// let security = Arc::new(SecurityPolicy::default());
/// let runtime = Arc::new(WasmRuntime::new(/* ... */));
/// let tool = WasmModuleTool::new(security, runtime);
/// ```
pub struct WasmModuleTool {
    /// 安全策略引用，用于速率限制和权限检查
    security: Arc<SecurityPolicy>,
    /// 运行时适配器引用，用于执行 WASM 模块操作
    runtime: Arc<dyn RuntimeAdapter>,
}

impl WasmModuleTool {
    /// 创建新的 WASM 模块工具实例
    ///
    /// # 参数
    ///
    /// - `security`: 安全策略的 Arc 引用，用于速率限制和权限检查
    /// - `runtime`: 运行时适配器的 Arc 引用，用于实际的 WASM 模块操作
    ///
    /// # 返回值
    ///
    /// 返回初始化后的 `WasmModuleTool` 实例
    pub fn new(security: Arc<SecurityPolicy>, runtime: Arc<dyn RuntimeAdapter>) -> Self {
        Self { security, runtime }
    }

    /// 尝试获取底层的 WASM 运行时引用
    ///
    /// 通过类型向下转换（downcast）尝试将通用的 `RuntimeAdapter` 转换为具体的 `WasmRuntime`。
    /// 如果运行时不是 WASM 类型，则返回 `None`。
    ///
    /// # 返回值
    ///
    /// - `Some(&WasmRuntime)`: 如果运行时适配器是 WASM 运行时
    /// - `None`: 如果运行时适配器不是 WASM 运行时
    fn wasm_runtime(&self) -> Option<&WasmRuntime> {
        self.runtime.as_any().downcast_ref::<WasmRuntime>()
    }

    /// 从 JSON 参数中解析 WASM 能力配置
    ///
    /// 从工具执行参数中提取并构建 `WasmCapabilities` 结构体。
    /// 该方法处理所有能力相关的配置项，包括文件系统访问权限、网络访问权限以及资源限制覆盖。
    ///
    /// # 参数
    ///
    /// - `args`: JSON 格式的工具参数，包含以下可选字段：
    ///   - `read_workspace`: 是否允许读取工作区（默认 false）
    ///   - `write_workspace`: 是否允许写入工作区（默认 false）
    ///   - `fuel_override`: 燃料限制覆盖值（默认 0，表示不覆盖）
    ///   - `memory_override_mb`: 内存限制覆盖值（默认 0，表示不覆盖）
    ///   - `allowed_hosts`: 允许访问的主机白名单数组
    ///
    /// # 返回值
    ///
    /// - `Ok(WasmCapabilities)`: 成功解析的能力配置
    /// - `Err`: 如果 `allowed_hosts` 字段格式不正确
    ///
    /// # 错误
    ///
    /// 当 `allowed_hosts` 存在但不是字符串数组时返回错误
    fn parse_caps(args: &serde_json::Value) -> anyhow::Result<WasmCapabilities> {
        // 解析工作区读取权限，默认为 false
        let read_workspace =
            args.get("read_workspace").and_then(serde_json::Value::as_bool).unwrap_or(false);
        // 解析工作区写入权限，默认为 false
        let write_workspace =
            args.get("write_workspace").and_then(serde_json::Value::as_bool).unwrap_or(false);
        // 解析燃料限制覆盖值，默认为 0（不覆盖）
        let fuel_override =
            args.get("fuel_override").and_then(serde_json::Value::as_u64).unwrap_or(0);
        // 解析内存限制覆盖值，默认为 0（不覆盖）
        let memory_override_mb =
            args.get("memory_override_mb").and_then(serde_json::Value::as_u64).unwrap_or(0);

        // 解析允许访问的主机白名单
        let allowed_hosts = match args.get("allowed_hosts") {
            Some(value) => {
                // 验证 allowed_hosts 必须是数组
                let arr = value.as_array().ok_or_else(|| {
                    anyhow::anyhow!("'allowed_hosts' must be an array of strings")
                })?;
                let mut hosts = Vec::with_capacity(arr.len());
                // 遍历数组，提取每个主机名
                for entry in arr {
                    let host = entry
                        .as_str()
                        .ok_or_else(|| {
                            anyhow::anyhow!("'allowed_hosts' must be an array of strings")
                        })?
                        .trim() // 去除首尾空白
                        .to_string();
                    // 只添加非空主机名
                    if !host.is_empty() {
                        hosts.push(host);
                    }
                }
                hosts
            }
            None => Vec::new(),
        };

        // 构建并返回能力配置结构体
        Ok(WasmCapabilities {
            read_workspace,
            write_workspace,
            allowed_hosts,
            fuel_override,
            memory_override_mb,
        })
    }
}

/// Tool trait 实现
///
/// 为 WASM 模块工具实现 Tool trait，使其能够被代理系统调用。
/// 该实现支持两种操作：列出可用模块和执行指定模块。
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for WasmModuleTool {
    /// 获取工具名称
    ///
    /// # 返回值
    ///
    /// 返回固定字符串 "wasm_module"，用于工具注册和调用时的标识
    fn name(&self) -> &str {
        "wasm_module"
    }

    /// 获取工具描述
    ///
    /// # 返回值
    ///
    /// 返回工具的功能描述字符串，说明该工具用于列出或执行沙箱化的 WASM 模块
    fn description(&self) -> &str {
        "列出或执行来自 runtime.wasm.tools_dir 的沙箱化 WASM 模块"
    }

    /// 获取工具参数的 JSON Schema
    ///
    /// 定义工具接受的参数结构，包括操作类型、模块名称以及各种能力请求参数。
    ///
    /// # 返回值
    ///
    /// 返回描述参数结构的 JSON Schema 对象，包含以下字段定义：
    /// - `action`: 必需字段，指定操作类型（"list" 或 "run"）
    /// - `module`: 执行操作时必需，指定要运行的 WASM 模块名称
    /// - `read_workspace`: 可选，请求文件系统读取权限
    /// - `write_workspace`: 可选，请求文件系统写入权限
    /// - `allowed_hosts`: 可选，指定允许访问的网络主机列表
    /// - `fuel_override`: 可选，覆盖默认的燃料限制
    /// - `memory_override_mb`: 可选，覆盖默认的内存限制
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list", "run"],
                    "description": "要执行的操作：list 列出模块，run 运行模块"
                },
                "module": {
                    "type": "string",
                    "description": "WASM 模块名称（不含 .wasm 扩展名），action=run 时必需"
                },
                "read_workspace": {
                    "type": "boolean",
                    "description": "请求 read_workspace 能力（必须被运行时策略允许）"
                },
                "write_workspace": {
                    "type": "boolean",
                    "description": "请求 write_workspace 能力（必须被运行时策略允许）"
                },
                "allowed_hosts": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "本次调用请求的主机白名单子集"
                },
                "fuel_override": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "可选的燃料覆盖值；不能超过 runtime.wasm.fuel_limit"
                },
                "memory_override_mb": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "可选的内存覆盖值（MB）；不能超过 runtime.wasm.memory_limit_mb"
                }
            },
            "required": ["action"]
        })
    }

    /// 执行工具操作
    ///
    /// 根据提供的参数执行相应的 WASM 模块操作。支持两种操作：
    /// - "list": 列出可用的 WASM 模块
    /// - "run": 执行指定的 WASM 模块
    ///
    /// # 参数
    ///
    /// - `args`: JSON 格式的工具参数，必须包含 `action` 字段
    ///
    /// # 返回值
    ///
    /// 返回 `ToolResult` 结构体，包含：
    /// - `success`: 操作是否成功
    /// - `output`: 操作输出（JSON 格式）
    /// - `error`: 错误信息（如果操作失败）
    ///
    /// # 错误处理
    ///
    /// 该方法在以下情况返回失败结果：
    /// - 缺少必需的 `action` 参数
    /// - 触发速率限制
    /// - 运行时不是 WASM 类型
    /// - `action=run` 时缺少 `module` 参数
    /// - 模块执行失败
    /// - 不支持的操作类型
    ///
    /// # 示例
    ///
    /// 列出模块：
    /// ```json
    /// {
    ///   "action": "list"
    /// }
    /// ```
    ///
    /// 执行模块：
    /// ```json
    /// {
    ///   "action": "run",
    ///   "module": "my_module",
    ///   "read_workspace": true,
    ///   "allowed_hosts": ["api.example.com"]
    /// }
    /// ```
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // 提取并验证 action 参数
        let action = args
            .get("action")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing 'action' parameter"))?;

        // 检查速率限制 - 检查是否超过小时限制
        if self.security.is_rate_limited() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Rate limit exceeded: too many actions in the last hour".into()),
            });
        }

        // 记录本次操作并检查是否耗尽操作预算
        if !self.security.record_action() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Rate limit exceeded: action budget exhausted".into()),
            });
        }

        // 尝试获取 WASM 运行时引用，如果不是 WASM 运行时则返回错误
        let Some(wasm_runtime) = self.wasm_runtime() else {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(
                    "wasm_module tool is only available when runtime.kind = \"wasm\"".into(),
                ),
            });
        };

        // 根据 action 类型执行相应操作
        match action {
            // 列出可用的 WASM 模块
            "list" => match wasm_runtime.list_modules(&self.security.workspace_dir) {
                // 成功获取模块列表，返回 JSON 格式的模块名数组
                Ok(modules) => Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({ "modules": modules }))?,
                    error: None,
                }),
                // 列表失败，返回错误信息
                Err(err) => Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(err.to_string()),
                }),
            },
            // 执行指定的 WASM 模块
            "run" => {
                // 提取必需的 module 参数
                let module = args
                    .get("module")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("Missing 'module' parameter for action=run"))?;

                // 解析能力配置参数
                let caps = Self::parse_caps(&args)?;

                // 执行 WASM 模块
                match wasm_runtime.execute_module(module, &self.security.workspace_dir, &caps) {
                    // 执行成功，构建详细的输出结果
                    Ok(result) => {
                        // 将执行结果格式化为 JSON 输出
                        let output = serde_json::to_string_pretty(&json!({
                            "module": module,
                            "module_sha256": result.module_sha256,
                            "exit_code": result.exit_code,
                            "fuel_consumed": result.fuel_consumed,
                            "stdout": result.stdout,
                            "stderr": result.stderr
                        }))?;

                        // 根据退出码判断是否成功
                        let success = result.exit_code == 0;

                        // 构建错误信息：如果执行失败但没有 stderr，使用通用错误消息
                        let error = if success {
                            None
                        } else if result.stderr.is_empty() {
                            Some(format!("WASM module exited with code {}", result.exit_code))
                        } else {
                            Some(result.stderr)
                        };

                        Ok(ToolResult { success, output, error })
                    }
                    // 模块执行失败，返回错误信息
                    Err(err) => Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(err.to_string()),
                    }),
                }
            }
            // 不支持的操作类型
            other => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Unsupported action '{other}'. Use 'list' or 'run'.")),
            }),
        }
    }
}

/// 单元测试模块
///
/// 包含 WasmModuleTool 的单元测试，测试文件位于 `tests/wasm_module.rs`
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
