//! # SOP 门控评估状态测试模块
//!
//! 本模块包含对 SOP（标准操作流程）门控评估系统的全面测试用例。
//!
//! ## 主要功能
//!
//! - 测试门控评估状态（`GateEvalState`）的各种行为
//! - 验证门控评估逻辑（提升/降级决策）
//! - 测试从文件加载门控配置
//! - 验证状态持久化与热启动
//! - 测试与真实指标收集器的集成
//!
//! ## 测试覆盖
//!
//! - 基本门控评估（通过/失败）
//! - 阶段转换逻辑
//! - 强制模式 vs 观察模式
//! - 人工审批流程
//! - 冷启动 vs 热启动
//! - 门控加载错误处理
//! - 降级优先级规则
//! - 幂等性保证
//!
//! ## Mock 对象
//!
//! 使用 `MockMetrics` 模拟指标提供者，支持预设指标值用于测试。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    use ampersona_core::errors::MetricError;
    use ampersona_core::spec::gates::Gate;
    use ampersona_core::traits::{MetricQuery, MetricSample};
    use ampersona_core::types::{CriterionOp, GateApproval, GateDirection, GateEnforcement};
    use serde_json::json;
    use std::collections::HashMap;

    // ─────────────────────────────────────────────────────────────
    // Mock 指标提供者
    // ─────────────────────────────────────────────────────────────

    /// Mock 指标提供者
    ///
    /// 用于测试环境中的指标数据模拟，允许预先设置指标值。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let metrics = MockMetrics::new(vec![
    ///     ("sop.completion_rate", json!(0.9)),
    ///     ("sop.deviation_rate", json!(0.1)),
    /// ]);
    /// ```
    struct MockMetrics {
        /// 存储指标名称到值的映射
        values: HashMap<String, serde_json::Value>,
    }

    impl MockMetrics {
        /// 创建新的 Mock 指标提供者
        ///
        /// # 参数
        ///
        /// - `values`: 指标名称和值的键值对列表
        ///
        /// # 返回值
        ///
        /// 返回初始化后的 `MockMetrics` 实例
        ///
        /// # 示例
        ///
        /// ```ignore
        /// let metrics = MockMetrics::new(vec![
        ///     ("metric_name", json!(42)),
        /// ]);
        /// ```
        fn new(values: Vec<(&str, serde_json::Value)>) -> Self {
            Self { values: values.into_iter().map(|(k, v)| (k.to_string(), v)).collect() }
        }
    }

    impl MetricsProvider for MockMetrics {
        /// 获取指定指标的值
        ///
        /// # 参数
        ///
        /// - `query`: 指标查询请求，包含指标名称
        ///
        /// # 返回值
        ///
        /// - `Ok(MetricSample)`: 找到指标时返回采样数据
        /// - `Err(MetricError::NotFound)`: 指标不存在时返回错误
        fn get_metric(&self, query: &MetricQuery) -> Result<MetricSample, MetricError> {
            self.values
                .get(&query.name)
                .cloned()
                .map(|value| MetricSample {
                    name: query.name.clone(),
                    value,
                    sampled_at: Utc::now(),
                })
                .ok_or_else(|| MetricError::NotFound(query.name.clone()))
        }
    }

    // ─────────────────────────────────────────────────────────────
    // 辅助函数
    // ─────────────────────────────────────────────────────────────

    /// 创建提升（Promote）类型的门控配置
    ///
    /// 快速创建用于测试的提升门控，简化测试用例的编写。
    ///
    /// # 参数
    ///
    /// - `id`: 门控唯一标识符
    /// - `metric`: 要检查的指标名称
    /// - `op`: 比较操作符（如 Gte 表示大于等于）
    /// - `value`: 阈值
    /// - `to_phase`: 目标阶段名称
    ///
    /// # 返回值
    ///
    /// 返回配置好的 `Gate` 实例
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let gate = make_promote_gate(
    ///     "gate-1",
    ///     "sop.completion_rate",
    ///     CriterionOp::Gte,
    ///     json!(0.8),
    ///     "active",
    /// );
    /// ```
    fn make_promote_gate(
        id: &str,
        metric: &str,
        op: CriterionOp,
        value: serde_json::Value,
        to_phase: &str,
    ) -> Gate {
        Gate {
            id: id.into(),
            direction: GateDirection::Promote,
            enforcement: GateEnforcement::Enforce,
            priority: 0,
            cooldown_seconds: 0,
            from_phase: None,
            to_phase: to_phase.into(),
            criteria: vec![ampersona_core::spec::gates::Criterion {
                metric: metric.into(),
                op,
                value,
                window_seconds: None,
            }],
            metrics_schema: None,
            approval: GateApproval::Auto,
            on_pass: None,
        }
    }

    /// 创建测试用的内存存储实例
    ///
    /// 创建基于 SQLite 后端的内存存储，用于测试状态持久化。
    /// 使用临时目录避免污染实际数据。
    ///
    /// # 返回值
    ///
    /// 返回线程安全的内存存储引用（`Arc<dyn Memory>`）
    ///
    /// # 注意
    ///
    /// 临时目录会在测试结束时自动清理
    fn test_memory() -> Arc<dyn Memory> {
        let mem_cfg = crate::app::agent::config::MemoryConfig {
            backend: "sqlite".into(),
            ..crate::app::agent::config::MemoryConfig::default()
        };
        let tmp = tempfile::tempdir().unwrap();
        Arc::from(crate::app::agent::memory::create_memory(&mem_cfg, tmp.path(), None).unwrap())
    }

    // ─────────────────────────────────────────────────────────────
    // 测试用例
    // ─────────────────────────────────────────────────────────────

    /// 测试无门控配置时的行为
    ///
    /// 验证当没有任何门控配置时，tick 操作应返回 None。
    ///
    /// # 测试场景
    ///
    /// - 空门控列表
    /// - 已过 tick 间隔
    /// - 期望返回 None
    #[test]
    fn tick_no_gates_returns_none() {
        let mem = test_memory();
        let ge = GateEvalState::new("test-agent", vec![], 1, mem);
        let metrics = MockMetrics::new(vec![]);

        // 强制设置已过间隔时间，确保满足 tick 条件
        {
            let mut inner = ge.inner.lock().unwrap();
            inner.last_tick = Instant::now().checked_sub(Duration::from_secs(10)).unwrap();
        }

        // 无门控时应返回 None
        assert!(ge.tick(&metrics).is_none());
    }

    /// 测试门控通过时返回决策记录
    ///
    /// 验证当指标满足门控条件时，tick 操作应返回正确的决策记录。
    ///
    /// # 测试场景
    ///
    /// - 单个提升门控，阈值为 0.8
    /// - 指标值为 0.9（满足条件）
    /// - 期望返回决策记录，包含正确的门控 ID 和目标阶段
    #[test]
    fn tick_with_passing_gate_returns_decision() {
        let mem = test_memory();
        let gate =
            make_promote_gate("g1", "sop.completion_rate", CriterionOp::Gte, json!(0.8), "active");
        let ge = GateEvalState::new("test-agent", vec![gate], 1, mem);

        // 设置指标值为 0.9，满足 >= 0.8 的条件
        let metrics = MockMetrics::new(vec![("sop.completion_rate", json!(0.9))]);

        {
            let mut inner = ge.inner.lock().unwrap();
            inner.last_tick = Instant::now().checked_sub(Duration::from_secs(10)).unwrap();
        }

        let record = ge.tick(&metrics);

        // 应返回决策记录
        assert!(record.is_some());
        let record = record.unwrap();
        assert_eq!(record.gate_id, "g1");
        assert_eq!(record.to_phase, "active");
    }

    /// 测试门控触发后阶段实际发生转换
    ///
    /// 验证门控决策生效后，阶段状态确实发生了变化。
    ///
    /// # 测试场景
    ///
    /// - 初始阶段为 None
    /// - 门控通过后应转换为 "active"
    /// - 状态版本号应增加
    /// - 应记录最后转换时间
    #[test]
    fn tick_transition_advances_phase() {
        let mem = test_memory();
        let gate =
            make_promote_gate("g1", "sop.completion_rate", CriterionOp::Gte, json!(0.8), "active");
        let ge = GateEvalState::new("test-agent", vec![gate], 1, mem);
        let metrics = MockMetrics::new(vec![("sop.completion_rate", json!(0.95))]);

        {
            let mut inner = ge.inner.lock().unwrap();
            inner.last_tick = Instant::now().checked_sub(Duration::from_secs(10)).unwrap();
        }

        // 触发门控评估
        ge.tick(&metrics);

        // 验证阶段状态已更新
        let snap = ge.phase_state_snapshot().unwrap();
        assert_eq!(snap.current_phase, Some("active".into()));
        assert!(snap.state_rev > 0); // 版本号应增加
        assert!(snap.last_transition.is_some()); // 应记录转换时间
    }

    /// 测试观察模式下的门控评估
    ///
    /// 验证当门控设置为观察（Observe）模式时：
    /// - 门控评估仍然执行
    /// - 但不实际改变阶段状态
    /// - 决策记录标记为 "observed"
    ///
    /// # 测试场景
    ///
    /// - 门控强制模式设为 Observe
    /// - 指标满足条件
    /// - 期望返回 "observed" 决策
    /// - 阶段状态保持不变
    #[test]
    fn tick_observed_no_state_change() {
        let mem = test_memory();
        let mut gate =
            make_promote_gate("g1", "sop.completion_rate", CriterionOp::Gte, json!(0.8), "active");
        gate.enforcement = GateEnforcement::Observe; // 设置为观察模式

        let ge = GateEvalState::new("test-agent", vec![gate], 1, mem);
        let metrics = MockMetrics::new(vec![("sop.completion_rate", json!(0.95))]);

        {
            let mut inner = ge.inner.lock().unwrap();
            inner.last_tick = Instant::now().checked_sub(Duration::from_secs(10)).unwrap();
        }

        let record = ge.tick(&metrics);

        // 应返回 "observed" 决策
        assert!(record.is_some());
        assert_eq!(record.unwrap().decision, "observed");

        // 阶段状态应保持不变
        let snap = ge.phase_state_snapshot().unwrap();
        assert!(snap.current_phase.is_none()); // 无变化
        assert_eq!(snap.state_rev, 0); // 版本号仍为 0
    }

    /// 测试需要人工审批的门控
    ///
    /// 验证当门控需要人工审批时：
    /// - 门控评估仍然执行
    /// - 决策记录标记为 "pending_human"
    /// - 阶段状态记录待定转换
    ///
    /// # 测试场景
    ///
    /// - 门控审批模式设为 Human
    /// - 指标满足条件
    /// - 期望返回 "pending_human" 决策
    /// - 阶段状态记录待定转换
    #[test]
    fn tick_pending_human_sets_pending() {
        let mem = test_memory();
        let mut gate =
            make_promote_gate("g1", "sop.completion_rate", CriterionOp::Gte, json!(0.8), "active");
        gate.approval = GateApproval::Human; // 设置为需要人工审批

        let ge = GateEvalState::new("test-agent", vec![gate], 1, mem);
        let metrics = MockMetrics::new(vec![("sop.completion_rate", json!(0.95))]);

        {
            let mut inner = ge.inner.lock().unwrap();
            inner.last_tick = Instant::now().checked_sub(Duration::from_secs(10)).unwrap();
        }

        let record = ge.tick(&metrics);

        // 应返回 "pending_human" 决策
        assert!(record.is_some());
        assert_eq!(record.unwrap().decision, "pending_human");

        // 阶段状态应记录待定转换
        let snap = ge.phase_state_snapshot().unwrap();
        assert!(snap.pending_transition.is_some());
        assert_eq!(snap.pending_transition.unwrap().to_phase, "active");
    }

    /// 测试加载不存在的门控文件
    ///
    /// 验证当门控配置文件不存在时，应返回空列表而不报错。
    ///
    /// # 测试场景
    ///
    /// - 文件路径不存在
    /// - 期望返回空列表
    #[test]
    fn load_gates_missing_file_returns_empty() {
        let gates = GateEvalState::load_gates_from_file(Path::new("/nonexistent/persona.json"));
        assert!(gates.is_empty());
    }

    /// 测试从有效的 persona 文件加载门控
    ///
    /// 验证能正确解析包含门控配置的 persona 文件。
    ///
    /// # 测试场景
    ///
    /// - 创建包含有效门控配置的临时文件
    /// - 期望成功加载门控列表
    /// - 门控 ID 应正确解析
    #[test]
    fn load_gates_valid_persona() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("persona.json");
        std::fs::write(
            &path,
            r#"{
                    "gates": [{
                        "id": "g1",
                        "direction": "promote",
                        "to_phase": "active",
                        "criteria": [{"metric": "sop.completion_rate", "op": "gte", "value": 0.8}]
                    }]
                }"#,
        )
        .unwrap();

        let gates = GateEvalState::load_gates_from_file(&path);

        assert_eq!(gates.len(), 1);
        assert_eq!(gates[0].id, "g1");
    }

    /// 测试从不含 gates 键的 persona 文件加载
    ///
    /// 验证当 persona 文件不包含 "gates" 键时，应返回空列表。
    ///
    /// # 测试场景
    ///
    /// - persona 文件仅包含其他字段（如 "name"）
    /// - 期望返回空列表
    #[test]
    fn load_gates_no_gates_key_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("persona.json");
        std::fs::write(&path, r#"{"name": "test"}"#).unwrap();

        let gates = GateEvalState::load_gates_from_file(&path);
        assert!(gates.is_empty());
    }

    /// 测试从无效 JSON 文件加载门控
    ///
    /// 验证当文件内容不是有效 JSON 时，应返回空列表而不崩溃。
    ///
    /// # 测试场景
    ///
    /// - 文件内容为无效 JSON
    /// - 期望返回空列表
    #[test]
    fn load_gates_invalid_json_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("persona.json");
        std::fs::write(&path, "not json at all {{{").unwrap();

        let gates = GateEvalState::load_gates_from_file(&path);
        assert!(gates.is_empty());
    }

    /// 测试热启动的状态恢复（完整往返）
    ///
    /// 验证从内存中恢复状态后，能正确重建门控评估状态，
    /// 包括已转换的阶段和门控配置。
    ///
    /// # 测试流程
    ///
    /// 1. 创建门控评估状态并触发阶段转换
    /// 2. 持久化状态到内存
    /// 3. 从内存重建状态
    /// 4. 验证重建后的状态与原状态一致
    #[tokio::test]
    async fn warm_start_roundtrip() {
        let mem = test_memory();
        let gate =
            make_promote_gate("g1", "sop.completion_rate", CriterionOp::Gte, json!(0.8), "active");

        // 创建状态，触发评估，持久化
        let ge = GateEvalState::new("test-agent", vec![gate.clone()], 1, Arc::clone(&mem));
        let metrics = MockMetrics::new(vec![("sop.completion_rate", json!(0.95))]);
        {
            let mut inner = ge.inner.lock().unwrap();
            inner.last_tick = Instant::now().checked_sub(Duration::from_secs(10)).unwrap();
        }
        ge.tick(&metrics);
        ge.persist().await.unwrap();

        // 写入门控配置文件用于重建
        let dir = tempfile::tempdir().unwrap();
        let gates_path = dir.path().join("persona.json");
        std::fs::write(
            &gates_path,
            serde_json::to_string(&serde_json::json!({"gates": [gate]})).unwrap(),
        )
        .unwrap();

        // 从内存重建状态
        let ge2 = GateEvalState::rebuild_from_memory(
            Arc::clone(&mem),
            "test-agent",
            Some(gates_path.as_path()),
            1,
        )
        .await
        .unwrap();

        // 验证重建后的状态
        let snap = ge2.phase_state_snapshot().unwrap();
        assert_eq!(snap.current_phase, Some("active".into()));
        assert!(snap.state_rev > 0);
        assert_eq!(ge2.gate_count(), 1);
    }

    /// 测试空内存的热启动
    ///
    /// 验证从空内存重建时，应返回初始状态。
    ///
    /// # 测试场景
    ///
    /// - 内存中无任何状态记录
    /// - 期望重建后为初始状态（无当前阶段，版本号为 0，无门控）
    #[tokio::test]
    async fn warm_start_empty_memory() {
        let mem = test_memory();
        let ge = GateEvalState::rebuild_from_memory(Arc::clone(&mem), "test-agent", None, 60)
            .await
            .unwrap();

        let snap = ge.phase_state_snapshot().unwrap();
        assert!(snap.current_phase.is_none()); // 无当前阶段
        assert_eq!(snap.state_rev, 0); // 版本号为 0
        assert_eq!(ge.gate_count(), 0); // 无门控
    }

    /// 测试降级门控优先于提升门控
    ///
    /// 验证当同时满足提升和降级条件时，降级门控应优先执行。
    /// 这是门控评估器的重要安全特性。
    ///
    /// # 测试场景
    ///
    /// - 当前阶段为 "active"
    /// - 提升门控条件满足（可提升到更高的阶段）
    /// - 降级门控条件也满足（应降级到 "restricted"）
    /// - 期望执行降级而非提升
    #[test]
    fn demote_priority_over_promote() {
        let mem = test_memory();

        // 创建提升门控
        let promote = make_promote_gate(
            "promote-g",
            "sop.completion_rate",
            CriterionOp::Gte,
            json!(0.8),
            "active",
        );

        // 创建降级门控
        let mut demote = make_promote_gate(
            "demote-g",
            "sop.deviation_rate",
            CriterionOp::Gte,
            json!(0.3),
            "restricted",
        );
        demote.direction = GateDirection::Demote;
        demote.from_phase = Some("active".into());

        // 设置当前阶段为 "active"
        let state = PhaseState {
            current_phase: Some("active".into()),
            ..PhaseState::new("test-agent".into())
        };
        let ge = GateEvalState::with_state(state, vec![promote, demote], 1, mem);

        // 设置指标：同时满足提升和降级条件
        let metrics = MockMetrics::new(vec![
            ("sop.completion_rate", json!(0.95)), // 满足提升
            ("sop.deviation_rate", json!(0.5)), // 满足降级
        ]);

        {
            let mut inner = ge.inner.lock().unwrap();
            inner.last_tick = Instant::now().checked_sub(Duration::from_secs(10)).unwrap();
        }

        let record = ge.tick(&metrics).unwrap();

        // 降级门控应优先触发（评估器按降级优先排序）
        assert_eq!(record.gate_id, "demote-g");
        assert_eq!(record.to_phase, "restricted");
    }

    /// 测试应用后的幂等 tick
    ///
    /// 验证门控评估的幂等性：当指标和状态未变化时，
    /// 不应重复触发相同的门控决策。
    ///
    /// # 测试场景
    ///
    /// - 第一次 tick 触发门控
    /// - 第二次 tick（相同指标，状态已更新）应返回 None
    /// - 通过 metrics_hash + state_rev 实现幂等性
    #[test]
    fn idempotent_tick_after_apply() {
        let mem = test_memory();
        let gate =
            make_promote_gate("g1", "sop.completion_rate", CriterionOp::Gte, json!(0.8), "active");
        let ge = GateEvalState::new("test-agent", vec![gate], 1, mem);
        let metrics = MockMetrics::new(vec![("sop.completion_rate", json!(0.95))]);

        // 第一次 tick — 应触发门控
        {
            let mut inner = ge.inner.lock().unwrap();
            inner.last_tick = Instant::now().checked_sub(Duration::from_secs(10)).unwrap();
        }
        let first = ge.tick(&metrics);
        assert!(first.is_some());

        // 第二次 tick（相同指标 + 已更新 state_rev）— 不应再次触发
        // （通过 metrics_hash + state_rev 实现幂等性）
        {
            let mut inner = ge.inner.lock().unwrap();
            inner.last_tick = Instant::now().checked_sub(Duration::from_secs(10)).unwrap();
        }
        let second = ge.tick(&metrics);
        assert!(second.is_none());
    }

    /// 测试与真实指标收集器的集成
    ///
    /// 验证门控评估能与实际的 SOP 指标收集器正确工作。
    ///
    /// # 测试场景
    ///
    /// - 使用真实的 `SopMetricsCollector`
    /// - 记录一次完整的 SOP 运行
    /// - 验证门控能基于收集的指标做出正确决策
    #[test]
    fn gate_tick_with_real_collector() {
        use crate::app::agent::sop::metrics::SopMetricsCollector;
        use crate::app::agent::sop::types::{
            SopEvent, SopRun, SopRunStatus, SopStepResult, SopStepStatus, SopTriggerSource,
        };

        let mem = test_memory();
        let collector = SopMetricsCollector::new();

        // 记录一次已完成的 SOP 运行
        let run = SopRun {
            run_id: "r1".into(),
            sop_name: "test-sop".into(),
            trigger_event: SopEvent {
                source: SopTriggerSource::Manual,
                topic: None,
                payload: None,
                timestamp: "2026-02-19T12:00:00Z".into(),
            },
            status: SopRunStatus::Completed,
            current_step: 1,
            total_steps: 1,
            started_at: "2026-02-19T12:00:00Z".into(),
            completed_at: Some("2026-02-19T12:05:00Z".into()),
            step_results: vec![SopStepResult {
                step_number: 1,
                status: SopStepStatus::Completed,
                output: "done".into(),
                started_at: "2026-02-19T12:00:00Z".into(),
                completed_at: Some("2026-02-19T12:01:00Z".into()),
            }],
            waiting_since: None,
        };
        collector.record_run_complete(&run);

        let gate =
            make_promote_gate("g1", "sop.completion_rate", CriterionOp::Gte, json!(0.8), "active");
        let ge = GateEvalState::new("test-agent", vec![gate], 1, mem);
        {
            let mut inner = ge.inner.lock().unwrap();
            inner.last_tick = Instant::now().checked_sub(Duration::from_secs(10)).unwrap();
        }

        let record = ge.tick(&collector);
        assert!(record.is_some());
        assert_eq!(record.unwrap().to_phase, "active");
    }

    /// 测试 tick 间隔限制
    ///
    /// 验证门控评估遵守配置的 tick 间隔。
    ///
    /// # 测试场景
    ///
    /// - 长间隔（3600 秒）：未到时间不触发
    /// - 零间隔：禁用门控评估
    #[test]
    fn tick_respects_interval() {
        let mem = test_memory();
        let gate =
            make_promote_gate("g1", "sop.completion_rate", CriterionOp::Gte, json!(0.8), "active");

        // 长间隔配置
        let ge = GateEvalState::new("test-agent", vec![gate.clone()], 3600, mem.clone());
        let metrics = MockMetrics::new(vec![("sop.completion_rate", json!(0.95))]);

        // last_tick 是 Instant::now() — 时间未到
        assert!(ge.tick(&metrics).is_none());

        // 零间隔 = 禁用门控
        let ge_disabled = GateEvalState::new("test-agent", vec![gate], 0, mem);
        assert!(ge_disabled.tick(&metrics).is_none());
    }

    /// 测试 ampersona 决策字符串的稳定性（金丝雀测试）
    ///
    /// 验证 `DefaultGateEvaluator` 产生的决策字符串符合预期。
    /// 如果 ampersona 库更改了这些字符串，此测试将失败。
    ///
    /// # 测试场景
    ///
    /// - Enforce 模式提升 → "transition"
    /// - Observe 模式提升 → "observed"
    /// - RequireApproval 模式提升 → "pending_human"
    #[test]
    fn ampersona_decision_strings_stable() {
        let state = PhaseState::new("test".into());

        // Enforce 提升模式 → "transition"
        let enforce_gate =
            make_promote_gate("g-enforce", "m", CriterionOp::Gte, json!(1), "phase-b");
        let metrics = MockMetrics::new(vec![("m", json!(1))]);
        let record = DefaultGateEvaluator.evaluate(&[enforce_gate], &state, &metrics);
        assert_eq!(record.as_ref().map(|r| r.decision.as_str()), Some("transition"));

        // Observe 提升模式 → "observed"
        let mut observe_gate =
            make_promote_gate("g-observe", "m", CriterionOp::Gte, json!(1), "phase-b");
        observe_gate.enforcement = GateEnforcement::Observe;
        let record = DefaultGateEvaluator.evaluate(&[observe_gate], &state, &metrics);
        assert_eq!(record.as_ref().map(|r| r.decision.as_str()), Some("observed"));

        // RequireApproval 提升模式 → "pending_human"
        let mut approval_gate =
            make_promote_gate("g-approval", "m", CriterionOp::Gte, json!(1), "phase-b");
        approval_gate.approval = GateApproval::Human;
        let record = DefaultGateEvaluator.evaluate(&[approval_gate], &state, &metrics);
        assert_eq!(record.as_ref().map(|r| r.decision.as_str()), Some("pending_human"));
    }
}
