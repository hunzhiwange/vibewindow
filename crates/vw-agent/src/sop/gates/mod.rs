//! ampersona 信任阶段转换的门控评估状态模块
//!
//! 本模块仅在 `ampersona-gates` 特性激活时编译
//! (模块声明在 `mod.rs` 中通过 `#[cfg]` 条件编译)
//!
//! # 模块职责
//!
//! 门控决策**不会**改变 SOP 执行行为 —— 本模块纯粹用于：
//! - 观察：监控门控条件的满足情况
//! - 阶段状态跟踪：记录和维护当前的信任阶段
//! - 审计日志：记录所有门控决策以供后续审查
//!
//! # 核心概念
//!
//! - **Gate（门控）**：定义阶段转换条件的规则
//! - **Phase（阶段）**：代理的信任级别状态
//! - **Decision（决策）**：门控评估的结果，决定是否进行阶段转换
//!
//! # 使用场景
//!
//! 用于 ampersona 系统中管理代理的信任级别自动升级或降级，确保代理行为在
//! 可控的信任范围内演变

use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use ampersona_core::spec::gates::Gate;
use ampersona_core::state::{PendingTransition, PhaseState, TransitionRecord};
use ampersona_core::traits::MetricsProvider;
use ampersona_engine::gates::decision::GateDecisionRecord;
use ampersona_engine::gates::evaluator::DefaultGateEvaluator;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use super::super::memory::traits::{Memory, MemoryCategory};

/// 阶段状态在内存存储中使用的键名
/// 用于持久化和恢复 GateEvalState 的状态
const PHASE_STATE_KEY: &str = "sop_phase_state";

/// 获取 SOP 相关数据在内存中的存储分类
///
/// # 返回值
///
/// 返回一个自定义的 MemoryCategory，用于将 SOP 相关数据与其他数据隔离存储
fn sop_category() -> MemoryCategory {
    MemoryCategory::Custom("sop".into())
}

// ── 内部状态结构 ────────────────────────────────────────────────

/// 门控评估的内部状态
///
/// 包含当前阶段状态和上次评估的时间戳，
/// 通过 Mutex 保护以实现线程安全的原子操作
struct GateEvalInner {
    /// 当前的信任阶段状态
    phase_state: PhaseState,
    /// 上次执行 tick 的时间点，用于间隔控制
    last_tick: Instant,
}

// ── GateEvalState 主结构 ──────────────────────────────────────────────

/// 管理信任阶段门控评估状态的核心结构
///
/// # 设计说明
///
/// - 使用单一的 `Mutex<GateEvalInner>` 确保间隔检查、评估和应用的原子性
/// - `DefaultGateEvaluator` 是一个单元结构体，直接内联调用而非存储
/// - 线程安全：通过 Mutex 保护内部状态，支持多线程环境
///
/// # 主要职责
///
/// 1. 定期检查门控条件（tick 机制）
/// 2. 评估是否满足阶段转换条件
/// 3. 应用门控决策到阶段状态
/// 4. 持久化阶段状态到内存后端
///
/// # 示例
///
/// ```ignore
/// use std::sync::Arc;
/// use std::time::Duration;
///
/// let memory = Arc::new(SomeMemoryBackend::new());
/// let gates = vec![]; // 从配置加载门控定义
///
/// let state = GateEvalState::new("my_agent", gates, 60, memory);
///
/// // 定期调用 tick 进行评估
/// if let Some(record) = state.tick(&metrics) {
///     println!("Gate fired: {:?}", record);
/// }
///
/// // 持久化状态
/// state.persist().await?;
/// ```
pub struct GateEvalState {
    /// 内部状态，通过 Mutex 保护以实现线程安全
    inner: Mutex<GateEvalInner>,
    /// 内存存储后端，用于持久化阶段状态
    memory: Arc<dyn Memory>,
    /// 门控定义列表，定义阶段转换的规则
    gates: Vec<Gate>,
    /// tick 调用之间的间隔时间
    tick_interval: Duration,
}

impl GateEvalState {
    /// 创建一个具有全新（默认）阶段状态的 GateEvalState 实例
    ///
    /// # 参数
    ///
    /// - `agent_name`: 代理名称，用于标识阶段状态
    /// - `gates`: 门控定义列表，定义阶段转换规则
    /// - `interval_secs`: tick 评估间隔时间（秒），设为 0 则禁用定期评估
    /// - `memory`: 内存存储后端，用于持久化状态
    ///
    /// # 返回值
    ///
    /// 返回一个新初始化的 GateEvalState 实例，阶段状态为默认初始状态
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let memory = Arc::new(InMemoryBackend::new());
    /// let gates = GateEvalState::load_gates_from_file(Path::new("persona.json"));
    /// let state = GateEvalState::new("agent_1", gates, 60, memory);
    /// ```
    pub fn new(
        agent_name: &str,
        gates: Vec<Gate>,
        interval_secs: u64,
        memory: Arc<dyn Memory>,
    ) -> Self {
        Self {
            inner: Mutex::new(GateEvalInner {
                phase_state: PhaseState::new(agent_name.to_string()),
                last_tick: Instant::now(),
            }),
            memory,
            gates,
            tick_interval: Duration::from_secs(interval_secs),
        }
    }

    /// 使用已知的阶段状态创建实例（热启动）
    ///
    /// 与 `new` 方法的区别在于，此方法接受一个现有的 PhaseState，
    /// 用于从持久化存储恢复状态或使用预设的阶段状态
    ///
    /// # 参数
    ///
    /// - `state`: 已有的阶段状态，将直接使用而非创建新的
    /// - `gates`: 门控定义列表
    /// - `interval_secs`: tick 评估间隔时间（秒）
    /// - `memory`: 内存存储后端
    ///
    /// # 返回值
    ///
    /// 返回一个使用指定阶段状态初始化的 GateEvalState 实例
    ///
    /// # 使用场景
    ///
    /// 通常在系统重启后从内存存储恢复状态时使用
    pub fn with_state(
        state: PhaseState,
        gates: Vec<Gate>,
        interval_secs: u64,
        memory: Arc<dyn Memory>,
    ) -> Self {
        Self {
            inner: Mutex::new(GateEvalInner { phase_state: state, last_tick: Instant::now() }),
            memory,
            gates,
            tick_interval: Duration::from_secs(interval_secs),
        }
    }

    /// 从 persona JSON 文件加载门控定义
    ///
    /// 期望的 JSON 格式：
    /// ```json
    /// {
    ///   "gates": [
    ///     { "id": "gate_1", "condition": "...", ... },
    ///     { "id": "gate_2", "condition": "...", ... }
    ///   ]
    /// }
    /// ```
    ///
    /// # 参数
    ///
    /// - `path`: persona JSON 文件的路径
    ///
    /// # 返回值
    ///
    /// - 成功时：返回解析出的门控定义列表
    /// - 文件不存在：返回空 Vec（静默处理）
    /// - 解析错误：记录警告日志并返回空 Vec
    ///
    /// # 错误处理
    ///
    /// - 文件读取失败：静默返回空列表
    /// - JSON 解析失败：记录警告日志，返回空列表
    /// - 不会抛出 panic 或返回 Result 错误
    pub fn load_gates_from_file(path: &Path) -> Vec<Gate> {
        // 尝试读取文件内容，失败则返回空列表
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        // 定义临时结构用于解析 JSON
        #[derive(serde::Deserialize)]
        struct PersonaGates {
            #[serde(default)]
            gates: Vec<Gate>,
        }

        // 解析 JSON，失败时记录警告并返回空列表
        match serde_json::from_str::<PersonaGates>(&content) {
            Ok(parsed) => parsed.gates,
            Err(e) => {
                warn!(path = %path.display(), error = %e, "failed to parse gates from persona file");
                Vec::new()
            }
        }
    }

    /// 从内存后端重建 GateEvalState（热启动）
    ///
    /// 此方法用于系统重启后恢复之前保存的状态，包括：
    /// 1. 从内存存储加载 PhaseState（键名：`sop_phase_state`）
    /// 2. 从文件加载门控定义
    /// 3. 在解析失败时回退到全新状态
    ///
    /// # 参数
    ///
    /// - `memory`: 内存存储后端，用于读取持久化的阶段状态
    /// - `agent_name`: 代理名称，在无法从内存恢复时用于创建新状态
    /// - `gates_file`: 可选的门控定义文件路径，None 表示不加载门控
    /// - `interval_secs`: tick 评估间隔时间（秒）
    ///
    /// # 返回值
    ///
    /// - `Ok(Self)`: 成功重建的 GateEvalState 实例
    /// - `Err`: 内存读取失败
    ///
    /// # 恢复策略
    ///
    /// - 内存中存在状态且解析成功 → 使用恢复的状态
    /// - 内存中存在但解析失败 → 记录警告，使用全新状态
    /// - 内存中不存在状态 → 使用全新状态
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let memory = Arc::new(SqliteMemory::new("state.db").await?);
    /// let state = GateEvalState::rebuild_from_memory(
    ///     memory,
    ///     "agent_1",
    ///     Some(Path::new("persona.json")),
    ///     60,
    /// ).await?;
    /// ```
    pub async fn rebuild_from_memory(
        memory: Arc<dyn Memory>,
        agent_name: &str,
        gates_file: Option<&Path>,
        interval_secs: u64,
    ) -> Result<Self> {
        // 从文件加载门控定义，无文件则使用空列表
        let gates = gates_file.map(Self::load_gates_from_file).unwrap_or_default();

        // 尝试从内存加载阶段状态
        let phase_state = match memory.get(PHASE_STATE_KEY).await? {
            Some(entry) => match serde_json::from_str::<PhaseState>(&entry.content) {
                Ok(state) => {
                    // 成功解析，记录恢复信息
                    info!(
                        phase = ?state.current_phase,
                        rev = state.state_rev,
                        "gate eval warm-started from memory"
                    );
                    state
                }
                Err(e) => {
                    // 解析失败，使用全新状态
                    warn!(error = %e, "failed to parse phase state from memory, using fresh state");
                    PhaseState::new(agent_name.to_string())
                }
            },
            // 内存中不存在状态，使用全新状态
            None => PhaseState::new(agent_name.to_string()),
        };

        Ok(Self::with_state(phase_state, gates, interval_secs, memory))
    }

    /// 原子性 tick 操作：间隔检查 + 评估 + 应用（在单个锁内完成）
    ///
    /// 这是门控评估的核心方法，定期调用以检查是否满足阶段转换条件。
    /// 整个操作在同一个 Mutex 锁内完成，确保原子性。
    ///
    /// # 参数
    ///
    /// - `metrics`: 指标提供者，用于评估门控条件（如成功率、延迟等指标）
    ///
    /// # 返回值
    ///
    /// - `Some(record)`: 有门控触发，返回决策记录
    /// - `None`: 无门控触发（可能是因为间隔未到、评估未通过或被禁用）
    ///
    /// # 执行流程
    ///
    /// 1. 检查是否禁用（interval_secs = 0）
    /// 2. 检查 Mutex 是否中毒
    /// 3. 检查是否到达评估间隔
    /// 4. 执行门控评估
    /// 5. 如果有决策，应用到阶段状态
    ///
    /// # 线程安全
    ///
    /// 所有操作在获取 Mutex 锁后执行，确保线程安全。
    /// 即使多个线程同时调用 tick，实际评估也会串行进行。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// // 在主循环中定期调用
    /// loop {
    ///     tokio::time::sleep(Duration::from_millis(100)).await;
    ///     if let Some(record) = gate_state.tick(&metrics) {
    ///         println!("Gate {} fired: {} -> {}", record.gate_id,
    ///                  record.from_phase, record.to_phase);
    ///     }
    /// }
    /// ```
    pub fn tick(&self, metrics: &dyn MetricsProvider) -> Option<GateDecisionRecord> {
        // 创建 tracing span 用于日志跟踪
        let _span = tracing::info_span!("gate_eval_tick", gates = self.gates.len()).entered();

        // interval_secs=0 表示禁用定期评估
        if self.tick_interval.is_zero() {
            return None;
        }

        // 检查 Mutex 是否中毒（之前持有锁的线程 panic）
        if self.inner.is_poisoned() {
            error!("gate eval mutex poisoned — loss of gate evaluation until restart");
            return None;
        }

        // 获取锁，失败则返回 None
        let mut inner = self.inner.lock().ok()?;

        // 检查是否到达评估间隔
        if inner.last_tick.elapsed() < self.tick_interval {
            return None;
        }
        // 更新最后评估时间
        inner.last_tick = Instant::now();

        // 使用默认评估器执行门控评估
        let record = DefaultGateEvaluator.evaluate(&self.gates, &inner.phase_state, metrics);

        // 处理评估结果
        match record {
            Some(ref record) => {
                // 在同一个锁内应用决策，确保原子性
                apply_decision(&mut inner.phase_state, record);
                info!(
                    gate_id = %record.gate_id,
                    decision = %record.decision,
                    from = ?record.from_phase,
                    to = %record.to_phase,
                    "gate decision"
                );
            }
            None => {
                // 无门控触发，仅记录调试日志
                debug!("no gate fired");
            }
        }

        record
    }

    /// 将当前阶段状态持久化到内存存储
    ///
    /// 将 PhaseState 序列化为 JSON 并存储到内存后端，
    /// 可用于系统重启后通过 `rebuild_from_memory` 恢复状态
    ///
    /// # 返回值
    ///
    /// - `Ok(())`: 持久化成功
    /// - `Err`: 锁获取失败或存储失败
    ///
    /// # 存储位置
    ///
    /// - 键名：`sop_phase_state`
    /// - 分类：自定义 `sop` 分类
    ///
    /// # 错误处理
    ///
    /// - Mutex 中毒：返回错误，包含中毒原因
    /// - 序列化失败：返回错误
    /// - 存储失败：返回底层存储错误
    ///
    /// # 调用时机
    ///
    /// 建议在以下场景调用：
    /// - 有门控决策触发后
    /// - 定期保存（如每隔 N 次评估）
    /// - 系统关闭前
    ///
    /// # 示例
    ///
    /// ```ignore
    /// if let Some(_) = gate_state.tick(&metrics) {
    ///     // 有决策触发，立即持久化
    ///     gate_state.persist().await?;
    /// }
    /// ```
    pub async fn persist(&self) -> Result<()> {
        // 在锁内序列化状态，避免长时间持有锁
        let content = {
            let inner =
                self.inner.lock().map_err(|e| anyhow::anyhow!("gate eval lock poisoned: {e}"))?;
            // 使用美化格式序列化，便于调试和审查
            serde_json::to_string_pretty(&inner.phase_state)?
        };
        // 存储到内存后端
        self.memory.store(PHASE_STATE_KEY, &content, sop_category(), None).await?;
        Ok(())
    }

    /// 获取当前阶段状态的快照（用于诊断和状态查询）
    ///
    /// 返回 PhaseState 的克隆副本，不会阻塞或影响 tick 评估
    ///
    /// # 返回值
    ///
    /// - `Some(state)`: 成功获取状态快照
    /// - `None`: Mutex 获取失败（锁中毒）
    ///
    /// # 用途
    ///
    /// - 诊断：检查当前阶段状态
    /// - 状态查询：对外暴露当前信任级别
    /// - 监控：上报指标和状态
    ///
    /// # 性能
    ///
    /// 克隆 PhaseState 的开销取决于其大小，但通常很小。
    /// 仅短暂持有锁（仅用于克隆），不会长时间阻塞。
    pub fn phase_state_snapshot(&self) -> Option<PhaseState> {
        self.inner.lock().ok().map(|g| g.phase_state.clone())
    }

    /// 获取已加载的门控定义数量
    ///
    /// # 返回值
    ///
    /// 返回当前加载的门控规则数量
    ///
    /// # 用途
    ///
    /// - 健康检查：确认门控定义已正确加载
    /// - 监控：统计配置的门控数量
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }
}

// ── 决策应用逻辑 ───────────────────────────────────────

/// 将门控决策应用到阶段状态
///
/// 根据决策类型更新 PhaseState，处理不同的决策场景：
/// - transition：立即执行阶段转换
/// - observed：仅观察，不改变状态
/// - pending_human：等待人工确认
///
/// # 参数
///
/// - `state`: 可变的阶段状态引用，将被更新
/// - `record`: 门控决策记录，包含决策详情
///
/// # 决策类型处理
///
/// ## transition - 立即转换
///
/// 执行即时阶段转换：
/// 1. 更新当前阶段为 `to_phase`
/// 2. 增加状态版本号 `state_rev`
/// 3. 记录转换历史到 `last_transition`
/// 4. 清除任何待处理的转换
///
/// ## observed - 仅观察
///
/// 门控条件被观察但无需转换：
/// - 不修改状态
/// - 记录调试日志
///
/// ## pending_human - 等待人工确认
///
/// 阶段转换需要人工批准：
/// - 将转换信息保存到 `pending_transition`
/// - 不立即改变当前阶段
/// - 等待外部系统或人工确认后再执行
///
/// ## 其他 - 未知决策
///
/// - 记录警告日志
/// - 跳过处理
fn apply_decision(state: &mut PhaseState, record: &GateDecisionRecord) {
    match record.decision.as_str() {
        // 立即执行阶段转换
        "transition" => {
            // 更新当前阶段
            state.current_phase = Some(record.to_phase.clone());
            // 增加状态版本号
            state.state_rev += 1;
            // 记录转换历史
            state.last_transition = Some(TransitionRecord {
                gate_id: record.gate_id.clone(),
                from_phase: record.from_phase.clone(),
                to_phase: record.to_phase.clone(),
                at: Utc::now(),
                // 生成唯一决策 ID
                decision_id: format!(
                    "{}-{}-{}",
                    record.gate_id, record.state_rev, record.metrics_hash
                ),
                metrics_hash: Some(record.metrics_hash.clone()),
                state_rev: state.state_rev,
            });
            // 清除待处理的转换
            state.pending_transition = None;
            // 更新修改时间
            state.updated_at = Utc::now();
        }
        // 仅观察，无状态变更
        "observed" => {
            debug!(
                gate_id = %record.gate_id,
                "observed gate — no state change"
            );
        }
        // 等待人工确认的转换
        "pending_human" => {
            state.pending_transition = Some(PendingTransition {
                gate_id: record.gate_id.clone(),
                from_phase: record.from_phase.clone(),
                to_phase: record.to_phase.clone(),
                decision: record.decision.clone(),
                metrics_hash: record.metrics_hash.clone(),
                state_rev: record.state_rev,
                created_at: Utc::now(),
            });
            state.updated_at = Utc::now();
        }
        // 未知决策类型
        other => {
            warn!(decision = %other, gate_id = %record.gate_id, "unknown gate decision — skipping");
        }
    }
}

// ── 单元测试模块 ──────────────────────────────────────────────────────

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
