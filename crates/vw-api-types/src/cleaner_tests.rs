use serde_json::json;

use crate::cleaner::{
    CleanerCleanupRequest, CleanerInfoResponse, CleanerRunResponse, CleanerScanReport,
    CleanerStatusResponse,
};

#[test]
fn cleaner_cleanup_request_defaults_missing_flags_to_false() {
    let request: CleanerCleanupRequest = serde_json::from_value(json!({})).unwrap();

    assert!(!request.clear_system_temp);
    assert!(!request.clear_chrome);
}

#[test]
fn cleaner_scan_report_roundtrips_as_json() {
    let report = CleanerScanReport { total_bytes: 42, matched_items: 1, groups: Vec::new() };

    let value = serde_json::to_value(&report).unwrap();
    let parsed: CleanerScanReport = serde_json::from_value(value).unwrap();

    assert_eq!(parsed, report);
}

#[test]
fn cleaner_run_response_carries_output_text() {
    let response: CleanerRunResponse = serde_json::from_value(json!({ "output": "done" })).unwrap();

    assert_eq!(response.output, "done");
}

#[test]
fn cleaner_status_defaults_to_idle_empty_output() {
    let status = CleanerStatusResponse::default();

    assert!(!status.running);
    assert!(status.output.is_empty());
}

#[test]
fn cleaner_info_response_carries_platform_support() {
    let info: CleanerInfoResponse =
        serde_json::from_value(json!({ "platform": "macos", "supported": true })).unwrap();

    assert_eq!(info.platform, "macos");
    assert!(info.supported);
}
