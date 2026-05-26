//! SOP 审计日志模块
//!
//! 本模块提供 SOP（标准操作流程）执行的审计日志记录功能，负责将 SOP 运行记录、
//! 步骤结果、操作员批准事件等信息持久化到 Memory 后端存储中。
//!
//! # 功能概述
//!
//! - 记录 SOP 运行的完整生命周期（启动、执行、完成）
//! - 记录每个步骤的执行结果
//! - 记录操作员批准和超时自动批准事件
//! - 提供审计记录的查询接口
//!
//! # 存储键格式
//!
//! - `sop_run_{run_id}` - 完整的 SOP 运行记录（JSON 格式）
//! - `sop_step_{run_id}_{step_number}` - 步骤执行结果（JSON 格式）
//! - `sop_approval_{run_id}_{step_number}` - 操作员批准记录
//! - `sop_timeout_approve_{run_id}_{step_number}` - 超时自动批准记录

use std::sync::Arc;

use anyhow::Result;
use tracing::{info, warn};

use super::super::memory::traits::{Memory, MemoryCategory};
use super::types::{SopRun, SopStepResult};

/// SOP 审计记录的存储分类常量
const SOP_CATEGORY: &str = "sop";

/// SOP 审计日志记录器
///
/// 负责将 SOP 执行过程中的各类事件（运行启动、步骤执行、批准操作等）
/// 持久化到 Memory 后端，为后续审计、追溯和分析提供数据支持。
///
/// # 存储结构
///
/// 所有审计记录使用 Memory 后端存储，采用以下键命名规范：
/// - 运行记录：`sop_run_{run_id}` — 完整的 `SopRun` JSON（启动时创建，完成时更新）
/// - 步骤结果：`sop_step_{run_id}_{step_number}` — `SopStepResult` JSON（每个步骤一条）
/// - 批准记录：`sop_approval_{run_id}_{step_number}` — 操作员批准记录
/// - 超时批准：`sop_timeout_approve_{run_id}_{step_number}` — 超时自动批准记录
///
/// # 线程安全性
///
/// 该结构体通过 `Arc<dyn Memory>` 持有 Memory 后端的引用，可以安全地在多线程环境中使用。
pub struct SopAuditLogger {
    /// Memory 后端存储接口
    memory: Arc<dyn Memory>,
}

impl SopAuditLogger {
    /// 创建新的 SOP 审计日志记录器实例
    ///
    /// # 参数
    ///
    /// - `memory` - Memory 后端存储接口的共享引用，用于持久化审计记录
    ///
    /// # 返回值
    ///
    /// 返回初始化后的 `SopAuditLogger` 实例
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let memory = Arc::new(SqliteMemory::new("audit.db")?);
    /// let logger = SopAuditLogger::new(memory);
    /// ```
    pub fn new(memory: Arc<dyn Memory>) -> Self {
        Self { memory }
    }

    /// 记录 SOP 运行开始事件
    ///
    /// 将完整的 SOP 运行信息存储到 Memory 后端，键格式为 `sop_run_{run_id}`。
    /// 该记录会在运行完成时被更新。
    ///
    /// # 参数
    ///
    /// - `run` - SOP 运行记录，包含运行 ID、SOP 名称、状态等完整信息
    ///
    /// # 返回值
    ///
    /// - `Ok(())` - 记录成功存储
    /// - `Err` - 序列化失败或存储失败
    ///
    /// # 副作用
    ///
    /// - 向 Memory 后端写入一条记录
    /// - 输出 info 级别的日志：`SOP audit: run {run_id} started for '{sop_name}'`
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let run = SopRun {
    ///     run_id: "run-123".to_string(),
    ///     sop_name: "backup_database".to_string(),
    ///     status: SopStatus::Running,
    ///     ..Default::default()
    /// };
    /// logger.log_run_start(&run).await?;
    /// ```
    pub async fn log_run_start(&self, run: &SopRun) -> Result<()> {
        let key = run_key(&run.run_id);
        let content = serde_json::to_string_pretty(run)?;
        self.memory.store(&key, &content, category(), None).await?;
        info!("SOP audit: run {} started for '{}'", run.run_id, run.sop_name);
        Ok(())
    }

    /// 记录步骤执行结果
    ///
    /// 将单个步骤的执行结果存储到 Memory 后端，键格式为 `sop_step_{run_id}_{step_number}`。
    /// 每个步骤执行完成后都应调用此方法进行记录。
    ///
    /// # 参数
    ///
    /// - `run_id` - SOP 运行的唯一标识符
    /// - `result` - 步骤执行结果，包含步骤编号、状态、输出、错误信息等
    ///
    /// # 返回值
    ///
    /// - `Ok(())` - 记录成功存储
    /// - `Err` - 序列化失败或存储失败
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let result = SopStepResult {
    ///     step_number: 1,
    ///     success: true,
    ///     output: Some("Database backup completed".to_string()),
    ///     error: None,
    ///     ..Default::default()
    /// };
    /// logger.log_step_result("run-123", &result).await?;
    /// ```
    pub async fn log_step_result(&self, run_id: &str, result: &SopStepResult) -> Result<()> {
        let key = step_key(run_id, result.step_number);
        let content = serde_json::to_string_pretty(result)?;
        self.memory.store(&key, &content, category(), None).await?;
        Ok(())
    }

    /// 记录 SOP 运行完成事件
    ///
    /// 更新 Memory 后端中的 SOP 运行记录，将其状态更新为最终状态（成功、失败等）。
    /// 该方法会覆盖运行开始时创建的记录。
    ///
    /// # 参数
    ///
    /// - `run` - 完整的 SOP 运行记录，应包含最终状态和结果信息
    ///
    /// # 返回值
    ///
    /// - `Ok(())` - 记录成功更新
    /// - `Err` - 序列化失败或存储失败
    ///
    /// # 副作用
    ///
    /// - 向 Memory 后端更新已存在的运行记录
    /// - 输出 info 级别的日志：`SOP audit: run {run_id} finished with status {status}`
    ///
    /// # 示例
    ///
    /// ```ignore
    /// run.status = SopStatus::Completed;
    /// run.end_time = Some(Utc::now());
    /// logger.log_run_complete(&run).await?;
    /// ```
    pub async fn log_run_complete(&self, run: &SopRun) -> Result<()> {
        let key = run_key(&run.run_id);
        let content = serde_json::to_string_pretty(run)?;
        self.memory.store(&key, &content, category(), None).await?;
        info!("SOP audit: run {} finished with status {}", run.run_id, run.status);
        Ok(())
    }

    /// 记录操作员批准事件
    ///
    /// 当操作员手动批准某个步骤的执行时，记录该批准事件。
    /// 键格式为 `sop_approval_{run_id}_{step_number}`。
    ///
    /// # 参数
    ///
    /// - `run` - SOP 运行记录
    /// - `step_number` - 被批准的步骤编号
    ///
    /// # 返回值
    ///
    /// - `Ok(())` - 记录成功存储
    /// - `Err` - 序列化失败或存储失败
    ///
    /// # 副作用
    ///
    /// - 向 Memory 后端写入一条批准记录
    /// - 输出 info 级别的日志：`SOP audit: run {run_id} step {step_number} approved by operator`
    ///
    /// # 使用场景
    ///
    /// 当 SOP 步骤配置为需要人工批准时，操作员批准后应调用此方法记录审批事件，
    /// 为后续审计提供完整的人工干预记录。
    pub async fn log_approval(&self, run: &SopRun, step_number: u32) -> Result<()> {
        let key = format!("sop_approval_{}_{step_number}", run.run_id);
        let content = serde_json::to_string_pretty(run)?;
        self.memory.store(&key, &content, category(), None).await?;
        info!("SOP audit: run {} step {step_number} approved by operator", run.run_id);
        Ok(())
    }

    /// 记录超时自动批准事件
    ///
    /// 当步骤配置了批准超时时间，且在超时后自动批准时，记录该自动批准事件。
    /// 键格式为 `sop_timeout_approve_{run_id}_{step_number}`。
    ///
    /// # 参数
    ///
    /// - `run` - SOP 运行记录
    /// - `step_number` - 自动批准的步骤编号
    ///
    /// # 返回值
    ///
    /// - `Ok(())` - 记录成功存储
    /// - `Err` - 序列化失败或存储失败
    ///
    /// # 副作用
    ///
    /// - 向 Memory 后端写入一条超时批准记录
    /// - 输出 info 级别的日志：`SOP audit: run {run_id} step {step_number} auto-approved after timeout`
    ///
    /// # 使用场景
    ///
    /// 当 SOP 步骤配置了 `approval_timeout` 且操作员未在指定时间内响应时，
    /// 系统会自动批准该步骤，此时应调用此方法记录自动批准事件。
    /// 这对于审计和合规性检查非常重要。
    pub async fn log_timeout_auto_approve(&self, run: &SopRun, step_number: u32) -> Result<()> {
        let key = format!("sop_timeout_approve_{}_{step_number}", run.run_id);
        let content = serde_json::to_string_pretty(run)?;
        self.memory.store(&key, &content, category(), None).await?;
        info!("SOP audit: run {} step {step_number} auto-approved after timeout", run.run_id);
        Ok(())
    }

    /// 根据 ID 获取存储的 SOP 运行记录
    ///
    /// 从 Memory 后端检索指定 ID 的 SOP 运行记录。
    ///
    /// # 参数
    ///
    /// - `run_id` - SOP 运行的唯一标识符
    ///
    /// # 返回值
    ///
    /// - `Ok(Some(SopRun))` - 找到并成功解析的运行记录
    /// - `Ok(None)` - 未找到该 ID 的运行记录
    /// - `Err` - 读取失败或 JSON 解析失败
    ///
    /// # 错误处理
    ///
    /// 当找到记录但 JSON 解析失败时，会：
    /// 1. 输出 warn 级别日志
    /// 2. 返回解析错误
    ///
    /// # 示例
    ///
    /// ```ignore
    /// match logger.get_run("run-123").await? {
    ///     Some(run) => println!("Found run: {}", run.sop_name),
    ///     None => println!("Run not found"),
    /// }
    /// ```
    pub async fn get_run(&self, run_id: &str) -> Result<Option<SopRun>> {
        let key = run_key(run_id);
        match self.memory.get(&key).await? {
            Some(entry) => {
                let run: SopRun = serde_json::from_str(&entry.content).map_err(|e| {
                    warn!("SOP audit: failed to parse run {run_id}: {e}");
                    e
                })?;
                Ok(Some(run))
            }
            None => Ok(None),
        }
    }

    /// 列出所有存储的 SOP 运行键
    ///
    /// 从 Memory 后端检索所有 SOP 类别的记录，并过滤出运行记录的键。
    ///
    /// # 返回值
    ///
    /// - `Ok(Vec<String>)` - 所有 SOP 运行记录的键列表（格式：`sop_run_{run_id}`）
    /// - `Err` - 列表查询失败
    ///
    /// # 过滤逻辑
    ///
    /// 1. 从 Memory 后端获取所有 SOP 类别的记录
    /// 2. 过滤出键以 `sop_run_` 开头的记录
    /// 3. 提取并返回这些键
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let run_keys = logger.list_runs().await?;
    /// for key in run_keys {
    ///     println!("Run key: {}", key);
    /// }
    /// ```
    pub async fn list_runs(&self) -> Result<Vec<String>> {
        let entries = self.memory.list(Some(&category()), None).await?;
        let run_keys: Vec<String> =
            entries.into_iter().filter(|e| e.key.starts_with("sop_run_")).map(|e| e.key).collect();
        Ok(run_keys)
    }
}

/// 生成 SOP 运行记录的存储键
///
/// # 参数
///
/// - `run_id` - SOP 运行的唯一标识符
///
/// # 返回值
///
/// 返回格式为 `sop_run_{run_id}` 的存储键
fn run_key(run_id: &str) -> String {
    format!("sop_run_{run_id}")
}

/// 生成 SOP 步骤结果的存储键
///
/// # 参数
///
/// - `run_id` - SOP 运行的唯一标识符
/// - `step_number` - 步骤编号
///
/// # 返回值
///
/// 返回格式为 `sop_step_{run_id}_{step_number}` 的存储键
fn step_key(run_id: &str, step_number: u32) -> String {
    format!("sop_step_{run_id}_{step_number}")
}

/// 创建 SOP 审计记录的 Memory 分类
///
/// # 返回值
///
/// 返回自定义分类 `MemoryCategory::Custom("sop")`，用于在 Memory 后端中
/// 区分 SOP 审计记录与其他类型的记录。
fn category() -> MemoryCategory {
    MemoryCategory::Custom(SOP_CATEGORY.into())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
