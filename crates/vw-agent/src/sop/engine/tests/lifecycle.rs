//! SOP 运行生命周期测试。
//!
//! 验证从启动、推进、完成、失败、取消到并发/冷却限制的核心状态变化，保证
//! 引擎不会遗留活跃运行或绕过运行数量限制。

use super::fixtures::{engine_with_sops, extract_run_id, manual_event, test_sop};
use super::*;

#[test]
fn start_run_returns_first_step() {
    let mut engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::Auto, SopPriority::Normal)]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    let run_id = extract_run_id(&action);
    assert!(run_id.starts_with("run-"));
    assert!(matches!(action, SopRunAction::ExecuteStep { .. }));
    assert_eq!(engine.active_runs().len(), 1);
}

#[test]
fn start_run_unknown_sop_fails() {
    let mut engine = engine_with_sops(vec![]);
    assert!(engine.start_run("nonexistent", manual_event()).is_err());
}

#[test]
fn advance_step_to_completion() {
    let mut engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::Auto, SopPriority::Normal)]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    let run_id = extract_run_id(&action).to_string();

    let action = engine
        .advance_step(
            &run_id,
            SopStepResult {
                step_number: 1,
                status: SopStepStatus::Completed,
                output: "done".into(),
                started_at: now_iso8601(),
                completed_at: Some(now_iso8601()),
            },
        )
        .unwrap();

    assert!(matches!(action, SopRunAction::ExecuteStep { .. }));

    let action = engine
        .advance_step(
            &run_id,
            SopStepResult {
                step_number: 2,
                status: SopStepStatus::Completed,
                output: "done".into(),
                started_at: now_iso8601(),
                completed_at: Some(now_iso8601()),
            },
        )
        .unwrap();

    assert!(matches!(action, SopRunAction::Completed { .. }));
    assert!(engine.active_runs().is_empty());
    assert_eq!(engine.finished_runs(None).len(), 1);
}

#[test]
fn step_failure_ends_run() {
    let mut engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::Auto, SopPriority::Normal)]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    let run_id = extract_run_id(&action).to_string();

    let action = engine
        .advance_step(
            &run_id,
            SopStepResult {
                step_number: 1,
                status: SopStepStatus::Failed,
                output: "valve stuck".into(),
                started_at: now_iso8601(),
                completed_at: Some(now_iso8601()),
            },
        )
        .unwrap();

    assert!(
        matches!(action, SopRunAction::Failed { ref reason, .. } if reason.contains("valve stuck"))
    );
    assert!(engine.active_runs().is_empty());
}

#[test]
fn cancel_run() {
    let mut engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::Auto, SopPriority::Normal)]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    let run_id = extract_run_id(&action).to_string();
    engine.cancel_run(&run_id).unwrap();
    assert!(engine.active_runs().is_empty());
    let finished = engine.finished_runs(None);
    assert_eq!(finished[0].status, SopRunStatus::Cancelled);
}

#[test]
fn cancel_unknown_run_fails() {
    let mut engine = engine_with_sops(vec![]);
    assert!(engine.cancel_run("nonexistent").is_err());
}

#[test]
fn per_sop_concurrency_limit() {
    let mut engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::Auto, SopPriority::Normal)]);
    engine.start_run("s1", manual_event()).unwrap();
    assert!(!engine.can_start("s1"));
    assert!(engine.start_run("s1", manual_event()).is_err());
}

#[test]
fn global_concurrency_limit() {
    let sops = vec![
        test_sop("s1", SopExecutionMode::Auto, SopPriority::Normal),
        test_sop("s2", SopExecutionMode::Auto, SopPriority::Normal),
    ];
    let mut engine = SopEngine::new(SopConfig { max_concurrent_total: 1, ..SopConfig::default() });
    engine.sops = sops;

    engine.start_run("s1", manual_event()).unwrap();
    assert!(!engine.can_start("s2"));
}

#[test]
fn cooldown_blocks_immediate_restart() {
    let mut sop = test_sop("s1", SopExecutionMode::Auto, SopPriority::Normal);
    // 冷却窗口以最近完成记录为依据，完整推进两步后再验证立即重启会被阻止。
    sop.cooldown_secs = 3600;
    let mut engine = engine_with_sops(vec![sop]);

    let action = engine.start_run("s1", manual_event()).unwrap();
    let run_id = extract_run_id(&action).to_string();

    engine
        .advance_step(
            &run_id,
            SopStepResult {
                step_number: 1,
                status: SopStepStatus::Completed,
                output: "ok".into(),
                started_at: now_iso8601(),
                completed_at: Some(now_iso8601()),
            },
        )
        .unwrap();
    engine
        .advance_step(
            &run_id,
            SopStepResult {
                step_number: 2,
                status: SopStepStatus::Completed,
                output: "ok".into(),
                started_at: now_iso8601(),
                completed_at: Some(now_iso8601()),
            },
        )
        .unwrap();

    assert!(!engine.can_start("s1"));
}
