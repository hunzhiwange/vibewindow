//! 验证性能指标的内存快照与 JSONL 捕获输出。
//!
//! 性能指标是进程全局状态，这些测试通过互斥锁串行化访问，确保计数器、仪表、
//! timing 和捕获文件的断言不会被并发用例互相影响。

use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use tokio::sync::Mutex;
use vw_acp::{
    CaptureReason, PerfMetricsCaptureOptions, checkpoint_perf_metrics_capture,
    flush_perf_metrics_capture, format_perf_metric, get_perf_metrics_snapshot,
    increment_perf_counter, install_perf_metrics_capture, measure_perf, record_perf_duration,
    reset_perf_metrics, set_perf_gauge, start_perf_timer,
};

/// 生成唯一捕获文件路径，避免性能指标 JSONL 输出覆盖其它测试运行。
fn unique_capture_file() -> std::path::PathBuf {
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    std::env::temp_dir().join(format!("vw-acp-perf-metrics-{unique}.jsonl"))
}

/// 串行化全局性能指标状态的测试访问。
static PERF_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// 验证计数器、仪表和 timing 快照会按约定收集并四舍五入到毫秒精度。
#[tokio::test]
async fn perf_metrics_collect_and_round_snapshot_values() {
    let _guard = PERF_TEST_LOCK.lock().await;
    reset_perf_metrics();

    increment_perf_counter("requests", 2);
    set_perf_gauge("depth", 1.23456);
    record_perf_duration("roundtrip", 12.34567);
    let stop = start_perf_timer("timer");
    let elapsed = stop();
    measure_perf("async", || async { 7_u8 }).await;

    let snapshot = get_perf_metrics_snapshot();
    assert_eq!(snapshot.counters.get("requests"), Some(&2));
    assert_eq!(snapshot.gauges.get("depth"), Some(&1.235));
    assert_eq!(snapshot.timings.get("roundtrip").map(|entry| entry.total_ms), Some(12.346));
    assert_eq!(snapshot.timings.get("roundtrip").map(|entry| entry.max_ms), Some(12.346));
    assert_eq!(snapshot.timings.get("timer").map(|entry| entry.count), Some(1));
    assert_eq!(snapshot.timings.get("async").map(|entry| entry.count), Some(1));
    assert!(elapsed >= 0.0);
    assert_eq!(format_perf_metric("db", 3.4567), "db=3.457ms");
}

/// 验证捕获器会写出 checkpoint 与 flush 两类 JSONL 记录，并递增 sequence。
#[test]
fn perf_metrics_capture_writes_checkpoint_and_flush_records() {
    let _guard = PERF_TEST_LOCK.blocking_lock();
    reset_perf_metrics();

    let capture_file = unique_capture_file();
    install_perf_metrics_capture(PerfMetricsCaptureOptions {
        argv: Some(vec!["vwacp".to_string(), "--json".to_string()]),
        role: Some("cli".to_string()),
        file_path: Some(capture_file.clone()),
    });

    increment_perf_counter("requests", 1);
    checkpoint_perf_metrics_capture();

    set_perf_gauge("depth", 2.5);
    flush_perf_metrics_capture(CaptureReason::Exit);

    let payload = std::fs::read_to_string(&capture_file).expect("capture file should exist");
    let entries = payload
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("capture line should be valid json"))
        .collect::<Vec<_>>();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].get("reason").and_then(Value::as_str), Some("checkpoint"));
    assert_eq!(entries[0].get("sequence").and_then(Value::as_u64), Some(0));
    assert_eq!(
        entries[0]
            .get("metrics")
            .and_then(|value| value.get("counters"))
            .and_then(|value| value.get("requests"))
            .and_then(Value::as_i64),
        Some(1)
    );

    assert_eq!(entries[1].get("reason").and_then(Value::as_str), Some("exit"));
    assert_eq!(entries[1].get("sequence").and_then(Value::as_u64), Some(1));
    assert_eq!(
        entries[1]
            .get("metrics")
            .and_then(|value| value.get("gauges"))
            .and_then(|value| value.get("depth"))
            .and_then(Value::as_f64),
        Some(2.5)
    );

    let _ = std::fs::remove_file(capture_file);
}
