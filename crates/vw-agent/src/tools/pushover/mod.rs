//! Pushover 通知工具
//!
//! 本模块实现了通过 Pushover 服务发送推送通知的工具。Pushover 是一个跨平台的
//! 推送通知服务，支持 iOS、Android、桌面端等多种设备。
//!
//! # 主要功能
//!
//! - 从工作区 `.env` 文件读取 API 凭据（`PUSHOVER_TOKEN` 和 `PUSHOVER_USER_KEY`）
//! - 支持发送文本消息通知
//! - 支持自定义通知标题、优先级和声音
//! - 实现了安全策略检查和速率限制
//!
//! # 配置要求
//!
//! 在工作区目录的 `.env` 文件中配置以下环境变量：
//!
//! ```text
//! PUSHOVER_TOKEN=your_app_token
//! PUSHOVER_USER_KEY=your_user_key
//! ```
//!
//! # 示例
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use std::path::PathBuf;
//! use crate::app::agent::tools::pushover::PushoverTool;
//! use crate::app::agent::security::SecurityPolicy;
//!
//! let security = Arc::new(SecurityPolicy::default());
//! let workspace = PathBuf::from("/path/to/workspace");
//! let tool = PushoverTool::new(security, workspace);
//! ```

use super::traits::{Tool, ToolResult};
use crate::app::agent::security::SecurityPolicy;
use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

/// Pushover API 消息发送端点 URL
///
/// 该端点用于向 Pushover 服务提交消息发送请求。
const PUSHOVER_API_URL: &str = "https://api.pushover.net/1/messages.json";

/// HTTP 请求超时时间（秒）
///
/// 设置为 15 秒以适应网络延迟，同时避免长时间阻塞。
const PUSHOVER_REQUEST_TIMEOUT_SECS: u64 = 15;

/// Pushover 通知工具
///
/// 该结构体封装了 Pushover 通知服务的调用逻辑，包括凭据管理、
/// 消息构建和 API 请求发送。
///
/// # 字段说明
///
/// - `security`: 安全策略引用，用于检查操作权限和速率限制
/// - `workspace_dir`: 工作区目录路径，用于定位 `.env` 配置文件
pub struct PushoverTool {
    /// 安全策略，控制操作权限和速率限制
    security: Arc<SecurityPolicy>,
    /// 工作区目录，包含 `.env` 配置文件
    workspace_dir: PathBuf,
}

impl PushoverTool {
    /// 创建新的 PushoverTool 实例
    ///
    /// # 参数
    ///
    /// - `security`: 安全策略的原子引用，用于权限检查和速率限制
    /// - `workspace_dir`: 工作区目录路径，工具将在此目录下查找 `.env` 文件
    ///
    /// # 返回值
    ///
    /// 返回配置好的 `PushoverTool` 实例
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let tool = PushoverTool::new(security, PathBuf::from("/workspace"));
    /// ```
    pub fn new(security: Arc<SecurityPolicy>, workspace_dir: PathBuf) -> Self {
        Self { security, workspace_dir }
    }

    /// 解析 `.env` 文件中的环境变量值
    ///
    /// 该方法处理环境变量值的多种格式，包括：
    /// - 带引号的值（单引号或双引号）
    /// - 不带引号的值
    /// - 包含行内注释的值（`#` 分隔）
    ///
    /// # 参数
    ///
    /// - `raw`: 原始的环境变量值字符串
    ///
    /// # 返回值
    ///
    /// 返回清理后的值字符串，已移除引号和注释
    ///
    /// # 处理逻辑
    ///
    /// 1. 首先去除首尾空白字符
    /// 2. 如果值被引号包裹（单引号或双引号），移除引号
    /// 3. 处理行内注释：以 ` #` 分隔，只保留注释前的部分
    fn parse_env_value(raw: &str) -> String {
        let raw = raw.trim();

        // 处理引号包裹的值：支持单引号和双引号
        let unquoted = if raw.len() >= 2
            && ((raw.starts_with('"') && raw.ends_with('"'))
                || (raw.starts_with('\'') && raw.ends_with('\'')))
        {
            &raw[1..raw.len() - 1]
        } else {
            raw
        };

        // 处理不带引号的值中的行内注释
        // 例如：KEY=value # 这是注释
        unquoted
            .split_once(" #")
            .map_or_else(|| unquoted.trim().to_string(), |(value, _)| value.trim().to_string())
    }

    /// 从工作区 `.env` 文件获取 Pushover API 凭据
    ///
    /// 读取工作区目录下的 `.env` 文件，解析并提取 `PUSHOVER_TOKEN`
    /// 和 `PUSHOVER_USER_KEY` 两个环境变量。
    ///
    /// # 返回值
    ///
    /// 成功时返回元组 `(token, user_key)`，包含：
    /// - `token`: Pushover 应用程序令牌
    /// - `user_key`: Pushover 用户密钥
    ///
    /// # 错误
    ///
    /// 在以下情况下返回错误：
    /// - `.env` 文件不存在或无法读取
    /// - `PUSHOVER_TOKEN` 未配置
    /// - `PUSHOVER_USER_KEY` 未配置
    ///
    /// # 解析规则
    ///
    /// - 跳过空行和以 `#` 开头的注释行
    /// - 支持 `export` 前缀（自动移除）
    /// - 键名匹配不区分大小写
    async fn get_credentials(&self) -> anyhow::Result<(String, String)> {
        // 构建 .env 文件路径
        let env_path = self.workspace_dir.join(".env");

        // 异步读取文件内容
        let content = tokio::fs::read_to_string(&env_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", env_path.display(), e))?;

        // 用于存储解析出的凭据
        let mut token = None;
        let mut user_key = None;

        // 逐行解析环境变量
        for line in content.lines() {
            let line = line.trim();

            // 跳过注释行和空行
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            // 移除可选的 export 前缀
            let line = line.strip_prefix("export ").map(str::trim).unwrap_or(line);

            // 尝试按等号分割键值对
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = Self::parse_env_value(value);

                // 匹配 Pushover 相关的环境变量（不区分大小写）
                if key.eq_ignore_ascii_case("PUSHOVER_TOKEN") {
                    token = Some(value);
                } else if key.eq_ignore_ascii_case("PUSHOVER_USER_KEY") {
                    user_key = Some(value);
                }
            }
        }

        // 验证必需的凭据是否都已找到
        let token = token.ok_or_else(|| anyhow::anyhow!("PUSHOVER_TOKEN not found in .env"))?;
        let user_key =
            user_key.ok_or_else(|| anyhow::anyhow!("PUSHOVER_USER_KEY not found in .env"))?;

        Ok((token, user_key))
    }
}

/// Tool trait 实现
///
/// 为 `PushoverTool` 实现 `Tool` trait，使其可作为代理工具使用。
/// 支持跨平台异步执行（包括 WASM 目标）。
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for PushoverTool {
    /// 返回工具名称
    ///
    /// 该名称用于工具注册和调用时的标识符。
    ///
    /// # 返回值
    ///
    /// 返回固定字符串 `"pushover"`
    fn name(&self) -> &str {
        "pushover"
    }

    /// 返回工具描述
    ///
    /// 描述了工具的功能、用途和配置要求，
    /// 供代理系统了解工具能力并决定何时调用。
    ///
    /// # 返回值
    ///
    /// 返回工具的中文描述字符串
    fn description(&self) -> &str {
        "向您的设备发送 Pushover 通知。需要在 .env 文件中配置 PUSHOVER_TOKEN 和 PUSHOVER_USER_KEY。"
    }

    /// 返回工具参数的 JSON Schema
    ///
    /// 定义了工具接受的参数结构，包括：
    /// - `message`: 必需，通知消息内容
    /// - `title`: 可选，通知标题
    /// - `priority`: 可选，消息优先级（-2 到 2）
    /// - `sound`: 可选，通知声音
    ///
    /// # 返回值
    ///
    /// 返回描述参数结构的 JSON Schema 对象
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "要发送的通知消息"
                },
                "title": {
                    "type": "string",
                    "description": "可选的通知标题"
                },
                "priority": {
                    "type": "integer",
                    "description": "消息优先级：-2（最低/静音），-1（低/无声），0（正常），1（高），2（紧急/重复）"
                },
                "sound": {
                    "type": "string",
                    "description": "通知声音覆盖（例如 'pushover'、'bike'、'bugle'、'cashregister' 等）"
                }
            },
            "required": ["message"]
        })
    }

    /// 执行 Pushover 通知发送
    ///
    /// 该方法是工具的核心执行逻辑，负责：
    /// 1. 安全策略检查（权限和速率限制）
    /// 2. 参数解析和验证
    /// 3. 从 `.env` 文件获取 API 凭据
    /// 4. 构建 HTTP 请求并发送到 Pushover API
    /// 5. 解析响应并返回执行结果
    ///
    /// # 参数
    ///
    /// - `args`: JSON 格式的参数对象，包含：
    ///   - `message` (必需): 通知消息内容
    ///   - `title` (可选): 通知标题
    ///   - `priority` (可选): 优先级整数，范围 -2 到 2
    ///   - `sound` (可选): 通知声音名称
    ///
    /// # 返回值
    ///
    /// 返回 `ToolResult` 结构体，包含：
    /// - `success`: 操作是否成功
    /// - `output`: 响应内容或状态信息
    /// - `error`: 错误信息（如果失败）
    ///
    /// # 错误情况
    ///
    /// - 安全策略阻止操作（只读模式）
    /// - 速率限制超出
    /// - 缺少必需参数
    /// - 优先级值超出有效范围
    /// - API 凭据未配置
    /// - Pushover API 返回错误
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // 安全策略检查：是否允许执行操作
        if !self.security.can_act() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Action blocked: autonomy is read-only".into()),
            });
        }

        // 速率限制检查
        if !self.security.record_action() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Action blocked: rate limit exceeded".into()),
            });
        }

        // 解析并验证必需的 message 参数
        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| anyhow::anyhow!("Missing 'message' parameter"))?
            .to_string();

        // 解析可选的 title 参数
        let title = args.get("title").and_then(|v| v.as_str()).map(String::from);

        // 解析并验证可选的 priority 参数
        // 有效范围：-2（最低）到 2（紧急）
        let priority = match args.get("priority").and_then(|v| v.as_i64()) {
            Some(value) if (-2..=2).contains(&value) => Some(value),
            Some(value) => {
                // 优先级值超出有效范围
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!(
                        "Invalid 'priority': {value}. Expected integer in range -2..=2"
                    )),
                });
            }
            None => None,
        };

        // 解析可选的 sound 参数
        let sound = args.get("sound").and_then(|v| v.as_str()).map(String::from);

        // 获取 API 凭据
        let (token, user_key) = self.get_credentials().await?;

        // 构建multipart表单数据
        // 必需字段：token、user、message
        let mut form = reqwest::multipart::Form::new()
            .text("token", token)
            .text("user", user_key)
            .text("message", message);

        // 添加可选的 title 字段
        if let Some(title) = title {
            form = form.text("title", title);
        }

        // 添加可选的 priority 字段
        if let Some(priority) = priority {
            form = form.text("priority", priority.to_string());
        }

        // 添加可选的 sound 字段
        if let Some(sound) = sound {
            form = form.text("sound", sound);
        }

        // 创建带超时配置的 HTTP 客户端并发送请求
        let client = crate::app::agent::config::build_runtime_proxy_client_with_timeouts(
            "tool.pushover",
            PUSHOVER_REQUEST_TIMEOUT_SECS,
            10,
        );
        let response = client.post(PUSHOVER_API_URL).multipart(form).send().await?;

        // 解析响应状态和内容
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        // 检查 HTTP 状态码
        if !status.is_success() {
            return Ok(ToolResult {
                success: false,
                output: body,
                error: Some(format!("Pushover API returned status {}", status)),
            });
        }

        // 解析响应 JSON，检查 API 级别的状态
        // Pushover API 成功时返回 {"status": 1, ...}
        let api_status = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|json| json.get("status").and_then(|value| value.as_i64()));

        // 根据API状态返回结果
        if api_status == Some(1) {
            Ok(ToolResult {
                success: true,
                output: format!("Pushover notification sent successfully. Response: {}", body),
                error: None,
            })
        } else {
            Ok(ToolResult {
                success: false,
                output: body,
                error: Some("Pushover API returned an application-level error".into()),
            })
        }
    }
}

/// 单元测试模块
///
/// 测试代码位于 `tests/pushover.rs` 文件中，
/// 包含对凭据解析、参数验证和 API 调用的测试用例。
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
