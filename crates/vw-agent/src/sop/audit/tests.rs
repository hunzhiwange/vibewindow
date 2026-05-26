//! SOP 审计日志测试模块
//!
//! 本模块提供 SOP（标准操作流程）审计日志功能的集成测试。
//!
//! # 主要测试内容
//!
//! - **审计往返测试**：验证 SOP 运行记录的创建、更新、查询和列表功能的完整性
//! - **审批日志持久化测试**：验证人工审批记录的存储和检索
//! - **超时自动审批测试**：验证超时自动审批机制的日志记录
//! - **边界条件测试**：验证查询不存在记录时的正确行为
//!
//! # 测试策略
//!
//! 所有测试使用临时的 SQLite 内存数据库作为存储后端，
//! 通过 `tempfile::tempdir()` 创建临时目录，确保测试之间相互隔离且不污染环境。

use super::*;

/// SOP 审计日志测试模块
///
/// 包含针对 `SopAuditLogger` 功能的各种测试用例
#[allow(dead_code)]
mod tests {
    use super::*;

    use crate::app::agent::sop::types::{SopEvent, SopRunStatus, SopStepStatus, SopTriggerSource};

    /// 创建测试用的 SOP 运行记录
    ///
    /// 返回一个标准的 `SopRun` 实例，用于测试目的。
    /// 该运行记录包含以下特征：
    /// - 运行 ID: `run-test-001`
    /// - SOP 名称: `test-sop`
    /// - 触发源: 手动触发
    /// - 状态: 正在运行
    /// - 当前步骤: 1
    /// - 总步骤数: 3
    ///
    /// # 返回值
    ///
    /// 返回一个预设好所有字段的 `SopRun` 实例
    fn test_run() -> SopRun {
        SopRun {
            run_id: "run-test-001".into(),
            sop_name: "test-sop".into(),
            trigger_event: SopEvent {
                source: SopTriggerSource::Manual,
                topic: None,
                payload: None,
                timestamp: "2026-02-19T12:00:00Z".into(),
            },
            status: SopRunStatus::Running,
            current_step: 1,
            total_steps: 3,
            started_at: "2026-02-19T12:00:00Z".into(),
            completed_at: None,
            step_results: Vec::new(),
            waiting_since: None,
        }
    }

    /// 创建测试用的 SOP 步骤结果
    ///
    /// 根据给定的步骤编号生成一个已完成的步骤结果记录。
    ///
    /// # 参数
    ///
    /// * `n` - 步骤编号
    ///
    /// # 返回值
    ///
    /// 返回一个已完成的 `SopStepResult` 实例，包含：
    /// - 指定的步骤编号
    /// - 状态: 已完成
    /// - 输出: "Step {n} completed"
    /// - 开始时间: 2026-02-19T12:00:00Z
    /// - 完成时间: 2026-02-19T12:00:05Z
    fn test_step_result(n: u32) -> SopStepResult {
        SopStepResult {
            step_number: n,
            status: SopStepStatus::Completed,
            output: format!("Step {n} completed"),
            started_at: "2026-02-19T12:00:00Z".into(),
            completed_at: Some("2026-02-19T12:00:05Z".into()),
        }
    }

    /// 测试审计日志的完整往返流程
    ///
    /// 该测试验证以下场景：
    /// 1. 记录 SOP 运行的开始
    /// 2. 记录步骤执行结果
    /// 3. 记录 SOP 运行的完成
    /// 4. 通过运行 ID 检索完整的运行记录
    /// 5. 列出所有 SOP 运行记录
    ///
    /// # 测试步骤
    ///
    /// - 创建 SQLite 后端的内存存储
    /// - 初始化审计日志记录器
    /// - 记录运行开始、步骤结果和运行完成
    /// - 验证检索到的记录与原始数据一致
    /// - 验证列表功能能正确返回记录键
    ///
    /// # 断言
    ///
    /// - 检索到的运行 ID 应为 "run-test-001"
    /// - 最终状态应为 Completed
    /// - 应包含 1 个步骤结果
    /// - 列表应包含正确的记录键
    #[tokio::test]
    async fn audit_roundtrip() {
        let mem_cfg = crate::app::agent::config::MemoryConfig {
            backend: "sqlite".into(),
            ..crate::app::agent::config::MemoryConfig::default()
        };

        // 创建临时目录用于测试，测试结束后自动清理
        let tmp = tempfile::tempdir().unwrap();

        // 使用 SQLite 后端初始化内存存储
        let memory: Arc<dyn Memory> = Arc::from(
            crate::app::agent::memory::create_memory(&mem_cfg, tmp.path(), None).unwrap(),
        );

        let logger = SopAuditLogger::new(memory);

        // 记录 SOP 运行开始
        let run = test_run();
        logger.log_run_start(&run).await.unwrap();

        // 记录步骤执行结果
        let step = test_step_result(1);
        logger.log_step_result(&run.run_id, &step).await.unwrap();

        // 记录 SOP 运行完成
        let mut completed_run = run.clone();
        completed_run.status = SopRunStatus::Completed;
        completed_run.completed_at = Some("2026-02-19T12:05:00Z".into());
        completed_run.step_results = vec![step];
        logger.log_run_complete(&completed_run).await.unwrap();

        // 检索并验证运行记录
        let retrieved = logger.get_run("run-test-001").await.unwrap().unwrap();
        assert_eq!(retrieved.run_id, "run-test-001");
        assert_eq!(retrieved.status, SopRunStatus::Completed);
        assert_eq!(retrieved.step_results.len(), 1);

        // 列出所有运行记录并验证包含目标记录
        let keys = logger.list_runs().await.unwrap();
        assert!(keys.contains(&"sop_run_run-test-001".to_string()));
    }

    /// 测试审批日志的持久化
    ///
    /// 验证 `log_approval` 方法能够正确记录人工审批事件，
    /// 并以正确的键格式存储到内存后端。
    ///
    /// # 测试步骤
    ///
    /// - 初始化 SQLite 内存存储和审计日志记录器
    /// - 记录步骤 1 的审批事件
    /// - 列出所有 SOP 类别的条目
    /// - 验证审批记录已正确持久化
    ///
    /// # 断言
    ///
    /// - 应存在 1 条以 "sop_approval_" 开头的记录
    /// - 记录键应包含运行 ID "run-test-001"
    #[tokio::test]
    async fn log_approval_persists_entry() {
        let mem_cfg = crate::app::agent::config::MemoryConfig {
            backend: "sqlite".into(),
            ..crate::app::agent::config::MemoryConfig::default()
        };

        // 创建临时目录和内存存储
        let tmp = tempfile::tempdir().unwrap();
        let memory: Arc<dyn Memory> = Arc::from(
            crate::app::agent::memory::create_memory(&mem_cfg, tmp.path(), None).unwrap(),
        );

        let logger = SopAuditLogger::new(memory.clone());
        let run = test_run();

        // 记录审批事件
        logger.log_approval(&run, 1).await.unwrap();

        // 检索所有 SOP 类别的条目并筛选审批记录
        let entries = memory.list(Some(&category()), None).await.unwrap();
        let approval_keys: Vec<_> =
            entries.iter().filter(|e| e.key.starts_with("sop_approval_")).collect();

        // 验证审批记录已正确持久化
        assert_eq!(approval_keys.len(), 1);
        assert!(approval_keys[0].key.contains("run-test-001"));
    }

    /// 测试超时自动审批日志的持久化
    ///
    /// 验证 `log_timeout_auto_approve` 方法能够正确记录超时自动审批事件，
    /// 并以正确的键格式存储到内存后端。
    ///
    /// # 测试步骤
    ///
    /// - 初始化 SQLite 内存存储和审计日志记录器
    /// - 记录步骤 1 的超时自动审批事件
    /// - 列出所有 SOP 类别的条目
    /// - 验证超时审批记录已正确持久化
    ///
    /// # 断言
    ///
    /// - 应存在 1 条以 "sop_timeout_approve_" 开头的记录
    /// - 记录键应包含运行 ID "run-test-001"
    #[tokio::test]
    async fn log_timeout_auto_approve_persists_entry() {
        let mem_cfg = crate::app::agent::config::MemoryConfig {
            backend: "sqlite".into(),
            ..crate::app::agent::config::MemoryConfig::default()
        };

        // 创建临时目录和内存存储
        let tmp = tempfile::tempdir().unwrap();
        let memory: Arc<dyn Memory> = Arc::from(
            crate::app::agent::memory::create_memory(&mem_cfg, tmp.path(), None).unwrap(),
        );

        let logger = SopAuditLogger::new(memory.clone());
        let run = test_run();

        // 记录超时自动审批事件
        logger.log_timeout_auto_approve(&run, 1).await.unwrap();

        // 检索所有 SOP 类别的条目并筛选超时审批记录
        let entries = memory.list(Some(&category()), None).await.unwrap();
        let timeout_keys: Vec<_> =
            entries.iter().filter(|e| e.key.starts_with("sop_timeout_approve_")).collect();

        // 验证超时审批记录已正确持久化
        assert_eq!(timeout_keys.len(), 1);
        assert!(timeout_keys[0].key.contains("run-test-001"));
    }

    /// 测试查询不存在的运行记录
    ///
    /// 验证当查询不存在的运行记录时，`get_run` 方法能够正确返回 `None`，
    /// 而不会抛出错误或产生异常行为。
    ///
    /// # 测试步骤
    ///
    /// - 初始化 SQLite 内存存储和审计日志记录器
    /// - 尝试获取不存在的运行记录（ID 为 "nonexistent"）
    /// - 验证返回值为 `None`
    ///
    /// # 断言
    ///
    /// - 查询不存在的记录应返回 `None`，而不是错误
    #[tokio::test]
    async fn get_nonexistent_run_returns_none() {
        let mem_cfg = crate::app::agent::config::MemoryConfig {
            backend: "sqlite".into(),
            ..crate::app::agent::config::MemoryConfig::default()
        };

        // 创建临时目录和内存存储
        let tmp = tempfile::tempdir().unwrap();
        let memory: Arc<dyn Memory> = Arc::from(
            crate::app::agent::memory::create_memory(&mem_cfg, tmp.path(), None).unwrap(),
        );

        let logger = SopAuditLogger::new(memory);

        // 尝试获取不存在的运行记录
        let result = logger.get_run("nonexistent").await.unwrap();

        // 验证返回值为 None
        assert!(result.is_none());
    }
}
