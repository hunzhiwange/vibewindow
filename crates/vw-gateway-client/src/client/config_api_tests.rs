use serde::Deserialize;
use serde_json::json;

use crate::client::test_support;

#[derive(Debug, Deserialize)]
struct AcpConfig {
    agent: String,
}

#[tokio::test]
async fn config_api_reads_patches_and_maps_nested_global_paths() {
    let server = test_support::server(vec![
        (200, json!({"model": "gpt"})),
        (200, json!({"model": "o3"})),
        (200, json!({"global": true})),
        (200, json!({})),
        (200, json!({"agent": "codex"})),
        (200, json!({"app_ui": {"system_settings": {"theme": "dark"}}})),
        (200, json!({})),
        (200, json!({"app_ui": {}})),
    ]);

    assert_eq!(
        server.client().config_get(Some("/tmp/project")).await.expect("config")["model"],
        "gpt"
    );
    assert_eq!(
        server
            .client()
            .config_patch(Some("/tmp/project"), &json!({"model": "o3"}))
            .await
            .expect("patch")["model"],
        "o3"
    );
    assert!(
        server.client().global_config_get().await.expect("global")["global"].as_bool().unwrap()
    );
    server.client().global_config_patch(&json!({"x": 1})).await.expect("global patch");
    let acp: AcpConfig = server.client().global_acp_config_get().await.expect("acp");
    assert_eq!(acp.agent, "codex");
    assert_eq!(
        server
            .client()
            .global_config_get_path(&["app_ui", "system_settings", "theme"])
            .await
            .expect("path"),
        Some(json!("dark"))
    );
    server
        .client()
        .global_config_patch_path(&["app_ui", "system_settings"], json!({"theme": "light"}))
        .await
        .expect("patch path");
    assert_eq!(
        server
            .client()
            .global_config_get_path(&["app_ui", "system_settings"])
            .await
            .expect("missing path"),
        None
    );

    assert!(server.take_request().path.contains("/v1/config?directory=%2Ftmp%2Fproject"));
    let request = server.take_request();
    assert_eq!(request.method, "PATCH");
    assert!(request.path.contains("/v1/config?directory=%2Ftmp%2Fproject"));
    assert_eq!(request.body["model"], "o3");
    assert_eq!(server.take_request().path, "/v1/global/config");
    assert_eq!(server.take_request().path, "/v1/global/config");
    assert_eq!(server.take_request().path, "/v1/global/config/acp");
    assert_eq!(server.take_request().path, "/v1/global/config");
    let request = server.take_request();
    assert_eq!(request.method, "PATCH");
    assert_eq!(request.body, json!({"app_ui": {"system_settings": {"theme": "light"}}}));
    assert_eq!(server.take_request().path, "/v1/global/config");
    server.join();
}
