use super::*;
use crate::app::agent::config::IdentityConfig;

fn config(format: &str, path: Option<&str>, inline: Option<&str>) -> IdentityConfig {
    IdentityConfig {
        format: format.to_string(),
        aieos_path: path.map(ToString::to_string),
        aieos_inline: inline.map(ToString::to_string),
    }
}

#[test]
fn is_aieos_configured_requires_aieos_format_and_source() {
    assert!(is_aieos_configured(&config("aieos", Some("identity.json"), None)));
    assert!(!is_aieos_configured(&config("local", Some("identity.json"), None)));
    assert!(!is_aieos_configured(&config("aieos", None, None)));
}

#[test]
fn parse_aieos_identity_rejects_non_object_payload() {
    let error = parse_aieos_identity("[]").expect_err("array is not a valid identity object");

    assert!(error.to_string().contains("JSON 对象"));
}
