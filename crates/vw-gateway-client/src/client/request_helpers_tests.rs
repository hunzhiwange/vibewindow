use serde_json::json;

use crate::client::test_support;

#[tokio::test]
async fn request_helpers_cover_http_verbs_success_404_decode_and_error_paths() {
    let server = test_support::server(vec![
        (200, json!({"ok": "get"})),
        (200, json!({"ok": "post"})),
        (200, json!({"ok": "patch"})),
        (200, json!({"ok": "put"})),
        (204, json!(null)),
        (200, json!({"ok": "delete_json"})),
        (200, json!({"ok": "optional"})),
        (404, json!({"message": "missing"})),
        (500, json!({"message": "boom"})),
        (200, json!("not a number")),
    ]);

    let get: serde_json::Value = server
        .client()
        .get_json("/helper/get", &[("q".to_string(), "one".to_string())])
        .await
        .expect("get");
    assert_eq!(get["ok"], "get");
    let post: serde_json::Value =
        server.client().post_json("/helper/post", &[], &json!({"a": 1})).await.expect("post");
    assert_eq!(post["ok"], "post");
    let patch: serde_json::Value =
        server.client().patch_json("/helper/patch", &[], &json!({"b": 2})).await.expect("patch");
    assert_eq!(patch["ok"], "patch");
    let put: serde_json::Value =
        server.client().put_json("/helper/put", &[], &json!({"c": 3})).await.expect("put");
    assert_eq!(put["ok"], "put");
    server.client().delete_empty("/helper/delete-empty", &[]).await.expect("delete empty");
    let deleted: serde_json::Value = server
        .client()
        .delete_json("/helper/delete-json", &[], &json!({"d": 4}))
        .await
        .expect("delete json");
    assert_eq!(deleted["ok"], "delete_json");
    let optional: Option<serde_json::Value> =
        server.client().get_json_with_404("/helper/optional", &[]).await.expect("optional");
    assert_eq!(optional.expect("some")["ok"], "optional");
    let missing: Option<serde_json::Value> =
        server.client().get_json_with_404("/helper/missing", &[]).await.expect("missing");
    assert!(missing.is_none());
    let status_error = server
        .client()
        .get_json::<serde_json::Value>("/helper/error", &[])
        .await
        .expect_err("status error");
    assert!(status_error.contains("gateway request failed: 500"));
    let decode_error = server
        .client()
        .get_json::<u64>("/helper/decode-error", &[])
        .await
        .expect_err("decode error");
    assert!(!decode_error.is_empty());

    assert_eq!(server.take_request().path, "/helper/get?q=one");
    let request = server.take_request();
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/helper/post");
    assert_eq!(request.body, json!({"a": 1}));
    let request = server.take_request();
    assert_eq!(request.method, "PATCH");
    assert_eq!(request.body, json!({"b": 2}));
    let request = server.take_request();
    assert_eq!(request.method, "PUT");
    assert_eq!(request.body, json!({"c": 3}));
    assert_eq!(server.take_request().method, "DELETE");
    let request = server.take_request();
    assert_eq!(request.method, "DELETE");
    assert_eq!(request.body, json!({"d": 4}));
    assert_eq!(server.take_request().path, "/helper/optional");
    assert_eq!(server.take_request().path, "/helper/missing");
    assert_eq!(server.take_request().path, "/helper/error");
    assert_eq!(server.take_request().path, "/helper/decode-error");
    server.join();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn get_json_blocking_maps_success_status_error_and_decode_error() {
    let server = test_support::server(vec![
        (200, json!({"ok": true})),
        (500, json!({"message": "boom"})),
        (200, json!("not a bool")),
    ]);

    let ok: serde_json::Value = super::get_json_blocking(
        server.client().endpoint(),
        "/blocking/ok",
        &[("mode".to_string(), "sync".to_string())],
    )
    .expect("blocking ok");
    assert_eq!(ok["ok"], true);
    let status_error = super::get_json_blocking::<serde_json::Value>(
        server.client().endpoint(),
        "/blocking/error",
        &[],
    )
    .expect_err("blocking status error");
    assert!(status_error.contains("gateway request failed: 500"));
    let decode_error =
        super::get_json_blocking::<bool>(server.client().endpoint(), "/blocking/decode", &[])
            .expect_err("blocking decode error");
    assert!(!decode_error.is_empty());

    assert_eq!(server.take_request().path, "/blocking/ok?mode=sync");
    assert_eq!(server.take_request().path, "/blocking/error");
    assert_eq!(server.take_request().path, "/blocking/decode");
    server.join();
}
