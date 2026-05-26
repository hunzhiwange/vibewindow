//! Brief 工具实现。
//!
//! Brief 用于向用户发送可见消息和本地附件，同时给模型返回简短投递结果。
//! 附件路径在读取元数据前会经过安全策略检查，避免工具借由“发送附件”暴露
//! 工作区外或未授权的文件。

use super::external_directory::{Kind, Options, assert_external_directory};
use super::traits::{
    Tool, ToolCallResult, ToolCallTelemetry, ToolRenderHint, ToolResult, ToolSpec,
};
use crate::app::agent::security::SecurityPolicy;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vw_api_types::tools::ToolResultContentDto;

#[derive(Debug, Clone, Deserialize)]
struct Args {
    message: String,
    #[serde(default)]
    attachments: Vec<String>,
    #[serde(default)]
    status: BriefStatus,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BriefStatus {
    #[default]
    Normal,
    Proactive,
}

impl BriefStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Proactive => "proactive",
        }
    }
}

#[derive(Clone)]
/// 向用户发送简报消息的工具。
pub struct BriefTool {
    security: Arc<SecurityPolicy>,
}

impl BriefTool {
    /// 创建新的 Brief 工具。
    ///
    /// # 参数
    ///
    /// - `security`: 当前安全策略，用于解析工作区相对路径并校验附件访问。
    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        Self { security }
    }

    fn resolve_path(&self, raw: &str) -> PathBuf {
        let path = Path::new(raw.trim());
        if path.is_absolute() { path.to_path_buf() } else { self.security.workspace_dir.join(path) }
    }

    fn is_image(path: &Path) -> bool {
        matches!(
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase())
                .as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico")
        )
    }

    fn summary_text(message: &str, attachment_count: usize) -> String {
        let trimmed = message.trim();
        if !trimmed.is_empty() {
            let mut shortened = trimmed.chars().take(96).collect::<String>();
            if trimmed.chars().count() > 96 {
                shortened.push_str("...");
            }
            return shortened.replace('\n', " ");
        }

        match attachment_count {
            0 => "用户消息".to_string(),
            1 => "1 个附件".to_string(),
            count => format!("{} 个附件", count),
        }
    }

    fn model_delivery_text(attachment_count: usize) -> String {
        match attachment_count {
            0 => "Message delivered to user.".to_string(),
            1 => "Message delivered to user. (1 attachment included)".to_string(),
            count => format!("Message delivered to user. ({} attachments included)", count),
        }
    }

    async fn resolve_attachments(&self, raw_paths: &[String]) -> anyhow::Result<Vec<Value>> {
        let mut attachments = Vec::with_capacity(raw_paths.len());
        for raw_path in raw_paths {
            let resolved = self.resolve_path(raw_path);
            let resolved_str = resolved.to_string_lossy().to_string();

            // 附件会直接展示给用户，读取前必须走外部目录策略校验，避免把
            // 未授权路径通过 UI 暴露出去。
            assert_external_directory(
                self.security.as_ref(),
                Some(&resolved_str),
                Some(Options { bypass: false, kind: Kind::File }),
            )
            .await
            .map_err(anyhow::Error::msg)?;

            let metadata = std::fs::metadata(&resolved)
                .map_err(|error| anyhow::anyhow!("failed to read attachment metadata: {error}"))?;
            if !metadata.is_file() {
                anyhow::bail!("attachment path does not point to a regular file");
            }

            attachments.push(json!({
                "path": resolved_str,
                "size": metadata.len(),
                "isImage": Self::is_image(&resolved),
            }));
        }

        Ok(attachments)
    }

    async fn build_result(&self, args: Args) -> anyhow::Result<ToolCallResult> {
        let message = args.message.trim().to_string();
        let attachments = self.resolve_attachments(&args.attachments).await?;
        if message.is_empty() && attachments.is_empty() {
            anyhow::bail!("either message or attachments is required");
        }

        let sent_at = chrono::Utc::now().to_rfc3339();
        let attachment_count = attachments.len();
        let summary = Self::summary_text(&message, attachment_count);
        let status = args.status.as_str();
        let data = json!({
            "message": message,
            "attachments": attachments,
            "status": status,
            "sentAt": sent_at,
        });

        Ok(ToolCallResult {
            data: data.clone(),
            model_result: Value::String(Self::model_delivery_text(attachment_count)),
            content_blocks: vec![ToolResultContentDto::Json { value: data.clone() }],
            render_hint: Some(ToolRenderHint {
                title: Some("Brief".to_string()),
                kind: Some("brief".to_string()),
                summary: Some(summary),
                metadata: json!({
                    "canonical_tool_id": "brief",
                    "status": status,
                    "attachment_count": attachment_count,
                }),
            }),
            telemetry: Some(ToolCallTelemetry { success: true, ..ToolCallTelemetry::default() }),
            ..ToolCallResult::default()
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for BriefTool {
    fn name(&self) -> &str {
        "Brief"
    }

    fn description(&self) -> &str {
        "向用户发送一条可见消息，可附带本地文件附件。"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "message": {
                    "type": "string",
                    "description": "要发送给用户的消息，支持 Markdown。"
                },
                "attachments": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "可选的本地附件路径，支持工作区相对路径或绝对路径。"
                },
                "status": {
                    "type": "string",
                    "enum": ["normal", "proactive"],
                    "description": "normal 表示响应用户刚刚的请求；proactive 表示主动提醒或状态更新。"
                }
            },
            "required": ["message"]
        })
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec::new(self.name(), self.description(), self.parameters_schema())
            .with_display_name("Brief")
            .with_aliases(vec!["brief".to_string()])
            .with_read_only(true)
            .with_destructive(false)
            .with_concurrency_safe(true)
            .with_requires_user_interaction(false)
            .with_strict(true)
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let args: Args = serde_json::from_value(args)?;
        let result = self.build_result(args).await?;
        Ok(ToolResult { success: true, output: result.model_text(), error: None })
    }

    async fn call(&self, input: Value) -> anyhow::Result<ToolCallResult> {
        let args: Args = serde_json::from_value(input)?;
        self.build_result(args).await
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
