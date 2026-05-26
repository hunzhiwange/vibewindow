//! SOP 引擎审批与超时策略测试。
//!
//! 这些用例验证 supervised/critical 流程在等待审批、人工批准和超时自动批准
//! 之间的状态转换，确保安全相关步骤不会因为时间配置而被普通优先级任务绕过。

use super::fixtures::{engine_with_sops, extract_run_id, manual_event, test_sop};
use super::*;

#[test]
fn approve_transitions_to_execute() {
    let mut engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::Supervised, SopPriority::Normal)]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    let run_id = extract_run_id(&action).to_string();

    let run = engine.active_runs().get(&run_id).unwrap();
    assert_eq!(run.status, SopRunStatus::WaitingApproval);

    let action = engine.approve_step(&run_id).unwrap();
    assert!(matches!(action, SopRunAction::ExecuteStep { .. }));

    let run = engine.active_runs().get(&run_id).unwrap();
    assert_eq!(run.status, SopRunStatus::Running);
}

#[test]
fn approve_non_waiting_fails() {
    let mut engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::Auto, SopPriority::Normal)]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    let run_id = extract_run_id(&action).to_string();
    assert!(engine.approve_step(&run_id).is_err());
}

#[test]
fn timeout_auto_approves_critical() {
    let mut engine = SopEngine::new(SopConfig { approval_timeout_secs: 1, ..SopConfig::default() });
    let mut sop = test_sop("s1", SopExecutionMode::Supervised, SopPriority::Critical);
    sop.execution_mode = SopExecutionMode::Supervised;
    engine.set_sops_for_test(vec![sop]);

    let action = engine.start_run("s1", manual_event()).unwrap();
    let run_id = extract_run_id(&action).to_string();
    assert!(matches!(action, SopRunAction::WaitApproval { .. }));

    let run = engine.active_runs.get_mut(&run_id).unwrap();
    // 使用固定的旧时间戳制造超时，避免测试依赖真实等待。
    run.waiting_since = Some("2020-01-01T00:00:00Z".into());

    let actions = engine.check_approval_timeouts();
    assert_eq!(actions.len(), 1);
    assert!(matches!(actions[0], SopRunAction::ExecuteStep { .. }));
}

#[test]
fn timeout_does_not_auto_approve_normal() {
    let mut engine = SopEngine::new(SopConfig { approval_timeout_secs: 1, ..SopConfig::default() });
    engine.set_sops_for_test(vec![test_sop(
        "s1",
        SopExecutionMode::Supervised,
        SopPriority::Normal,
    )]);

    let action = engine.start_run("s1", manual_event()).unwrap();
    let run_id = extract_run_id(&action).to_string();

    let run = engine.active_runs.get_mut(&run_id).unwrap();
    // 普通优先级即便超时也保持等待审批，防止超时机制扩大自动执行范围。
    run.waiting_since = Some("2020-01-01T00:00:00Z".into());

    let actions = engine.check_approval_timeouts();
    assert!(actions.is_empty());
    assert_eq!(engine.get_run(&run_id).unwrap().status, SopRunStatus::WaitingApproval);
}

#[test]
fn timeout_zero_disables_check() {
    let mut engine = SopEngine::new(SopConfig { approval_timeout_secs: 0, ..SopConfig::default() });
    engine.set_sops_for_test(vec![test_sop(
        "s1",
        SopExecutionMode::Supervised,
        SopPriority::Critical,
    )]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    let run_id = extract_run_id(&action).to_string();

    let run = engine.active_runs.get_mut(&run_id).unwrap();
    run.waiting_since = Some("2020-01-01T00:00:00Z".into());

    let actions = engine.check_approval_timeouts();
    assert!(actions.is_empty());
}

#[test]
fn waiting_since_set_on_wait_approval() {
    let mut engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::Supervised, SopPriority::Normal)]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    let run_id = extract_run_id(&action).to_string();

    let run = engine.get_run(&run_id).unwrap();
    assert_eq!(run.status, SopRunStatus::WaitingApproval);
    assert!(run.waiting_since.is_some());
}

#[test]
fn waiting_since_cleared_on_approve() {
    let mut engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::Supervised, SopPriority::Normal)]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    let run_id = extract_run_id(&action).to_string();
    engine.approve_step(&run_id).unwrap();

    let run = engine.get_run(&run_id).unwrap();
    assert_eq!(run.status, SopRunStatus::Running);
    assert!(run.waiting_since.is_none());
}
