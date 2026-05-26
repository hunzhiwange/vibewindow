use super::*;
use crate::app::agent::security::SecurityPolicy;
use std::sync::Arc;

#[tokio::test]
async fn execute_action_rejects_invalid_endpoint_before_network_call() {
    let client = ComputerUseClient::new(
        Arc::new(SecurityPolicy::default()),
        vec!["example.com".to_string()],
        Some("session".to_string()),
        super::super::config::ComputerUseConfig {
            endpoint: "not a url".to_string(),
            ..Default::default()
        },
    );

    let err = client
        .execute_action("open", &serde_json::json!({"url":"https://example.com"}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("Invalid browser.computer_use.endpoint"));
}
