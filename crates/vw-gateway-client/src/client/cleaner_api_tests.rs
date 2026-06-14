use serde_json::json;
use vw_api_types::cleaner::CleanerCleanupRequest;

use crate::client::test_support;

#[tokio::test]
async fn cleaner_api_methods_use_expected_gateway_routes() {
    let server = test_support::server(vec![
        (200, json!({"platform": "macos", "supported": true})),
        (200, json!({"total_bytes": 0, "matched_items": 0, "groups": []})),
        (200, json!({"output": "removed 3 files"})),
        (200, json!({"running": true, "output": "working"})),
        (200, json!({"ok": true})),
    ]);

    let info = server.client().cleaner_info().await.expect("info");
    assert_eq!(info.platform, "macos");
    let scan = server.client().cleaner_scan().await.expect("scan");
    assert_eq!(scan.total_bytes, 0);
    let output = server
        .client()
        .cleaner_run(&CleanerCleanupRequest { clear_system_temp: true, ..Default::default() })
        .await
        .expect("run");
    assert_eq!(output, "removed 3 files");
    let status = server.client().cleaner_status().await.expect("status");
    assert!(status.running);
    server.client().cleaner_cancel().await.expect("cancel");

    assert_eq!(server.take_request().path, "/v1/desktop/cleaner/info");
    let request = server.take_request();
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/v1/desktop/cleaner/scan");
    assert_eq!(request.body, json!({}));
    let request = server.take_request();
    assert_eq!(request.path, "/v1/desktop/cleaner/run");
    assert_eq!(request.body["clear_system_temp"], true);
    assert_eq!(server.take_request().path, "/v1/desktop/cleaner/status");
    assert_eq!(server.take_request().path, "/v1/desktop/cleaner/cancel");
    server.join();
}
