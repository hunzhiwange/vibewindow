//! SOP 查询与时间工具测试。
//!
//! 覆盖运行查询、步骤上下文渲染和 ISO8601 时间解析，确保执行器生成的上下文
//! 可读且已完成运行仍可被追踪。

use super::fixtures::{engine_with_sops, extract_run_id, manual_event, test_sop};
use super::*;

#[test]
fn step_context_includes_sop_name_and_step() {
    let sop = test_sop("pump-shutdown", SopExecutionMode::Auto, SopPriority::Critical);
    let run = SopRun {
        run_id: "run-001".into(),
        sop_name: "pump-shutdown".into(),
        trigger_event: manual_event(),
        status: SopRunStatus::Running,
        current_step: 1,
        total_steps: 2,
        started_at: now_iso8601(),
        completed_at: None,
        step_results: Vec::new(),
        waiting_since: None,
    };
    let ctx = format_step_context(&sop, &run, &sop.steps[0]);
    assert!(ctx.contains("pump-shutdown"));
    assert!(ctx.contains("Step 1 of 2"));
    assert!(ctx.contains("Step one"));
}

#[test]
fn get_run_finds_active_and_finished() {
    let mut engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::Auto, SopPriority::Normal)]);
    let action = engine.start_run("s1", manual_event()).unwrap();
    let run_id = extract_run_id(&action).to_string();

    assert!(engine.get_run(&run_id).is_some());
    assert_eq!(engine.get_run(&run_id).unwrap().status, SopRunStatus::Running);

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

    assert!(engine.get_run(&run_id).is_some());
    assert_eq!(engine.get_run(&run_id).unwrap().status, SopRunStatus::Completed);

    assert!(engine.get_run("nonexistent").is_none());
}

#[test]
fn iso8601_roundtrip() {
    let ts = now_iso8601();
    let secs = parse_iso8601_secs(&ts);
    assert!(secs.is_some());
    // 只要求接近当前时间，避免测试因为秒级边界抖动而变脆。
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    assert!(now.abs_diff(secs.unwrap()) < 2);
}

#[test]
fn parse_known_timestamp() {
    let secs = parse_iso8601_secs("2026-01-01T00:00:00Z").unwrap();
    assert_eq!(secs, 20454 * 86400);
}
