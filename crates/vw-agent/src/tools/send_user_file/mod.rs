//! 面向用户的本地文件附件工具。
//!
//! 本模块负责把工作区内或经安全策略允许的本地文件转换为模型可引用的附件标记。
//! 工具不会读取文件内容，只校验文件边界、推断附件类型并返回结构化渲染信息。

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
    path: String,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    caption: Option<String>,
}

#[derive(Clone)]
/// 准备发送给用户的本地文件附件工具。
///
/// 工具依赖安全策略解析相对路径，并通过外部目录校验避免暴露未授权文件。
pub struct SendUserFileTool {
    security: Arc<SecurityPolicy>,
}

impl SendUserFileTool {
    /// 创建文件附件工具。
    ///
    /// # 参数
    ///
    /// - `security`: 当前会话安全策略，包含工作区路径和外部目录访问规则。
    ///
    /// # 返回值
    ///
    /// 返回绑定该安全策略的工具实例。
    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        Self { security }
    }

    fn resolve_path(&self, raw: &str) -> PathBuf {
        let path = Path::new(raw.trim());
        if path.is_absolute() { path.to_path_buf() } else { self.security.workspace_dir.join(path) }
    }

    fn infer_kind(path: &Path, requested: Option<&str>) -> &'static str {
        match requested.map(|value| value.trim().to_ascii_lowercase()) {
            Some(value) if value == "image" => "IMAGE",
            Some(value) if value == "video" => "VIDEO",
            Some(value) if value == "audio" => "AUDIO",
            Some(value) if value == "voice" => "VOICE",
            Some(_) | None => match path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase())
                .as_deref()
            {
                Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "svg") => "IMAGE",
                Some("mp4" | "mov" | "avi" | "mkv") => "VIDEO",
                Some("mp3" | "wav" | "ogg" | "m4a") => "AUDIO",
                _ => "DOCUMENT",
            },
        }
    }

    async fn build_result(&self, input: Value) -> anyhow::Result<ToolCallResult> {
        let args: Args = serde_json::from_value(input)?;
        let resolved = self.resolve_path(&args.path);
        let resolved_str = resolved.to_string_lossy().to_string();
        assert_external_directory(
            self.security.as_ref(),
            Some(&resolved_str),
            Some(Options { bypass: false, kind: Kind::File }),
        )
        .await
        .map_err(anyhow::Error::msg)?;

        // 只接受普通文件，避免目录、管道或特殊设备被包装成可发送附件。
        let metadata = std::fs::metadata(&resolved)
            .map_err(|error| anyhow::anyhow!("failed to read file metadata: {error}"))?;
        if !metadata.is_file() {
            anyhow::bail!("path does not point to a regular file");
        }

        let marker_kind = Self::infer_kind(&resolved, args.kind.as_deref());
        let marker = format!("[{marker_kind}:{resolved_str}]");
        let data = json!({
            "path": resolved_str,
            "marker": marker,
            "kind": marker_kind,
            "size_bytes": metadata.len(),
            "caption": args.caption,
        });

        Ok(ToolCallResult {
            data: data.clone(),
            model_result: Value::String(marker.clone()),
            content_blocks: vec![ToolResultContentDto::Json { value: data.clone() }],
            render_hint: Some(ToolRenderHint {
                title: Some("SendUserFile".to_string()),
                kind: Some("send_user_file".to_string()),
                summary: Some(format!("Prepared {marker_kind} attachment")),
                metadata: data,
            }),
            telemetry: Some(ToolCallTelemetry { success: true, ..ToolCallTelemetry::default() }),
            ..ToolCallResult::default()
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for SendUserFileTool {
    fn name(&self) -> &str {
        "SendUserFile"
    }

    fn description(&self) -> &str {
        "为当前会话准备一个可发送给用户的本地文件附件标记。"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "path": {
                    "type": "string",
                    "description": "要投递的文件路径，支持工作区相对路径或绝对路径。"
                },
                "kind": {
                    "type": "string",
                    "enum": ["document", "image", "video", "audio", "voice"],
                    "description": "可选的附件类型；未提供时按扩展名推断。"
                },
                "caption": {
                    "type": "string",
                    "description": "可选说明，会附加在返回结果中。"
                }
            },
            "required": ["path"]
        })
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec::new(self.name(), self.description(), self.parameters_schema())
            .with_display_name("SendUserFile")
            .with_aliases(vec!["send_user_file".to_string()])
            .with_read_only(true)
            .with_destructive(false)
            .with_concurrency_safe(true)
            .with_requires_user_interaction(false)
            .with_strict(true)
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let result = self.build_result(args).await?;
        Ok(ToolResult {
            success: result.is_success(),
            output: result.model_text(),
            error: result.error_text(),
        })
    }

    async fn call(&self, input: Value) -> anyhow::Result<ToolCallResult> {
        self.build_result(input).await
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
