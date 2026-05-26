//! SOP 事件分发模块单元测试
//!
//! 本模块包含 SOP（标准操作流程）事件分发逻辑的测试用例，涵盖：
//! - 事件触发与 SOP 匹配
//! - 冷却期（cooldown）机制
//! - 批量锁定与并发控制
//! - 不同执行模式（Auto/Supervised）的行为
//! - Cron 定时触发器的解析与调度
//!
//! 测试使用临时的内存引擎和审计日志器，确保每个测试用例的隔离性和可重复性。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    use crate::app::agent::config::{MemoryConfig, SopConfig};
    use crate::app::agent::memory::Memory;
    use crate::app::agent::sop::types::{
        Sop, SopExecutionMode, SopPriority, SopRunAction, SopStep, SopTrigger, SopTriggerSource,
    };

    /// 创建测试用的 SOP 实例
    ///
    /// # 参数
    /// - `name`: SOP 名称
    /// - `triggers`: 触发器列表
    ///
    /// # 返回值
    /// 返回一个配置了基础属性的 SOP 实例，包含单个测试步骤
    fn test_sop(name: &str, triggers: Vec<SopTrigger>) -> Sop {
        Sop {
            name: name.into(),
            description: format!("Test SOP: {name}"),
            version: "1.0.0".into(),
            priority: SopPriority::Normal,
            execution_mode: SopExecutionMode::Auto,
            triggers,
            steps: vec![SopStep {
                number: 1,
                title: "Step one".into(),
                body: "Do step one".into(),
                suggested_tools: vec![],
                requires_confirmation: false,
            }],
            cooldown_secs: 0,
            max_concurrent: 2,
            location: None,
        }
    }

    /// 创建测试用的 SOP 引擎实例
    ///
    /// # 参数
    /// - `sops`: 要加载到引擎中的 SOP 列表
    ///
    /// # 返回值
    /// 返回一个线程安全的 `Arc<Mutex<SopEngine>>` 实例，用于并发测试
    fn test_engine(sops: Vec<Sop>) -> Arc<Mutex<SopEngine>> {
        let mut engine = SopEngine::new(SopConfig::default());
        engine.set_sops_for_test(sops);
        Arc::new(Mutex::new(engine))
    }

    /// 创建测试用的审计日志器实例
    ///
    /// 创建一个基于 SQLite 内存数据库的审计日志器，用于记录测试中的 SOP 执行事件。
    /// 为了避免临时目录在测试结束前被删除，使用 `std::mem::forget` 防止清理。
    ///
    /// # 返回值
    /// 返回一个配置好的 `SopAuditLogger` 实例
    fn test_audit() -> SopAuditLogger {
        let mem_cfg = MemoryConfig { backend: "sqlite".into(), ..MemoryConfig::default() };
        let tmp = tempfile::tempdir().unwrap();
        let memory: Arc<dyn Memory> = Arc::from(
            crate::app::agent::memory::create_memory(&mem_cfg, tmp.path(), None).unwrap(),
        );
        // 故意泄漏临时目录，确保其在整个测试周期内存活
        std::mem::forget(tmp);
        SopAuditLogger::new(memory)
    }

    /// 测试：分发匹配的 SOP 事件应成功启动执行
    ///
    /// 验证当接收到与 SOP 触发器匹配的 MQTT 事件时，
    /// `dispatch_sop_event` 能够正确识别并启动对应的 SOP。
    #[tokio::test]
    async fn dispatch_starts_matching_sop() {
        let engine = test_engine(vec![test_sop(
            "mqtt-sop",
            vec![SopTrigger::Mqtt { topic: "sensors/temp".into(), condition: None }],
        )]);
        let audit = test_audit();

        let event = SopEvent {
            source: SopTriggerSource::Mqtt,
            topic: Some("sensors/temp".into()),
            payload: Some(r#"{"value": 42}"#.into()),
            timestamp: now_iso8601(),
        };

        let results = dispatch_sop_event(&engine, &audit, event).await;
        assert_eq!(results.len(), 1);
        assert!(
            matches!(&results[0], DispatchResult::Started { sop_name, action, .. } if sop_name == "mqtt-sop" && matches!(action, SopRunAction::ExecuteStep { .. }))
        );
    }

    /// 测试：冷却期激活时应跳过 SOP 执行
    ///
    /// 验证当 SOP 处于冷却期（cooldown）时，即使收到匹配的触发事件，
    /// 分发器也应跳过执行并返回 `DispatchResult::Skipped`。
    #[tokio::test]
    async fn dispatch_skips_when_cooldown_active() {
        let mut sop = test_sop("cooldown-sop", vec![SopTrigger::Manual]);
        sop.cooldown_secs = 3600;
        sop.max_concurrent = 1;
        let engine = test_engine(vec![sop]);
        let audit = test_audit();

        // 手动启动一次执行，以便完成后能触发冷却期
        {
            let mut eng = engine.lock().unwrap();
            let _action = eng
                .start_run(
                    "cooldown-sop",
                    SopEvent {
                        source: SopTriggerSource::Manual,
                        topic: None,
                        payload: None,
                        timestamp: now_iso8601(),
                    },
                )
                .unwrap();
            // 完成该次执行
            let run_id = eng.active_runs().keys().next().unwrap().clone();
            eng.advance_step(
                &run_id,
                crate::app::agent::sop::types::SopStepResult {
                    step_number: 1,
                    status: crate::app::agent::sop::types::SopStepStatus::Completed,
                    output: "done".into(),
                    started_at: now_iso8601(),
                    completed_at: Some(now_iso8601()),
                },
            )
            .unwrap();
        }

        // 现在分发事件 —— 由于冷却期应该被跳过
        let event = SopEvent {
            source: SopTriggerSource::Manual,
            topic: None,
            payload: None,
            timestamp: now_iso8601(),
        };
        let results = dispatch_sop_event(&engine, &audit, event).await;
        assert_eq!(results.len(), 1);
        assert!(
            matches!(&results[0], DispatchResult::Skipped { sop_name, .. } if sop_name == "cooldown-sop")
        );
    }

    /// 测试：未知事件应返回 NoMatch 结果
    ///
    /// 验证当接收到与所有已注册 SOP 触发器都不匹配的事件时，
    /// 分发器应返回 `DispatchResult::NoMatch`。
    #[tokio::test]
    async fn dispatch_returns_no_match_for_unknown_event() {
        let engine = test_engine(vec![test_sop("manual-sop", vec![SopTrigger::Manual])]);
        let audit = test_audit();

        // 发送 MQTT 事件 —— 但 SOP 只有手动触发器
        let event = SopEvent {
            source: SopTriggerSource::Mqtt,
            topic: Some("some/topic".into()),
            payload: None,
            timestamp: now_iso8601(),
        };
        let results = dispatch_sop_event(&engine, &audit, event).await;
        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], DispatchResult::NoMatch));
    }

    /// 测试：批量锁定应能同时启动多个匹配的 SOP
    ///
    /// 验证当单个事件匹配多个 SOP 时（如多个 SOP 监听同一个 Webhook 路径），
    /// 分发器应能够启动所有匹配的 SOP。
    #[tokio::test]
    async fn dispatch_batch_lock_starts_multiple_sops() {
        let sop1 =
            test_sop("webhook-sop-1", vec![SopTrigger::Webhook { path: "/api/deploy".into() }]);
        let sop2 =
            test_sop("webhook-sop-2", vec![SopTrigger::Webhook { path: "/api/deploy".into() }]);
        let engine = test_engine(vec![sop1, sop2]);
        let audit = test_audit();

        let event = SopEvent {
            source: SopTriggerSource::Webhook,
            topic: Some("/api/deploy".into()),
            payload: None,
            timestamp: now_iso8601(),
        };

        let results = dispatch_sop_event(&engine, &audit, event).await;
        let started_count =
            results.iter().filter(|r| matches!(r, DispatchResult::Started { .. })).count();
        assert_eq!(started_count, 2);
    }

    /// 测试（B1 DoD）：监督模式 SOP 应返回 WaitApproval 动作
    ///
    /// 验证监督模式的 SOP 在分发时返回的 action 是 `SopRunAction::WaitApproval`，
    /// 而不是被静默丢弃。这确保了 `start_run` 返回的动作能正确传递到 `DispatchResult::Started` 中。
    #[tokio::test]
    async fn dispatch_captures_action_for_wait_approval() {
        // 监督模式 → 第一步需要等待审批
        let mut sop = test_sop(
            "supervised-sop",
            vec![SopTrigger::Mqtt { topic: "alert".into(), condition: None }],
        );
        sop.execution_mode = SopExecutionMode::Supervised;
        let engine = test_engine(vec![sop]);
        let audit = test_audit();

        let event = SopEvent {
            source: SopTriggerSource::Mqtt,
            topic: Some("alert".into()),
            payload: None,
            timestamp: now_iso8601(),
        };

        let results = dispatch_sop_event(&engine, &audit, event).await;
        assert_eq!(results.len(), 1);
        match &results[0] {
            DispatchResult::Started { run_id, sop_name, action } => {
                assert_eq!(sop_name, "supervised-sop");
                assert!(!run_id.is_empty());
                assert!(
                    matches!(action, SopRunAction::WaitApproval { .. }),
                    "Supervised SOP must return WaitApproval, got {:?}",
                    action
                );
            }
            other => panic!("Expected Started, got {other:?}"),
        }
    }

    /// B1 DoD: Auto-mode SOP returns ExecuteStep action in dispatch result.
    #[tokio::test]
    async fn dispatch_captures_action_for_execute_step() {
        let engine = test_engine(vec![test_sop("auto-sop", vec![SopTrigger::Manual])]);
        let audit = test_audit();

        let event = SopEvent {
            source: SopTriggerSource::Manual,
            topic: None,
            payload: None,
            timestamp: now_iso8601(),
        };

        let results = dispatch_sop_event(&engine, &audit, event).await;
        assert_eq!(results.len(), 1);
        match &results[0] {
            DispatchResult::Started { action, .. } => {
                assert!(
                    matches!(action, SopRunAction::ExecuteStep { .. }),
                    "Auto SOP must return ExecuteStep, got {:?}",
                    action
                );
            }
            other => panic!("Expected Started, got {other:?}"),
        }
    }

    #[test]
    fn cron_cache_skips_invalid_expression() {
        let sop =
            test_sop("bad-cron", vec![SopTrigger::Cron { expression: "not a valid cron".into() }]);
        let engine = test_engine(vec![sop]);
        let cache = SopCronCache::from_engine(&engine);
        assert!(cache.schedules().is_empty());
    }

    #[test]
    fn cron_cache_parses_valid_expression() {
        let sop =
            test_sop("valid-cron", vec![SopTrigger::Cron { expression: "0 */5 * * *".into() }]);
        let engine = test_engine(vec![sop]);
        let cache = SopCronCache::from_engine(&engine);
        assert_eq!(cache.schedules().len(), 1);
        assert_eq!(cache.schedules()[0].0, "valid-cron");
        assert_eq!(cache.schedules()[0].1, "0 */5 * * *");
    }

    #[tokio::test]
    async fn cron_sop_trigger_fires_on_schedule() {
        let sop = test_sop("cron-sop", vec![SopTrigger::Cron { expression: "* * * * *".into() }]);
        let engine = test_engine(vec![sop]);
        let audit = test_audit();
        let cache = SopCronCache::from_engine(&engine);

        // Set last_check to 2 minutes ago so the window contains a tick
        let mut last_check = chrono::Utc::now() - chrono::Duration::minutes(2);
        let results = check_sop_cron_triggers(&engine, &audit, &cache, &mut last_check).await;

        let started =
            results.iter().filter(|r| matches!(r, DispatchResult::Started { .. })).count();
        assert!(started >= 1, "Expected at least 1 started SOP from cron");
    }

    #[tokio::test]
    async fn cron_sop_only_matching_expression_fires() {
        let sop1 = test_sop("every-min", vec![SopTrigger::Cron { expression: "* * * * *".into() }]);
        // An expression that won't fire in a 2-minute window from now:
        // "0 0 1 1 *" = midnight Jan 1
        let sop2 = test_sop("yearly", vec![SopTrigger::Cron { expression: "0 0 1 1 *".into() }]);
        let engine = test_engine(vec![sop1, sop2]);
        let audit = test_audit();
        let cache = SopCronCache::from_engine(&engine);

        let mut last_check = chrono::Utc::now() - chrono::Duration::minutes(2);
        let results = check_sop_cron_triggers(&engine, &audit, &cache, &mut last_check).await;

        // Only "every-min" should have fired
        let started_names: Vec<&str> = results
            .iter()
            .filter_map(|r| match r {
                DispatchResult::Started { sop_name, .. } => Some(sop_name.as_str()),
                _ => None,
            })
            .collect();
        assert!(started_names.contains(&"every-min"));
        assert!(!started_names.contains(&"yearly"));
    }

    #[tokio::test]
    async fn cron_sop_window_check_does_not_miss_tick() {
        let sop = test_sop("every-min", vec![SopTrigger::Cron { expression: "* * * * *".into() }]);
        let engine = test_engine(vec![sop]);
        let audit = test_audit();
        let cache = SopCronCache::from_engine(&engine);

        // Simulate: last_check was 5 minutes ago, poll just now
        let mut last_check = chrono::Utc::now() - chrono::Duration::minutes(5);
        let results = check_sop_cron_triggers(&engine, &audit, &cache, &mut last_check).await;

        // At least one tick should have been caught
        let started =
            results.iter().filter(|r| matches!(r, DispatchResult::Started { .. })).count();
        assert!(started >= 1, "Window-based check should catch ticks from 5 minutes ago");

        // last_check should be updated to approximately now
        let now = chrono::Utc::now();
        assert!((now - last_check).num_seconds() < 2, "last_check should be updated to now");
    }
}
