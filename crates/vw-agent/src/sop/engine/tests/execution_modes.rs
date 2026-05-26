//! SOP 引擎执行模式测试。
//!
//! 覆盖自动、监督、逐步和按优先级执行模式，验证每种模式在第一步和后续步骤
//! 上是否应直接执行或等待审批。

use super::fixtures::{engine_with_sops, extract_run_id, manual_event, test_sop};
use super::*;

#[test]
fn auto_mode_executes_immediately() {
    let mut engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::Auto, SopPriority::Normal)]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    assert!(matches!(action, SopRunAction::ExecuteStep { .. }));
}

#[test]
fn supervised_mode_waits_on_first_step() {
    let mut engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::Supervised, SopPriority::Normal)]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    assert!(matches!(action, SopRunAction::WaitApproval { .. }));
}

#[test]
fn step_by_step_waits_on_every_step() {
    let mut engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::StepByStep, SopPriority::Normal)]);

    let action = engine.start_run("s1", manual_event()).unwrap();
    let run_id = extract_run_id(&action).to_string();
    assert!(matches!(action, SopRunAction::WaitApproval { .. }));

    let action = engine.approve_step(&run_id).unwrap();
    assert!(matches!(action, SopRunAction::ExecuteStep { .. }));

    let action = engine
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
    assert!(matches!(action, SopRunAction::WaitApproval { .. }));
}

#[test]
fn priority_based_critical_auto() {
    let mut engine = engine_with_sops(vec![test_sop(
        "s1",
        SopExecutionMode::PriorityBased,
        SopPriority::Critical,
    )]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    assert!(matches!(action, SopRunAction::ExecuteStep { .. }));
}

#[test]
fn priority_based_normal_supervised() {
    let mut engine = engine_with_sops(vec![test_sop(
        "s1",
        SopExecutionMode::PriorityBased,
        SopPriority::Normal,
    )]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    assert!(matches!(action, SopRunAction::WaitApproval { .. }));
}

#[test]
fn requires_confirmation_overrides_auto() {
    let mut sop = test_sop("s1", SopExecutionMode::Auto, SopPriority::Critical);
    // 步骤级确认要求优先于自动模式，用于守住单个高风险动作的人工审批边界。
    sop.steps[0].requires_confirmation = true;
    let mut engine = engine_with_sops(vec![sop]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    assert!(matches!(action, SopRunAction::WaitApproval { .. }));
}
