//! Claude Tools V2 共享 DTO。
//!
//! 本模块为工具规格、工具调用与工具结果提供稳定的跨 crate 协议类型，
//! 供 agent、gateway、ACP、desktop 等边界层复用。

use crate::id::ToolId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

fn default_strict_tool_schema() -> bool {
    true
}

/// V2 工具规格 DTO。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolSpecDto {
    /// 稳定工具 ID。
    pub id: ToolId,
    /// 工具展示名。
    pub display_name: String,
    /// 工具描述。
    pub description: String,
    /// 输入 schema。
    pub input_schema: Value,
    /// 可接受的别名。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    /// 是否只读。
    #[serde(default)]
    pub read_only: bool,
    /// 是否包含破坏性写操作。
    #[serde(default)]
    pub destructive: bool,
    /// 是否允许并发执行。
    #[serde(default)]
    pub concurrency_safe: bool,
    /// 是否需要用户交互。
    #[serde(default)]
    pub requires_user_interaction: bool,
    /// 是否要求严格 schema。
    #[serde(default = "default_strict_tool_schema")]
    pub strict: bool,
}

/// 工具规格列表响应。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListToolSpecsResponse {
    /// 工具规格列表。
    pub items: Vec<ToolSpecDto>,
}

/// 工具使用 DTO。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolUseDto {
    /// 工具使用 ID，通常对应 provider 的 tool_call_id。
    pub id: String,
    /// 被调用的工具 ID。
    pub tool_id: ToolId,
    /// 工具参数。
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub arguments: Value,
}

/// 结构化补丁 hunk DTO。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredPatchHunkDto {
    /// hunk 头部，例如 @@ -1,3 +1,4 @@。
    pub header: String,
    /// 关联文件路径。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// 旧文件起始行。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_start: Option<u32>,
    /// 旧文件影响行数。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_lines: Option<u32>,
    /// 新文件起始行。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_start: Option<u32>,
    /// 新文件影响行数。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_lines: Option<u32>,
    /// hunk 内部的逐行内容。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<String>,
}

/// 工具渲染提示 DTO。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RenderHintDto {
    /// 展示标题。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// 渲染种类。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// 摘要。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// 结构化扩展元数据。
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub metadata: Value,
}

/// 审批请求 DTO。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionRequestDto {
    /// 需要审批的原因。
    pub reason: String,
    /// 可选警告信息。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    /// 审批前最终归一化后的输入。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_input: Option<Value>,
}

/// 工具结果内容 DTO。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolResultContentDto {
    /// 纯文本结果。
    Text { text: String },
    /// 任意 JSON 结果。
    Json { value: Value },
    /// 结构化补丁结果。
    StructuredPatch { hunks: Vec<StructuredPatchHunkDto> },
}

/// V2 工具结果 DTO。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResultDto {
    /// 对应的 tool use id。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    /// 对应的工具 ID。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<ToolId>,
    /// 调用是否成功。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
    /// 面向消费层的结果内容块。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<ToolResultContentDto>,
    /// 工具内部结构化数据。
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub data: Value,
    /// 面向模型的结果。
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub model_result: Value,
    /// 渲染提示。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_hint: Option<RenderHintDto>,
    /// 可选审批请求。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_request: Option<PermissionRequestDto>,
    /// 写回上下文的更新。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_updates: Vec<Value>,
    /// 需要追加的额外消息。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_messages: Vec<Value>,
    /// 遥测扩展字段。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<Value>,
}