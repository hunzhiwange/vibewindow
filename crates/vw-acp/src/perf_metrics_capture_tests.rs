use super::*;
use std::collections::HashMap;

#[test]
fn perf_metrics_capture_file_from_env_trims_and_rejects_empty_values() {
    let mut env = HashMap::new();
    env.insert(PERF_METRICS_FILE_ENV.to_string(), "  /tmp/vwacp-metrics.ndjson  ".to_string());

    assert_eq!(
        perf_metrics_capture_file_from_env(&env).as_deref(),
        Some(std::path::Path::new("/tmp/vwacp-metrics.ndjson"))
    );

    env.insert(PERF_METRICS_FILE_ENV.to_string(), "   ".to_string());
    assert_eq!(perf_metrics_capture_file_from_env(&env), None);
}

#[test]
fn capture_state_default_is_not_captureable() {
    let state = PerfMetricsCaptureState::default();

    assert!(!state.installed);
    assert!(!state.flushed);
    assert!(!should_capture(&state));
    assert_eq!(state.capture_role, "cli");
}
