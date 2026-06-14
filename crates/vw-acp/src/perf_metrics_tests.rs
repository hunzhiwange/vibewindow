use super::*;

use std::sync::{Mutex, MutexGuard};

static TEST_LOCK: Mutex<()> = Mutex::new(());

fn lock_test_metrics() -> MutexGuard<'static, ()> {
    TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[test]
fn perf_metrics_snapshot_rounds_values_and_accumulates() {
    let _guard = lock_test_metrics();
    reset_perf_metrics();

    increment_perf_counter("requests", 2);
    increment_perf_counter("requests", 3);
    set_perf_gauge("depth", 1.23456);
    record_perf_duration("run", 1.1111);
    record_perf_duration("run", 2.2222);

    let snapshot = get_perf_metrics_snapshot();
    assert_eq!(snapshot.counters["requests"], 5);
    assert_eq!(snapshot.gauges["depth"], 1.235);
    assert_eq!(snapshot.timings["run"].count, 2);
    assert_eq!(snapshot.timings["run"].total_ms, 3.333);
    assert_eq!(snapshot.timings["run"].max_ms, 2.222);
}

#[test]
fn format_perf_metric_uses_same_rounding_rule() {
    assert_eq!(format_perf_metric("load", 1.23456), "load=1.235ms");
}

#[test]
fn measure_perf_returns_result_and_records_timing() {
    let _guard = lock_test_metrics();
    reset_perf_metrics();

    let runtime =
        tokio::runtime::Builder::new_current_thread().build().expect("test runtime should build");
    let result = runtime.block_on(measure_perf("async-run", || async { "done" }));

    let snapshot = get_perf_metrics_snapshot();
    let timing = &snapshot.timings["async-run"];
    assert_eq!(result, "done");
    assert_eq!(timing.count, 1);
    assert!(timing.total_ms >= 0.0);
    assert_eq!(timing.max_ms, timing.total_ms);
}

#[test]
fn start_perf_timer_records_elapsed_duration_once() {
    let _guard = lock_test_metrics();
    reset_perf_metrics();

    let stop_timer = start_perf_timer("manual-run");
    let elapsed_ms = stop_timer();

    let snapshot = get_perf_metrics_snapshot();
    let timing = &snapshot.timings["manual-run"];
    assert!(elapsed_ms >= 0.0);
    assert_eq!(timing.count, 1);
    assert_eq!(timing.total_ms, timing.max_ms);
}
