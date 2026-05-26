use super::*;
use crate::app::agent::security::SecurityPolicy;
use std::sync::Arc;

#[test]
fn new_preserves_security_boundary_inputs() {
    let client = ComputerUseClient::new(
        Arc::new(SecurityPolicy::default()),
        vec!["example.com".to_string()],
        Some("session-a".to_string()),
        ComputerUseConfig {
            timeout_ms: 42,
            ..Default::default()
        },
    );

    assert_eq!(client.allowed_domains, vec!["example.com"]);
    assert_eq!(client.session_name.as_deref(), Some("session-a"));
    assert_eq!(client.config.timeout_ms, 42);
}
