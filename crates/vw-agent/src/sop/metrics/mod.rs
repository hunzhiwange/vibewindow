//! SOP（标准操作流程）指标收集器模块
//!
//! 本模块提供了线程安全的 SOP 运行指标收集与聚合功能，用于：
//! - 追踪全局和单个 SOP 的运行统计（完成数、失败数、取消数等）
//! - 计算协议遵从率、偏离率等关键指标
//! - 支持滑动时间窗口查询（7天、30天、90天）
//! - 从持久化存储重建指标状态（重启后恢复）
//!
//! # 核心组件
//!
//! - [`SopMetricsCollector`]: 线程安全的指标收集器主结构
//! - [`MetricCounters`]: 基础指标计数器结构
//! - [`RunSnapshot`]: 单次运行的快照记录
//!
//! # 使用示例
//!
//! ```ignore
//! let collector = SopMetricsCollector::new();
//! collector.record_run_complete(&run);
//! collector.record_approval("my_sop", "run_123");
//!
//! // 查询指标
//! let rate = collector.get_metric_value("sop.completion_rate");
//! let rate_7d = collector.get_metric_value("sop.completion_rate_7d");
//! ```

use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;
use std::time::Instant;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::json;
use tracing::warn;

use super::super::memory::traits::{Memory, MemoryCategory};
use super::types::{SopRun, SopRunStatus, SopStepStatus};

/// 每个环形缓冲区保留的最大最近运行记录数（全局 + 每个 SOP）
///
/// 设计依据：覆盖约 90 天的时间窗口（假设约 11 次/天）。
/// 如果吞吐量超过此限制，窗口指标会优雅地低估而不是报错，
/// 以确保系统在高负载下仍然稳定运行。
const MAX_RECENT_RUNS: usize = 1000;

/// 待审批条目的过期时间（秒）
///
/// 超过此时间的待审批条目将被清除，防止内存泄漏。
/// 默认为 1 小时（3600 秒）。
const PENDING_EVICT_SECS: u64 = 3600;

// ═══════════════════════════════════════════════════════════════════
// MetricCounters - 基础指标计数器
// ═══════════════════════════════════════════════════════════════════

/// 基础指标计数器结构
///
/// 提供所有时间和时间窗口聚合共用的计数器字段集合。
/// 将这些字段提取到独立结构体中，避免了在 `SopCounters` 和
/// 窗口累加器之间重复定义相同的 9 个字段。
///
/// # 字段说明
///
/// - `runs_completed`: 成功完成的运行次数
/// - `runs_failed`: 失败的运行次数
/// - `runs_cancelled`: 取消的运行次数
/// - `steps_executed`: 执行的步骤总数
/// - `steps_defined`: 定义的步骤总数
/// - `steps_failed`: 失败的步骤数
/// - `steps_skipped`: 跳过的步骤数
/// - `human_approvals`: 人工审批次数
/// - `timeout_auto_approvals`: 超时自动审批次数
#[derive(Debug, Default, Clone)]
struct MetricCounters {
    runs_completed: u64,
    runs_failed: u64,
    runs_cancelled: u64,
    steps_executed: u64,
    steps_defined: u64,
    steps_failed: u64,
    steps_skipped: u64,
    human_approvals: u64,
    timeout_auto_approvals: u64,
}

// ═══════════════════════════════════════════════════════════════════
// RunSnapshot - 运行快照
// ═══════════════════════════════════════════════════════════════════

/// 轻量级运行快照结构
///
/// 存储单次终止状态运行的快照，用于时间窗口指标计算。
///
/// # 设计理念
///
/// 存储**事件级计数**（而非布尔值），确保窗口指标和全时指标
/// 在语义上保持一致：两者都统计审批事件次数，而不是运行次数。
/// 这样可以正确处理一次运行中有多次审批的情况。
///
/// # 字段说明
///
/// - `completed_at`: 运行完成时间（UTC 时区）
/// - `terminal_status`: 终止状态（Completed/Failed/Cancelled）
/// - `steps_executed`: 实际执行的步骤数
/// - `steps_defined`: 定义的步骤总数
/// - `steps_failed`: 失败的步骤数
/// - `steps_skipped`: 跳过的步骤数
/// - `human_approval_count`: 人工审批事件计数
/// - `timeout_approval_count`: 超时自动审批事件计数
#[derive(Debug, Clone)]
struct RunSnapshot {
    completed_at: DateTime<Utc>,
    terminal_status: SopRunStatus,
    steps_executed: u64,
    steps_defined: u64,
    steps_failed: u64,
    steps_skipped: u64,
    human_approval_count: u64,
    timeout_approval_count: u64,
}

// ═══════════════════════════════════════════════════════════════════
// SopCounters - 单个 SOP 的计数器
// ═══════════════════════════════════════════════════════════════════

/// 单个 SOP（或全局聚合）的累积计数器
///
/// 维护一个 SOP 的所有时间指标和最近的运行快照列表。
/// 最近运行列表使用环形缓冲区实现，自动淘汰旧记录。
#[derive(Debug, Default)]
struct SopCounters {
    counters: MetricCounters,
    recent_runs: VecDeque<RunSnapshot>,
}

// ═══════════════════════════════════════════════════════════════════
// CollectorState - 收集器内部状态
// ═══════════════════════════════════════════════════════════════════

/// 收集器内部状态结构
///
/// 维护全局聚合、每个 SOP 的独立计数器，以及待处理的审批记录。
/// 该结构被 `RwLock` 保护以确保线程安全。
#[derive(Debug, Default)]
struct CollectorState {
    /// 全局聚合计数器
    global: SopCounters,
    /// 每个 SOP 的独立计数器（按 SOP 名称索引）
    per_sop: HashMap<String, SopCounters>,
    /// 待处理的人工审批记录：run_id → (最后更新时间, 事件计数)
    pending_approvals: HashMap<String, (Instant, u64)>,
    /// 待处理的超时自动审批记录：run_id → (最后更新时间, 事件计数)
    pending_timeout_approvals: HashMap<String, (Instant, u64)>,
}

// ═══════════════════════════════════════════════════════════════════
// SopMetricsCollector - 主收集器
// ═══════════════════════════════════════════════════════════════════

/// 线程安全的 SOP 指标聚合器
///
/// 这是本模块的主要公开类型，负责将原始 SOP 审计事件转换为
/// 可查询的指标数据，供以下场景使用：
///
/// - 门控评估（Gate Evaluation）：基于指标决定是否触发某些动作
/// - 健康检查端点：暴露系统运行状况
/// - 诊断和调试：提供详细的运行统计
///
/// # 线程安全性
///
/// 内部使用 `RwLock` 保护状态，支持多读单写。
/// 在锁中毒时会记录警告并优雅降级。
///
/// # 示例
///
/// ```ignore
/// // 创建收集器
/// let collector = SopMetricsCollector::new();
///
/// // 记录运行完成
/// collector.record_run_complete(&run);
///
/// // 记录人工审批
/// collector.record_approval("backup_sop", "run_123");
///
/// // 查询指标
/// let value = collector.get_metric_value("sop.completion_rate");
/// ```
pub struct SopMetricsCollector {
    inner: RwLock<CollectorState>,
}

impl SopMetricsCollector {
    /// 创建一个新的空收集器（冷启动）
    ///
    /// 返回一个没有任何历史数据的收集器实例。
    /// 如果需要从持久化存储恢复数据，请使用 [`rebuild_from_memory`](Self::rebuild_from_memory)。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let collector = SopMetricsCollector::new();
    /// ```
    pub fn new() -> Self {
        Self { inner: RwLock::new(CollectorState::default()) }
    }

    // ─────────────────────────────────────────────────────────────
    // 推送方法（同步，需要写锁）
    // ─────────────────────────────────────────────────────────────

    /// 记录一次终止状态的运行（Completed/Failed/Cancelled）
    ///
    /// 在调用 `audit.log_run_complete()` 之后调用此方法。
    /// 该方法会：
    /// 1. 清理过期的待审批条目（>1小时）
    /// 2. 提取该运行关联的审批计数
    /// 3. 创建运行快照并应用到全局和 SOP 级别的计数器
    ///
    /// # 参数
    ///
    /// - `run`: 完成的 SOP 运行记录
    ///
    /// # 线程安全
    ///
    /// 该方法需要获取写锁。如果锁中毒，会记录警告并直接返回。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let run = SopRun { /* ... */ };
    /// collector.record_run_complete(&run);
    /// ```
    pub fn record_run_complete(&self, run: &SopRun) {
        let Ok(mut state) = self.inner.write() else {
            warn!("SOP metrics collector lock poisoned in record_run_complete");
            return;
        };

        // 清理过期的待审批条目（超过 1 小时）
        let now = Instant::now();
        state
            .pending_approvals
            .retain(|_, (ts, _)| now.duration_since(*ts).as_secs() < PENDING_EVICT_SECS);
        state
            .pending_timeout_approvals
            .retain(|_, (ts, _)| now.duration_since(*ts).as_secs() < PENDING_EVICT_SECS);

        // 提取该运行 ID 对应的审批计数，并从待处理映射中移除
        let human_count = state.pending_approvals.remove(&run.run_id).map(|(_, c)| c).unwrap_or(0);
        let timeout_count =
            state.pending_timeout_approvals.remove(&run.run_id).map(|(_, c)| c).unwrap_or(0);

        // 创建快照并应用到全局和 SOP 级别计数器
        let snapshot = build_snapshot(run, human_count, timeout_count);
        apply_run(&mut state.global, &snapshot);
        let counters = state.per_sop.entry(run.sop_name.clone()).or_default();
        apply_run(counters, &snapshot);
    }

    /// 记录一次人工审批事件
    ///
    /// 在调用 `audit.log_approval()` 之后调用此方法。
    /// 该方法会：
    /// 1. 增加全局和 SOP 级别的人工审批计数
    /// 2. 更新待审批映射（如果运行尚未完成）
    ///
    /// # 参数
    ///
    /// - `sop_name`: SOP 名称
    /// - `run_id`: 运行 ID
    ///
    /// # 线程安全
    ///
    /// 该方法需要获取写锁。如果锁中毒，会记录警告并直接返回。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// collector.record_approval("backup_sop", "run_123");
    /// ```
    pub fn record_approval(&self, sop_name: &str, run_id: &str) {
        let Ok(mut state) = self.inner.write() else {
            warn!("SOP metrics collector lock poisoned in record_approval");
            return;
        };
        // 增加全局计数
        state.global.counters.human_approvals += 1;
        // 增加 SOP 级别计数
        state.per_sop.entry(sop_name.to_string()).or_default().counters.human_approvals += 1;
        // 更新待审批映射（为运行完成时累计审批次数）
        let entry =
            state.pending_approvals.entry(run_id.to_string()).or_insert((Instant::now(), 0));
        entry.0 = Instant::now();
        entry.1 += 1;
    }

    /// 记录一次超时自动审批事件
    ///
    /// 在调用 `audit.log_timeout_auto_approve()` 之后调用此方法。
    /// 该方法会：
    /// 1. 增加全局和 SOP 级别的超时自动审批计数
    /// 2. 更新待处理超时审批映射（如果运行尚未完成）
    ///
    /// # 参数
    ///
    /// - `sop_name`: SOP 名称
    /// - `run_id`: 运行 ID
    ///
    /// # 线程安全
    ///
    /// 该方法需要获取写锁。如果锁中毒，会记录警告并直接返回。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// collector.record_timeout_auto_approve("backup_sop", "run_123");
    /// ```
    pub fn record_timeout_auto_approve(&self, sop_name: &str, run_id: &str) {
        let Ok(mut state) = self.inner.write() else {
            warn!("SOP metrics collector lock poisoned in record_timeout_auto_approve");
            return;
        };
        // 增加全局计数
        state.global.counters.timeout_auto_approvals += 1;
        // 增加 SOP 级别计数
        state.per_sop.entry(sop_name.to_string()).or_default().counters.timeout_auto_approvals += 1;
        // 更新待处理超时审批映射（为运行完成时累计审批次数）
        let entry = state
            .pending_timeout_approvals
            .entry(run_id.to_string())
            .or_insert((Instant::now(), 0));
        entry.0 = Instant::now();
        entry.1 += 1;
    }

    // ─────────────────────────────────────────────────────────────
    // 热启动（异步）
    // ─────────────────────────────────────────────────────────────

    /// 从 Memory 后端重建收集器状态（单次扫描 O(n) 复杂度）
    ///
    /// 扫描 `MemoryCategory::Custom("sop")` 中的所有条目，
    /// 重建收集器的完整状态。失败时回退到空收集器。
    ///
    /// # 重建逻辑
    ///
    /// 1. **第一遍扫描**：收集所有终止状态的运行和每个 run_id 的审批计数
    /// 2. **构建状态**：从终止运行构建全局和 SOP 级别计数器
    /// 3. **处理审批**：累计所有审批事件到全时计数器
    /// 4. **填充待处理映射**：对于尚未终止的运行，将其审批记录存入待处理映射，
    ///    以便运行完成后（通过实时推送）能正确传播审批标志到 `RunSnapshot`
    ///
    /// # 参数
    ///
    /// - `memory`: 实现 `Memory` trait 的存储后端
    ///
    /// # 返回值
    ///
    /// 返回 `anyhow::Result<Self>`，成功时返回重建的收集器，失败时返回错误。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let memory = SqliteMemory::new("sop.db").await?;
    /// let collector = SopMetricsCollector::rebuild_from_memory(&memory).await?;
    /// ```
    pub async fn rebuild_from_memory(memory: &dyn Memory) -> anyhow::Result<Self> {
        let category = MemoryCategory::Custom("sop".into());
        let entries = memory.list(Some(&category), None).await?;

        // 第一遍扫描：收集终止运行和每个 run_id 的审批计数
        let mut runs: HashMap<String, SopRun> = HashMap::new();
        let mut approval_counts: HashMap<String, u64> = HashMap::new();
        let mut timeout_counts: HashMap<String, u64> = HashMap::new();
        // 追踪每个 run_id 对应的 sop_name（用于待处理映射和 SOP 级别计数器）
        let mut approval_sop_names: HashMap<String, String> = HashMap::new();

        for entry in &entries {
            if entry.key.starts_with("sop_run_") {
                // 处理运行记录
                if let Ok(run) = serde_json::from_str::<SopRun>(&entry.content) {
                    if matches!(
                        run.status,
                        SopRunStatus::Completed | SopRunStatus::Failed | SopRunStatus::Cancelled
                    ) {
                        runs.insert(run.run_id.clone(), run);
                    }
                }
            } else if entry.key.starts_with("sop_approval_") {
                // 处理人工审批记录
                if let Ok(run) = serde_json::from_str::<SopRun>(&entry.content) {
                    *approval_counts.entry(run.run_id.clone()).or_default() += 1;
                    approval_sop_names.entry(run.run_id.clone()).or_insert(run.sop_name);
                }
            } else if entry.key.starts_with("sop_timeout_approve_") {
                // 处理超时自动审批记录
                if let Ok(run) = serde_json::from_str::<SopRun>(&entry.content) {
                    *timeout_counts.entry(run.run_id.clone()).or_default() += 1;
                    approval_sop_names.entry(run.run_id.clone()).or_insert(run.sop_name);
                }
            }
        }

        // 从终止运行构建状态
        let mut state = CollectorState::default();
        for (run_id, run) in &runs {
            let human_count = approval_counts.get(run_id).copied().unwrap_or(0);
            let timeout_count = timeout_counts.get(run_id).copied().unwrap_or(0);
            let snapshot = build_snapshot(run, human_count, timeout_count);
            apply_run(&mut state.global, &snapshot);
            let counters = state.per_sop.entry(run.sop_name.clone()).or_default();
            apply_run(counters, &snapshot);
        }

        // 全时审批计数器：统计每个审批事件
        for (run_id, count) in &approval_counts {
            state.global.counters.human_approvals += count;
            if let Some(sop_name) = approval_sop_names.get(run_id) {
                state.per_sop.entry(sop_name.clone()).or_default().counters.human_approvals +=
                    count;
            }
        }
        for (run_id, count) in &timeout_counts {
            state.global.counters.timeout_auto_approvals += count;
            if let Some(sop_name) = approval_sop_names.get(run_id) {
                state
                    .per_sop
                    .entry(sop_name.clone())
                    .or_default()
                    .counters
                    .timeout_auto_approvals += count;
            }
        }

        // 填充非终止运行的待处理映射，以便运行完成后（通过实时推送）
        // 审批标志能正确传播到 RunSnapshot
        for (run_id, count) in &approval_counts {
            if !runs.contains_key(run_id) {
                state.pending_approvals.insert(run_id.clone(), (Instant::now(), *count));
            }
        }
        for (run_id, count) in &timeout_counts {
            if !runs.contains_key(run_id) {
                state.pending_timeout_approvals.insert(run_id.clone(), (Instant::now(), *count));
            }
        }

        Ok(Self { inner: RwLock::new(state) })
    }

    // ─────────────────────────────────────────────────────────────
    // 内部指标 API
    // ─────────────────────────────────────────────────────────────

    /// 解析指标名称并返回当前值
    ///
    /// # 指标名称格式
    ///
    /// - 全局指标：`sop.<metric>`（例如 `sop.completion_rate`）
    /// - 单个 SOP 指标：`sop.<sop_name>.<metric>`（例如 `sop.backup_sop.completion_rate`）
    ///
    /// # SOP 名称匹配策略
    ///
    /// 使用最长匹配优先策略，防止较短的 SOP 名称遮蔽较长的名称。
    /// 例如，如果有 SOP `"backup"` 和 `"backup_daily"`，`sop.backup_daily.completion_rate`
    /// 会正确匹配到 `backup_daily` 而不是 `backup`。
    ///
    /// # 已知边缘情况
    ///
    /// 如果 SOP 名称恰好与指标后缀相同（例如 SOP 名为 `"runs_completed"`），
    /// `sop.runs_completed` 会解析为**全局**指标。
    /// 此类 SOP 的指标只能通过完整路径 `sop.runs_completed.runs_completed` 访问。
    ///
    /// # 参数
    ///
    /// - `name`: 指标名称，必须以 `sop.` 开头
    ///
    /// # 返回值
    ///
    /// - `Some(value)`: 成功解析的指标值（JSON 格式）
    /// - `None`: 指标不存在或锁获取失败
    ///
    /// # 示例
    ///
    /// ```ignore
    /// // 全局指标
    /// let rate = collector.get_metric_value("sop.completion_rate");
    ///
    /// // 单个 SOP 指标
    /// let rate = collector.get_metric_value("sop.backup_sop.completion_rate");
    ///
    /// // 7 天窗口指标
    /// let rate_7d = collector.get_metric_value("sop.completion_rate_7d");
    /// ```
    pub fn get_metric_value(&self, name: &str) -> Option<serde_json::Value> {
        let Ok(state) = self.inner.read() else {
            return None;
        };

        let rest = name.strip_prefix("sop.")?;

        // 首先尝试全局指标（无点分隔的 SOP 名称前缀）
        if let Some(val) = resolve_metric(&state.global, rest) {
            return Some(val);
        }

        // 单个 SOP：最长匹配优先
        let mut best_key: Option<&str> = None;
        let mut best_len = 0;
        for key in state.per_sop.keys() {
            if rest.starts_with(key.as_str()) {
                let next_char_idx = key.len();
                // 必须后跟 '.' 才是有效的 SOP 名称匹配
                if rest.len() > next_char_idx
                    && rest.as_bytes()[next_char_idx] == b'.'
                    && key.len() > best_len
                {
                    best_key = Some(key.as_str());
                    best_len = key.len();
                }
            }
        }

        if let Some(sop_key) = best_key {
            let suffix = &rest[sop_key.len() + 1..]; // 跳过 "sop_name."
            if let Some(counters) = state.per_sop.get(sop_key) {
                return resolve_metric(counters, suffix);
            }
        }

        None
    }

    // ─────────────────────────────────────────────────────────────
    // 诊断
    // ─────────────────────────────────────────────────────────────

    /// 解析指定时间窗口内的指标值
    ///
    /// 与 [`get_metric_value`](Self::get_metric_value) 类似，但支持自定义时间窗口。
    /// 通常从评估器的 `Criterion.window_seconds` 字段获取窗口大小。
    ///
    /// # 参数
    ///
    /// - `name`: 基础指标名称（例如 `"sop.completion_rate"`）
    /// - `window`: 时间窗口 Duration，从评估器传入
    ///
    /// # 返回值
    ///
    /// - `Some(value)`: 成功解析的窗口化指标值（JSON 格式）
    /// - `None`: 指标不存在或锁获取失败
    ///
    /// # 示例
    ///
    /// ```ignore
    /// use std::time::Duration;
    ///
    /// // 查询最近 1 小时的完成率
    /// let rate = collector.get_metric_value_windowed(
    ///     "sop.completion_rate",
    ///     &Duration::from_secs(3600)
    /// );
    /// ```
    pub fn get_metric_value_windowed(
        &self,
        name: &str,
        window: &std::time::Duration,
    ) -> Option<serde_json::Value> {
        let state = self.inner.read().ok()?;
        let rest = name.strip_prefix("sop.")?;

        // 提取前缀（全局 vs 单个 SOP）和基础指标名称
        let (counters, metric_name) = if let Some(dot) = rest.find('.') {
            // 可能是单个 SOP: "sop.<sop_name>.<metric>"
            // 使用最长匹配优先策略以保持与 get_metric_value 一致
            let mut best_key: Option<&str> = None;
            let mut best_len = 0;
            for key in state.per_sop.keys() {
                if rest.starts_with(key.as_str()) {
                    let next_char_idx = key.len();
                    if rest.len() > next_char_idx
                        && rest.as_bytes()[next_char_idx] == b'.'
                        && key.len() > best_len
                    {
                        best_key = Some(key.as_str());
                        best_len = key.len();
                    }
                }
            }
            if let Some(sop_key) = best_key {
                let suffix = &rest[sop_key.len() + 1..];
                match state.per_sop.get(sop_key) {
                    Some(c) => (c, suffix),
                    None => return None,
                }
            } else {
                // 没有匹配的 SOP 名称前缀 — 视为全局指标
                // （处理指标名称包含点但不是单个 SOP 的情况）
                let _ = dot; // 消除未使用警告
                (&state.global, rest)
            }
        } else {
            // "sop." 后只有裸指标名：全局
            (&state.global, rest)
        };

        // 计算时间窗口截止点
        let cutoff = Utc::now() - chrono::Duration::from_std(*window).ok()?;
        let wc = aggregate_windowed(&counters.recent_runs, cutoff);
        resolve_from_counters(&wc, metric_name)
    }

    /// 返回收集器状态的完整快照（用于健康检查/调试）
    ///
    /// 返回一个包含以下信息的 JSON 对象：
    /// - `global`: 全局聚合计数器
    /// - `per_sop`: 每个 SOP 的独立计数器
    /// - `pending_approvals`: 待处理的人工审批数量
    /// - `pending_timeout_approvals`: 待处理的超时自动审批数量
    ///
    /// # 返回值
    ///
    /// 返回 `serde_json::Value`，如果锁中毒则返回包含错误信息的 JSON。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let snapshot = collector.snapshot();
    /// println!("{}", serde_json::to_string_pretty(&snapshot)?);
    /// ```
    pub fn snapshot(&self) -> serde_json::Value {
        let Ok(state) = self.inner.read() else {
            return json!({"error": "lock poisoned"});
        };

        let per_sop: serde_json::Map<String, serde_json::Value> =
            state.per_sop.iter().map(|(name, c)| (name.clone(), counters_to_json(c))).collect();

        json!({
            "global": counters_to_json(&state.global),
            "per_sop": per_sop,
            "pending_approvals": state.pending_approvals.len(),
            "pending_timeout_approvals": state.pending_timeout_approvals.len(),
        })
    }
}

impl Default for SopMetricsCollector {
    /// 提供默认实现，等同于 `SopMetricsCollector::new()`
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// 辅助函数
// ═══════════════════════════════════════════════════════════════════

/// 构建运行快照
///
/// 从 `SopRun` 实例和审批计数创建 `RunSnapshot`。
///
/// # 参数
///
/// - `run`: SOP 运行记录
/// - `human_count`: 人工审批事件计数
/// - `timeout_count`: 超时自动审批事件计数
///
/// # 返回值
///
/// 返回包含运行关键信息的 `RunSnapshot` 实例。
fn build_snapshot(run: &SopRun, human_count: u64, timeout_count: u64) -> RunSnapshot {
    // 解析完成时间，优先使用 RFC 3339 格式，失败则使用当前时间
    let completed_at =
        run.completed_at.as_deref().and_then(parse_completed_at).unwrap_or_else(Utc::now);

    // 统计步骤执行情况
    let steps_executed = run.step_results.len() as u64;
    let steps_failed =
        run.step_results.iter().filter(|s| s.status == SopStepStatus::Failed).count() as u64;
    let steps_skipped =
        run.step_results.iter().filter(|s| s.status == SopStepStatus::Skipped).count() as u64;

    RunSnapshot {
        completed_at,
        terminal_status: run.status,
        steps_executed,
        steps_defined: u64::from(run.total_steps),
        steps_failed,
        steps_skipped,
        human_approval_count: human_count,
        timeout_approval_count: timeout_count,
    }
}

/// 将运行快照应用到 SOP 计数器
///
/// 更新计数器并根据终止状态增加相应计数，
/// 同时将快照添加到最近运行列表（环形缓冲区）。
///
/// # 参数
///
/// - `sop`: 要更新的 SOP 计数器
/// - `snap`: 运行快照
fn apply_run(sop: &mut SopCounters, snap: &RunSnapshot) {
    let c = &mut sop.counters;
    // 根据终止状态增加相应计数
    match snap.terminal_status {
        SopRunStatus::Completed => c.runs_completed += 1,
        SopRunStatus::Failed => c.runs_failed += 1,
        SopRunStatus::Cancelled => c.runs_cancelled += 1,
        _ => {}
    }
    // 累加步骤统计
    c.steps_executed += snap.steps_executed;
    c.steps_defined += snap.steps_defined;
    c.steps_failed += snap.steps_failed;
    c.steps_skipped += snap.steps_skipped;

    // 添加到环形缓冲区，超出限制时淘汰最旧记录
    sop.recent_runs.push_back(snap.clone());
    if sop.recent_runs.len() > MAX_RECENT_RUNS {
        sop.recent_runs.pop_front();
    }
}

/// 解析完成时间字符串
///
/// 支持多种时间格式：
/// 1. RFC 3339 格式（优先）
/// 2. 无时区后缀的 ISO 格式（回退）
///
/// # 参数
///
/// - `ts`: 时间字符串
///
/// # 返回值
///
/// - `Some(datetime)`: 成功解析的 UTC 时间
/// - `None`: 解析失败
fn parse_completed_at(ts: &str) -> Option<DateTime<Utc>> {
    // 优先尝试 RFC 3339 格式
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
        return Some(dt.with_timezone(&Utc));
    }
    // 回退：无时区后缀的朴素时间
    if let Ok(n) = NaiveDateTime::parse_from_str(ts.trim_end_matches('Z'), "%Y-%m-%dT%H:%M:%S") {
        return Some(n.and_utc());
    }
    // 最后手段：记录警告
    warn!("SOP metrics: could not parse completed_at timestamp: {ts}");
    None
}

/// 聚合指定时间窗口内的运行快照
///
/// 遍历所有最近运行，筛选出完成时间在截止点之后的运行，
/// 并累加其指标到计数器。
///
/// # 参数
///
/// - `recent_runs`: 最近运行快照列表
/// - `cutoff`: 时间窗口截止点（早于此时间的运行被忽略）
///
/// # 返回值
///
/// 返回聚合后的 `MetricCounters` 实例。
fn aggregate_windowed(
    recent_runs: &VecDeque<RunSnapshot>,
    cutoff: DateTime<Utc>,
) -> MetricCounters {
    let mut wc = MetricCounters::default();
    for snap in recent_runs {
        // 只统计窗口内的运行
        if snap.completed_at >= cutoff {
            match snap.terminal_status {
                SopRunStatus::Completed => wc.runs_completed += 1,
                SopRunStatus::Failed => wc.runs_failed += 1,
                SopRunStatus::Cancelled => wc.runs_cancelled += 1,
                _ => {}
            }
            wc.steps_executed += snap.steps_executed;
            wc.steps_defined += snap.steps_defined;
            wc.steps_failed += snap.steps_failed;
            wc.steps_skipped += snap.steps_skipped;
            wc.human_approvals += snap.human_approval_count;
            wc.timeout_auto_approvals += snap.timeout_approval_count;
        }
    }
    wc
}

/// 解析指标后缀并返回对应的值
///
/// 从 `SopCounters` 结构中提取指定指标的值。
/// 支持窗口化指标（后缀 `_7d`、`_30d`、`_90d`）。
///
/// # 参数
///
/// - `sop`: SOP 计数器
/// - `suffix`: 指标名称后缀（例如 `"completion_rate"` 或 `"completion_rate_7d"`）
///
/// # 返回值
///
/// - `Some(value)`: 成功解析的指标值
/// - `None`: 指标不存在
fn resolve_metric(sop: &SopCounters, suffix: &str) -> Option<serde_json::Value> {
    // 检查是否有窗口化变体后缀
    let (base, window_days) = if let Some(base) = suffix.strip_suffix("_7d") {
        (base, Some(7i64))
    } else if let Some(base) = suffix.strip_suffix("_30d") {
        (base, Some(30i64))
    } else if let Some(base) = suffix.strip_suffix("_90d") {
        (base, Some(90i64))
    } else {
        (suffix, None)
    };

    if let Some(days) = window_days {
        // 窗口化指标：计算时间窗口截止点并聚合
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let wc = aggregate_windowed(&sop.recent_runs, cutoff);
        resolve_from_counters(&wc, base)
    } else {
        // 全时指标
        resolve_from_counters(&sop.counters, base)
    }
}

/// 核心指标解析函数
///
/// 从 `MetricCounters` 实例中提取指定指标的值。
/// 此函数被全时和窗口化指标路径共用，消除了
/// 之前 `resolve_alltime`/`resolve_windowed` 之间约 100 行的重复代码。
///
/// # 支持的指标
///
/// - `runs_completed`: 完成的运行数
/// - `runs_failed`: 失败的运行数
/// - `runs_cancelled`: 取消的运行数
/// - `deviation_rate`: 偏离率 = (失败步骤 + 跳过步骤) / 执行步骤
/// - `protocol_adherence_rate`: 协议遵从率 = 成功步骤 / 定义步骤
/// - `human_intervention_count`: 人工干预次数
/// - `human_intervention_rate`: 人工干预率 = 人工审批数 / 完成运行数
/// - `timeout_auto_approvals`: 超时自动审批数
/// - `timeout_approval_rate`: 超时审批率 = 超时审批数 / 完成运行数
/// - `completion_rate`: 完成率 = 完成运行数 / 总运行数
///
/// # 参数
///
/// - `c`: 指标计数器
/// - `metric`: 指标名称
///
/// # 返回值
///
/// - `Some(value)`: 成功解析的指标值（JSON 格式）
/// - `None`: 指标不存在
fn resolve_from_counters(c: &MetricCounters, metric: &str) -> Option<serde_json::Value> {
    match metric {
        "runs_completed" => Some(json!(c.runs_completed)),
        "runs_failed" => Some(json!(c.runs_failed)),
        "runs_cancelled" => Some(json!(c.runs_cancelled)),
        // 偏离率：(失败步骤 + 跳过步骤) / 执行步骤，无执行步骤时为 0
        "deviation_rate" => {
            if c.steps_executed == 0 {
                Some(json!(0.0))
            } else {
                Some(json!((c.steps_failed + c.steps_skipped) as f64 / c.steps_executed as f64))
            }
        }
        // 协议遵从率：成功步骤 / 定义步骤
        "protocol_adherence_rate" => {
            if c.steps_defined == 0 {
                Some(json!(0.0))
            } else {
                let good =
                    c.steps_executed.saturating_sub(c.steps_failed).saturating_sub(c.steps_skipped);
                Some(json!(good as f64 / c.steps_defined as f64))
            }
        }
        "human_intervention_count" => Some(json!(c.human_approvals)),
        // 人工干预率：人工审批数 / 完成运行数（至少为 1 避免除零）
        "human_intervention_rate" => {
            Some(json!(c.human_approvals as f64 / c.runs_completed.max(1) as f64))
        }
        "timeout_auto_approvals" => Some(json!(c.timeout_auto_approvals)),
        // 超时审批率：超时审批数 / 完成运行数（至少为 1 避免除零）
        "timeout_approval_rate" => {
            Some(json!(c.timeout_auto_approvals as f64 / c.runs_completed.max(1) as f64))
        }
        // 完成率：完成运行数 / 总运行数（至少为 1 避免除零）
        "completion_rate" => {
            let total = c.runs_completed + c.runs_failed + c.runs_cancelled;
            Some(json!(c.runs_completed as f64 / total.max(1) as f64))
        }
        _ => None,
    }
}

/// 将 SOP 计数器转换为 JSON 格式
///
/// 用于诊断和调试，输出所有计数器字段和最近运行深度。
///
/// # 参数
///
/// - `sop`: SOP 计数器
///
/// # 返回值
///
/// 返回包含所有计数器信息的 JSON 对象。
fn counters_to_json(sop: &SopCounters) -> serde_json::Value {
    let c = &sop.counters;
    json!({
        "runs_completed": c.runs_completed,
        "runs_failed": c.runs_failed,
        "runs_cancelled": c.runs_cancelled,
        "steps_executed": c.steps_executed,
        "steps_defined": c.steps_defined,
        "steps_failed": c.steps_failed,
        "steps_skipped": c.steps_skipped,
        "human_approvals": c.human_approvals,
        "timeout_auto_approvals": c.timeout_auto_approvals,
        "recent_runs_depth": sop.recent_runs.len(),
    })
}

// ═══════════════════════════════════════════════════════════════════
// 单元测试
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
