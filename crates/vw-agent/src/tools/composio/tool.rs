//! Composio 工具的 Tool trait 实现
//!
//! 本模块为 `ComposioTool` 实现 `Tool` trait，提供与 Composio 平台的交互能力。
//! Composio 是一个统一的应用集成平台，支持 1000+ 第三方应用（如 Gmail、Notion、GitHub、Slack 等）。
//!
//! # 功能概述
//!
//! - **列出可用操作**：查询 Composio 平台支持的所有工具和操作
//! - **列出已连接账户**：查看当前实体下已授权的 OAuth 账户
//! - **执行操作**：在指定应用上执行具体操作（如发送邮件、创建任务等）
//! - **获取授权链接**：生成 OAuth 连接 URL，用于新应用的授权
//!
//! # 安全考虑
//!
//! - `execute` 和 `connect` 操作需要通过安全策略检查
//! - 支持多实体（entity）隔离，每个用户使用独立的 `entity_id`
//! - 敏感操作会在执行前进行权限验证

use super::core::ComposioTool;
use super::util;
use crate::app::agent::security::policy::ToolOperation;
use crate::app::agent::tools::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::json;
use std::fmt::Write;

/// 为 ComposioTool 实现 Tool trait
///
/// 该实现将 Composio 平台的操作能力封装为统一的工具接口，
/// 使代理能够通过标准化的方式与 1000+ 应用进行交互。
/// 所有操作都通过异步方式执行，以适应网络 I/O 的特性。
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for ComposioTool {
    /// 返回工具的名称标识符
    ///
    /// 该名称用于工具注册和调用时的唯一标识。
    fn name(&self) -> &str {
        "composio"
    }

    /// 返回工具的详细描述
    ///
    /// 描述中包含工具的基本功能说明以及各操作的用法指南，
    /// 帮助 LLM 理解如何正确使用此工具。
    fn description(&self) -> &str {
        "通过 Composio 在 1000+ 应用上执行操作（Gmail、Notion、GitHub、Slack 等）。\
         使用 action='list' 查看可用操作（包含参数名称）。\
         使用 action='execute' 配合 action_name/tool_slug 和 params 运行操作。\
         如果不确定确切参数，可传递 'text' 替代自然语言描述（Composio 将通过 NLP 解析正确参数）。\
         action='list_accounts' 或 action='connected_accounts' 列出 OAuth 连接的账户。\
         action='connect' 配合 app/auth_config_id 获取 OAuth URL。\
         省略 connected_account_id 时自动解析。"
    }

    /// 返回工具参数的 JSON Schema 定义
    ///
    /// 该 Schema 描述了 `execute` 方法接受的参数结构，
    /// 包括参数类型、描述、枚举值和必需字段等信息。
    ///
    /// # 参数说明
    ///
    /// - `action`：必需参数，指定要执行的操作类型
    /// - `app`：可选参数，应用标识符，用于过滤或指定目标应用
    /// - `action_name` / `tool_slug`：执行操作时必需，指定要执行的具体操作
    /// - `params`：可选参数，结构化的操作参数
    /// - `text`：可选参数，自然语言描述，与 `params` 互斥
    /// - `entity_id`：可选参数，多用户场景下的用户标识
    /// - `auth_config_id`：可选参数，授权配置 ID
    /// - `connected_account_id`：可选参数，指定使用的已连接账户
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "The operation: 'list' (list available actions), 'list_accounts'/'connected_accounts' (list connected accounts), 'execute' (run an action), or 'connect' (get OAuth URL)",
                    "enum": ["list", "list_accounts", "connected_accounts", "execute", "connect"]
                },
                "app": {
                    "type": "string",
                    "description": "Toolkit slug filter for 'list' or 'list_accounts', optional app hint for 'execute', or toolkit/app for 'connect' (e.g. 'gmail', 'notion', 'github')"
                },
                "action_name": {
                    "type": "string",
                    "description": "Action/tool identifier to execute (legacy aliases supported)"
                },
                "tool_slug": {
                    "type": "string",
                    "description": "Preferred v3 tool slug to execute (alias of action_name)"
                },
                "params": {
                    "type": "object",
                    "description": "Structured parameters to pass to the action (use the key names shown by action='list')"
                },
                "text": {
                    "type": "string",
                    "description": "Natural-language description of what you want the action to do (alternative to 'params' when you are unsure of the exact parameter names). Composio will resolve the correct parameters via NLP. Mutually exclusive with 'params'."
                },
                "entity_id": {
                    "type": "string",
                    "description": "Entity/user ID for multi-user setups (defaults to composio.entity_id from config)"
                },
                "auth_config_id": {
                    "type": "string",
                    "description": "Optional Composio v3 auth config id for connect flow"
                },
                "connected_account_id": {
                    "type": "string",
                    "description": "Optional connected account ID for execute flow when a specific account is required"
                }
            },
            "required": ["action"]
        })
    }

    /// 执行工具操作
    ///
    /// 根据传入的 `action` 参数执行相应的 Composio 操作。
    /// 支持的操作包括：列出可用操作、列出已连接账户、执行应用操作、获取授权链接。
    ///
    /// # 参数
    ///
    /// - `args`：JSON 格式的参数对象，必须包含 `action` 字段
    ///
    /// # 返回值
    ///
    /// 返回 `ToolResult`，包含：
    /// - `success`：操作是否成功
    /// - `output`：成功时的输出内容
    /// - `error`：失败时的错误信息
    ///
    /// # 支持的 action 类型
    ///
    /// - `"list"`：列出可用的 Composio 操作
    /// - `"list_accounts"` / `"connected_accounts"`：列出已连接的 OAuth 账户
    /// - `"execute"`：执行具体的 Composio 操作（需通过安全检查）
    /// - `"connect"`：获取 OAuth 授权链接（需通过安全检查）
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // 提取必需的 action 参数
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'action' parameter"))?;

        // 获取 entity_id，如果未指定则使用默认值
        let entity_id =
            args.get("entity_id").and_then(|v| v.as_str()).unwrap_or(self.default_entity_id());

        match action {
            // 列出可用的 Composio 操作
            "list" => {
                let app = args.get("app").and_then(|v| v.as_str());
                match self.list_actions(app).await {
                    Ok(actions) => {
                        // 格式化前 20 个操作为可读的摘要列表
                        let summary: Vec<String> = actions
                            .iter()
                            .take(20)
                            .map(|a| {
                                // 获取参数提示信息
                                let params_hint =
                                    util::format_input_params_hint(a.input_parameters.as_ref());
                                format!(
                                    "- {} ({}): {}{}",
                                    a.name,
                                    a.app_name.as_deref().unwrap_or("?"),
                                    a.description.as_deref().unwrap_or(""),
                                    params_hint,
                                )
                            })
                            .collect();
                        let total = actions.len();
                        // 构建输出字符串，如果超过 20 个则显示省略提示
                        let output = format!(
                            "Found {total} available actions:\n{}{}",
                            summary.join("\n"),
                            if total > 20 {
                                format!("\n... and {} more", total - 20)
                            } else {
                                String::new()
                            }
                        );
                        Ok(ToolResult { success: true, output, error: None })
                    }
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to list actions: {e}")),
                    }),
                }
            }

            // 列出已连接的 OAuth 账户（支持两种拼写）
            "list_accounts" | "connected_accounts" => {
                let app = args.get("app").and_then(|v| v.as_str());
                match self.list_connected_accounts(app, Some(entity_id)).await {
                    Ok(accounts) => {
                        // 如果没有已连接的账户，返回提示信息
                        if accounts.is_empty() {
                            let app_hint =
                                app.map(|value| format!(" for app '{value}'")).unwrap_or_default();
                            return Ok(ToolResult {
                                success: true,
                                output: format!(
                                    "No connected accounts found{app_hint} for entity '{entity_id}'. Run action='connect' first."
                                ),
                                error: None,
                            });
                        }

                        // 格式化前 20 个账户为可读的摘要列表
                        let summary: Vec<String> = accounts
                            .iter()
                            .take(20)
                            .map(|account| {
                                let toolkit = account.toolkit_slug().unwrap_or("?");
                                format!("- {} [{}] toolkit={toolkit}", account.id, account.status)
                            })
                            .collect();
                        let total = accounts.len();
                        // 构建输出字符串，包含使用提示
                        let output = format!(
                            "Found {total} connected accounts (entity '{entity_id}'):\n{}{}\nUse connected_account_id in action='execute' when needed.",
                            summary.join("\n"),
                            if total > 20 {
                                format!("\n... and {} more", total - 20)
                            } else {
                                String::new()
                            }
                        );
                        Ok(ToolResult { success: true, output, error: None })
                    }
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to list connected accounts: {e}")),
                    }),
                }
            }

            // 执行具体的 Composio 操作
            "execute" => {
                // 执行安全策略检查，确保有权限执行此操作
                if let Err(error) =
                    self.security().enforce_tool_operation(ToolOperation::Act, "composio.execute")
                {
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(error),
                    });
                }

                // 获取要执行的操作名称（支持 tool_slug 和 action_name 两种参数名）
                let action_name = args
                    .get("tool_slug")
                    .or_else(|| args.get("action_name"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing 'action_name' (or 'tool_slug') for execute")
                    })?;

                // 提取可选参数
                let app = args.get("app").and_then(|v| v.as_str());
                let params = args.get("params").cloned().unwrap_or(json!({}));
                let text = args.get("text").and_then(|v| v.as_str());
                let acct_ref = args.get("connected_account_id").and_then(|v| v.as_str());

                // 执行操作
                match self
                    .execute_action(action_name, app, params, text, Some(entity_id), acct_ref)
                    .await
                {
                    Ok(result) => {
                        // 将结果格式化为 JSON 字符串
                        let output = serde_json::to_string_pretty(&result)
                            .unwrap_or_else(|_| format!("{result:?}"));
                        Ok(ToolResult { success: true, output, error: None })
                    }
                    Err(e) => {
                        // 执行失败时，尝试获取工具的参数 schema 以帮助 LLM 自我修正
                        let schema_hint = self
                            .get_tool_schema(action_name)
                            .await
                            .ok()
                            .and_then(|s| util::format_schema_hint(&s))
                            .unwrap_or_default();
                        Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Action execution failed: {e}{schema_hint}")),
                        })
                    }
                }
            }

            // 获取 OAuth 授权链接
            "connect" => {
                // 执行安全策略检查，确保有权限执行此操作
                if let Err(error) =
                    self.security().enforce_tool_operation(ToolOperation::Act, "composio.connect")
                {
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(error),
                    });
                }

                let app = args.get("app").and_then(|v| v.as_str());
                let auth_config_id = args.get("auth_config_id").and_then(|v| v.as_str());

                // app 和 auth_config_id 至少需要提供一个
                if app.is_none() && auth_config_id.is_none() {
                    anyhow::bail!("Missing 'app' or 'auth_config_id' for connect");
                }

                match self.get_connection_url(app, auth_config_id, entity_id).await {
                    Ok(link) => {
                        // 确定连接目标名称（优先使用 app，否则使用 auth_config_id）
                        let target =
                            app.unwrap_or(auth_config_id.unwrap_or("provided auth config"));
                        let mut output =
                            format!("Open this URL to connect {target}:\n{}", link.redirect_url);
                        // 如果返回了 connected_account_id，缓存并显示
                        if let Some(connected_account_id) = link.connected_account_id.as_deref() {
                            if let Some(app_name) = app {
                                // 缓存账户关联关系，便于后续自动解析
                                self.cache_connected_account(
                                    app_name,
                                    entity_id,
                                    connected_account_id,
                                );
                            }
                            let _ =
                                write!(output, "\nConnected account ID: {connected_account_id}");
                        }
                        Ok(ToolResult { success: true, output, error: None })
                    }
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to get connection URL: {e}")),
                    }),
                }
            }

            // 未知的 action 类型
            _ => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "Unknown action '{action}'. Use 'list', 'list_accounts', 'execute', or 'connect'."
                )),
            }),
        }
    }
}
#[cfg(test)]
#[path = "tool_tests.rs"]
mod tool_tests;
