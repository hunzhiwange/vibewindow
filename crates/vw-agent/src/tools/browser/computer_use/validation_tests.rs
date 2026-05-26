use super::*;
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::tools::browser::computer_use::ComputerUseConfig;
use std::sync::Arc;

fn client(config: ComputerUseConfig, allowed_domains: Vec<String>) -> ComputerUseClient {
    ComputerUseClient::new(Arc::new(SecurityPolicy::default()), allowed_domains, None, config)
}

#[test]
fn endpoint_url_keeps_remote_public_hosts_https_only() {
    let local = client(ComputerUseConfig::default(), vec!["example.com".into()]);
    assert!(local.endpoint_url().is_ok());

    let remote_http = client(
        ComputerUseConfig {
            endpoint: "http://example.com/actions".into(),
            allow_remote_endpoint: true,
            ..Default::default()
        },
        vec!["example.com".into()],
    );
    assert!(remote_http.endpoint_url().is_err());

    let remote_https = client(
        ComputerUseConfig {
            endpoint: "https://example.com/actions".into(),
            allow_remote_endpoint: true,
            ..Default::default()
        },
        vec!["example.com".into()],
    );
    assert!(remote_https.endpoint_url().is_ok());
}

#[test]
fn validate_coordinate_enforces_lower_and_upper_bounds() {
    let c = client(ComputerUseConfig::default(), vec![]);
    assert!(c.validate_coordinate("x", 0, Some(10)).is_ok());
    assert!(c.validate_coordinate("x", -1, Some(10)).is_err());
    assert!(c.validate_coordinate("x", 11, Some(10)).is_err());
    assert!(c.validate_coordinate("x", 1, Some(-1)).is_err());
}

#[test]
fn read_required_i64_rejects_missing_or_wrong_type() {
    let params = serde_json::json!({"x": 3, "y": "4"}).as_object().unwrap().clone();
    assert_eq!(read_required_i64(&params, "x").unwrap(), 3);
    assert!(read_required_i64(&params, "y").is_err());
    assert!(read_required_i64(&params, "z").is_err());
}
