//! SOP 完成记录保留策略测试。
//!
//! 验证完成运行列表的上限裁剪语义，以及 `0` 作为“不限制”的特殊配置含义。

use super::fixtures::{extract_run_id, manual_event, test_sop};
use super::*;

#[test]
fn max_finished_runs_evicts_oldest() {
    let mut engine = SopEngine::new(SopConfig { max_finished_runs: 2, ..SopConfig::default() });
    let mut sop = test_sop("s1", SopExecutionMode::Auto, SopPriority::Normal);
    sop.steps = vec![sop.steps[0].clone()];
    sop.max_concurrent = 10;
    engine.sops = vec![sop];

    let mut finished_ids = Vec::new();
    for _ in 0..3 {
        let action = engine.start_run("s1", manual_event()).unwrap();
        let rid = extract_run_id(&action).to_string();
        engine
            .advance_step(
                &rid,
                SopStepResult {
                    step_number: 1,
                    status: SopStepStatus::Completed,
                    output: "ok".into(),
                    started_at: now_iso8601(),
                    completed_at: Some(now_iso8601()),
                },
            )
            .unwrap();
        finished_ids.push(rid);
    }

    let finished = engine.finished_runs(None);
    // 裁剪应保留最近完成的运行，最旧记录被淘汰。
    assert_eq!(finished.len(), 2, "eviction should cap at max_finished_runs");
    assert_eq!(finished[0].run_id, finished_ids[1]);
    assert_eq!(finished[1].run_id, finished_ids[2]);
}

#[test]
fn max_finished_runs_zero_means_unlimited() {
    let mut engine = SopEngine::new(SopConfig { max_finished_runs: 0, ..SopConfig::default() });
    let mut sop = test_sop("s1", SopExecutionMode::Auto, SopPriority::Normal);
    sop.steps = vec![sop.steps[0].clone()];
    sop.max_concurrent = 10;
    engine.sops = vec![sop];

    for _ in 0..5 {
        let action = engine.start_run("s1", manual_event()).unwrap();
        let rid = extract_run_id(&action).to_string();
        engine
            .advance_step(
                &rid,
                SopStepResult {
                    step_number: 1,
                    status: SopStepStatus::Completed,
                    output: "ok".into(),
                    started_at: now_iso8601(),
                    completed_at: Some(now_iso8601()),
                },
            )
            .unwrap();
    }

    assert_eq!(engine.finished_runs(None).len(), 5, "zero means unlimited");
}
