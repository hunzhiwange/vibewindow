//! 会话处理器类型定义模块
//!
//! 本模块定义了会话处理过程中使用的核心数据类型，包括请求结构、流式事件和工具会话状态。
//! 这些类型用于在会话处理管道中传递和处理数据。

use crate::agent::loop_::NonCliApprovalContext;
use crate::app::agent::approval::ApprovalManager;
use crate::session::ui_types as models;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;

/// 会话处理请求
///
/// 表示一个完整的会话处理请求，包含查询内容、会话标识、模型配置和历史消息等信息。
/// 该结构体用于启动或继续一个会话处理流程。
#[derive(Debug, Clone)]
pub struct Request {
    /// 流标识符，用于唯一标识一个流式会话
    pub stream: u64,

    /// 会话标识符，用于关联同一会话中的多个请求
    pub session: String,

    /// 用户查询文本，包含用户的问题或指令
    pub query: String,

    /// 根目录路径，可选的工作目录或上下文根路径
    pub root: Option<String>,

    /// 模型标识符，指定要使用的 AI 模型（如 "gpt-4"、"claude-3" 等）
    pub model: Option<String>,

    /// 额外调用选项，用于透传 ACP 等适配器特定参数
    pub options: Value,

    /// 交互式审批管理器
    pub approval: Option<Arc<ApprovalManager>>,

    /// 当前请求所属通道
    pub channel_name: Option<String>,

    /// 非 CLI 审批上下文
    pub non_cli_approval_context: Option<NonCliApprovalContext>,

    /// 当前助手消息 ID（可预分配，用于工具/审批关联）
    pub assistant_message_id: Option<String>,

    /// 历史消息列表，包含之前对话的上下文信息
    pub history: Vec<models::ChatMessage>,

    /// 是否持久化会话工件（如生成的文件、中间结果等）
    pub persist_app_session_artifacts: bool,
}

/// 流式事件枚举
///
/// 定义了会话处理过程中可能产生的各种流式事件类型。
/// 这些事件用于实时通知客户端会话处理的进度和状态变化。
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// 增量文本事件，包含新生成的文本片段
    Delta(String),

    /// 步骤开始事件，表示一个新的处理步骤开始执行
    StepStart {
        /// 步骤索引，从 0 开始递增
        step_index: u32,

        /// 步骤创建时间戳（毫秒级）
        created_ms: u64,

        /// 当前步骤使用的模型标识符
        model: Option<String>,
    },

    /// 步骤完成事件，表示一个处理步骤执行完毕
    StepFinish {
        /// 步骤索引，与 StepStart 中的 step_index 对应
        step_index: u32,

        /// 步骤完成时间戳（毫秒级）
        finished_ms: u64,

        /// 当前步骤的令牌使用统计
        usage: models::TokenUsage,

        /// 完成原因（如 "stop"、"length"、"content_filter" 等）
        finish_reason: Option<String>,

        /// 完成该步骤时使用的模型标识符
        model: Option<String>,
    },

    /// 工具轮次完成事件，表示本轮工具执行已结束，下一轮模型请求尚未开始
    PostToolRound {
        /// 对应的步骤索引
        step_index: u32,
    },

    /// 全部完成事件，表示整个会话处理结束
    Done(models::TokenUsage),

    /// 错误事件，表示处理过程中发生了错误
    Error(String),
}

/// 工具会话状态
///
/// 跟踪会话中工具执行的状态信息，用于防止重复执行和统计执行次数。
/// 该结构体仅在处理器内部使用。
#[derive(Default)]
pub(crate) struct ToolSessionState {
    /// 已见的工具调用标识符集合，用于去重和避免重复执行
    pub(crate) seen: HashSet<u64>,

    /// 非 TODO 类型的工具运行次数统计
    pub(crate) non_todo_tool_runs: u64,
}
#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
