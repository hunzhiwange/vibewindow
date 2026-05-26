//! 工作树生命周期与切换相关类型。
//!
//! 本模块定义项目工作树的生命周期协议，包括：
//! - 创建与删除工作树
//! - 查询工作树列表与单个工作树详情
//! - 在工作树内切换分支或执行 reset
//! - 通过状态字段反馈创建中、忙碌、异常或已移除等阶段

use crate::common::TimestampMs;
use crate::id::{ProjectId, WorktreeId};
use serde::{Deserialize, Serialize};

/// 工作树状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorktreeStatus {
    /// 正在创建。
    Creating,
    /// 已准备就绪。
    Ready,
    /// 当前有操作在进行中。
    Busy,
    /// 处于异常状态。
    Error,
    /// 已被移除。
    Removed,
}

/// 工作树详情。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorktreeDto {
    /// 工作树 ID。
    pub id: WorktreeId,
    /// 所属项目 ID。
    pub project_id: ProjectId,
    /// 工作树名称。
    pub name: String,
    /// 关联分支。
    pub branch: String,
    /// 工作树目录。
    pub directory: String,
    /// 当前状态。
    pub status: WorktreeStatus,
    /// 创建时间。
    pub created_at_ms: TimestampMs,
    /// 更新时间。
    pub updated_at_ms: TimestampMs,
}

/// 列出工作树响应。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListWorktreesResponse {
    /// 工作树列表。
    pub items: Vec<WorktreeDto>,
}

/// 创建工作树请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateWorktreeRequest {
    /// 工作树名称。
    pub name: String,
    /// 目标分支。
    pub branch: String,
    /// 可选起始引用。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_ref: Option<String>,
    /// 是否切换到新工作树。
    #[serde(default)]
    pub checkout: bool,
}

/// 创建工作树响应。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateWorktreeResponse {
    /// 新建工作树详情。
    pub worktree: WorktreeDto,
}

/// 删除工作树请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DeleteWorktreeRequest {
    /// 是否强制删除。
    #[serde(default)]
    pub force: bool,
}

/// Git reset 模式。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetMode {
    /// 仅移动 HEAD。
    Soft,
    /// 重置索引但保留工作区内容。
    Mixed,
    /// 同时重置索引与工作区内容。
    Hard,
}

/// 重置工作树请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResetWorktreeRequest {
    /// 重置模式。
    pub mode: ResetMode,
    /// 目标引用。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_ref: Option<String>,
}

/// 切换工作树分支请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckoutWorktreeRequest {
    /// 目标分支。
    pub branch: String,
    /// 分支不存在时是否创建。
    #[serde(default)]
    pub create_if_missing: bool,
    /// 新分支的起始引用。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_ref: Option<String>,
}

/// 获取单个工作树响应。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetWorktreeResponse {
    /// 工作树详情。
    pub worktree: WorktreeDto,
}
