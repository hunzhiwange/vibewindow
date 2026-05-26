use super::*;

#[test]
fn perf_metrics_snapshot_rounds_values_and_accumulates() {
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
