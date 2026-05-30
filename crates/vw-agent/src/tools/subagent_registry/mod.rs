//! 线程安全的子代理会话注册表
//!
//! 本模块提供 [`SubAgentRegistry`] 用于跟踪后台子代理会话，主要功能包括：
//! - 会话状态生命周期管理（运行中、已完成、失败、已终止）
//! - 并发安全访问（基于 `parking_lot::RwLock`）
//! - 自动清理过期会话
//! - 会话终止（通过 tokio 任务句柄）
//!
//! # 典型用法
//!
//! ```ignore
//! use crate::app::agent::tools::subagent_registry::{SubAgentRegistry, SubAgentSession, SubAgentStatus};
//!
//! let registry = SubAgentRegistry::new();
//!
//! // 插入新会话
//! let session = SubAgentSession {
//!     id: "session-001".to_string(),
//!     agent_name: "researcher".to_string(),
//!     task: "分析市场趋势".to_string(),
//!     status: SubAgentStatus::Running,
//!     started_at: chrono::Utc::now(),
//!     completed_at: None,
//!     result: None,
//!     #[cfg(not(target_arch = "wasm32"))]
//!     handle: None,
//! };
//! registry.insert(session);
//!
//! // 查询会话状态
//! if let Some(snapshot) = registry.get_status("session-001") {
//!     println!("会话状态: {:?}", snapshot.status);
//! }
//! ```

use crate::app::agent::tools::traits::ToolResult;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::JoinHandle;

/// 已完成/失败/终止会话在自动清理前的最大存活时间（秒）
///
/// 超过此时间的非运行中会话将在调用 [`SubAgentRegistry::list`] 时被自动清理。
const SESSION_MAX_AGE_SECS: i64 = 3600;

/// 子代理会话的状态枚举
///
/// 表示子代理会话在其生命周期中的不同阶段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubAgentStatus {
    /// 运行中 - 会话当前正在执行任务
    Running,
    /// 已完成 - 会话已成功完成其任务
    Completed,
    /// 失败 - 会话执行过程中发生错误
    Failed,
    /// 已终止 - 会话被用户或系统主动终止
    Killed,
}

impl SubAgentStatus {
    /// 将状态转换为静态字符串表示
    ///
    /// # 返回值
    ///
    /// 返回对应状态的小写字符串：
    /// - `Running` -> `"running"`
    /// - `Completed` -> `"completed"`
    /// - `Failed` -> `"failed"`
    /// - `Killed` -> `"killed"`
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let status = SubAgentStatus::Running;
    /// assert_eq!(status.as_str(), "running");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            SubAgentStatus::Running => "running",
            SubAgentStatus::Completed => "completed",
            SubAgentStatus::Failed => "failed",
            SubAgentStatus::Killed => "killed",
        }
    }
}

impl std::fmt::Display for SubAgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 子代理会话记录
///
/// 存储单个子代理会话的完整信息，包括标识、任务描述、
/// 状态时间戳和执行结果。
pub struct SubAgentSession {
    /// 会话的唯一标识符
    pub id: String,
    /// 执行此任务的子代理名称
    pub agent_name: String,
    /// 可选的人类可读标题。
    pub title: Option<String>,
    /// 任务描述或提示词
    pub task: String,
    /// 可选的结构化元数据。
    pub metadata: Value,
    /// 当前会话状态
    pub status: SubAgentStatus,
    /// 会话启动时间（UTC）
    pub started_at: DateTime<Utc>,
    /// 最近一次元数据或状态更新时间。
    pub updated_at: DateTime<Utc>,
    /// 会话完成时间（UTC），若未完成则为 `None`
    pub completed_at: Option<DateTime<Utc>>,
    /// 执行结果，包含成功/失败状态、输出和错误信息
    pub result: Option<ToolResult>,
    /// tokio 任务句柄，用于通过 `abort()` 取消正在运行的任务
    ///
    /// 注意：仅在非 WASM 目标平台上可用，因为 WASM 不支持完整的 tokio 运行时。
    #[cfg(not(target_arch = "wasm32"))]
    pub handle: Option<JoinHandle<()>>,
}

/// 线程安全的子代理会话注册表
///
/// 使用 `Arc<RwLock<HashMap>>` 实现的并发安全注册表，
/// 支持多读单写的高效并发访问模式。
///
/// # 线程安全
///
/// - 读操作（如 [`get_status`](Self::get_status)、[`list`](Self::list)）使用读锁
/// - 写操作（如 [`insert`](Self::insert)、[`complete`](Self::complete)）使用写锁
/// - 自动清理旧会话以防止内存泄漏
#[derive(Clone)]
pub struct SubAgentRegistry {
    /// 内部会话存储，使用 `Arc<RwLock>` 包装以支持跨线程共享
    sessions: Arc<RwLock<HashMap<String, SubAgentSession>>>,
}

impl SubAgentRegistry {
    /// 创建新的空注册表
    ///
    /// # 返回值
    ///
    /// 返回一个不包含任何会话的新注册表实例。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let registry = SubAgentRegistry::new();
    /// assert_eq!(registry.running_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self { sessions: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// 将新会话插入注册表
    ///
    /// 如果已存在相同 ID 的会话，则会覆盖原有记录。
    ///
    /// # 参数
    ///
    /// - `session`: 要插入的会话记录
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let registry = SubAgentRegistry::new();
    /// registry.insert(SubAgentSession {
    ///     id: "session-001".to_string(),
    ///     // ... 其他字段
    /// });
    /// ```
    pub fn insert(&self, session: SubAgentSession) {
        let mut sessions = self.sessions.write();
        sessions.insert(session.id.clone(), session);
    }

    /// 原子性地检查并发会话限制并插入会话
    ///
    /// 此方法在持有写锁的情况下同时检查运行中会话数量并决定是否插入，
    /// 确保并发检查的原子性。
    ///
    /// # 参数
    ///
    /// - `session`: 要插入的会话记录
    /// - `max_concurrent`: 允许的最大并发运行会话数
    ///
    /// # 返回值
    ///
    /// - `Ok(())`: 会话已成功插入
    /// - `Err(running_count)`: 已达到并发限制，返回当前运行中的会话数量
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let registry = SubAgentRegistry::new();
    /// let session = SubAgentSession { /* ... */ };
    ///
    /// match registry.try_insert(session, 5) {
    ///     Ok(()) => println!("会话已启动"),
    ///     Err(count) => println!("已达并发限制，当前运行中: {}", count),
    /// }
    /// ```
    pub fn try_insert(&self, session: SubAgentSession, max_concurrent: usize) -> Result<(), usize> {
        let mut sessions = self.sessions.write();
        // 统计当前运行中的会话数量
        let running =
            sessions.values().filter(|s| matches!(s.status, SubAgentStatus::Running)).count();
        // 检查是否达到并发限制
        if running >= max_concurrent {
            return Err(running);
        }
        sessions.insert(session.id.clone(), session);
        Ok(())
    }

    /// 为会话设置 tokio 任务句柄（用于启用取消功能）
    ///
    /// 通过保存任务句柄，后续可以通过 [`kill`](Self::kill) 方法终止该会话。
    ///
    /// # 参数
    ///
    /// - `session_id`: 目标会话的 ID
    /// - `handle`: tokio 任务的 `JoinHandle`
    ///
    /// # 注意
    ///
    /// 此方法仅在非 WASM 目标平台上可用。
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_handle(&self, session_id: &str, handle: JoinHandle<()>) {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.handle = Some(handle);
        }
    }

    /// 将会话标记为已完成并记录结果
    ///
    /// 更新会话状态为 `Completed`，设置完成时间，并保存执行结果。
    /// 同时清除任务句柄以释放资源。
    ///
    /// # 参数
    ///
    /// - `session_id`: 目标会话的 ID
    /// - `result`: 执行结果（包含成功状态、输出和可能的错误）
    ///
    /// # 注意
    ///
    /// 如果会话 ID 不存在，此方法静默不做任何操作。
    pub fn complete(&self, session_id: &str, result: ToolResult) {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = SubAgentStatus::Completed;
            session.completed_at = Some(Utc::now());
            session.updated_at = Utc::now();
            session.result = Some(result);
            // 清除任务句柄，允许 tokio 运行时回收资源
            #[cfg(not(target_arch = "wasm32"))]
            {
                session.handle = None;
            }
        }
    }

    /// 将会话标记为失败并记录错误
    ///
    /// 更新会话状态为 `Failed`，设置完成时间，并构造包含错误信息的 `ToolResult`。
    ///
    /// # 参数
    ///
    /// - `session_id`: 目标会话的 ID
    /// - `error`: 错误描述信息
    ///
    /// # 注意
    ///
    /// 如果会话 ID 不存在，此方法静默不做任何操作。
    pub fn fail(&self, session_id: &str, error: String) {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = SubAgentStatus::Failed;
            session.completed_at = Some(Utc::now());
            session.updated_at = Utc::now();
            // 构造失败结果
            session.result =
                Some(ToolResult { success: false, output: String::new(), error: Some(error) });
            // 清除任务句柄
            #[cfg(not(target_arch = "wasm32"))]
            {
                session.handle = None;
            }
        }
    }

    /// 通过终止 tokio 任务来停止运行中的会话
    ///
    /// 调用任务句柄的 `abort()` 方法取消正在执行的任务，
    /// 并更新会话状态为 `Killed`。
    ///
    /// # 参数
    ///
    /// - `session_id`: 目标会话的 ID
    ///
    /// # 返回值
    ///
    /// - `true`: 会话被找到并成功终止
    /// - `false`: 会话不存在或不在运行状态
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let registry = SubAgentRegistry::new();
    /// // ... 插入会话 ...
    ///
    /// if registry.kill("session-001") {
    ///     println!("会话已终止");
    /// } else {
    ///     println!("无法终止会话（不存在或未运行）");
    /// }
    /// ```
    pub fn kill(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            // 只能终止运行中的会话
            if session.status != SubAgentStatus::Running {
                return false;
            }
            // 通过 tokio 任务句柄取消执行
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(handle) = session.handle.take() {
                handle.abort();
            }
            // 更新会话状态和结果
            session.status = SubAgentStatus::Killed;
            session.completed_at = Some(Utc::now());
            session.updated_at = Utc::now();
            session.result = Some(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Session killed by user".to_string()),
            });
            true
        } else {
            false
        }
    }

    /// 获取会话的状态快照
    ///
    /// 返回会话的当前状态信息，包括状态、代理名称、任务描述和时间戳。
    ///
    /// # 参数
    ///
    /// - `session_id`: 目标会话的 ID
    ///
    /// # 返回值
    ///
    /// - `Some(SubAgentStatusSnapshot)`: 会话存在，返回状态快照
    /// - `None`: 会话不存在
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let registry = SubAgentRegistry::new();
    /// if let Some(snapshot) = registry.get_status("session-001") {
    ///     println!("代理: {}", snapshot.agent_name);
    ///     println!("状态: {}", snapshot.status);
    /// }
    /// ```
    pub fn get_status(&self, session_id: &str) -> Option<SubAgentStatusSnapshot> {
        let sessions = self.sessions.read();
        sessions.get(session_id).map(|s| SubAgentStatusSnapshot {
            status: s.status.clone(),
            agent_name: s.agent_name.clone(),
            title: s.title.clone(),
            task: s.task.clone(),
            metadata: s.metadata.clone(),
            started_at: s.started_at,
            updated_at: s.updated_at,
            completed_at: s.completed_at,
            result: s.result.clone(),
        })
    }

    /// 更新会话的标题和元数据。
    pub fn update_metadata(
        &self,
        session_id: &str,
        title: Option<Option<String>>,
        metadata: Option<Value>,
    ) -> bool {
        let mut sessions = self.sessions.write();
        let Some(session) = sessions.get_mut(session_id) else {
            return false;
        };

        if let Some(title) = title {
            session.title =
                title.map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
        }
        if let Some(metadata) = metadata {
            session.metadata = metadata;
        }
        session.updated_at = Utc::now();
        true
    }

    /// 列出会话列表，支持按状态过滤
    ///
    /// 返回会话信息列表，同时执行惰性清理过期会话。
    ///
    /// # 参数
    ///
    /// - `status_filter`: 可选的状态过滤条件，支持：
    ///   - `"running"` - 仅返回运行中的会话
    ///   - `"completed"` - 仅返回已完成的会话
    ///   - `"failed"` - 仅返回失败的会话
    ///   - `"killed"` - 仅返回已终止的会话
    ///   - `None` 或其他值 - 返回所有会话
    ///
    /// # 返回值
    ///
    /// 返回包含会话简要信息的向量，每个元素包含：
    /// - 会话 ID
    /// - 代理名称
    /// - 任务描述（截断至 100 字符）
    /// - 状态字符串
    /// - 开始/完成时间（RFC3339 格式）
    /// - 执行时长（毫秒）
    ///
    /// # 自动清理
    ///
    /// 此方法会自动清理超过 [`SESSION_MAX_AGE_SECS`] 的非运行中会话，
    /// 以防止内存泄漏。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let registry = SubAgentRegistry::new();
    ///
    /// // 获取所有会话
    /// let all_sessions = registry.list(None);
    ///
    /// // 仅获取运行中的会话
    /// let running = registry.list(Some("running"));
    /// ```
    pub fn list(&self, status_filter: Option<&str>) -> Vec<SubAgentSessionInfo> {
        // 惰性清理过期会话
        self.cleanup_old_sessions();

        let sessions = self.sessions.read();
        sessions
            .values()
            // 按状态过滤
            .filter(|s| match status_filter {
                Some("running") => s.status == SubAgentStatus::Running,
                Some("completed") => s.status == SubAgentStatus::Completed,
                Some("failed") => s.status == SubAgentStatus::Failed,
                Some("killed") => s.status == SubAgentStatus::Killed,
                _ => true, // 无过滤或未知过滤值，返回所有会话
            })
            .map(|s| {
                // 计算会话执行时长（毫秒）
                let duration_ms = s.completed_at.map(|end| {
                    u64::try_from((end - s.started_at).num_milliseconds()).unwrap_or_default()
                });
                SubAgentSessionInfo {
                    session_id: s.id.clone(),
                    agent: s.agent_name.clone(),
                    title: s.title.clone(),
                    task: truncate_task(&s.task, 100),
                    metadata: s.metadata.clone(),
                    status: s.status.as_str().to_string(),
                    started_at: s.started_at.to_rfc3339(),
                    updated_at: s.updated_at.to_rfc3339(),
                    completed_at: s.completed_at.map(|t| t.to_rfc3339()),
                    duration_ms,
                }
            })
            .collect()
    }

    /// 清理过期的已完成/失败/终止会话
    ///
    /// 移除所有非运行状态且超过 [`SESSION_MAX_AGE_SECS`] 的会话记录。
    /// 此方法在 [`list`](Self::list) 中被自动调用以实现惰性清理。
    fn cleanup_old_sessions(&self) {
        let now = Utc::now();
        let mut sessions = self.sessions.write();
        // 保留运行中的会话，或未过期的已完成会话
        sessions.retain(|_, s| {
            // 运行中的会话始终保留
            if s.status == SubAgentStatus::Running {
                return true;
            }
            // 检查已完成会话是否过期
            match s.completed_at {
                Some(completed) => (now - completed).num_seconds() < SESSION_MAX_AGE_SECS,
                None => true, // 无完成时间的非运行会话保留（异常情况）
            }
        });
    }

    /// 检查会话是否存在
    ///
    /// # 参数
    ///
    /// - `session_id`: 要检查的会话 ID
    ///
    /// # 返回值
    ///
    /// - `true`: 会话存在
    /// - `false`: 会话不存在
    pub fn exists(&self, session_id: &str) -> bool {
        self.sessions.read().contains_key(session_id)
    }

    /// 获取当前运行中的会话数量
    ///
    /// # 返回值
    ///
    /// 返回状态为 `Running` 的会话总数。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let registry = SubAgentRegistry::new();
    /// println!("当前运行中的会话: {}", registry.running_count());
    /// ```
    pub fn running_count(&self) -> usize {
        self.sessions.read().values().filter(|s| s.status == SubAgentStatus::Running).count()
    }
}

impl Default for SubAgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 会话状态快照
///
/// 由 [`SubAgentRegistry::get_status`] 返回的不可变状态信息，
/// 包含会话的完整状态数据但不含任务句柄。
#[derive(Debug, Clone)]
pub struct SubAgentStatusSnapshot {
    /// 会话当前状态
    pub status: SubAgentStatus,
    /// 执行任务的子代理名称
    pub agent_name: String,
    /// 可选的人类可读标题。
    pub title: Option<String>,
    /// 任务描述或提示词
    pub task: String,
    /// 结构化元数据。
    pub metadata: Value,
    /// 会话启动时间（UTC）
    pub started_at: DateTime<Utc>,
    /// 最近一次元数据或状态更新时间。
    pub updated_at: DateTime<Utc>,
    /// 会话完成时间（UTC），若未完成则为 `None`
    pub completed_at: Option<DateTime<Utc>>,
    /// 执行结果
    pub result: Option<ToolResult>,
}

/// 可序列化的会话简要信息
///
/// 用于 [`SubAgentRegistry::list`] 返回的会话列表项，
/// 设计为可 JSON 序列化以便在 API 响应中使用。
#[derive(Debug, Clone, serde::Serialize)]
pub struct SubAgentSessionInfo {
    /// 会话唯一标识符
    pub session_id: String,
    /// 执行任务的子代理名称
    pub agent: String,
    /// 可选的人类可读标题。
    pub title: Option<String>,
    /// 任务描述（可能被截断）
    pub task: String,
    /// 结构化元数据。
    pub metadata: Value,
    /// 状态字符串（"running"、"completed"、"failed"、"killed"）
    pub status: String,
    /// 会话启动时间（RFC3339 格式）
    pub started_at: String,
    /// 最近更新时间（RFC3339 格式）。
    pub updated_at: String,
    /// 会话完成时间（RFC3339 格式），若未完成则为 `None`
    pub completed_at: Option<String>,
    /// 执行时长（毫秒），若未完成则为 `None`
    pub duration_ms: Option<u64>,
}

/// 截断任务描述字符串
///
/// 如果任务描述超过指定长度，则截断并添加省略号后缀。
/// 正确处理多字节 UTF-8 字符边界。
///
/// # 参数
///
/// - `task`: 原始任务描述
/// - `max_len`: 最大字符数（不是字节数）
///
/// # 返回值
///
/// 截断后的字符串，若超过 `max_len` 则添加 `"..."`
///
/// # 示例
///
/// ```ignore
/// let task = "这是一个很长的任务描述";
/// assert_eq!(truncate_task(task, 5), "这是一个很...");
/// ```
fn truncate_task(task: &str, max_len: usize) -> String {
    // 检查字符数是否超过限制
    if task.chars().count() <= max_len {
        task.to_string()
    } else {
        // 找到第 max_len 个字符的字节索引，确保正确处理 UTF-8 边界
        let byte_idx = task.char_indices().nth(max_len).map(|(i, _)| i).unwrap_or(task.len());
        format!("{}...", &task[..byte_idx])
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
